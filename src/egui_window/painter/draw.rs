use std::num::NonZeroU32;

use egui::{ClippedPrimitive, epaint::Primitive};
use glow::{HasContext, NativeTexture};

use crate::{debugln, geometry::RectExt};

impl super::Painter {
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn prepare(&mut self) {
        unsafe {
            self.gl.enable(glow::SCISSOR_TEST);
            // xplane does not like counter clockwise winding order, but egui needs it
            self.gl.disable(glow::CULL_FACE);

            self.last_texture = self.gl.get_parameter_i32(glow::TEXTURE_BINDING_2D);

            self.gl
                .get_parameter_i32_slice(glow::VIEWPORT, &mut self.old_viewport);
            self.gl.viewport(
                self.viewport.origin.x,
                self.viewport.origin.y,
                self.viewport.size.width,
                self.viewport.size.height,
            );

            self.gl.color_mask(true, true, true, true);

            self.gl.scissor(
                self.viewport.left(),
                self.viewport.bottom(),
                self.viewport.width(),
                self.viewport.height(),
            );
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl
                .blend_equation_separate(glow::FUNC_ADD, glow::FUNC_ADD);
            // egui uses premultiplied alpha
            self.gl.blend_func_separate(
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE_MINUS_DST_ALPHA,
                glow::ONE,
            );
            if self.supports_srgb_framebuffer {
                self.gl.disable(glow::FRAMEBUFFER_SRGB);
                crate::check_for_gl_error!(&self.gl, "FRAMEBUFFER_SRGB");
            }

            self.gl.use_program(Some(self.program));

            self.gl.uniform_2_f32(
                self.u_screen_size.as_ref(),
                self.viewport.width() as f32,
                self.viewport.height() as f32,
            );
            self.gl.uniform_1_i32(self.u_sampler.as_ref(), 0);
            self.bind_texture();
        }

        crate::check_for_gl_error!(&self.gl, "prepare");
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub(super) fn draw(&mut self, primitive: ClippedPrimitive) {
        let ClippedPrimitive {
            clip_rect,
            primitive,
        } = primitive;
        let Primitive::Mesh(mesh) = primitive else {
            return;
        };

        self.set_clip_rect(clip_rect);

        unsafe {
            self.gl.bind_vertex_array(self.vao);
            self.gl.bind_buffer(glow::ARRAY_BUFFER, self.vbo);
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.ebo);
            if self.vao.is_none() {
                let _ = self
                    .bind_vertex_attributes()
                    .inspect_err(|e| debugln!("Error binding vertex attributes: {e}"));
            }
            self.gl.enable_vertex_attrib_array(self.a_pos_loc);
            self.gl.enable_vertex_attrib_array(self.a_tc_loc);
            self.gl.enable_vertex_attrib_array(self.a_srgba_loc);

            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&mesh.vertices),
                glow::STREAM_DRAW,
            );

            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&mesh.indices),
                glow::STATIC_DRAW,
            );

            self.gl.draw_elements(
                glow::TRIANGLES,
                mesh.indices.len() as _,
                glow::UNSIGNED_INT,
                0,
            );
        }

        crate::check_for_gl_error!(&self.gl, "draw");
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn set_clip_rect(&self, rect: egui::Rect) {
        // egui space to window space
        let left = self.viewport.left() + rect.min.x.round() as i32;
        let right = left + rect.width().round() as i32;
        let top = self.viewport.top() - rect.min.y.round() as i32;
        let bottom = top - rect.height().round() as i32;

        // clamp to viewport
        let left = left.clamp(self.viewport.left(), self.viewport.right());
        let right = right.clamp(self.viewport.left(), self.viewport.right());
        let top = top.clamp(self.viewport.bottom(), self.viewport.top());
        let bottom = bottom.clamp(self.viewport.bottom(), self.viewport.top());

        unsafe {
            self.gl.scissor(left, bottom, right - left, top - bottom);
        }
    }

    pub(super) fn cleanup(&mut self) {
        unsafe {
            self.gl.bind_texture(
                glow::TEXTURE_2D,
                NonZeroU32::new(self.last_texture.cast_unsigned()).map(NativeTexture),
            );
            self.gl.disable_vertex_attrib_array(self.a_pos_loc);
            self.gl.disable_vertex_attrib_array(self.a_tc_loc);
            self.gl.disable_vertex_attrib_array(self.a_srgba_loc);
            self.gl.bind_vertex_array(None);
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            self.gl.bind_buffer(glow::ARRAY_BUFFER, None);
            self.gl.use_program(None);

            self.gl.enable(glow::CULL_FACE);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.viewport(
                self.old_viewport[0],
                self.old_viewport[1],
                self.old_viewport[2],
                self.old_viewport[3],
            );
        }

        crate::check_for_gl_error!(&self.gl, "cleanup");
    }
}
