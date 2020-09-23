use crate::config::{WIN_WIDTH_MIN, WIN_HEIGHT_MIN};
use crate::helper;
use crate::screen::Screen;
use crate::x::{XConn, XWindow};

use std::collections::VecDeque;

fn ensure_in_bounds(val: &mut i32, min: i32, max: i32) {
    if *val < min {
        *val = min;
    } else if *val > max {
        *val = max;
    }
}

#[derive(Copy, Clone)]
pub struct Window {
    id: xcb::Window,

    pub x: i32,
    pub y: i32,

    pub width: i32,
    pub height: i32,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}

impl From<xcb::Window> for Window {
    fn from(window_id: xcb::Window) -> Self {
        Self {
            id: window_id,

            x: 0,
            y: 0,

            width: 0,
            height: 0,
        }
    }
}

impl XWindow for Window {
    fn id(&self) -> xcb::Window {
        return self.id;
    }

    fn set(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }
}

impl Window {
    pub fn do_resize(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current size values
        self.width += dx;
        self.height += dy;

        // Ensure the window sizes are within set bounds
        ensure_in_bounds(&mut self.width, WIN_WIDTH_MIN as i32, screen.x + screen.width - self.x);
        ensure_in_bounds(&mut self.height, WIN_HEIGHT_MIN as i32, screen.y + screen.height - self.y);

        // Untrack before configuration
        conn.change_window_attributes(self.id, &helper::values_attributes_no_events());

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_resize(self.width as u32, self.height as u32));

        // Re-enable tracking
        conn.change_window_attributes(self.id, &helper::values_attributes_child_events());
    }

    // we're using this name because `move` is reseeeeerved, bleh
    pub fn do_move(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current position values
        self.x += dx;
        self.y += dy;

        // Ensure the window coords are within set bounds (still pick-up-able)
        ensure_in_bounds(&mut self.x, screen.x - self.width  + 10, screen.x + screen.width  - 10);
        ensure_in_bounds(&mut self.y, screen.y - self.height + 10, screen.y + screen.height - 10);

        // Untrack before configuration
        conn.change_window_attributes(self.id, &helper::values_attributes_no_events());

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_move(self.x as u32, self.y as u32));

        // Re-enable tracking
        conn.change_window_attributes(self.id, &helper::values_attributes_child_events());
    }
}

#[derive(Default)]
pub struct Windows {
    windows: VecDeque<Window>,
}

impl Windows {
    pub fn len(&self) -> usize {
        return self.windows.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.windows.len() == 0;
    }

    pub fn index_of(&self, window_id: xcb::Window) -> Option<usize> {
        let mut idx: usize = 0;
        for w in self.windows.iter() {
            if w.id == window_id {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }

    pub fn add(&mut self, window: Window) {
        self.windows.push_front(window);
    }

    pub fn remove(&mut self, idx: usize) {
        self.windows.remove(idx);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        return self.windows.iter();
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        return self.windows.iter_mut();
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &Window> {
        return self.windows.iter().rev();
    }

    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        return self.windows.iter_mut().rev();
    }

    pub fn get(&self, idx: usize) -> Option<&Window> {
        return self.windows.get(idx);
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Window> {
        return self.windows.get_mut(idx);
    }

    pub fn contains(&self, window_id: xcb::Window) -> Option<usize> {
        let mut idx: usize = 0;
        for window in self.windows.iter() {
            if window.id == window_id {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }

    pub fn is_focused(&self, window_id: xcb::Window) -> bool {
        match self.focused() {
            Some(window) => return window_id == window.id,
            None => return false,
        }
    }

    pub fn focused(&self) -> Option<&Window> {
        return self.windows.get(0);
    }

    pub fn focused_mut(&mut self) -> Option<&mut Window> {
        return self.windows.get_mut(0);
    }
}