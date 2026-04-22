use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::{ffi::NulError, mem};

use bitflags::bitflags;
use euclid::{Rect, point2};
use xplm_sys::{self, XPLMSetWindowGravity};

use crate::geometry::{RectExt, ScreenPoint, ScreenRect, WindowRect};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum Cursor {
    /// X-Plane manages the cursor normally, plugin does not affect the cusrsor.
    #[default]
    Default = xplm_sys::xplm_CursorDefault,
    /// X-Plane hides the cursor.
    Hidden = xplm_sys::xplm_CursorHidden,
    /// X-Plane shows the cursor as the default arrow.
    Arrow = xplm_sys::xplm_CursorArrow,
    /// X-Plane shows the cursor but lets you select an OS cursor.
    Custom = xplm_sys::xplm_CursorCustom,
    /// X-Plane shows a small bi-directional knob-rotating cursor.
    RotateSmall = xplm_sys::xplm_CursorRotateSmall,
    /// X-Plane shows a small counter-clockwise knob-rotating cursor.
    RotateSmallLeft = xplm_sys::xplm_CursorRotateSmallLeft,
    /// X-Plane shows a small clockwise knob-rotating cursor.
    RotateSmallRight = xplm_sys::xplm_CursorRotateSmallRight,
    /// X-Plane shows a medium bi-directional knob-rotating cursor.
    RotateMedium = xplm_sys::xplm_CursorRotateMedium,
    /// X-Plane shows a medium counter-clockwise knob-rotating cursor.
    RotateMediumLeft = xplm_sys::xplm_CursorRotateMediumLeft,
    /// X-Plane shows a medium clockwise knob-rotating cursor.
    RotateMediumRight = xplm_sys::xplm_CursorRotateMediumRight,
    /// X-Plane shows a large bi-directional knob-rotating cursor.
    RotateLarge = xplm_sys::xplm_CursorRotateLarge,
    /// X-Plane shows a large counter-clockwise knob-rotating cursor.
    RotateLargeLeft = xplm_sys::xplm_CursorRotateLargeLeft,
    /// X-Plane shows a large clockwise knob-rotating cursor.
    RotateLargeRight = xplm_sys::xplm_CursorRotateLargeRight,
    /// X-Plane shows an up-and-down arrows cursor.
    UpDown = xplm_sys::xplm_CursorUpDown,
    /// X-Plane shows a down arrow
    Down = xplm_sys::xplm_CursorDown,
    /// X-Plane shows an up arrow cursor.
    Up = xplm_sys::xplm_CursorUp,
    /// X-Plane shows a left-right arrow cursor.
    LeftRight = xplm_sys::xplm_CursorLeftRight,
    /// X-Plane shows a left arrow cursor.
    Left = xplm_sys::xplm_CursorLeft,
    /// X-Plane shows a right arrow cursor.
    Right = xplm_sys::xplm_CursorRight,
    /// X-Plane shows a button-pushing cursor.
    Button = xplm_sys::xplm_CursorButton,
    /// X-Plane shows a handle-grabbing cursor.
    Handle = xplm_sys::xplm_CursorHandle,
    /// X-Plane shows a four-arrows cursor.
    FourArrows = xplm_sys::xplm_CursorFourArrows,
    /// X-Plane shows a cursor to drag a horizontal splitter bar.
    SplitterH = xplm_sys::xplm_CursorSplitterH,
    /// X-Plane shows a cursor to drag a vertical splitter bar.
    SplitterV = xplm_sys::xplm_CursorSplitterV,
    /// X-Plane shows an I-Beam cursor for text editing.
    Text = xplm_sys::xplm_CursorText,
}

impl From<egui::CursorIcon> for Cursor {
    fn from(icon: egui::CursorIcon) -> Self {
        use Cursor as C;
        #[allow(unreachable_patterns)]
        #[allow(clippy::match_same_arms)]
        match icon {
            egui::CursorIcon::Default => C::Default,
            egui::CursorIcon::None => C::Hidden,
            egui::CursorIcon::ContextMenu => C::Arrow,
            egui::CursorIcon::Help => C::Arrow,
            egui::CursorIcon::PointingHand => C::Button,
            egui::CursorIcon::Progress => C::Arrow,
            egui::CursorIcon::Wait => C::Arrow,
            egui::CursorIcon::Cell => C::Arrow,
            egui::CursorIcon::Crosshair => C::Default,
            egui::CursorIcon::Text => C::Text,
            egui::CursorIcon::VerticalText => C::Text,
            egui::CursorIcon::Alias => C::Arrow,
            egui::CursorIcon::Copy => C::Arrow,
            egui::CursorIcon::Move => C::FourArrows,
            egui::CursorIcon::NoDrop => C::Arrow,
            egui::CursorIcon::NotAllowed => C::Arrow,
            egui::CursorIcon::Grab => C::Handle,
            egui::CursorIcon::Grabbing => C::Handle,
            egui::CursorIcon::AllScroll => C::FourArrows,
            egui::CursorIcon::ResizeHorizontal => C::LeftRight,
            egui::CursorIcon::ResizeNeSw => C::UpDown,
            egui::CursorIcon::ResizeNwSe => C::UpDown,
            egui::CursorIcon::ResizeVertical => C::UpDown,
            egui::CursorIcon::ResizeEast => C::Right,
            egui::CursorIcon::ResizeSouthEast => C::Down,
            egui::CursorIcon::ResizeSouth => C::Down,
            egui::CursorIcon::ResizeSouthWest => C::Down,
            egui::CursorIcon::ResizeWest => C::Left,
            egui::CursorIcon::ResizeNorthWest => C::Up,
            egui::CursorIcon::ResizeNorth => C::Up,
            egui::CursorIcon::ResizeNorthEast => C::Up,
            egui::CursorIcon::ResizeColumn => C::LeftRight,
            egui::CursorIcon::ResizeRow => C::UpDown,
            egui::CursorIcon::ZoomIn => C::Arrow,
            egui::CursorIcon::ZoomOut => C::Arrow,
            _ => C::Default,
        }
    }
}

/// Trait for things that can define the behavior of a window
#[allow(unused_variables)]
pub trait WindowDelegate: 'static {
    /// Draws this window
    fn draw(&mut self, window: &Window);
    /// Handles a keyboard event
    ///
    /// The default implementation does nothing
    fn keyboard_event(&mut self, window: &Window, event: KeyEvent) {}
    /// Handles a mouse event
    ///
    /// Return false to consume the event or true to propagate it.
    ///
    /// The default implementation does nothing and allows the event to propagate.
    fn mouse_event(&mut self, window: &Window, event: MouseEvent) -> bool {
        true
    }
    /// Handles a right-click event
    ///
    /// Return false to consume the event or true to propagate it.
    ///
    /// The default implementation does nothing and allows the event to propagate.
    fn right_mouse_event(&mut self, window: &Window, event: MouseEvent) -> bool {
        true
    }
    /// Handles a scroll event
    ///
    /// Return false to consume the event or true to propagate it.
    ///
    /// The default implementation does nothing and allows the event to propagate.
    fn scroll_event(&mut self, window: &Window, event: ScrollEvent) -> bool {
        true
    }
    /// Tells X-Plane what cursor to draw over a section of the window
    ///
    /// The default implementation allows X-Plane to draw the default cursor.
    fn cursor(&mut self, window: &Window, position: ScreenPoint) -> Cursor {
        Cursor::Default
    }
}

/// A reference to a window
pub struct WindowRef {
    /// The window
    window: Box<Window>,
}

impl Deref for WindowRef {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        self.window.deref()
    }
}

/// Defines what layer the window should be positioned in.
/// The default is to create a floating window.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[allow(clippy::cast_possible_wrap)]
pub enum WindowLayer {
    FlightOverlay = xplm_sys::xplm_WindowLayerFlightOverlay as _,
    #[default]
    FloatingWindows = xplm_sys::xplm_WindowLayerFloatingWindows as _,
    Modal = xplm_sys::xplm_WindowLayerModal as _,
    GrowlNotifications = xplm_sys::xplm_WindowLayerGrowlNotifications as _,
}

/// Defines what decorations should be applied to the window.
/// The default is to use X-Plane's native rounded window title bar.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[allow(clippy::cast_possible_wrap)]
pub enum WindowDecorations {
    None = xplm_sys::xplm_WindowDecorationNone as _,
    #[default]
    RoundRectangle = xplm_sys::xplm_WindowDecorationRoundRectangle as _,
    SelfDecorated = xplm_sys::xplm_WindowDecorationSelfDecorated as _,
    SelfDecoratedResizable = xplm_sys::xplm_WindowDecorationSelfDecoratedResizable as _,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[allow(clippy::cast_possible_wrap)]
pub enum WindowPositioningMode {
    #[default]
    Free = xplm_sys::xplm_WindowPositionFree as _,
    CenterOnMonitor = xplm_sys::xplm_WindowCenterOnMonitor as _,
    FullScreenOnMonitor = xplm_sys::xplm_WindowFullScreenOnMonitor as _,
    FullScreenOnAllMonitors = xplm_sys::xplm_WindowFullScreenOnAllMonitors as _,
    PopOut = xplm_sys::xplm_WindowPopOut as _,
    Vr = xplm_sys::xplm_WindowVR as _,
}

/// A basic window that may appear on the screen
///
/// A window has a position and size, but no appearance. Plugins must draw in their draw callbacks
/// to make windows appear.
pub struct Window {
    /// The window ID
    id: xplm_sys::XPLMWindowID,
    /// The delegate
    delegate: Box<dyn WindowDelegate>,
    /// Decorations
    decorations: WindowDecorations,
}

impl Window {
    /// Creates a new window with the provided geometry and returns a reference to it
    ///
    /// The window is originally not visible.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<D: WindowDelegate>(geometry: impl Into<ScreenRect>, delegate: D) -> WindowRef {
        Self::new_custom(
            geometry,
            WindowLayer::FloatingWindows,
            WindowDecorations::None,
            delegate,
        )
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn new_custom<D: WindowDelegate>(
        geometry: impl Into<ScreenRect>,
        layer: WindowLayer,
        decorations: WindowDecorations,
        delegate: D,
    ) -> WindowRef {
        let geometry = geometry.into();

        let mut window_box = Box::new(Window {
            id: ptr::null_mut(),
            delegate: Box::new(delegate),
            decorations,
        });
        let window_ptr: *mut Window = &raw mut *window_box;

        let mut window_info = xplm_sys::XPLMCreateWindow_t {
            structSize: mem::size_of::<xplm_sys::XPLMCreateWindow_t>() as _,
            left: geometry.left(),
            top: geometry.top(),
            right: geometry.right(),
            bottom: geometry.bottom(),
            visible: 0,
            drawWindowFunc: Some(window_draw),
            handleMouseClickFunc: Some(window_mouse),
            handleKeyFunc: Some(window_key),
            handleCursorFunc: Some(window_cursor),
            handleMouseWheelFunc: Some(window_scroll),
            refcon: window_ptr.cast(),
            decorateAsFloatingWindow: decorations as _,
            layer: layer as _,
            handleRightClickFunc: Some(window_right_mouse),
        };

        let window_id = unsafe { xplm_sys::XPLMCreateWindowEx(&raw mut window_info) };
        window_box.id = window_id;

        WindowRef { window: window_box }
    }

    /// Use `.geometry()` for drawing
    /// Returns the geometry of this window, with x + 100'000 in the case of a pop-out window
    #[must_use]
    pub fn screen_geometry(&self) -> ScreenRect {
        unsafe {
            let mut left = 0;
            let mut top = 0;
            let mut right = 0;
            let mut bottom = 0;
            xplm_sys::XPLMGetWindowGeometry(
                self.id,
                &raw mut left,
                &raw mut top,
                &raw mut right,
                &raw mut bottom,
            );
            let rect = Rect::from_left_top_right_bottom(left, top, right, bottom);
            if self.decorations == WindowDecorations::RoundRectangle {
                // round rectangle adds 10px padding, remove.
                rect.inflate(10, 10)
            } else {
                rect
            }
        }
    }

    /// Returns the geometry of this window's opengl viewport
    #[must_use]
    pub fn geometry(&self) -> WindowRect {
        self.screen_geometry().to_window_space()
    }

    /// Sets the geometry of this window
    pub fn set_geometry(&self, geometry: impl Into<ScreenRect>) {
        let geometry = geometry.into();

        unsafe {
            xplm_sys::XPLMSetWindowGeometry(
                self.id,
                geometry.left(),
                geometry.top(),
                geometry.right(),
                geometry.bottom(),
            );
        }
    }

    /// Returns true if this window is visible
    #[must_use]
    pub fn visible(&self) -> bool {
        1 == unsafe { xplm_sys::XPLMGetWindowIsVisible(self.id) }
    }
    /// Sets the window as visible or invisible
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            xplm_sys::XPLMSetWindowIsVisible(self.id, visible.into());
        }
    }
    /// Sets the window title, which is shown when using the standard X-Plane decorations.
    pub fn set_title<S: AsRef<str>>(&self, title: S) -> Result<(), NulError> {
        let title = CString::new(title.as_ref())?;
        unsafe { xplm_sys::XPLMSetWindowTitle(self.id, title.as_ptr()) };
        Ok(())
    }
    /// Sets the window positioning mode.
    pub fn set_positioning_mode(&self, mode: WindowPositioningMode, monitor_index: i32) {
        unsafe { xplm_sys::XPLMSetWindowPositioningMode(self.id, mode as _, monitor_index) };
    }
    /// Forces the window to take keyboard focus.
    pub fn take_keyboard_focus(&self) {
        unsafe { xplm_sys::XPLMTakeKeyboardFocus(self.id) };
    }
    /// Returns whether the window current has keyboard focus.
    #[must_use]
    pub fn has_keyboard_focus(&self) -> bool {
        unsafe { xplm_sys::XPLMHasKeyboardFocus(self.id) != 0 }
    }
    /// Brings a floating window to the top of the window stack.
    pub fn bring_to_front(&self) {
        unsafe { xplm_sys::XPLMBringWindowToFront(self.id) };
    }
    /// Returns whether the window is current at the top of the window stack.
    #[must_use]
    pub fn is_in_front(&self) -> bool {
        unsafe { xplm_sys::XPLMIsWindowInFront(self.id) != 0 }
    }
    pub fn set_resizing_limits(
        &self,
        min_w: impl Into<Option<i32>>,
        min_h: impl Into<Option<i32>>,
        max_w: impl Into<Option<i32>>,
        max_h: impl Into<Option<i32>>,
    ) {
        let min_w = min_w.into().unwrap_or(0).clamp(0, i32::from(i16::MAX));
        let max_w = max_w
            .into()
            .unwrap_or(i32::MAX)
            .clamp(0, i32::from(i16::MAX));
        let min_h = min_h.into().unwrap_or(0).clamp(0, i32::from(i16::MAX));
        let max_h = max_h
            .into()
            .unwrap_or(i32::MAX)
            .clamp(0, i32::from(i16::MAX));

        unsafe {
            xplm_sys::XPLMSetWindowResizingLimits(self.id, min_w, min_h, max_w, max_h);
        }
    }
    /// Returns the underlying `XPLMWindowID` from XPLM.
    ///
    /// # Safety
    /// The window will be destroyed when this `Window` struct is dropped, so the caller must
    /// take care not to use to the returned `XPLMWindowID` pointer afterwards.
    #[must_use]
    pub unsafe fn id(&self) -> xplm_sys::XPLMWindowID {
        self.id
    }

    /// A window's "gravity" controls how the window shifts as the whole X-Plane
    /// window resizes. A gravity of 1 means the window maintains its positioning
    /// relative to the right or top edges, 0 the left/bottom, and 0.5 keeps it
    /// centered.
    ///
    /// Default gravity is (0, 1, 0, 1), meaning your window will maintain its
    /// position relative to the top left and will not change size as its
    /// containing window grows.
    ///
    /// If you wanted, say, a window that sticks to the top of the screen (with a
    /// constant height), but which grows to take the full width of the window, you
    /// would pass (0, 1, 1, 1). Because your left and right edges would maintain
    /// their positioning relative to their respective edges of the screen, the
    /// whole width of your window would change with the X-Plane window.
    ///
    /// Only applies to modern windows. (Windows created using the deprecated
    /// XPLMCreateWindow(), or windows compiled against a pre-XPLM300 version of
    /// the SDK will simply get the default gravity.)
    pub fn set_gravity(&self, left: f32, top: f32, right: f32, bottom: f32) {
        unsafe {
            XPLMSetWindowGravity(self.id(), left, top, right, bottom);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            xplm_sys::XPLMDestroyWindow(self.id);
        }
    }
}

/// Callback in which windows are drawn
extern "C" fn window_draw(_window: xplm_sys::XPLMWindowID, refcon: *mut c_void) {
    unsafe {
        let window = refcon.cast::<Window>();
        (*window).delegate.draw(&*window);
    }
}

/// Keyboard callback
extern "C" fn window_key(
    _window: xplm_sys::XPLMWindowID,
    key: c_char,
    flags: xplm_sys::XPLMKeyFlags,
    virtual_key: c_char,
    refcon: *mut c_void,
    losing_focus: c_int,
) {
    unsafe {
        if losing_focus == 0 {
            let window = refcon.cast::<Window>();
            (*window)
                .delegate
                .keyboard_event(&*window, KeyEvent::from_xplm(key, flags, virtual_key));
        }
    }
}

/// Mouse callback
extern "C" fn window_mouse(
    _window: xplm_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    status: xplm_sys::XPLMMouseStatus,
    refcon: *mut c_void,
) -> c_int {
    let window = refcon.cast::<Window>();
    if let Some(action) = MouseAction::from_xplm(status) {
        let position = point2(x, y);
        let event = MouseEvent::new(position, action);
        unsafe { c_int::from(!(*window).delegate.mouse_event(&*window, event)) }
    } else {
        // Propagate
        0
    }
}

/// Right-mouse callback
extern "C" fn window_right_mouse(
    _window: xplm_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    status: xplm_sys::XPLMMouseStatus,
    refcon: *mut c_void,
) -> c_int {
    let window = refcon.cast::<Window>();
    if let Some(action) = MouseAction::from_xplm(status) {
        let position = point2(x, y);
        let event = MouseEvent::new(position, action);
        unsafe { c_int::from(!(*window).delegate.right_mouse_event(&*window, event)) }
    } else {
        // Propagate
        0
    }
}

/// Cursor callback
extern "C" fn window_cursor(
    _window: xplm_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    refcon: *mut c_void,
) -> xplm_sys::XPLMCursorStatus {
    let window = refcon.cast::<Window>();
    let cursor = unsafe { (*window).delegate.cursor(&*window, point2(x, y)) };
    cursor as i32
}

/// Scroll callback
extern "C" fn window_scroll(
    _window: xplm_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    wheel: c_int,
    clicks: c_int,
    refcon: *mut c_void,
) -> c_int {
    let window = refcon.cast::<Window>();

    let position = point2(x, y);
    let (dx, dy) = if wheel == 1 {
        // Horizontal
        (clicks, 0)
    } else {
        // Vertical
        (0, clicks)
    };
    let event = ScrollEvent::new(position, dx, dy);

    let propagate = unsafe { (*window).delegate.scroll_event(&*window, event) };
    i32::from(!propagate)
}

/// An event associated with a key press
#[derive(Debug)]
pub struct KeyEvent {
    /// Text
    pub basic_char: Option<char>,

    /// Key
    pub key: Option<egui::Key>,

    /// Flags
    pub flags: KeyFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyFlags: u32 {
        const SHIFT      = xplm_sys::xplm_ShiftFlag;
        const OPTION_ALT = xplm_sys::xplm_OptionAltFlag;
        const CONTROL    = xplm_sys::xplm_ControlFlag;
        const DOWN       = xplm_sys::xplm_DownFlag;
        const UP         = xplm_sys::xplm_UpFlag;
    }
}

impl KeyFlags {
    #[must_use]
    pub fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    #[must_use]
    pub fn option_alt(self) -> bool {
        self.contains(Self::OPTION_ALT)
    }

    #[must_use]
    pub fn control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    #[must_use]
    pub fn down(self) -> bool {
        self.contains(Self::DOWN)
    }

    #[must_use]
    pub fn up(self) -> bool {
        self.contains(Self::UP)
    }
}

impl KeyEvent {
    /// Creates an egui key event from XPLM key information
    fn from_xplm(key: c_char, flags: xplm_sys::XPLMKeyFlags, vkey: c_char) -> Self {
        let key = key.cast_unsigned();
        let vkey = vkey.cast_unsigned();
        let flags = KeyFlags::from_bits_truncate(flags.cast_unsigned());

        let Some(desc) = (unsafe {
            let cstr = xplm_sys::XPLMGetVirtualKeyDescription(vkey.cast_signed());
            std::ffi::CStr::from_ptr(cstr).to_str().ok()
        }) else {
            return KeyEvent {
                basic_char: None,
                key: None,
                flags,
            };
        };
        let desc = desc.replace("Numpad-", "Numpad");

        let ascii = if key.is_ascii() {
            Some(key as char)
        } else {
            None
        };

        let egui_key = egui::Key::from_name(&desc);
        let ascii_is_wrong = matches!(vkey, 13 | 9 | 8 | 28..=31 | 46 | 0x08..=0x0D | 0x1B | 0x20..=0x2F | 0x70..=0x87);

        if vkey == 188 {
            KeyEvent {
                basic_char: None,
                key: Some(egui::Key::Enter),
                flags,
            }
        } else if !ascii_is_wrong && ascii.is_some_and(|c| c.is_ascii_graphic()) {
            KeyEvent {
                basic_char: ascii,
                key: egui_key,
                flags,
            }
        } else if egui_key.is_some() {
            KeyEvent {
                basic_char: None,
                key: egui_key,
                flags,
            }
        } else {
            KeyEvent {
                basic_char: None,
                key: None,
                flags,
            }
        }
    }
}

/// Actions that the mouse/cursor can perform
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MouseAction {
    /// The user pressed the mouse button down
    Down,
    /// The user moved the mouse with the mouse button down
    Drag,
    /// The user released the mouse button
    Up,
}

impl MouseAction {
    fn from_xplm(status: xplm_sys::XPLMMouseStatus) -> Option<MouseAction> {
        if status == xplm_sys::xplm_MouseDown.cast_signed() {
            Some(MouseAction::Down)
        } else if status == xplm_sys::xplm_MouseDrag.cast_signed() {
            Some(MouseAction::Drag)
        } else if status == xplm_sys::xplm_MouseUp.cast_signed() {
            Some(MouseAction::Up)
        } else {
            None
        }
    }
}

/// A mouse event
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    /// The position of the mouse, in global window coordinates
    position: ScreenPoint,
    /// The action of the mouse
    action: MouseAction,
}

impl MouseEvent {
    /// Creates a new event
    fn new(position: ScreenPoint, action: MouseAction) -> Self {
        MouseEvent { position, action }
    }
    /// Returns the position of the mouse, in global coordinates relative to the X-Plane
    /// main window
    #[must_use]
    pub fn position(&self) -> ScreenPoint {
        self.position
    }
    /// Returns the action that the user performed with the mouse
    #[must_use]
    pub fn action(&self) -> MouseAction {
        self.action
    }
}

/// A scroll event
#[derive(Debug, Clone)]
pub struct ScrollEvent {
    /// The position of the mouse, in global window coordinates
    position: ScreenPoint,
    /// The amount of scroll in the X direction
    scroll_x: i32,
    /// The amount of scroll in the Y direction
    scroll_y: i32,
}

impl ScrollEvent {
    /// Creates a new event
    fn new(position: ScreenPoint, scroll_x: i32, scroll_y: i32) -> Self {
        ScrollEvent {
            position,
            scroll_x,
            scroll_y,
        }
    }
    /// Returns the position of the mouse, in global coordinates relative to the X-Plane
    /// main window
    #[must_use]
    pub fn position(&self) -> ScreenPoint {
        self.position
    }
    /// Returns the amount of scroll in the X direction
    #[must_use]
    pub fn scroll_x(&self) -> i32 {
        self.scroll_x
    }
    /// Returns the amount of scroll in the Y direction
    #[must_use]
    pub fn scroll_y(&self) -> i32 {
        self.scroll_y
    }
}
