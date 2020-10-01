use crate::config::{WIN_WIDTH_MIN, WIN_HEIGHT_MIN};
use crate::helper;
use crate::screen::Screen;
use crate::x::{XConn, XWindow};

use std::collections::VecDeque;

const MIN_SCREEN_ONSCREEN: i32 = 10;

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
        ensure_in_bounds(&mut self.width,  WIN_WIDTH_MIN  as i32, screen.x + screen.width  - self.x);
        ensure_in_bounds(&mut self.height, WIN_HEIGHT_MIN as i32, screen.y + screen.height - self.y);

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_resize(self.width as u32, self.height as u32));
    }

    // we're using this name because `move` is reseeeeerved, bleh
    pub fn do_move(&mut self, conn: &XConn, screen: &Screen, dx: i32, dy: i32) {
        // Iterate current position values
        self.x += dx;
        self.y += dy;

        // Ensure the window coords are within set bounds (still pick-up-able)
        ensure_in_bounds(&mut self.x, screen.x - self.width  + MIN_SCREEN_ONSCREEN, screen.x + screen.width  - MIN_SCREEN_ONSCREEN);
        ensure_in_bounds(&mut self.y, screen.y - self.height + MIN_SCREEN_ONSCREEN, screen.y + screen.height - MIN_SCREEN_ONSCREEN);

        // Send new window configuration to X
        conn.configure_window(self.id, &helper::values_configure_move(self.x as u32, self.y as u32));
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

    pub fn index_of(&self, window_id: xcb::Window) -> Option<usize> {
        let mut idx: usize = 0;
        for w in self.0.iter() {
            if w.id == window_id {
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

    pub fn contains(&self, window_id: xcb::Window) -> Option<usize> {
        let mut idx: usize = 0;
        for window in self.0.iter() {
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
        return self.0.get(0);
    }

    pub fn focused_mut(&mut self) -> Option<&mut Window> {
        return self.0.get_mut(0);
    }
}