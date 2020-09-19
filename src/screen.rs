use crate::config::BAR_SIZE;
use crate::xconn::XConn;

pub struct Screen {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            x: 0,
            y: BAR_SIZE as i32,
            width: 0,
            height: 0,
        }
    }
}

impl Screen {
    pub fn update(&mut self, conn: &XConn) {
        // Get new window geometry
        let (x, y, w, h) = conn.get_geometry(conn.root);

        // Set new window geometry
        self.width = w;
        self.height = h - (BAR_SIZE as i32);
    }
}