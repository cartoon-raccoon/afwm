use crate::config::{WIN_WIDTH_MIN, WIN_HEIGHT_MIN};
use crate::helper;
use crate::screen::Screen;
use crate::x::{XConn, XWindow, XWindowID};

use std::collections::{VecDeque, HashSet};

const MIN_SCREEN_ONSCREEN: i32 = 10;

fn ensure_in_bounds(val: &mut i32, min: i32, max: i32) {
    if *val < min {
        *val = min;
    } else if *val > max {
        *val = max;
    }
}

#[derive(Clone)]
pub struct Window {
    pub xwindow: XWindow,
    protocols: HashSet<xcb::Atom>,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        return self.xwindow.id == other.xwindow.id;
    }
}

impl From<XWindowID> for Window {
    fn from(window_id: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(window_id),
            protocols: HashSet::new(),
        }
    }
}

impl Window {
    pub fn do_resize(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current size values
        self.xwindow.width += dx;
        self.xwindow.height += dy;

        // Ensure the window sizes are within set bounds
        ensure_in_bounds(&mut self.xwindow.width,  WIN_WIDTH_MIN  as i32, screen.xwindow.x + screen.xwindow.width  - self.xwindow.x);
        ensure_in_bounds(&mut self.xwindow.height, WIN_HEIGHT_MIN as i32, screen.xwindow.y + screen.xwindow.height - self.xwindow.y);

        // Send new window configuration to X
        conn.configure_window(self.xwindow.id, &helper::values_configure_resize(self.xwindow.width as u32, self.xwindow.height as u32));
    }

    pub fn do_move(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current position values
        self.xwindow.x += dx;
        self.xwindow.y += dy;

        // Ensure the window coords are within set bounds (still pick-up-able)
        ensure_in_bounds(&mut self.xwindow.x, screen.xwindow.x - self.xwindow.width  + MIN_SCREEN_ONSCREEN, screen.xwindow.x + screen.xwindow.width  - MIN_SCREEN_ONSCREEN);
        ensure_in_bounds(&mut self.xwindow.y, screen.xwindow.y - self.xwindow.height + MIN_SCREEN_ONSCREEN, screen.xwindow.y + screen.xwindow.height - MIN_SCREEN_ONSCREEN);

        // Send new window configuration to X
        conn.configure_window(self.xwindow.id, &helper::values_configure_move(self.xwindow.x as u32, self.xwindow.y as u32));
    }

    pub fn set_supported_protocols(&mut self, conn: &XConn) {
        // Attempt to get wm protocols for window, and add to our
        // hashset of supported atoms
        if let Some(protocols) = conn.get_wm_protocols(self.xwindow.id) {
            for protocol in protocols {
                debug!("{}", conn._get_atom_name(protocol));
                self.protocols.insert(protocol);
            }
        }
    }

    pub fn supports_protocol(&self, atom: &xcb::Atom) -> bool {
        return self.protocols.contains(atom);
    }
}

#[derive(Default)]
pub struct Windows(VecDeque<Window>);

impl Windows {
    pub fn len(&self) -> usize {
        return self.0.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.0.len() == 0;
    }

    pub fn move_front(&mut self, idx: usize) {
        // Only swap with front if window isn't already there
        if idx != 0 { self.0.swap(0, idx); }
    }

    pub fn index_of(&self, window_id: XWindowID) -> Option<usize> {
        let mut idx: usize = 0;
        for window in self.0.iter() {
            if window.xwindow.id == window_id {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }

    pub fn add(&mut self, window: Window) {
        self.0.push_front(window);
    }

    pub fn remove(&mut self, idx: usize) {
        self.0.remove(idx);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        return self.0.iter();
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &Window> {
        return self.0.iter().rev();
    }

    pub fn get(&self, idx: usize) -> Option<&Window> {
        return self.0.get(idx);
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Window> {
        return self.0.get_mut(idx);
    }

    pub fn contains(&self, window_id: XWindowID) -> Option<usize> {
        let mut idx: usize = 0;
        for window in self.0.iter() {
            if window.xwindow.id == window_id {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }

    pub fn is_focused(&self, window_id: XWindowID) -> bool {
        match self.focused() {
            Some(window) => return window_id == window.xwindow.id,
            None => return false,
        }
    }

    pub fn focused(&self) -> Option<&Window> {
        return self.0.get(0);
    }

    pub fn focused_mut(&mut self) -> Option<&mut Window> {
        return self.0.get_mut(0);
    }
}