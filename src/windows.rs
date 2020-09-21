use crate::helper;
use crate::screen::Screen;
use crate::x::{XConn, XWindow};

use std::collections::VecDeque;

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

        // If at screen max, scale it back
        let end_x = screen.x + screen.width;
        if self.x + self.width > end_x {
            self.width = end_x - self.x;
        }
        let end_y = screen.y + screen.height;
        if self.y + self.height > end_y {
            self.height = end_y - self.y;
        }

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_resize(self.width as u32, self.height as u32));
    }

    // we're using this name because `move` is reseeeeerved, bleh
    pub fn do_move(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current position values
        self.x += dx;
        self.y += dy;

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_move(self.x as u32, self.y as u32));
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