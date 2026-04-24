pub(super) mod draw;
pub(super) mod ui;
pub(super) mod util;

use egui::epaint::Vertex;
use glow::{HasContext, NativeProgram, NativeTexture};
use memoffset::offset_of;
use std::num::NonZeroU32;

use crate::{
    egui_window::painter::util::{compile_shader, link_program},
    geometry::WindowRect,
};

fn init_gl() -> glow::Context {
    gl_loader::init_gl();
    unsafe { glow::Context::from_loader_function(|f| gl_loader::get_proc_address(f).cast()) }
}

#[derive(Debug)]
pub struct Painter {
    viewport: WindowRect,
    old_viewport: [i32; 4],

    gl: glow::Context,
    program: glow::Program,
    u_screen_size: Option<glow::UniformLocation>,
    u_sampler: Option<glow::UniformLocation>,

    vao: Option<glow::NativeVertexArray>,
    vbo: Option<glow::Buffer>,
    ebo: Option<glow::Buffer>,

    supports_srgb_framebuffer: bool,
    max_texture_side: usize,
    xp_texture_id: i32,

    destroyed: bool,
}

impl Painter {
    #[allow(clippy::cast_sign_loss)]
    pub fn new() -> anyhow::Result<Self> {
        let gl = init_gl();
        crate::check_for_gl_error_even_in_release!(&gl, "before init painter");

        unsafe {
            let version = gl.get_parameter_string(glow::VERSION);
            let renderer = gl.get_parameter_string(glow::RENDERER);
            let vendor = gl.get_parameter_string(glow::VENDOR);
            debug!("\nOpenGL Version: {version}\n  Renderer: {renderer}\n  Vendor: {vendor}");
        }

        let max_texture_side = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) } as usize;
        debug!("max texture size: {max_texture_side}");
        let supported_extensions = gl.supported_extensions();
        let supports_srgb_framebuffer = !cfg!(target_arch = "wasm32")
            && supported_extensions.iter().any(|extension| {
                // {GL,GLX,WGL}_ARB_framebuffer_sRGB, …
                extension.ends_with("ARB_framebuffer_sRGB")
            });
        debug!("SRGB framebuffer Support: {supports_srgb_framebuffer}");

        let mut painter = Painter {
            viewport: WindowRect::default(),
            old_viewport: [0; 4],

            program: Self::create_program(&gl)?,
            gl,
            u_screen_size: None,
            u_sampler: None,

            vao: None,
            vbo: None,
            ebo: None,

            supports_srgb_framebuffer,
            max_texture_side,
            xp_texture_id: 0,
            destroyed: false,
        };

        painter.setup_uniforms();
        crate::check_for_gl_error_even_in_release!(&painter.gl, "after setup_uniforms");
        painter.setup_buffers()?;
        crate::check_for_gl_error_even_in_release!(&painter.gl, "after setup_buffers");

        unsafe {
            xplm_sys::XPLMGenerateTextureNumbers(&raw mut painter.xp_texture_id, 1);
        }

        Ok(painter)
    }

    pub fn set_viewport(&mut self, viewport: WindowRect) {
        self.viewport = viewport;
    }

    pub fn max_texture_side(&self) -> usize {
        self.max_texture_side
    }

    fn create_program(gl: &glow::Context) -> anyhow::Result<NativeProgram> {
        let vert = compile_shader(gl, glow::VERTEX_SHADER, include_str!("shader.vert"))?;
        let frag = compile_shader(gl, glow::FRAGMENT_SHADER, include_str!("shader.frag"))?;
        let program = link_program(gl, [vert, frag].iter())?;
        unsafe {
            gl.detach_shader(program, vert);
            gl.detach_shader(program, frag);
            gl.delete_shader(vert);
            gl.delete_shader(frag);
        }
        Ok(program)
    }

    fn setup_uniforms(&mut self) {
        unsafe {
            self.u_screen_size = self.gl.get_uniform_location(self.program, "u_screen_size");
            self.u_sampler = self.gl.get_uniform_location(self.program, "u_sampler");
        }
    }

    fn setup_buffers(&mut self) -> anyhow::Result<()> {
        let gl = &self.gl;
        let program = self.program;
        unsafe {
            self.vao = Some(
                gl.create_vertex_array()
                    .map_err(|e| anyhow!("gl error create vao: {e}"))?,
            );
            gl.bind_vertex_array(self.vao);

            self.vbo = Some(
                gl.create_buffer()
                    .map_err(|e| anyhow!("vbo create error: {e}"))?,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, self.vbo);

            self.ebo = Some(
                gl.create_buffer()
                    .map_err(|e| anyhow!("ebo create error: {e}"))?,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.ebo);

            let a_pos_loc = gl
                .get_attrib_location(program, "a_pos")
                .ok_or(anyhow!("attribute a_pos not found"))?;
            let a_tc_loc = gl
                .get_attrib_location(program, "a_tc")
                .ok_or(anyhow!("attribute a_tc not found"))?;
            let a_srgba_loc = gl
                .get_attrib_location(program, "a_srgba")
                .ok_or(anyhow!("attribute a_srgba not found"))?;

            let stride = i32::try_from(std::mem::size_of::<Vertex>())?;
            gl.vertex_attrib_pointer_f32(
                a_pos_loc,
                2,
                glow::FLOAT,
                false,
                stride,
                i32::try_from(offset_of!(Vertex, pos))?,
            );
            gl.vertex_attrib_pointer_f32(
                a_tc_loc,
                2,
                glow::FLOAT,
                false,
                stride,
                i32::try_from(offset_of!(Vertex, uv))?,
            );
            gl.vertex_attrib_pointer_f32(
                a_srgba_loc,
                4,
                glow::UNSIGNED_BYTE,
                false,
                stride,
                i32::try_from(offset_of!(Vertex, color))?,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.use_program(None);
        }

        Ok(())
    }

    fn bind_texture(&self) {
        unsafe {
            xplm_sys::XPLMBindTexture2d(self.xp_texture_id, 0);
        }
    }

    pub fn destroy(&mut self) {
        if !self.destroyed {
            unsafe {
                self.gl.delete_program(self.program);
                if let Some(vbo) = self.vbo {
                    self.gl.delete_buffer(vbo);
                }
                if let Some(ebo) = self.ebo {
                    self.gl.delete_buffer(ebo);
                }
                if let Some(vao) = self.vao {
                    self.gl.delete_vertex_array(vao);
                }

                if let Some(id) = NonZeroU32::new(self.xp_texture_id.cast_unsigned()) {
                    self.bind_texture();
                    self.gl.delete_texture(NativeTexture(id));
                }
            }
            self.destroyed = true;
        }
    }
}

impl Drop for Painter {
    fn drop(&mut self) {
        if !self.destroyed {
            warn!("Painter was not destroyed before drop, resources will leak");
        }
    }
}
