use crate::config::BAR_SIZE;
use crate::x::{XConn, XWindow};

pub struct Screen {
    pub x: i32,
    pub y: i32,

    pub width: i32,
    pub height: i32,

    pub idx: i32,
    root_id: xcb::Window,

//    bar: i32,
}

impl XWindow for Screen {
    fn id(&self) -> xcb::Window {
        return self.root_id;
    }

    fn set(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.x = x;
        self.y = y;
        self.width = width; 
        self.height = height;
    }
}

impl Screen {
    pub fn new(screen_idx: i32, root_id: xcb::Window) -> Self {
        Self {
            x: 0,
            y: 0,

            width: 0,
            height: 0,

            idx: screen_idx,
            root_id: root_id,

//            bar: BAR_SIZE as i32,
        }
    }
}