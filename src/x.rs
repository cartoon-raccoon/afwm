use crate::helper;

use xcb_util::cursor;
use xcb_util::keysyms::KeySymbols;

pub enum CursorIndex {
    LeftPtr,
}

pub trait XWindow {
    fn id(&self) -> xcb::Window;
    fn set(&mut self, x: i32, y: i32, width: i32, height: i32);
}

pub struct XConn<'a> {
    // X server connection
    pub conn: &'a xcb::Connection,

    // Stored loaded cursors + optional core cursor font
    cursors: [u32; 1],

    // KeySymbol lookup object
    key_syms: KeySymbols<'a>,
}

impl<'a> XConn<'a> {
    pub fn new(conn: &'a xcb::Connection) -> Self {
        Self {
            conn:             conn,
            cursors:          [0; 1],
            key_syms:         KeySymbols::new(conn),
        }
    }

    pub fn create_core_cursor(&mut self, cursor: CursorIndex, cursor_glyph: u16) {
        // Try load cursor for supplied cursor glyp
        let cursor_id = cursor::create_font_cursor_checked(self.conn, cursor_glyph).expect("Creating font cursor");

        // Store the cursor id in the cursors array at supplied index
        self.cursors[cursor as usize] = cursor_id;
    }

    pub fn set_cursor(&mut self, window_id: xcb::Window, cursor: CursorIndex) {
        // Get the cursor id at index in the stored cursors array
        let cursor_id = self.cursors[cursor as usize];

        // Set the window cursor attributes
        self.change_window_attributes(window_id, &helper::values_attributes_cursor(cursor_id));
    }

    pub fn get_setup(&self) -> xcb::Setup {
        debug!("Getting setup");
        return self.conn.get_setup();
    }

    pub fn map_window(&self, window_id: xcb::Window) {
        debug!("Mapping window: {}", window_id);
        xcb::map_window(self.conn, window_id);
    }

    pub fn unmap_window(&self, window_id: xcb::Window) {
        debug!("Unmapping window: {}", window_id);
        xcb::unmap_window(self.conn, window_id);
    }

    pub fn configure_window(&self, window_id: xcb::Window, values: &[(u16, u32)]) {
        debug!("Configuring window: {}", window_id);
        xcb::configure_window(self.conn, window_id, values);
    }

    pub fn change_window_attributes(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);
        xcb::change_window_attributes(self.conn, window_id, values);
    }

    pub fn change_window_attributes_checked(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);
        xcb::change_window_attributes_checked(self.conn, window_id, values).request_check().expect("Changing window attributes");
    }

    pub fn set_input_focus(&self, window_id: xcb::Window) {
        debug!("Setting input focus window: {}", window_id);
        xcb::set_input_focus(self.conn, xcb::INPUT_FOCUS_POINTER_ROOT as u8, window_id, xcb::CURRENT_TIME);
    }

    pub fn kill_client(&self, window_id: xcb::Window) {
        debug!("Killing client window: {}", window_id);
        xcb::kill_client(self.conn, window_id);
    }

    pub fn grab_key(&self, window_id: xcb::Window, mask: xcb::ModMask, keysym: xcb::Keysym) {
        debug!("Grabbing key with mask:{} sym:{} for window: {}", mask, keysym, window_id);

        // Get code iter for keysym
        let code = self.key_syms.get_keycode(keysym).next();

        // If no code, log and move-on. Else, unwrap
        if code.is_none() {
            warn!("Keysym {} translated to zero-length keycode iter, not grabbing", keysym);
            return;
        }
        let code = code.unwrap();

        // Register key code to grab with X
        xcb::grab_key(
            self.conn,
            false,                       // owner events (a.k.a don't pass on events to root window)
            window_id,                   // window id
            mask as u16,                 // key mod mask
            code,                        // keycode
            xcb::GRAB_MODE_ASYNC as u8,  // pointer mode
            xcb::GRAB_MODE_ASYNC as u8   // keyboard mode
        );
    }

    pub fn grab_button(&self, window_id: xcb::Window, mask: xcb::ButtonMask, button: xcb::ButtonIndex, modmask: xcb::ModMask, confine: bool) {
        debug!("Grabbing button {} for window: {}", window_id, button);
        xcb::grab_button(
            self.conn,
            false,                                       // owner events (a.k. don't pass on events to root window)
            window_id,                                   // window id
            mask as u16,                                 // button event mask
            xcb::GRAB_MODE_ASYNC as u8,                  // pointer mode
            xcb::GRAB_MODE_ASYNC as u8,                  // keyboard mode
            if confine { window_id } else { xcb::NONE }, // confine pointer to window (or no confine)
            xcb::NONE,                                   // cursor to use
            button as u8,                                // button to grab (right click)
            modmask as u16,                              // Modifiers to grab mouse with
        );
    }

    pub fn grab_pointer(&self, window_id: xcb::Window, mask: xcb::EventMask, confine: bool) {
        debug!("Grabbing pointer for window: {}", window_id);
        xcb::grab_pointer(
            self.conn,
            false,                                       // owner events (a.k. don't pass on events to root window)
            window_id,                                   //
            mask as u16,                                 //
            xcb::GRAB_MODE_ASYNC as u8,                  //
            xcb::GRAB_MODE_ASYNC as u8,                  //
            if confine { window_id } else { xcb::NONE }, //
            xcb::NONE,                                   //
            xcb::CURRENT_TIME,                           //
        );
    }

    pub fn ungrab_pointer(&self) {
        debug!("Ungrabbing pointer");
        xcb::ungrab_pointer(self.conn, xcb::CURRENT_TIME);
    }

    pub fn update_geometry(&self, window: &mut impl XWindow) -> bool {
        match self.get_geometry(window.id()) {
            Some((x, y, width, height)) => {
                // Update supplied window's geometry, return true
                window.set(x, y, width, height);
                return true;
            },

            None => return false,
        }
    }

    pub fn get_geometry(&self, window_id: xcb::Window) -> Option<(i32, i32, i32, i32)> {
        debug!("Getting geometry for window: {}", window_id);
        match xcb::get_geometry(self.conn, window_id).get_reply() {
            Ok(dimens) => return Some((dimens.x() as i32, dimens.y() as i32, dimens.width() as i32, dimens.height() as i32)),
            Err(_) => {
                warn!("Failed getting window geometry for {}. Was window destroyed and not yet unmapped?", window_id);
                return None;
            },
        }
    }

    pub fn query_pointer(&self, window_id: xcb::Window) -> (i32, i32, xcb::Window) {
        debug!("Querying pointer location for window: {}", window_id);
        let pointer = xcb::query_pointer(self.conn, window_id).get_reply().expect("Querying window pointer location");
        return (pointer.root_x() as i32, pointer.root_y() as i32, pointer.child())
    }

    pub fn lookup_keysym(&self, event: &xcb::KeyPressEvent) -> (xcb::ModMask, xcb::Keysym) {
        // Get keysym for event
        let keysym = self.key_syms.press_lookup_keysym(event, 0);

        // Create new key object
        return (event.state() as u32, keysym);
    }

    pub fn next_event(&self) -> xcb::GenericEvent {
        // Flush connection to ensure clean
        self.conn.flush();

        // Check for queued, else wait for next
        return if let Some(event) = self.conn.poll_for_queued_event() {
            event
        } else {
            self.conn.wait_for_event().expect("I/O error getting event from X server")
        };
    }
}