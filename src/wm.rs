use crate::config::{KEYBINDS, MODKEY};
use crate::desktop::Desktop;
use crate::helper;
use crate::screen::Screen;
use crate::windows::Window;
use crate::x::{CursorIndex, XConn, XWindow};

use xcb_util::{cursor, ewmh};

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
    pub fn register(conn: &'a ewmh::Connection, screen_idx: i32) -> Self {
        // Create new XConn wrapping xcb::Connection
        let mut xconn = XConn::new(conn);

        // Get root window id for screen index
        let root_id = xconn.get_setup().roots().nth(screen_idx as usize).expect("Getting root window id for screen index").root();

        // Create new screen object
        let mut screen = Screen::new(screen_idx, root_id);

        // Try register the root window for necessary window management events
        xconn.change_window_attributes_checked(root_id, &helper::values_attributes_root());

        // Set supported atoms
        xconn.set_supported(screen_idx, &[xconn.atoms.WM_PROTOCOLS, xconn.atoms.WM_DELETE_WINDOW]);

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

        // Create Desktop object
        let mut desktop = Desktop::default();

        // Perform initial client fetch
        for existing_id in xconn.query_tree(root_id).iter() {
            // Get attributes for id
            let attr = xconn.get_window_attributes(*existing_id);

            // Ignore windows in override redirect mode / invisible
            if attr.override_redirect() || attr.map_state() as u32 != xcb::MAP_STATE_VIEWABLE {
                continue;
            }
            debug!("Adding existing window: {}", *existing_id);

            // Create Window from existing id, get geometry
            let mut window = Window::from(*existing_id);
            xconn.update_geometry(&mut window);

            // Add window to current workspace!
            desktop.current_mut().windows.add(window);
        }

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

        // Perform an initial activation of current workspace in case contains any windows
        self.desktop.current_mut().activate(&self.conn, &self.screen);

        while self.running {
            // Get next event
            let event = self.conn.next_event();

            // Cast (this is unsafe) and pass event to appropriate function
            unsafe {
                match event.response_type() {
                    // Handle necessary events
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

                    unhandled => debug!("unhandled event type: {}", unhandled),
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

        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            // Get the referenced window at index
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

            // Configure window using filtered values
            self.conn.configure_window(event.window(), &values);
        } else {
            debug!("Received configure request event for non-tracked window!");
        }
    }

    fn on_map_request(&mut self, event: &xcb::MapRequestEvent) {
        debug!("on_map_request: {}", event.window());

        if self.desktop.contains(event.window()).is_none() {
            // Window not already tracked! Check it's one of the types we _want_ to track
            let window_type = self.conn.get_wm_window_type(event.window());
            if !(window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_NORMAL)  ||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_DIALOG)  ||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_TOOLBAR) ||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_UTILITY) ||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_SPLASH)) {
                // We don't want to track this, but we still want it to be displayed
                self.conn.map_window(event.window());
                return;
            }

            // Create new window
            let mut window = Window::from(event.window());

            // Fetch window geometry
            self.conn.update_geometry(&mut window);

            // Add the Window to the current workspace
            self.desktop.current_mut().window_add(&self.conn, &self.screen, window);
        }
    }

    fn on_unmap_notify(&mut self, event: &xcb::UnmapNotifyEvent) {
        debug!("on_unmap_notify: {}", event.window());
        self._unmap_window(event.window());
    }

    fn on_destroy_notify(&mut self, event: &xcb::DestroyNotifyEvent) {
        debug!("on_destroy_notify: {}", event.window());
        self._unmap_window(event.window());
    }

    fn _unmap_window(&mut self, window_id: xcb::Window) {
        // Unmap / destroy event shouldn't be generated by ourselves (we toggle tracking to ensure this).
        // We can safely assume that we should just remove whatever Window from whatever workspace it may be in
        if let Some((ws, idx)) = self.desktop.contains_mut(window_id) {
            ws.window_del(&self.conn, &self.screen, idx, window_id);
        } else {
            debug!("Recieved unmap/destroy notify event for non-tracked window!");
        }
    }

    fn on_enter_notify(&mut self, event: &xcb::EnterNotifyEvent) {
        debug!("on_enter_notify: {}", event.event());

        // We only care about normal / ungrab events
        if !(event.mode() as u32 == xcb::NOTIFY_MODE_NORMAL ||
             event.mode() as u32 == xcb::NOTIFY_MODE_UNGRAB) {
            return;
        }

        // We should only receive these from child windows we've tracked, so if in current workspace we set input focus
        if self.desktop.current().windows.contains(event.event()).is_some() {
            self.conn.set_input_focus(event.event());
        } else {
            debug!("Received enter notify event for window either not in current workspace or non-tracked!");
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

        // Unset the running flag
        self.running = false;
    }
}
