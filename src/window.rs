use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::{ffi::NulError, mem};

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
        let window = refcon.cast::<Window>();
        if losing_focus == 0 {
            match KeyEvent::from_xplm(key, flags, virtual_key) {
                Ok(event) => (*window).delegate.keyboard_event(&*window, event),
                Err(e) => super::debugln!("Invalid key event received: {:?}", e),
            }
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

/// Key actions
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// The key was pressed down
    Press,
    /// The key was released
    Release,
}

/// Keys that may be pressed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Back,
    Tab,
    Clear,
    Return,
    Escape,
    Space,
    Prior,
    Next,
    End,
    Home,
    Left,
    Up,
    Right,
    Down,
    Select,
    Print,
    Execute,
    Snapshot,
    Insert,
    Delete,
    Help,
    /// The 0 key at the top of a keyboard
    Key0,
    /// The 1 key at the top of a keyboard
    Key1,
    /// The 2 key at the top of a keyboard
    Key2,
    /// The 3 key at the top of a keyboard
    Key3,
    /// The 4 key at the top of a keyboard
    Key4,
    /// The 5 key at the top of a keyboard
    Key5,
    /// The 6 key at the top of a keyboard
    Key6,
    /// The 7 key at the top of a keyboard
    Key7,
    /// The 8 key at the top of a keyboard
    Key8,
    /// The 9 key at the top of a keyboard
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// The 0 key on the numerical keypad
    Numpad0,
    /// The 1 key on the numerical keypad
    Numpad1,
    /// The 2 key on the numerical keypad
    Numpad2,
    /// The 3 key on the numerical keypad
    Numpad3,
    /// The 4 key on the numerical keypad
    Numpad4,
    /// The 5 key on the numerical keypad
    Numpad5,
    /// The 6 key on the numerical keypad
    Numpad6,
    /// The 7 key on the numerical keypad
    Numpad7,
    /// The 8 key on the numerical keypad
    Numpad8,
    /// The 9 key on the numerical keypad
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Equal,
    Minus,
    ClosingBrace,
    OpeningBrace,
    Quote,
    Semicolon,
    Backslash,
    Comma,
    Slash,
    Period,
    Backquote,
    /// Enter, also known as return in Mac OS
    Enter,
    NumpadEnter,
    NumpadEqual,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dbg = format!("{self:?}");
        write!(f, "{}", dbg.strip_prefix("Key::").unwrap_or(&dbg))
    }
}

#[allow(clippy::cast_sign_loss, clippy::too_many_lines)]
impl Key {
    /// Converts an XPLM virtual key code into a Key
    fn from_xplm(virtual_key: c_char) -> Option<Self> {
        match virtual_key as u32 {
            xplm_sys::XPLM_VK_BACK => Some(Key::Back),
            xplm_sys::XPLM_VK_TAB => Some(Key::Tab),
            xplm_sys::XPLM_VK_CLEAR => Some(Key::Clear),
            xplm_sys::XPLM_VK_RETURN => Some(Key::Return),
            xplm_sys::XPLM_VK_ESCAPE => Some(Key::Escape),
            xplm_sys::XPLM_VK_SPACE => Some(Key::Space),
            xplm_sys::XPLM_VK_PRIOR => Some(Key::Prior),
            xplm_sys::XPLM_VK_NEXT => Some(Key::Next),
            xplm_sys::XPLM_VK_END => Some(Key::End),
            xplm_sys::XPLM_VK_HOME => Some(Key::Home),
            xplm_sys::XPLM_VK_LEFT => Some(Key::Left),
            xplm_sys::XPLM_VK_UP => Some(Key::Up),
            xplm_sys::XPLM_VK_RIGHT => Some(Key::Right),
            xplm_sys::XPLM_VK_DOWN => Some(Key::Down),
            xplm_sys::XPLM_VK_SELECT => Some(Key::Select),
            xplm_sys::XPLM_VK_PRINT => Some(Key::Print),
            xplm_sys::XPLM_VK_EXECUTE => Some(Key::Execute),
            xplm_sys::XPLM_VK_SNAPSHOT => Some(Key::Snapshot),
            xplm_sys::XPLM_VK_INSERT => Some(Key::Insert),
            xplm_sys::XPLM_VK_DELETE => Some(Key::Delete),
            xplm_sys::XPLM_VK_HELP => Some(Key::Help),
            xplm_sys::XPLM_VK_0 => Some(Key::Key0),
            xplm_sys::XPLM_VK_1 => Some(Key::Key1),
            xplm_sys::XPLM_VK_2 => Some(Key::Key2),
            xplm_sys::XPLM_VK_3 => Some(Key::Key3),
            xplm_sys::XPLM_VK_4 => Some(Key::Key4),
            xplm_sys::XPLM_VK_5 => Some(Key::Key5),
            xplm_sys::XPLM_VK_6 => Some(Key::Key6),
            xplm_sys::XPLM_VK_7 => Some(Key::Key7),
            xplm_sys::XPLM_VK_8 => Some(Key::Key8),
            xplm_sys::XPLM_VK_9 => Some(Key::Key9),
            xplm_sys::XPLM_VK_A => Some(Key::A),
            xplm_sys::XPLM_VK_B => Some(Key::B),
            xplm_sys::XPLM_VK_C => Some(Key::C),
            xplm_sys::XPLM_VK_D => Some(Key::D),
            xplm_sys::XPLM_VK_E => Some(Key::E),
            xplm_sys::XPLM_VK_F => Some(Key::F),
            xplm_sys::XPLM_VK_G => Some(Key::G),
            xplm_sys::XPLM_VK_H => Some(Key::H),
            xplm_sys::XPLM_VK_I => Some(Key::I),
            xplm_sys::XPLM_VK_J => Some(Key::J),
            xplm_sys::XPLM_VK_K => Some(Key::K),
            xplm_sys::XPLM_VK_L => Some(Key::L),
            xplm_sys::XPLM_VK_M => Some(Key::M),
            xplm_sys::XPLM_VK_N => Some(Key::N),
            xplm_sys::XPLM_VK_O => Some(Key::O),
            xplm_sys::XPLM_VK_P => Some(Key::P),
            xplm_sys::XPLM_VK_Q => Some(Key::Q),
            xplm_sys::XPLM_VK_R => Some(Key::R),
            xplm_sys::XPLM_VK_S => Some(Key::S),
            xplm_sys::XPLM_VK_T => Some(Key::T),
            xplm_sys::XPLM_VK_U => Some(Key::U),
            xplm_sys::XPLM_VK_V => Some(Key::V),
            xplm_sys::XPLM_VK_W => Some(Key::W),
            xplm_sys::XPLM_VK_X => Some(Key::X),
            xplm_sys::XPLM_VK_Y => Some(Key::Y),
            xplm_sys::XPLM_VK_Z => Some(Key::Z),
            xplm_sys::XPLM_VK_NUMPAD0 => Some(Key::Numpad0),
            xplm_sys::XPLM_VK_NUMPAD1 => Some(Key::Numpad1),
            xplm_sys::XPLM_VK_NUMPAD2 => Some(Key::Numpad2),
            xplm_sys::XPLM_VK_NUMPAD3 => Some(Key::Numpad3),
            xplm_sys::XPLM_VK_NUMPAD4 => Some(Key::Numpad4),
            xplm_sys::XPLM_VK_NUMPAD5 => Some(Key::Numpad5),
            xplm_sys::XPLM_VK_NUMPAD6 => Some(Key::Numpad6),
            xplm_sys::XPLM_VK_NUMPAD7 => Some(Key::Numpad7),
            xplm_sys::XPLM_VK_NUMPAD8 => Some(Key::Numpad8),
            xplm_sys::XPLM_VK_NUMPAD9 => Some(Key::Numpad9),
            xplm_sys::XPLM_VK_MULTIPLY => Some(Key::Multiply),
            xplm_sys::XPLM_VK_ADD => Some(Key::Add),
            xplm_sys::XPLM_VK_SEPARATOR => Some(Key::Separator),
            xplm_sys::XPLM_VK_SUBTRACT => Some(Key::Subtract),
            xplm_sys::XPLM_VK_DECIMAL => Some(Key::Decimal),
            xplm_sys::XPLM_VK_DIVIDE => Some(Key::Divide),
            xplm_sys::XPLM_VK_F1 => Some(Key::F1),
            xplm_sys::XPLM_VK_F2 => Some(Key::F2),
            xplm_sys::XPLM_VK_F3 => Some(Key::F3),
            xplm_sys::XPLM_VK_F4 => Some(Key::F4),
            xplm_sys::XPLM_VK_F5 => Some(Key::F5),
            xplm_sys::XPLM_VK_F6 => Some(Key::F6),
            xplm_sys::XPLM_VK_F7 => Some(Key::F7),
            xplm_sys::XPLM_VK_F8 => Some(Key::F8),
            xplm_sys::XPLM_VK_F9 => Some(Key::F9),
            xplm_sys::XPLM_VK_F10 => Some(Key::F10),
            xplm_sys::XPLM_VK_F11 => Some(Key::F11),
            xplm_sys::XPLM_VK_F12 => Some(Key::F12),
            xplm_sys::XPLM_VK_F13 => Some(Key::F13),
            xplm_sys::XPLM_VK_F14 => Some(Key::F14),
            xplm_sys::XPLM_VK_F15 => Some(Key::F15),
            xplm_sys::XPLM_VK_F16 => Some(Key::F16),
            xplm_sys::XPLM_VK_F17 => Some(Key::F17),
            xplm_sys::XPLM_VK_F18 => Some(Key::F18),
            xplm_sys::XPLM_VK_F19 => Some(Key::F19),
            xplm_sys::XPLM_VK_F20 => Some(Key::F20),
            xplm_sys::XPLM_VK_F21 => Some(Key::F21),
            xplm_sys::XPLM_VK_F22 => Some(Key::F22),
            xplm_sys::XPLM_VK_F23 => Some(Key::F23),
            xplm_sys::XPLM_VK_F24 => Some(Key::F24),
            xplm_sys::XPLM_VK_EQUAL => Some(Key::Equal),
            xplm_sys::XPLM_VK_MINUS => Some(Key::Minus),
            xplm_sys::XPLM_VK_RBRACE => Some(Key::ClosingBrace),
            xplm_sys::XPLM_VK_LBRACE => Some(Key::OpeningBrace),
            xplm_sys::XPLM_VK_QUOTE => Some(Key::Quote),
            xplm_sys::XPLM_VK_SEMICOLON => Some(Key::Semicolon),
            xplm_sys::XPLM_VK_BACKSLASH => Some(Key::Backslash),
            xplm_sys::XPLM_VK_COMMA => Some(Key::Comma),
            xplm_sys::XPLM_VK_SLASH => Some(Key::Slash),
            xplm_sys::XPLM_VK_PERIOD => Some(Key::Period),
            xplm_sys::XPLM_VK_BACKQUOTE => Some(Key::Backquote),
            xplm_sys::XPLM_VK_ENTER => Some(Key::Enter),
            xplm_sys::XPLM_VK_NUMPAD_ENT => Some(Key::NumpadEnter),
            xplm_sys::XPLM_VK_NUMPAD_EQ => Some(Key::NumpadEqual),
            _ => None,
        }
    }

    /// Converts an XPLM (non-virtual) key code into a Key
    fn from_xplm_non_virtual(xplm_key: c_char) -> Option<Self> {
        match xplm_key as u32 {
            xplm_sys::XPLM_KEY_DECIMAL => Some(Key::Period),
            _ => None,
        }
    }
}

/// An event associated with a key press
#[derive(Debug)]
pub struct KeyEvent {
    /// A character representing the key
    basic_char: Option<char>,
    /// The key
    key: Key,
    /// The action
    action: KeyAction,
    /// If the control key was pressed
    control_pressed: bool,
    /// If the option/alt key was pressed
    option_pressed: bool,
    /// If the shift key was pressed
    shift_pressed: bool,
}

impl KeyEvent {
    /// Creates a key event from XPLM key information
    fn from_xplm(
        key: c_char,
        flags: xplm_sys::XPLMKeyFlags,
        virtual_key: c_char,
    ) -> Result<Self, KeyEventError> {
        let basic_char = match key.cast_unsigned() {
            // Accept printable characters, including spaces and tabs
            b'\t' | b' '..=b'~' => Some(key.cast_unsigned() as char),
            _ => None,
        };
        let action = if flags & xplm_sys::xplm_DownFlag.cast_signed() != 0 {
            KeyAction::Press
        } else if flags & xplm_sys::xplm_UpFlag.cast_signed() != 0 {
            KeyAction::Release
        } else {
            return Err(KeyEventError::InvalidFlags(flags));
        };
        let control_pressed = flags & xplm_sys::xplm_ControlFlag.cast_signed() != 0;
        let shift_pressed = flags & xplm_sys::xplm_ShiftFlag.cast_signed() != 0;
        let option_pressed = flags & xplm_sys::xplm_OptionAltFlag.cast_signed() != 0;
        let key = match Key::from_xplm(virtual_key) {
            Some(key) => key,
            // some keys (notably period on the main keyboard) don't have virtual keys
            None => match Key::from_xplm_non_virtual(key) {
                Some(key) => key,
                None => return Err(KeyEventError::InvalidKey(virtual_key)),
            },
        };

        Ok(KeyEvent {
            basic_char,
            key,
            action,
            control_pressed,
            option_pressed,
            shift_pressed,
        })
    }
    /// Returns the character corresponding to the key associated with this event, if one exists
    ///
    /// Some key combinations, including combinations with non-Shift modifiers, may not have
    /// corresponding characters.
    #[must_use]
    pub fn char(&self) -> Option<char> {
        self.basic_char
    }
    /// Returns the key associated with this event
    #[must_use]
    pub fn key(&self) -> Key {
        self.key.clone()
    }
    /// Returns true if the control key was held down when the action occurred
    #[must_use]
    pub fn control_pressed(&self) -> bool {
        self.control_pressed
    }
    /// Returns true if the option/alt key was held down when the action occurred
    #[must_use]
    pub fn option_pressed(&self) -> bool {
        self.option_pressed
    }
    /// Returns true if a shift key was held down when the action occurred
    #[must_use]
    pub fn shift_pressed(&self) -> bool {
        self.shift_pressed
    }
    /// Returns the key action that occurred
    #[must_use]
    pub fn action(&self) -> KeyAction {
        self.action.clone()
    }
}

/// Key event creation error
#[derive(thiserror::Error, Debug)]
enum KeyEventError {
    #[error("Unexpected key flags {0:b}")]
    InvalidFlags(xplm_sys::XPLMKeyFlags),

    #[error("Invalid or unsupported key with code: 0x{0:x}")]
    InvalidKey(c_char),
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
