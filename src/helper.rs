// Root window mouse button event mask
pub const ROOT_BUTTON_GRAB_MASK: xcb::ButtonMask = xcb::EVENT_MASK_BUTTON_PRESS|xcb::EVENT_MASK_BUTTON_RELEASE;

// Root window pointer event mask
pub const ROOT_POINTER_GRAB_MASK: xcb::EventMask = xcb::EVENT_MASK_BUTTON_RELEASE|xcb::EVENT_MASK_BUTTON_MOTION;

// Values array of configurations setting window position
pub fn values_configure_move(x: u32, y: u32) -> [(u16, u32); 2] {
    debug!("VALUES: configure move");
    return [(xcb::CONFIG_WINDOW_X as u16, x), (xcb::CONFIG_WINDOW_Y as u16, y)];
}

// Values array of configurations setting window size
pub fn values_configure_resize(width: u32, height: u32) -> [(u16, u32); 2] {
    debug!("VALUES: configure resize");
    return [(xcb::CONFIG_WINDOW_WIDTH as u16, width), (xcb::CONFIG_WINDOW_HEIGHT as u16, height)];
}

// Values array of configurations setting window geometry
pub fn values_configure_geometry(x: u32, y: u32, width: u32, height: u32) -> [(u16, u32); 4] {
    debug!("VALUES: configure geometry");
    return [ (xcb::CONFIG_WINDOW_X as u16, x), (xcb::CONFIG_WINDOW_Y as u16, y), (xcb::CONFIG_WINDOW_WIDTH as u16, width), (xcb::CONFIG_WINDOW_HEIGHT as u16, height) ];
}

// Values array of configuration setting window ontop of stack
pub fn values_configure_stack_above() -> [(u16, u32); 1] {
    debug!("VALUES: configure stack above");
    return [(xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE)];
}

// Values array of attributes setting the cursor to use
pub fn values_attributes_cursor(cursor_id: u32) -> [(u32, u32); 1] {
    debug!("VALUES: attributes cursor");
    return [(xcb::CW_CURSOR, cursor_id)];
}

// Values array of attributes setting root window event mask
pub fn values_attributes_root() -> [(u32, u32); 1] {
    debug!("VALUES: attributes root");
    return [(xcb::CW_EVENT_MASK,
        xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT| // -> Child window CirculateRequest, ConfigureRequest, MapRequest events
        xcb::EVENT_MASK_STRUCTURE_NOTIFY       // -> Self CirculateNotify, ConfigureNotify, DestroyNotify, GravityNotify, MapNotify, ReparentNotify, UnmapNotify events
    )];
}

// Values array of attributes setting child window event mask
pub fn values_attributes_child_events() -> [(u32, u32); 1] {
    debug!("VALUES: attributes child events");
    return [(xcb::CW_EVENT_MASK,
        xcb::EVENT_MASK_ENTER_WINDOW|    // -> Self EnterNotify events
        xcb::EVENT_MASK_STRUCTURE_NOTIFY // -> Self CirculateNotify, ConfigureNotify, DestroyNotify, GravityNotify, MapNotify, ReparentNotify, UnmapNotify events
    )];
}

// Values array of attributes setting no event mask
pub fn values_attributes_no_events() -> [(u32, u32); 1] {
    debug!("VALUES: attributes no events");
    return [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
}