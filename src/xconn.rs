use crate::config::MODKEY;
use crate::event::{Event, KeyEvent, MouseButton};

use xcb_util::keysyms::KeySymbols;

pub struct XConn<'a> {
    // X server connection
    pub conn: &'a xcb::Connection,

    // Connected screen index
    pub screen_idx: i32,

    // Root window ID
    pub root: xcb::Window,

    // KeySymbol lookup object
    key_syms: KeySymbols<'a>,
}

impl<'a> XConn<'a> {
    pub fn register(conn: &'a xcb::Connection, screen_idx: i32, keys: &Vec<(xcb::ModMask, xcb::Keysym)>) -> Self {
        // Get the root window id
        let root = conn.get_setup().roots().nth(screen_idx as usize)
                       .unwrap_or_else(||{panic!("Fetching root window for screen {}", screen_idx)}).root();
        outlog::debug!("Fetched root window ID");

        // Register root window to receive events
        xcb::change_window_attributes_checked(conn, root, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY|xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY|xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT),
        ]).request_check().expect("Registering with X server as window manager");
        outlog::debug!("Registered as window manager");

        // New KeySymbols object for keysym lookup
        let key_syms = KeySymbols::new(conn);

        // Register mouse events to grab
        xcb::grab_button(
            conn,
            false,                                                                // owner events, (a.k. don't pass on events to root window)
            root,                                                                 // window id
            xcb::EVENT_MASK_BUTTON_PRESS as u16|xcb::EVENT_MASK_BUTTON_RELEASE as u16, // button event mask
            xcb::GRAB_MODE_ASYNC as u8,                                           // pointer mode
            xcb::GRAB_MODE_ASYNC as u8,                                           // keyboard mode
            root,                                                                 // confine pointer to window (or no confine)
            xcb::NONE,                                                            // cursor to use
            xcb::BUTTON_INDEX_1 as u8,                                            // button to grab (left click)
            MODKEY as u16,                                                        // Modifiers to grab mouse with
        );
        xcb::grab_button(
            conn,
            false,                                                                       // owner events, (a.k. don't pass on events to root window)
            root,                                                                        // window id
            xcb::EVENT_MASK_BUTTON_PRESS as u16|xcb::EVENT_MASK_BUTTON_RELEASE as u16, // button event mask
            xcb::GRAB_MODE_ASYNC as u8,                                           // pointer mode
            xcb::GRAB_MODE_ASYNC as u8,                                           // keyboard mode
            root,                                                                 // confine pointer to window (or no confine)
            xcb::NONE,                                                            // cursor to use
            xcb::BUTTON_INDEX_3 as u8,                                            // button to grab (right click)
            MODKEY as u16,                                                        // Modifiers to grab mouse with
        );

        // Register keys events to grab
        for (mask, keysym) in keys.iter() {
            // Get code iter for keysym
            let code = key_syms.get_keycode(*keysym).next();

            // If no code, log and move-on
            if code.is_none() {
                outlog::warn!("Keysym translated to zero-length keycode iter");
                continue;
            }

            let code = code.unwrap();

            // Register the keycode
            xcb::grab_key(
                conn,
                false,                       // owner events (a.k.a don't pass on events to root window)
                root,                        // window id
                *mask as u16,                // key mod mask
                code,                        // keycode
                xcb::GRAB_MODE_ASYNC as u8,  // pointer mode
                xcb::GRAB_MODE_ASYNC as u8   // keyboard mode
            );
            outlog::debug!("Registered grabbed key: {} {}", *mask, code);
        }

        // Return new XConn
        Self {
            conn:       conn,
            screen_idx: screen_idx,
            root:       root,
            key_syms:   key_syms,
        }
    }

    pub fn window_map(&self, window: xcb::Window) {
        outlog::debug!("Mapping window: {}", window);
        xcb::map_window(self.conn, window);
    }

    pub fn window_unmap(&self, window: xcb::Window) {
        outlog::debug!("Unmapping window: {}", window);
        xcb::unmap_window(self.conn, window);
    }

    pub fn window_ontop(&self, window_id: xcb::Window) {
        outlog::debug!("Moving window to top of stack: {}", window_id);
        xcb::configure_window(self.conn, window_id, &[(xcb::CONFIG_WINDOW_STACK_MODE as u16, xcb::STACK_MODE_ABOVE)]);
    }

    pub fn window_move(&self, window_id: xcb::Window, x: u32, y: u32) {
        outlog::debug!("Moving window: {}", window_id);
        xcb::configure_window(self.conn, window_id, &[(xcb::CONFIG_WINDOW_X as u16, x), (xcb::CONFIG_WINDOW_Y as u16, y)]);
    }

    pub fn window_resize(&self, window_id: xcb::Window, width: u32, height: u32) {
        outlog::debug!("Resizing window: {}", window_id);
        xcb::configure_window(self.conn, window_id, &[(xcb::CONFIG_WINDOW_WIDTH as u16, width), (xcb::CONFIG_WINDOW_HEIGHT as u16, height)]);
    }

    pub fn window_configure(&self, window_id: xcb::Window, x: u32, y: u32, width: u32, height: u32) {
        outlog::debug!("Configuring window: {}", window_id);
        xcb::configure_window(self.conn, window_id, &[
            (xcb::CONFIG_WINDOW_X as u16, x),
            (xcb::CONFIG_WINDOW_Y as u16, y),
            (xcb::CONFIG_WINDOW_WIDTH as u16, width),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, height),
        ]);
    }

    pub fn window_enable_tracking(&self, window: xcb::Window) {
        outlog::debug!("Enabling window tracking on: {}", window);
        xcb::change_window_attributes(self.conn, window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_ENTER_WINDOW|xcb::EVENT_MASK_STRUCTURE_NOTIFY),
        ]);
    }

    pub fn window_disable_tracking(&self, window: xcb::Window) {
        outlog::debug!("Disabling window tracking on: {}", window);
        xcb::change_window_attributes(self.conn, window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT),
        ]);
    }

    pub fn window_focus(&self, window: xcb::Window) {
        outlog::debug!("Focusing window: {}", window);
        xcb::set_input_focus(self.conn, xcb::INPUT_FOCUS_POINTER_ROOT as u8, window, xcb::CURRENT_TIME);
    }

    pub fn window_close(&self, window: xcb::Window) {
        outlog::debug!("Destroying window: {}", window);
        xcb::destroy_window(self.conn, window);
    }

    pub fn grab_pointer(&self) {
        outlog::debug!("Grabbing pointer for root window");
        xcb::grab_pointer(
            self.conn,
            false,
            self.root,
            (xcb::EVENT_MASK_BUTTON_RELEASE|xcb::EVENT_MASK_BUTTON_MOTION|xcb::EVENT_MASK_POINTER_MOTION_HINT) as u16,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            self.root,
            xcb::NONE,
            xcb::CURRENT_TIME,
        );
    }

    pub fn ungrab_pointer(&self) {
        outlog::debug!("Ungrabbing pointer for root window");
        xcb::ungrab_pointer(self.conn, xcb::CURRENT_TIME);
    }

    pub fn get_geometry(&self, window_id: xcb::Window) -> (i32, i32, i32, i32) {
        outlog::debug!("Getting window geometry: {}", window_id);
        let dimens = xcb::get_geometry(self.conn, window_id).get_reply().expect("Getting root window geometry");
        return (dimens.x() as i32, dimens.y() as i32, dimens.width() as i32, dimens.height() as i32);
    }

    pub fn get_pointer(&self, window_id: xcb::Window) -> (i32, i32, xcb::Window) {
        outlog::debug!("Querying window pointer location: {}", window_id);
        let pointer = xcb::query_pointer(self.conn, window_id).get_reply().expect("Querying root pointer location");
        return (pointer.root_x() as i32, pointer.root_y() as i32, pointer.child())
    }

    pub fn next_event(&self) -> Event {
        loop {
            // Flush connection to ensure clean
            self.conn.flush();

            // Check for queued
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
                    // Configure request handled internally (always none)
                    xcb::CONFIGURE_REQUEST => self.on_configure_request(xcb::cast_event(&event)),

                    // All others generate necessary Event and return
                    xcb::MAP_REQUEST    => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY   => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    xcb::ENTER_NOTIFY   => self.on_enter_notify(xcb::cast_event(&event)),
                    xcb::MOTION_NOTIFY  => self.on_motion_notify(xcb::cast_event(&event)),
                    xcb::KEY_PRESS      => self.on_key_press(xcb::cast_event(&event)),
                    xcb::BUTTON_PRESS   => self.on_button_press(xcb::cast_event(&event)),
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
        outlog::debug!("on_configure_request");

        // Value vector we use at end
        let mut values: Vec<(u16, u32)> = Vec::new();

        // Has CONFIG_WINDOW_X mask, add to values
        if xcb::CONFIG_WINDOW_X as u16 & event.value_mask() != 0 {
            values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32));
        }

        // Has CONFIG_WINDOW_Y mask, add to values
        if xcb::CONFIG_WINDOW_Y as u16 & event.value_mask() != 0 {
            values.push((xcb::CONFIG_WINDOW_Y as u16, event.y() as u32));
        }

        // Has CONFIG_WINDOW_WIDTH mask, add to values
        if xcb::CONFIG_WINDOW_WIDTH as u16 & event.value_mask() != 0 {
            values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
        }

        // Has CONFIG_WINDOW_HEIGHT mask, add to values
        if xcb::CONFIG_WINDOW_HEIGHT as u16 & event.value_mask() != 0 {
            values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
        }

        // Has CONFIG_WINDOW_SIBLING mask, add to values
        if xcb::CONFIG_WINDOW_SIBLING as u16 & event.value_mask() != 0 {
            values.push((xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32));
        }

        // Has CONFIG_WINDOW_BORDER_WIDTH mask, add to values
        //if xcb::CONFIG_WINDOW_BORDER_WIDTH as u16 & event.value_mask() != 0 {
        //    values.push((xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, event.border_width() as u32));
        //}

        // Configure window using filtered values
        xcb::configure_window(&self.conn, event.window(), &values);

        // Nothing to return
        return None;
    }

    fn on_map_request(&self, event: &xcb::MapRequestEvent) -> Option<Event> {
        // Log this!
        outlog::debug!("on_map_request: {}", event.window());

        // Return new MapRequest Event
        return Some(Event::MapRequest(event.window()));
    }

    fn on_unmap_notify(&self, event: &xcb::UnmapNotifyEvent) -> Option<Event> {
        // Ignore those from our root SUBSTRUCTURE_NOTIFY mask
        if event.event() == self.root {
            return None;
        }

        // Log this!
        outlog::debug!("on_unmap_notify: {}", event.window());

        // Return new UnmapNotify Event
        return Some(Event::UnmapNotify(event.window()));
    }

    fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) -> Option<Event> {
        // Log this!
        outlog::debug!("on_destroy_notify: {}", event.window());

        // Return new DestroyNotify Event
        return Some(Event::DestroyNotify(event.window()));
    }

    fn on_enter_notify(&self, event: &xcb::EnterNotifyEvent) -> Option<Event> {
        // Log this!
        outlog::debug!("on_enter_notify: {}", event.event());

        // Return new EnterNotify Event
        return Some(Event::EnterNotify(event.event()));
    }

    fn on_motion_notify(&self, event: &xcb::MotionNotifyEvent) -> Option<Event> {
        // Log this!
        outlog::debug!("on_motion_notify: {}", event.child());

        // Return new MotionNotify Event
        return Some(Event::MotionNotify);
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
        outlog::debug!("on_key_press: {} {}", key_ev.mask, key_ev.key);

        // Return KeyPress Event
        return Some(Event::KeyPress(key_ev));
    }

    fn on_button_press(&self, event: &xcb::ButtonPressEvent) -> Option<Event> {
        // Get MouseButton for event
        let tuple = match event.detail() as u32 {
            // Left click
            xcb::BUTTON_INDEX_1 => {
                outlog::debug!("on_button_press: mouse left click");
                Event::ButtonPress((MouseButton::LeftClick, event.child()))
            }

            // Right click
            xcb::BUTTON_INDEX_3 => {
                outlog::debug!("on_button_press: mouse right click");
                Event::ButtonPress((MouseButton::RightClick, event.child()))
            }

            // Invalid button press, return nothing
            b => {
                outlog::debug!("on_button_press: unhandled button {}", b);
                return None;
            },
        };

        // Grab pointer while button pressed
        self.grab_pointer();

        // Return the event
        return Some(tuple);
    }

    fn on_button_release(&self, event: &xcb::ButtonReleaseEvent) -> Option<Event> {
        // Get MouseButton for event
        let ev = match event.detail() as u32 {
            // Left click
            xcb::BUTTON_INDEX_1 => {
                outlog::debug!("on_button_release: mouse left click");
                Event::ButtonRelease(MouseButton::LeftClick)
            }

            // Right click
            xcb::BUTTON_INDEX_3 => {
                outlog::debug!("on_button_release: mouse right click");
                Event::ButtonRelease(MouseButton::RightClick)
            }

            // Invalid button press, return nothing
            b => {
                outlog::debug!("on_button_release: unhandled button {}", b);
                return None;
            },
        };

        // Ungrab pointer now button released
        self.ungrab_pointer();

        // Return the event
        return Some(ev);
    }
}