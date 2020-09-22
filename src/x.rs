use crate::event::{Event, KeyEvent, MouseButton};
use crate::helper;

use xcb_util::cursor;
use xcb_util::keysyms::KeySymbols;

pub enum CursorIndex {
    LeftPtr,
}

pub trait XWindow {
    fn id(&self) -> xcb::Window;
    fn set(&mut self, x: i32, y: i32, width: i32, height: i32);
}

pub struct XConn<'a> {
    // X server connection
    pub conn: &'a xcb::Connection,

    // Stored loaded cursors + optional core cursor font
    cursors: [u32; 1],

    // KeySymbol lookup object
    key_syms: KeySymbols<'a>,
}

impl<'a> XConn<'a> {
    pub fn new(conn: &'a xcb::Connection) -> Self {
        Self {
            conn:             conn,
            cursors:          [0; 1],
            key_syms:         KeySymbols::new(conn),
        }
    }

    pub fn create_core_cursor(&mut self, cursor: CursorIndex, cursor_glyph: u16) {
        // Try load cursor for supplied cursor glyp
        let cursor_id = cursor::create_font_cursor_checked(self.conn, cursor::LEFT_PTR).expect("Creating font cursor");

        // Store the cursor id in the cursors array at supplied index
        self.cursors[cursor as usize] = cursor_id;
    }

    pub fn create_pixmap_cursor(&mut self, cursor: CursorIndex) {
        // Allocate new pixmap id
        let pixmap_id = self.conn.generate_id();
    }

    pub fn set_cursor(&mut self, window_id: xcb::Window, cursor: CursorIndex) {
        // Get the cursor id at index in the stored cursors array
        let cursor_id = self.cursors[cursor as usize];

        // Set the window cursor attributes
        self.change_window_attributes(window_id, &helper::values_attributes_cursor(cursor_id));
    }

    pub fn get_setup(&self) -> xcb::Setup {
        debug!("Getting setup");
        return self.conn.get_setup();
    }

    pub fn query_tree(&self, window_id: xcb::Window) -> Vec<xcb::Window> {
        debug!("Querying tree for window: {}", window_id);
        let reply = xcb::query_tree(&self.conn, window_id).get_reply().expect("Querying window tree");
        return reply.children().iter().map(|w| { *w }).collect();
    }

    pub fn map_window(&self, window_id: xcb::Window) {
        debug!("Mapping window: {}", window_id);
        xcb::map_window(self.conn, window_id);
    }

    pub fn unmap_window(&self, window_id: xcb::Window) {
        debug!("Unmapping window: {}", window_id);
        xcb::unmap_window(self.conn, window_id);
    }

    pub fn configure_window(&self, window_id: xcb::Window, values: &[(u16, u32)]) {
        debug!("Configuring window: {}", window_id);
        xcb::configure_window(self.conn, window_id, values);
    }

    pub fn change_window_attributes(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);
        xcb::change_window_attributes(self.conn, window_id, values);
    }

    pub fn change_window_attributes_checked(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);
        xcb::change_window_attributes_checked(self.conn, window_id, values).request_check().expect("Changing window attributes");
    }

    pub fn set_input_focus(&self, window_id: xcb::Window) {
        debug!("Setting input focus window: {}", window_id);
        xcb::set_input_focus(self.conn, xcb::INPUT_FOCUS_POINTER_ROOT as u8, window_id, xcb::CURRENT_TIME);
    }

    pub fn destroy_window(&self, window_id: xcb::Window) {
        debug!("Destroying window: {}", window_id);
        xcb::destroy_window(self.conn, window_id);
    }

    pub fn grab_key(&self, window_id: xcb::Window, mask: xcb::ModMask, keysym: xcb::Keysym, confine: bool) {
        debug!("Grabbing key with mask:{} sym:{} for window: {}", mask, keysym, window_id);

        // Get code iter for keysym
        let code = self.key_syms.get_keycode(keysym).next();

        // If no code, log and move-on. Else, unwrap
        if code.is_none() {
            warn!("Keysym {} translated to zero-length keycode iter, not grabbing", keysym);
            return;
        }
        let code = code.unwrap();

        // Register key code to grab with X
        xcb::grab_key(
            self.conn,
            false,                       // owner events (a.k.a don't pass on events to root window)
            window_id,                   // window id
            mask as u16,                 // key mod mask
            code,                        // keycode
            xcb::GRAB_MODE_ASYNC as u8,  // pointer mode
            xcb::GRAB_MODE_ASYNC as u8   // keyboard mode
        );
    }

    pub fn grab_button(&self, window_id: xcb::Window, mask: xcb::ButtonMask, button: xcb::ButtonIndex, modmask: xcb::ModMask, confine: bool) {
        debug!("Grabbing button {} for window: {}", window_id, button);
        xcb::grab_button(
            self.conn,
            false,                                       // owner events (a.k. don't pass on events to root window)
            window_id,                                   // window id
            mask as u16,                                 // button event mask
            xcb::GRAB_MODE_ASYNC as u8,                  // pointer mode
            xcb::GRAB_MODE_ASYNC as u8,                  // keyboard mode
            if confine { window_id } else { xcb::NONE }, // confine pointer to window (or no confine)
            xcb::NONE,                                   // cursor to use
            button as u8,                                // button to grab (right click)
            modmask as u16,                              // Modifiers to grab mouse with
        );
    }

    pub fn grab_pointer(&self, window_id: xcb::Window, mask: xcb::EventMask, confine: bool) {
        debug!("Grabbing pointer for window: {}", window_id);
        xcb::grab_pointer(
            self.conn,
            false,                                       // owner events (a.k. don't pass on events to root window)
            window_id,                                   //
            mask as u16,                                 //
            xcb::GRAB_MODE_ASYNC as u8,                  //
            xcb::GRAB_MODE_ASYNC as u8,                  //
            if confine { window_id } else { xcb::NONE }, //
            xcb::NONE,                                   //
            xcb::CURRENT_TIME,                           //
        );
    }

    pub fn ungrab_pointer(&self) {
        debug!("Ungrabbing pointer");
        xcb::ungrab_pointer(self.conn, xcb::CURRENT_TIME);
    }

    pub fn update_geometry(&self, window: &mut impl XWindow) -> bool {
        match self.get_geometry(window.id()) {
            Some((x, y, width, height)) => {
                // Update supplied window's geometry, return true
                window.set(x, y, width, height);
                return true;
            },

            None => return false,
        }
    }

    pub fn get_geometry(&self, window_id: xcb::Window) -> Option<(i32, i32, i32, i32)> {
        debug!("Getting geometry for window: {}", window_id);
        match xcb::get_geometry(self.conn, window_id).get_reply() {
            Ok(dimens) => return Some((dimens.x() as i32, dimens.y() as i32, dimens.width() as i32, dimens.height() as i32)),
            Err(_) => {
                warn!("Failed getting window geometry for {}. Was window destroyed and not yet unmapped?", window_id);
                return None;
            },
        }
    }

    pub fn query_pointer(&self, window_id: xcb::Window) -> (i32, i32, xcb::Window) {
        debug!("Querying pointer location for window: {}", window_id);
        let pointer = xcb::query_pointer(self.conn, window_id).get_reply().expect("Querying window pointer location");
        return (pointer.root_x() as i32, pointer.root_y() as i32, pointer.child())
    }

    pub fn next_event(&self) -> Event {
        loop {
            // Flush connection to ensure clean
            self.conn.flush();

            // Check for queued, else wait for next
            let event = if let Some(ev) = self.conn.poll_for_queued_event() {
                ev
            } else {
                self.conn.wait_for_event().expect("I/O error getting event from X server")
            };

            // Declare in this scope
            let opt;

            // Set opt 'unsafely' as this is what xcb::cast_event requires
            unsafe {
                opt = match event.response_type() {
                    xcb::CONFIGURE_REQUEST => self.on_configure_request(xcb::cast_event(&event)),
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    xcb::ENTER_NOTIFY => self.on_enter_notify(xcb::cast_event(&event)),
                    xcb::MOTION_NOTIFY => self.on_motion_notify(xcb::cast_event(&event)),
                    xcb::KEY_PRESS => self.on_key_press(xcb::cast_event(&event)),
                    xcb::BUTTON_PRESS => self.on_button_press(xcb::cast_event(&event)),
                    xcb::BUTTON_RELEASE => self.on_button_release(xcb::cast_event(&event)),

                    // Unhandled event type
                    _ => None,
                };
            }

            // If we have an event to return, do so! Else, continue loop
            if let Some(ev) = opt {
                return ev;
            }
        }
    }

    fn on_configure_request(&self, event: &xcb::ConfigureRequestEvent) -> Option<Event> {
        debug!("on_configure_request");

        // Value vector we use at end
        let mut values: Vec<(u16, u32)> = Vec::new();

        // Set values we can find masks for
        if xcb::CONFIG_WINDOW_X as u16 & event.value_mask()          != 0 { values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32)); }
        if xcb::CONFIG_WINDOW_Y as u16 & event.value_mask()          != 0 { values.push((xcb::CONFIG_WINDOW_Y as u16, event.y() as u32)); }
        if xcb::CONFIG_WINDOW_WIDTH as u16 & event.value_mask()      != 0 { values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32)); }
        if xcb::CONFIG_WINDOW_HEIGHT as u16 & event.value_mask()     != 0 { values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32)); }
        if xcb::CONFIG_WINDOW_SIBLING as u16 & event.value_mask()    != 0 { values.push((xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32)); }
        if xcb::CONFIG_WINDOW_STACK_MODE as u16 & event.value_mask() != 0 { values.push((xcb::CONFIG_WINDOW_STACK_MODE as u16, event.stack_mode() as u32)) }

        // Configure window using filtered values
        xcb::configure_window(&self.conn, event.window(), &values);

        // Nothing to return
        return Some(Event::ConfigureRequest(((event.x() as i32, event.y() as i32, event.width() as i32, event.height() as i32), event.window())));
    }

    fn on_map_request(&self, event: &xcb::MapRequestEvent) -> Option<Event> {
        // Log this!
        debug!("on_map_request: {}", event.window());

        // Return new MapRequest Event
        return Some(Event::MapRequest(event.window()));
    }

    fn on_unmap_notify(&self, event: &xcb::UnmapNotifyEvent) -> Option<Event> {
        // Log this!
        debug!("on_unmap_notify: {}", event.window());

        // Return new UnmapNotify Event
        return Some(Event::UnmapNotify(event.window()));
    }

    fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) -> Option<Event> {
        // Log this!
        debug!("on_destroy_notify: {}", event.window());

        // Return new DestroyNotify Event
        return Some(Event::DestroyNotify(event.window()));
    }

    fn on_enter_notify(&self, event: &xcb::EnterNotifyEvent) -> Option<Event> {
        // Log this!
        debug!("on_enter_notify: {}", event.event());

        // Return new EnterNotify Event
        return Some(Event::EnterNotify(event.event()));
    }

    fn on_motion_notify(&self, event: &xcb::MotionNotifyEvent) -> Option<Event> {
        // If button press happens not in sub-window to root, we don't care
        if event.child() == xcb::WINDOW_NONE {
            return None;
        }

        // Log this!
        debug!("on_motion_notify: {}", event.child());

        // Return new MotionNotify Event
        return Some(Event::MotionNotify((event.root_x() as i32, event.root_y() as i32)));
    }

    fn on_key_press(&self, event: &xcb::KeyPressEvent) -> Option<Event> {
        // Get keysym for event
        let keysym = self.key_syms.press_lookup_keysym(event, 0);

        // Create new key object
        let key_ev = KeyEvent{
            mask: event.state() as u32,
            key:  keysym,
        };

        // Log this!
        debug!("on_key_press: {} {}", key_ev.mask, key_ev.key);

        // Return KeyPress Event
        return Some(Event::KeyPress((key_ev, event.child())));
    }

    fn on_button_press(&self, event: &xcb::ButtonPressEvent) -> Option<Event> {
        // If button press not in sub-window to root, we don't care
        if event.child() == xcb::WINDOW_NONE {
            return None;
        }

        // Get MouseButton for event
        let tuple = match event.detail() as u32 {
            // Left click
            xcb::BUTTON_INDEX_1 => {
                debug!("on_button_press: mouse left click");
                Event::ButtonPress(((event.root_x() as i32, event.root_y() as i32), MouseButton::LeftClick, event.child()))
            }

            // Right click
            xcb::BUTTON_INDEX_3 => {
                debug!("on_button_press: mouse right click");
                Event::ButtonPress(((event.root_x() as i32, event.root_y() as i32), MouseButton::RightClick, event.child()))
            }

            // Invalid button press, return nothing
            _ => {
                debug!("on_button_press: unhandled button");
                return None;
            },
        };

        // Return the event
        return Some(tuple);
    }

    fn on_button_release(&self, event: &xcb::ButtonReleaseEvent) -> Option<Event> {
        // If button press not in sub-window to root, we don't care
        if event.child() == xcb::WINDOW_NONE {
            return None;
        }

        // Get MouseButton for event
        let ev = match event.detail() as u32 {
            // Left click
            xcb::BUTTON_INDEX_1 => {
                debug!("on_button_release: mouse left click");
                Event::ButtonRelease(MouseButton::LeftClick)
            }

            // Right click
            xcb::BUTTON_INDEX_3 => {
                debug!("on_button_release: mouse right click");
                Event::ButtonRelease(MouseButton::RightClick)
            }

            // Invalid button press, return nothing
            b => {
                debug!("on_button_release: unhandled button {}", b);
                return None;
            },
        };

        // Return the event
        return Some(ev);
    }
}