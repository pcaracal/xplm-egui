// https://github.com/emilk/egui/blob/main/crates/egui_glow/src/lib.rs
// https://github.com/emilk/egui/blob/main/crates/egui_glow/src/misc_util.rs

use glow::HasContext;

pub(super) fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> anyhow::Result<glow::Shader> {
    unsafe {
        let shader = gl
            .create_shader(shader_type)
            .map_err(|e| anyhow!("gl error create_shader: {e}"))?;

        gl.shader_source(shader, source);

        gl.compile_shader(shader);

        if gl.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            bail!(
                "gl error compile_shader: {}",
                gl.get_shader_info_log(shader)
            )
        }
    }
}

pub(super) fn link_program<'a, T: IntoIterator<Item = &'a glow::Shader>>(
    gl: &glow::Context,
    shaders: T,
) -> anyhow::Result<glow::Program> {
    unsafe {
        let program = gl
            .create_program()
            .map_err(|e| anyhow!("gl error create_program: {e}"))?;

        for shader in shaders {
            gl.attach_shader(program, *shader);
        }

        gl.link_program(program);

        if gl.get_program_link_status(program) {
            Ok(program)
        } else {
            bail!(
                "gl error link_program: {}",
                gl.get_program_info_log(program)
            )
        }
    }
}

/// Check for OpenGL error and report it using `log::error`.
///
/// Only active in debug builds!
///
/// ``` no_run
/// # let glow_context = todo!();
/// use egui_glow::check_for_gl_error;
/// check_for_gl_error!(glow_context);
/// check_for_gl_error!(glow_context, "during painting");
/// ```
#[macro_export]
macro_rules! check_for_gl_error {
    ($gl: expr) => {{
        if cfg!(debug_assertions) || option_env!("CHECK_GL_ERROR_IN_RELEASE").is_some() {
            $crate::egui_window::painter::util::check_for_gl_error_impl($gl, file!(), line!(), "")
        }
    }};
    ($gl: expr, $context: literal) => {{
        if cfg!(debug_assertions) || option_env!("CHECK_GL_ERROR_IN_RELEASE").is_some() {
            $crate::egui_window::painter::util::check_for_gl_error_impl(
                $gl,
                file!(),
                line!(),
                $context,
            )
        }
    }};
}

/// Check for OpenGL error and report it using `log::error`.
///
/// WARNING: slow! Only use during setup!
///
/// ``` no_run
/// # let glow_context = todo!();
/// use egui_glow::check_for_gl_error_even_in_release;
/// check_for_gl_error_even_in_release!(glow_context);
/// check_for_gl_error_even_in_release!(glow_context, "during painting");
/// ```
#[macro_export]
macro_rules! check_for_gl_error_even_in_release {
    ($gl: expr) => {{ $crate::egui_window::painter::util::check_for_gl_error_impl($gl, file!(), line!(), "") }};
    ($gl: expr, $context: literal) => {{ $crate::egui_window::painter::util::check_for_gl_error_impl($gl, file!(), line!(), $context) }};
}

#[doc(hidden)]
pub fn check_for_gl_error_impl(gl: &glow::Context, file: &str, line: u32, context: &str) {
    use glow::HasContext as _;
    #[expect(unsafe_code)]
    let error_code = unsafe { gl.get_error() };
    if error_code != glow::NO_ERROR {
        let error_str = match error_code {
            glow::INVALID_ENUM => "GL_INVALID_ENUM",
            glow::INVALID_VALUE => "GL_INVALID_VALUE",
            glow::INVALID_OPERATION => "GL_INVALID_OPERATION",
            glow::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            glow::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            glow::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            glow::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            glow::CONTEXT_LOST => "GL_CONTEXT_LOST",
            0x8031 => "GL_TABLE_TOO_LARGE1",
            0x9242 => "CONTEXT_LOST_WEBGL",
            _ => "<unknown>",
        };

        if context.is_empty() {
            crate::debugln!(
                "GL error, at {file}:{line}: {error_str} (0x{error_code:X}). Please file a bug at https://github.com/emilk/egui/issues"
            );
        } else {
            crate::debugln!(
                "GL error, at {file}:{line} ({context}): {error_str} (0x{error_code:X}). Please file a bug at https://github.com/emilk/egui/issues"
            );
        }
    }
}
