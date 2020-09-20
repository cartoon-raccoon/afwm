use crate::config::KEYBINDS;
use crate::desktop::Desktop;
use crate::event::{Event, MouseButton};
use crate::screen::Screen;
use crate::xconn::XConn;

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

    // Should we continue running?
    running: bool,
}

impl<'a> WM<'a> {
    pub fn new(conn: &'a xcb::Connection, screen_idx: i32, keys: &Vec<(xcb::ModMask, xcb::Keysym)>) -> Self {
        // Register our XConn
        let xconn = XConn::register(conn, screen_idx, keys);

        // Create new WM object
        let mut new = Self {
            conn: xconn,
            desktop: Desktop::default(),
            screen:  Screen::default(),
            mouse_mode: MouseMode::Ground,
            last_mouse_x: 0,
            last_mouse_y: 0,
            running: true,
        };

        // Perform initial screen geometry fetch
        new.screen.update_geometry(&new.conn);

        // Return new self!
        return new;
    }

    pub fn run(&mut self) {
        outlog::info!("Started running");

        while self.running {
            // Get next event
            let event = self.conn.next_event();

            // Handle event
            match event {
                Event::MapRequest(window_id) => {
                    if let Some((ws, _)) = self.desktop.contains_mut(window_id) {
                        // We already have this window, if in the current then focus!
                        if ws.is_active() {
                            ws.window_focus(&self.conn, &self.screen, window_id);
                        }
                    } else {
                        // Add to current workspace
                        self.desktop.current_mut().window_add(&self.conn, &self.screen, window_id);
                    }
                },

                Event::UnmapNotify(window_id) => {
                    // Remove window (if there!)
                    if let Some((ws, idx)) = self.desktop.contains_mut(window_id) {
                        ws.window_del(&self.conn, &self.screen, window_id);

                        // View may have changed
                        self.screen.update_geometry(&self.conn);
                    }
                },

                Event::DestroyNotify(window_id) => {
                    // Remove window (if there!)
                    if let Some((ws, idx)) = self.desktop.contains_mut(window_id) {
                        ws.window_del(&self.conn, &self.screen, window_id);

                        // View may have changed
                        self.screen.update_geometry(&self.conn);
                    }
                },

                Event::EnterNotify(window_id) => {
                    // Focus this windo
                    self.conn.window_focus(window_id);
                },

                Event::MotionNotify => {
                    // If no tracked windows, nothing to do
                    if self.desktop.current().windows.is_empty() {
                        continue;
                    }

                    // Get current pointer location
                    let (px, py, _) = self.conn.get_pointer(self.conn.root);

                    // Calculate dx, dy
                    let dx = (px - self.last_mouse_x) as i32;
                    let dy = (py - self.last_mouse_y) as i32;

                    // Set new last mouse positions
                    self.last_mouse_x = px;
                    self.last_mouse_y = py;

                    // React depending on current MouseMode
                    match self.mouse_mode {
                        MouseMode::Move => {
                            // Move currently focused window
                            self.desktop.current_mut().windows.focused_mut().unwrap().do_move(&self.conn, &self.screen, dx, dy);
                        },

                        MouseMode::Resize => {
                            // Resize currently focused window
                            self.desktop.current_mut().windows.focused_mut().unwrap().do_resize(&self.conn, &self.screen, dx, dy);
                        },

                        MouseMode::Ground => panic!("MouseMode::Ground state registered in MotionNotify"),
                    }
                },

                Event::KeyPress((key_ev, window_id)) => {
                    // Try get function for keybind
                    for (mask, key, keyfn) in KEYBINDS {
                        if *mask == key_ev.mask &&
                           *key == key_ev.key {
                            // If window id isn't the focused window id, refocus
                            if !self.desktop.current_mut().windows.is_focused(window_id) {
                                self.desktop.current_mut().window_focus(&self.conn, &self.screen, window_id);
                            }

                            // Execute! And return
                            keyfn(self);
                            break;
                        }
                    }
                },

                Event::ButtonPress((but, window_id)) => {
                    // If no windows, nothing to do
                    if self.desktop.current().windows.is_empty() {
                        continue;
                    }

                    // Get current pointer position
                    let (px, py, _) = self.conn.get_pointer(self.conn.root);
                    self.last_mouse_x = px;
                    self.last_mouse_y = py;

                    // If window id different to focused, focus this one
                    if window_id != self.desktop.current().windows.focused().unwrap().id {
                        self.desktop.current_mut().window_focus(&self.conn, &self.screen, window_id);
                    }

                    // Handle button press
                    match but {
                        MouseButton::LeftClick => {
                            // Enter move mode
                            self.mouse_mode = MouseMode::Move;
                        },

                        MouseButton::RightClick => {
                            // Enter resize mode
                            self.mouse_mode = MouseMode::Resize;
                        },
                    }
                },

                Event::ButtonRelease(_) => {
                    // Regardless of button, current state etc, we unset the mouse mode
                    self.mouse_mode = MouseMode::Ground;
                },
            }
        }

        outlog::info!("Finished running");
    }

    pub fn kill(&mut self) {
        outlog::info!("Killing");
        self.running = false;
    }
}
