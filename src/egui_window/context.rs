use std::time::Instant;

use crate::{
    display::get_mouse_location_global,
    draw::GraphicsState,
    egui_window::{App, painter::Painter},
    geometry::{PointExt, SizeExt, WindowPoint, WindowRect},
    window::{MouseAction, MouseEvent, WindowDelegate},
};

pub(super) struct EguiWindowContext {
    app: Box<dyn App>,

    start: Instant,
    /// Viewport in the current draw
    viewport: WindowRect,
    painter: Painter,
    egui_ctx: egui::Context,
    egui_input: egui::RawInput,

    cursor_pos: WindowPoint,
    cursor_icon: egui::CursorIcon,
}

impl EguiWindowContext {
    pub(super) fn new(app: Box<dyn App>) -> anyhow::Result<Self> {
        Ok(Self {
            app,
            start: Instant::now(),
            viewport: WindowRect::default(),
            painter: Painter::new()?,
            egui_ctx: egui::Context::default(),
            egui_input: egui::RawInput::default(),
            cursor_pos: WindowPoint::default(),
            cursor_icon: egui::CursorIcon::Default,
        })
    }
}

impl WindowDelegate for EguiWindowContext {
    fn draw(&mut self, window: &crate::window::Window) {
        GraphicsState::new()
            .alpha_testing(true)
            .alpha_blending(true)
            .textures(1)
            .apply();
        self.viewport = window.geometry();
        self.painter.set_viewport(self.viewport);
        self.update_mouse();

        if self.egui_ctx.egui_wants_keyboard_input() {
            window.take_keyboard_focus();
        }

        if self.egui_ctx.egui_wants_pointer_input() {
            window.bring_to_front();
        }

        self.egui_input.system_theme = Some(egui::Theme::Dark);
        self.egui_input.max_texture_side = Some(self.painter.max_texture_side());
        self.egui_input.time = Some(self.start.elapsed().as_secs_f64());
        self.egui_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::default(),
            self.viewport.size.to_egui(),
        ));
        self.egui_input
            .viewports
            .get_mut(&self.egui_input.viewport_id)
            .unwrap()
            .native_pixels_per_point = Some(1.0);

        let platform_output = self
            .painter
            .run_ui(&self.egui_ctx, &mut self.egui_input, |ui| {
                self.app.ui(ui, window);
            });

        self.cursor_icon = platform_output.cursor_icon;
    }

    fn keyboard_event(&mut self, _: &crate::window::Window, event: crate::window::KeyEvent) {
        let modifiers = egui::Modifiers {
            ctrl: event.flags.control(),
            alt: event.flags.option_alt(),
            shift: event.flags.shift(),
            command: event.flags.control(),
            ..Default::default()
        };
        self.egui_input.modifiers = modifiers;

        if event.flags.down()
            && let Some(c) = event.basic_char
        {
            self.egui_input
                .events
                .push(egui::Event::Text(c.to_string()));
        }

        if let Some(key) = event.key {
            self.egui_input.events.push(egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: event.flags.down(),
                repeat: false,
                modifiers,
            });
        }
    }

    fn mouse_event(&mut self, _: &crate::window::Window, event: crate::window::MouseEvent) -> bool {
        self.handle_mouse_event(&event, true)
    }

    fn right_mouse_event(
        &mut self,
        _: &crate::window::Window,
        event: crate::window::MouseEvent,
    ) -> bool {
        self.handle_mouse_event(&event, false)
    }

    fn scroll_event(
        &mut self,
        _: &crate::window::Window,
        event: crate::window::ScrollEvent,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        let delta = egui::vec2(event.scroll_x() as f32, event.scroll_y() as f32);
        self.egui_input.events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta,
            phase: egui::TouchPhase::Move,
            modifiers: self.egui_input.modifiers,
        });

        !self.egui_ctx.egui_wants_pointer_input()
    }

    fn cursor(
        &mut self,
        _: &crate::window::Window,
        _: crate::geometry::ScreenPoint,
    ) -> crate::window::Cursor {
        self.egui_input.events.push(egui::Event::PointerMoved(
            self.cursor_pos.to_egui(self.viewport),
        ));
        crate::window::Cursor::Default
    }
}

impl EguiWindowContext {
    fn update_mouse(&mut self) {
        let pos = get_mouse_location_global().to_window_space();
        self.cursor_pos = pos;
        if !self.viewport.contains(pos) {
            self.egui_input.events.push(egui::Event::PointerGone);
        }
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, left: bool) -> bool {
        let button = if left {
            egui::PointerButton::Primary
        } else {
            egui::PointerButton::Secondary
        };

        if event.action() != MouseAction::Drag {
            self.egui_input.events.push(egui::Event::PointerButton {
                pos: self.cursor_pos.to_egui(self.viewport),
                button,
                pressed: event.action() == MouseAction::Down,
                modifiers: self.egui_input.modifiers,
            });
        }

        !self.egui_ctx.egui_wants_pointer_input()
    }
}

impl Drop for EguiWindowContext {
    fn drop(&mut self) {
        self.painter.destroy();
    }
}
