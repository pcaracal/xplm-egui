use glow::HasContext;

use crate::egui_window::painter::Painter;

impl Painter {
    pub fn run_ui(
        &mut self,
        ctx: &egui::Context,
        input: &mut egui::RawInput,
        ui: impl FnMut(&mut egui::Ui),
    ) -> egui::PlatformOutput {
        let egui::FullOutput {
            textures_delta,
            shapes,
            pixels_per_point,
            platform_output,
            ..
        } = ctx.run_ui(input.take(), ui);

        for (_, delta) in &textures_delta.set {
            match &delta.image {
                egui::ImageData::Color(image) => {
                    let data: &[u8] = bytemuck::cast_slice(image.pixels.as_ref());
                    self.set_texture(delta.pos, image.size, delta.options, data);
                }
            }
        }

        self.prepare();

        let primitives = ctx.tessellate(shapes, pixels_per_point);
        for primitive in primitives {
            self.draw(primitive);
        }

        self.cleanup();

        platform_output
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    fn set_texture(
        &mut self,
        pos: Option<[usize; 2]>,
        [w, h]: [usize; 2],
        options: egui::TextureOptions,
        data: &[u8],
    ) {
        self.bind_texture();

        unsafe {
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                options.magnification.glow_code(None).cast_signed(),
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                options
                    .minification
                    .glow_code(options.mipmap_mode)
                    .cast_signed(),
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                options.wrap_mode.glow_code().cast_signed(),
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                options.wrap_mode.glow_code().cast_signed(),
            );
            crate::check_for_gl_error!(&self.gl, "tex_parameter");

            let internal_format = glow::RGBA8;
            let src_format = glow::RGBA;

            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            let level = 0;
            if let Some([x, y]) = pos {
                self.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    level,
                    x as _,
                    y as _,
                    w as _,
                    h as _,
                    src_format,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                crate::check_for_gl_error!(&self.gl, "tex_sub_image_2d");
            } else {
                let border = 0;
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    level,
                    internal_format.cast_signed(),
                    w as _,
                    h as _,
                    border,
                    src_format,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                crate::check_for_gl_error!(&self.gl, "tex_image_2d");
            }

            if options.mipmap_mode.is_some() {
                self.gl.generate_mipmap(glow::TEXTURE_2D);
                crate::check_for_gl_error!(&self.gl, "generate_mipmap");
            }
        }
    }
}

trait TextureFilterExt {
    fn glow_code(&self, mipmap: Option<egui::TextureFilter>) -> u32;
}

impl TextureFilterExt for egui::TextureFilter {
    fn glow_code(&self, mipmap: Option<egui::TextureFilter>) -> u32 {
        match (self, mipmap) {
            (Self::Linear, None) => glow::LINEAR,
            (Self::Nearest, None) => glow::NEAREST,
            (Self::Linear, Some(Self::Linear)) => glow::LINEAR_MIPMAP_LINEAR,
            (Self::Nearest, Some(Self::Linear)) => glow::NEAREST_MIPMAP_LINEAR,
            (Self::Linear, Some(Self::Nearest)) => glow::LINEAR_MIPMAP_NEAREST,
            (Self::Nearest, Some(Self::Nearest)) => glow::NEAREST_MIPMAP_NEAREST,
        }
    }
}

trait TextureWrapModeExt {
    fn glow_code(&self) -> u32;
}

impl TextureWrapModeExt for egui::TextureWrapMode {
    fn glow_code(&self) -> u32 {
        match self {
            Self::ClampToEdge => glow::CLAMP_TO_EDGE,
            Self::Repeat => glow::REPEAT,
            Self::MirroredRepeat => glow::MIRRORED_REPEAT,
        }
    }
}
