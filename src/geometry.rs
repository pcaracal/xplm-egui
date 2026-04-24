use euclid::{Point2D, Rect, Size2D, point2, size2, vec2};

use crate::display::get_screen_bounds_global;

/// X-Plane screen space in pixels
/// Origin is the bottom left corner of the main X-Plane OpenGL window at (0, 0)
///
/// Shouldn't be used when working with windows, since pop-out windows special
pub struct XPScreenSpace;

/// X-Plane window space in pixels
/// Origin is the bottom left corner of the window's drawable area at (0, 0)
pub struct XPWindowSpace;

pub type ScreenRect = Rect<i32, XPScreenSpace>;
/// Size is the same for screen, window and egui coordinate spaces. This is just for convenience
pub type ScreenSize = Size2D<i32, XPScreenSpace>;
pub type ScreenPoint = Point2D<i32, XPScreenSpace>;

pub type WindowRect = Rect<i32, XPWindowSpace>;
pub type WindowPoint = Point2D<i32, XPWindowSpace>;

pub trait RectExt<T, U> {
    fn from_left_top_right_bottom(left: T, top: T, right: T, bottom: T) -> Rect<T, U>;
    fn left(&self) -> T;
    fn top(&self) -> T;
    fn right(&self) -> T;
    fn bottom(&self) -> T;
}

pub trait RectConv<T> {
    fn to_window_space(&self) -> Rect<T, XPWindowSpace>;
}

impl<U> RectExt<i32, U> for Rect<i32, U> {
    fn from_left_top_right_bottom(left: i32, top: i32, right: i32, bottom: i32) -> Rect<i32, U> {
        Rect::new(point2(left, bottom), size2(right - left, top - bottom))
    }
    fn left(&self) -> i32 {
        self.min_x()
    }
    fn top(&self) -> i32 {
        self.max_y()
    }
    fn right(&self) -> i32 {
        self.max_x()
    }
    fn bottom(&self) -> i32 {
        self.min_y()
    }
}

impl RectConv<i32> for Rect<i32, XPScreenSpace> {
    fn to_window_space(&self) -> Rect<i32, XPWindowSpace> {
        if self.left() >= 100_000 {
            self.translate(vec2(-100_000, 0)).cast_unit()
        } else {
            self.cast_unit()
                .translate(-get_screen_bounds_global().origin.to_vector().cast_unit())
        }
    }
}

pub trait PointExt<T, U> {
    fn to_window_space(&self) -> WindowPoint;
    fn to_egui(&self, rect: Rect<T, U>) -> egui::Pos2;
}

impl PointExt<i32, XPScreenSpace> for ScreenPoint {
    fn to_window_space(&self) -> WindowPoint {
        if self.x >= 100_000 {
            (*self - vec2(100_000, 0)).cast_unit()
        } else {
            self.cast_unit() - get_screen_bounds_global().origin.to_vector().cast_unit()
        }
    }

    fn to_egui(&self, rect: ScreenRect) -> egui::Pos2 {
        self.to_window_space().to_egui(rect.to_window_space())
    }
}

impl PointExt<i32, XPWindowSpace> for WindowPoint {
    fn to_window_space(&self) -> WindowPoint {
        *self
    }
    #[allow(clippy::cast_precision_loss)]
    fn to_egui(&self, rect: WindowRect) -> egui::Pos2 {
        let x = self.x - rect.left();
        let y = self.y - rect.bottom();
        let y = rect.height() - y;

        egui::pos2(x as f32, y as f32)
    }
}

pub trait SizeExt<T> {
    fn to_egui(&self) -> egui::Vec2;
}

impl<U> SizeExt<U> for Size2D<i32, U> {
    fn to_egui(&self) -> egui::Vec2 {
        #[allow(clippy::cast_precision_loss)]
        egui::vec2(self.width as f32, self.height as f32)
    }
}
