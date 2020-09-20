use crate::config::BAR_SIZE;
use crate::xconn::XConn;

pub struct Screen {
    pub x: i32,
    pub y: i32,

    pub width: i32,
    pub height: i32,

    bar: i32,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,

            width: 0,
            height: 0,

            bar: BAR_SIZE as i32,
        }
    }
}

impl Screen {
    pub fn update_geometry(&mut self, conn: &XConn) {
        // Get new window geometry
        let (x, y, w, h) = conn.get_geometry(conn.root);

        // Set x, y start
        self.x = x;
        self.y = y + self.bar; // starts after bar

        // Set sizes
        self.width = w;
        self.height = h - self.bar;
    }
}