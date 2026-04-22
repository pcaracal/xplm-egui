use euclid::{Rect, point2, size2, vec2};

use crate::geometry::{ScreenPoint, ScreenRect, ScreenSize};

#[must_use]
pub fn get_screen_size() -> ScreenSize {
    let mut width = 0;
    let mut height = 0;
    unsafe { xplm_sys::XPLMGetScreenSize(&raw mut width, &raw mut height) };
    vec2(width, height).into()
}

#[must_use]
pub fn get_screen_bounds_global() -> ScreenRect {
    let mut left = 0;
    let mut top = 0;
    let mut right = 0;
    let mut bottom = 0;
    unsafe {
        xplm_sys::XPLMGetScreenBoundsGlobal(
            &raw mut left,
            &raw mut top,
            &raw mut right,
            &raw mut bottom,
        );
    }
    Rect::new(point2(left, bottom), size2(right - left, top - bottom))
}

#[must_use]
pub fn get_mouse_location_global() -> ScreenPoint {
    let mut x = 0;
    let mut y = 0;
    unsafe { xplm_sys::XPLMGetMouseLocationGlobal(&raw mut x, &raw mut y) };
    point2(x, y)
}
