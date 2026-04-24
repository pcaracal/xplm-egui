mod context;
mod painter;

use std::ops::Deref;

use crate::{
    egui_window::context::EguiWindowContext,
    geometry::ScreenRect,
    window::{Window, WindowDecorations, WindowLayer, WindowRef},
};

#[allow(unused)]
pub trait App: 'static {
    /// Draw your ui here
    fn ui(&mut self, ui: &mut egui::Ui, window: &crate::window::Window);
}

/// A window using egui ui
pub struct EguiWindow {
    window: WindowRef,
}

impl EguiWindow {
    pub fn new(app: impl App, geometry: ScreenRect) -> anyhow::Result<Self> {
        let window = Window::new_custom(
            geometry,
            WindowLayer::FloatingWindows,
            WindowDecorations::RoundRectangle,
            EguiWindowContext::new(Box::new(app))?,
        );

        Ok(Self { window })
    }
}

impl Deref for EguiWindow {
    type Target = WindowRef;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
