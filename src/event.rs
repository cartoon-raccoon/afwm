pub enum Event {
    MapRequest(xcb::Window),
    UnmapNotify(xcb::Window),
    DestroyNotify(xcb::Window),
    EnterNotify(xcb::Window),
    MotionNotify((i32, i32)),
    KeyPress((KeyEvent, xcb::Window)),
    ButtonPress(((i32, i32), MouseButton, xcb::Window)),
    ButtonRelease(MouseButton),
}

pub struct KeyEvent {
    pub mask: xcb::ModMask,
    pub key:  xcb::Keysym,
}

pub enum MouseButton {
    LeftClick,
    RightClick,
}