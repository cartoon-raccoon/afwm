use crate::helper;

use xcb_util::{cursor, ewmh, icccm};
use xcb_util::keysyms::KeySymbols;

pub enum CursorIndex {
    LeftPtr,
}

pub trait XWindow {
    fn id(&self) -> xcb::Window;
    fn set(&mut self, x: i32, y: i32, width: i32, height: i32);
}

pub struct InternedAtoms {
    pub SUPPORTED:              xcb::Atom,
    pub WM_DELETE_WINDOW:       xcb::Atom,
    pub WM_PROTOCOLS:           xcb::Atom,
    pub WM_WINDOW_TYPE_NORMAL:  xcb::Atom,
    pub WM_WINDOW_TYPE_DIALOG:  xcb::Atom,
    pub WM_WINDOW_TYPE_TOOLBAR: xcb::Atom,
    pub WM_WINDOW_TYPE_UTILITY: xcb::Atom,
    pub WM_WINDOW_TYPE_SPLASH:  xcb::Atom,
}

pub struct XConn<'a> {
    // X server connection
    pub conn: &'a ewmh::Connection,

    // Stored loaded cursor ids
    cursors: [u32; 1],

    // KeySymbol lookup object
    key_syms: KeySymbols<'a>,

    // Interned atoms
    pub atoms: InternedAtoms,
}

impl<'a> XConn<'a> {
    pub fn new(conn: &'a ewmh::Connection) -> Self {
        // Create new atoms object
        let atoms = InternedAtoms {
            SUPPORTED:              conn.SUPPORTED(),
            WM_DELETE_WINDOW:       xcb::intern_atom(conn, false, "WM_DELETE_WINDOW").get_reply().expect("Interning WM_DELETE_WINDOW atom").atom(),
            WM_PROTOCOLS:           conn.WM_PROTOCOLS(),
            WM_WINDOW_TYPE_NORMAL:  conn.WM_WINDOW_TYPE_NORMAL(),
            WM_WINDOW_TYPE_DIALOG:  conn.WM_WINDOW_TYPE_DIALOG(),
            WM_WINDOW_TYPE_TOOLBAR: conn.WM_WINDOW_TYPE_TOOLBAR(),
            WM_WINDOW_TYPE_UTILITY: conn.WM_WINDOW_TYPE_UTILITY(),
            WM_WINDOW_TYPE_SPLASH:  conn.WM_WINDOW_TYPE_SPLASH(),
        };

        // Create new Self
        let new = Self {
            conn:     conn,
            cursors:  [0; 1],
            key_syms: KeySymbols::new(conn),
            atoms:    atoms,
        };

        // Return the new Self
        return new;
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

    pub fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]) {
        debug!("Set supported atoms for screen: {}\n{}", screen_idx, self._atom_slice_as_string(atoms));

        // Set supplied atoms slice as all supported
        ewmh::set_supported(self.conn, screen_idx, &atoms);
    }

    pub fn get_setup(&self) -> xcb::Setup {
        debug!("Getting setup");

        // Return the current X connection's setup
        return self.conn.get_setup();
    }

    pub fn query_tree(&self, window_id: xcb::Window) -> Vec<xcb::Window> {
        debug!("Querying tree");

        // Query tree for window. Don't bother checking returning option, we only ever query tree for root (for now...)
        return xcb::query_tree(self.conn, window_id).get_reply().expect("Querying tree").children().to_owned();
    }

    pub fn map_window(&self, window_id: xcb::Window) {
        debug!("Mapping window: {}", window_id);

        // Map window. Don't bother checking, if it failed, it failed :shrug:
        xcb::map_window(self.conn, window_id);
    }

    pub fn unmap_window(&self, window_id: xcb::Window) {
        debug!("Unmapping window: {}", window_id);

        // Unmap window. Don't bother checking, if it failed, it failed :shrug:
        xcb::unmap_window(self.conn, window_id);
    }

    pub fn configure_window(&self, window_id: xcb::Window, values: &[(u16, u32)]) {
        debug!("Configuring window: {}", window_id);

        // Configure window. Don't bother checking, if it failed, it failed :shrug:
        xcb::configure_window(self.conn, window_id, values);
    }

    pub fn change_window_attributes(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);

        // Change window attributes. Don't bother checking, if it failed, it failed :shrug:
        xcb::change_window_attributes(self.conn, window_id, values);
    }

    pub fn change_window_attributes_checked(&self, window_id: xcb::Window, values: &[(u32, u32)]) {
        debug!("Changing window attributes: {}", window_id);

        // Change window attributes, ensure it goes through okay!
        xcb::change_window_attributes_checked(self.conn, window_id, values).request_check().expect("Changing window attributes");
    }

    pub fn set_input_focus(&self, window_id: xcb::Window) {
        debug!("Setting input focus window: {}", window_id);

        // Set input focus on window. Don't bother checking, if it failed, it failed :shrug:
        xcb::set_input_focus(self.conn, xcb::INPUT_FOCUS_POINTER_ROOT as u8, window_id, xcb::CURRENT_TIME);
    }

    pub fn destroy_window(&self, window_id: xcb::Window) {
        debug!("Destroying window: {}", window_id);

        // First check we can get the protocols for window, if not then it was probably closed already
        let protocols = self.get_wm_protocols(window_id);
        if protocols.is_none() { return; }
        let protocols = protocols.unwrap();

        // Now check how best to destroy window
        if protocols.contains(&self.atoms.WM_DELETE_WINDOW) {
            // Window support ICCCM method of WM_DELETE_WINDOW
            debug!("Destroy window via ICCCM WM_DELETE_WINDOW");

            // Create client message data
            let msg_data = xcb::ClientMessageData::from_data32([self.atoms.WM_DELETE_WINDOW, xcb::CURRENT_TIME, 0, 0, 0]);

            // Create event from message data
            let event = xcb::ClientMessageEvent::new(32, window_id, self.atoms.WM_PROTOCOLS, msg_data);

            // Send the event!
            xcb::send_event(
                self.conn,                // connection
                false,                    // propagate?
                window_id,                // destination window
                xcb::EVENT_MASK_NO_EVENT, // event mask
                &event,                   // event object
            );
        } else {
            // Use plain-old X destroy window
            debug!("Destroy window via xcb_destroy_window");
            xcb::destroy_window(self.conn, window_id);
        }
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

        // Register key code to grab with X. We don't bother checking as only ever for root window
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

        // Register button to grab with X. We don't bother checking as only ever for root window
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

    pub fn grab_pointer(&self, window_id: xcb::Window, mask: xcb::EventMask) {
        debug!("Grabbing pointer for window: {}", window_id);

        // Register to grab pointer. We don't bother checking as only ever for root window
        xcb::grab_pointer(
            self.conn,
            false,                                       // owner events (a.k. don't pass on events to root window)
            window_id,                                   // grab window, i.e. where to grab pointer movement
            mask as u16,                                 // event mask
            xcb::GRAB_MODE_ASYNC as u8,                  // pointer mode
            xcb::GRAB_MODE_ASYNC as u8,                  // keyboard mode
            xcb::NONE,                                   // confine to window
            xcb::NONE,                                   // cursor to display
            xcb::CURRENT_TIME,                           // time
        );
    }

    pub fn ungrab_pointer(&self) {
        debug!("Ungrabbing pointer");

        // Unregister grabbing the pointer. We don't bother checking as only ever for root window
        xcb::ungrab_pointer(self.conn, xcb::CURRENT_TIME);
    }

    pub fn update_geometry(&self, window: &mut impl XWindow) {
        // Try get window geometry for current window, update if so
        if let Some((x, y, width, height)) = self.get_geometry(window.id()) {
            window.set(x, y, width, height);
        }
    }

    pub fn get_geometry(&self, window_id: xcb::Window) -> Option<(i32, i32, i32, i32)> {
        debug!("Getting geometry for window: {}", window_id);
        match xcb::get_geometry(self.conn, window_id).get_reply() {
            Ok(dimens) => return Some((dimens.x() as i32, dimens.y() as i32, dimens.width() as i32, dimens.height() as i32)),
            Err(err) => {
                warn!("Failed getting window geometry for {} ({}). Was window closed and not yet unmapped?", window_id, err);
                return None;
            },
        }
    }

    pub fn get_window_attributes(&self, window_id: xcb::Window) -> Option<xcb::GetWindowAttributesReply> {
        debug!("Getting attributes for window: {}", window_id);
        match xcb::get_window_attributes(self.conn, window_id).get_reply() {
            Ok(reply) => return Some(reply),
            Err(err) => {
                warn!("Failed getting attributes for window {} ({}). Was window closed and not yet unmapped?", window_id, err);
                return None;
            }
        }
    }

    pub fn get_wm_protocols(&self, window_id: xcb::Window) -> Option<Vec<xcb::Atom>> {
        debug!("Getting wm protocols for window: {}", window_id);
        match icccm::get_wm_protocols(self.conn, window_id, self.atoms.WM_PROTOCOLS).get_reply() {
            Ok(reply) => return Some(reply.atoms().to_owned()),
            Err(_) => return None,
        }
    }

    pub fn get_wm_window_type(&self, window_id: xcb::Window) -> Option<Vec<xcb::Atom>> {
        debug!("Getting wm type for window: {}", window_id);
        match ewmh::get_wm_window_type(self.conn, window_id).get_reply() {
            Ok(reply) => return Some(reply.atoms().to_owned()),
            Err(_) => return None,
        }
    }

    pub fn query_pointer(&self, window_id: xcb::Window) -> (i32, i32, xcb::Window) {
        debug!("Querying pointer location for window: {}", window_id);

        // We don't bother requesting check here as this is only ever used for root window
        let pointer = xcb::query_pointer(self.conn, window_id).get_reply().expect("Querying window pointer location");
        return (pointer.root_x() as i32, pointer.root_y() as i32, pointer.child())
    }

    #[cfg(debug_assertions)]
    pub fn _get_atom_name(&self, atom: xcb::Atom) -> String {
        // don't debug log because it's being used for debug anyway
        return xcb::get_atom_name(self.conn, atom).get_reply().expect("Getting atom name").name().to_owned();
    }

    #[cfg(debug_assertions)]
    pub fn _atom_slice_as_string(&self, atoms: &[xcb::Atom]) -> String {
        // don't debug log because it's being used for debug anyway
        let mut fmt = String::new();
        for a in atoms.iter() {
            fmt.push_str(&format!("> {}\n", self._get_atom_name(*a)));
        }
        return fmt;
    }

    pub fn lookup_keysym(&self, event: &xcb::KeyPressEvent) -> (xcb::ModMask, xcb::Keysym) {
        // Get keysym for event
        let keysym = self.key_syms.press_lookup_keysym(event, 0);

        // Create new tuple of (mod_mask, key_sym)
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
