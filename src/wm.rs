use crate::config::{KEYBINDS, MODKEY};
use crate::desktop::Desktop;
use crate::helper;
use crate::screen::Screen;
use crate::windows::Window;
use crate::x::{CursorIndex, XConn, XWindow};

use xcb_util::cursor;

#[derive(PartialEq)]
enum MouseMode {
    Ground,
    Resize,
    Move,
}

pub struct WM<'a> {
    // X connection
    pub conn: XConn<'a>,

    // Windows in workspaces stored
    pub desktop: Desktop,
    pub screen:  Screen,

    // Mouse mode from button press events
    mouse_mode: MouseMode,
    last_mouse_x: i32,
    last_mouse_y: i32,
    selected: Option<xcb::Window>,

    // Should we continue running?
    running: bool,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a xcb::Connection, screen_idx: i32) -> Self {
        // Create new XConn wrapping xcb::Connection
        let mut xconn = XConn::new(conn);

        // Get root window id for screen index
        let root_id = xconn.get_setup().roots().nth(screen_idx as usize).expect("Getting root window id for screen index").root();

        // Create new screen object
        let mut screen = Screen::new(screen_idx, root_id);

        // Try register the root window for necessary window management events
        xconn.change_window_attributes_checked(root_id, &helper::values_attributes_root());

        // For configured keybinds, register X to grab keys on the root window
        for (mask, keysym, _) in KEYBINDS {
            xconn.grab_key(root_id, *mask, *keysym);
        }

        // Register root window to grab necessary mouse button events
        xconn.grab_button(root_id, helper::ROOT_BUTTON_GRAB_MASK, xcb::BUTTON_INDEX_1, MODKEY, true);
        xconn.grab_button(root_id, helper::ROOT_BUTTON_GRAB_MASK, xcb::BUTTON_INDEX_3, MODKEY, true);

        // Create necessary core cursors
        xconn.create_core_cursor(CursorIndex::LeftPtr, cursor::LEFT_PTR);

        // Now set the default starting cursor
        xconn.set_cursor(root_id, CursorIndex::LeftPtr);

        // Perform initial screen geometry fetch
        xconn.update_geometry(&mut screen);

        // Return new WM object
        return Self {
            conn: xconn,
            desktop: Desktop::default(),
            screen:  screen,
            mouse_mode: MouseMode::Ground,
            last_mouse_x: 0,
            last_mouse_y: 0,
            selected: None,
            running: true,
        };
    }

    pub fn run(&mut self) {
        info!("Started running");

        while self.running {
            // Get next event
            let event = self.conn.next_event();

            // Cast (this is unsafe) and pass event to appropriate function
            unsafe {
                match event.response_type() {
                    xcb::CONFIGURE_NOTIFY => self.on_configure_notify(xcb::cast_event(&event)),
                    xcb::CONFIGURE_REQUEST => self.on_configure_request(xcb::cast_event(&event)),
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    xcb::ENTER_NOTIFY => self.on_enter_notify(xcb::cast_event(&event)),
                    xcb::MOTION_NOTIFY => self.on_motion_notify(xcb::cast_event(&event)),
                    xcb::BUTTON_PRESS => self.on_button_press(xcb::cast_event(&event)),
                    xcb::BUTTON_RELEASE => self.on_button_release(xcb::cast_event(&event)),
                    xcb::KEY_PRESS => self.on_key_press(xcb::cast_event(&event)),

                    _ => debug!("unhandled event type: {}", event.response_type()),
                }
            }
        }

        info!("Finished running");
    }

    fn on_configure_notify(&mut self, event: &xcb::ConfigureNotifyEvent) {
        // We only care about this if it's the route window being configured
        if event.window() == self.screen.id() {
            debug!("on_configure_notify: root window");

            // Set new root window geometry
            self.screen.x = event.x() as i32;
            self.screen.y = event.y() as i32;
            self.screen.width = event.width() as i32;
            self.screen.height = event.height() as i32;

            // Deactivate / active current workspace to redraw
            self.desktop.current_mut().deactivate(&self.conn);
            self.desktop.current_mut().activate(&self.conn, &self.screen);
        }
    }

    fn on_configure_request(&mut self, event: &xcb::ConfigureRequestEvent) {
        debug!("on_configure_request: {}", event.window());

        // Get the referenced Window
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            // Get the window at index
            let window = ws.windows.get_mut(idx).unwrap();

            // Value vector we use at end
            let mut values: Vec<(u16, u32)> = Vec::new();

            // If x configuration mask found, push to values vector and set Window geometry
            if xcb::CONFIG_WINDOW_X as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32));
                window.x = event.x() as i32;
            }

            // If y configuration mask found, push to values vector and set Window geometry
            if xcb::CONFIG_WINDOW_Y as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_Y as u16, event.y() as u32));
                window.y = event.y() as i32;
            }

            // If width configuration mask found, push to values vector and set Window geometry
            if xcb::CONFIG_WINDOW_WIDTH as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                window.width = event.width() as i32;
            }

            // If height configuration mask found, push to values vector and set Window geometry
            if xcb::CONFIG_WINDOW_HEIGHT as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                window.height = event.height() as i32;
            }

            // If stack mode configuration mask found, push to values vector and set Window
            //if xcb::CONFIG_WINDOW_STACK_MODE as u16 & event.value_mask() != 0 {
            //    values.push((xcb::CONFIG_WINDOW_STACK_MODE as u16, event.stack_mode() as u32));
            //}

            // If sibling configuration mask found, push to values vector and set Window
            //if xcb::CONFIG_WINDOW_SIBLING as u16 & event.value_mask() != 0 {
            //    values.push((xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32));
            //}

            // Configure window using filtered values
            self.conn.configure_window(event.window(), &values);
        } else {
            warn!("Received configure request for non-tracked window!");
        }
    }

    fn on_map_request(&mut self, event: &xcb::MapRequestEvent) {
        debug!("on_map_request: {}", event.window());

        // Try get Window specified
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            // We found the window, if in the current workspace then focus!
            if ws.active {
                ws.window_focus_idx(&self.conn, &self.screen, idx);
            }
        } else {
            // Not found, create new window
            let mut window = Window::from(event.window());

            // Fetch window geometry
            self.conn.update_geometry(&mut window);

            // Add the Window to the current workspace
            self.desktop.current_mut().window_add(&self.conn, &self.screen, window);
        }
    }

    fn on_unmap_notify(&mut self, event: &xcb::UnmapNotifyEvent) {
        debug!("on_unmap_notify: {}", event.window());

        // This event shouldn't be generated by ourselves (we toggle tracking during unmap to ensure this).
        // We can safely assume that we should just remove whatever Window from whatever workspace it may be in
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            ws.window_del(&self.conn, &self.screen, idx, event.window());
        } else {
            warn!("Recieved unmap notify event for non-trackked window");
        }
    }

    fn on_destroy_notify(&mut self, event: &xcb::DestroyNotifyEvent) {
        debug!("on_destroy_notify: {}", event.window());

        // Whether this event was generated by us, or someone else, it doesn't matter.
        // We can safely assume that we should just remove whatever Window from whatever workspace it may be in
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            ws.window_del(&self.conn, &self.screen, idx, event.window());
        } else {
            warn!("Received destroy notify event for non-tracked window!");
        }
    }

    fn on_enter_notify(&mut self, event: &xcb::EnterNotifyEvent) {
        debug!("on_enter_notify: {}", event.event());

        // We should only receive these from child windows we've tracked, so if in current workspace we set input focus
        if self.desktop.current().windows.contains(event.event()).is_some() {
            self.conn.set_input_focus(event.event());
        } else {
            warn!("Received enter notify event for window either not in current workspace or non-tracked!");
        }
    }

    fn on_motion_notify(&mut self, event: &xcb::MotionNotifyEvent) {
        // Only perform something if there's a window selected
        if let Some(selected) = self.selected {
            debug!("on_motion_notify");

            // Calculate dx, dy
            let dx = event.root_x() as i32 - self.last_mouse_x;
            let dy = event.root_y() as i32 - self.last_mouse_y;

            // Set new last mouse positions
            self.last_mouse_x = event.root_x() as i32;
            self.last_mouse_y = event.root_y() as i32;

            // Get the selected Window, this should be focused but may not always
            if let Some(idx) = self.desktop.current().windows.contains(selected) {
                let selected = self.desktop.current_mut().windows.get_mut(idx).unwrap();

                // React depending on current MouseMode
                match self.mouse_mode {
                    MouseMode::Move => {
                        selected.do_move(&self.conn, &self.screen, dx, dy);
                    },

                    MouseMode::Resize => {
                        selected.do_resize(&self.conn, &self.screen, dx, dy);
                    },

                    _ => panic!("MouseMode::Ground reached in on_motion_notify()"),
                }
            }
        }
    }

    fn on_button_press(&mut self, event: &xcb::ButtonPressEvent) {
        // If button press not in a child window to root, we don't care
        if event.child() == xcb::WINDOW_NONE {
            return;
        }

        // Set the selected window
        self.selected = Some(event.child());

        // Set current mouse position
        self.last_mouse_x = event.root_x() as i32;
        self.last_mouse_y = event.root_y() as i32;

        // Start grabbing pointer
        self.conn.grab_pointer(self.screen.id(), helper::ROOT_POINTER_GRAB_MASK, false);

        // If window id different to focused, focus it
        if !self.desktop.current().windows.is_focused(event.child()) {
            self.desktop.current_mut().window_focus(&self.conn, &self.screen, event.child());
        }

        // Get MouseButton for event
        match event.detail() as u32 {
            // Left click, set mouse mode
            xcb::BUTTON_INDEX_1 => {
                debug!("on_button_press: mouse left click");
                self.mouse_mode = MouseMode::Move;
            },

            // Right click, set mouse mode
            xcb::BUTTON_INDEX_3 => {
                debug!("on_button_press: mouse right click");
                self.mouse_mode = MouseMode::Resize;
            },

            _ => panic!("Unhandled button press in on_button_press"),
        }
    }

    fn on_button_release(&mut self, event: &xcb::ButtonReleaseEvent) {
        // We only log these in debug builds
        #[cfg(debug_assertions)]
        match event.detail() as u32  {
            xcb::BUTTON_INDEX_1 => debug!("on_button_release: mouse left click"),
            xcb::BUTTON_INDEX_3 => debug!("on_button_release: mouse right click"),
            _ => panic!("Unhandled button release in on_button_release"),
        }

        // Unselect the window and unset MouseMode
        self.selected = None;
        self.mouse_mode = MouseMode::Ground;

        // Ungrab the pointer
        self.conn.ungrab_pointer();
    }

    fn on_key_press(&mut self, event: &xcb::KeyPressEvent) {
        // Decode KeyEvent
        let (press_mask, press_key) = self.conn.lookup_keysym(event);

        debug!("on_key_press: {} {}", press_mask, press_key);

        // Try get function for keybind
        for (mask, key, keyfn) in KEYBINDS {
            // Check for match
            if *mask == press_mask && *key == press_key {
                // If window id isn't the focused window id, refocus
                if !self.desktop.current().windows.is_focused(event.child()) {
                    self.desktop.current_mut().window_focus(&self.conn, &self.screen, event.child());
                }

                // Execute! And return
                keyfn(self);
                return;
            }
        }
    }

    pub fn kill(&mut self) {
        info!("Killing");
        self.running = false;
    }
}
