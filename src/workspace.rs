use crate::layout::{floating, tiling, LayoutType};
use crate::screen::Screen;
use crate::windows::Windows;
use crate::xconn::XConn;

pub struct Workspace {
    // Internal window id tracking
    pub windows: Windows,

    // Track if Workspace active (on-screen)
    active:  bool,

    // Layout functions
    _activate:           fn(&mut Workspace, &XConn, &Screen),
    _deactivate:         fn(&mut Workspace, &XConn),
    _window_add:         fn(&mut Workspace, &XConn, &Screen, xcb::Window),
    _window_del:         fn(&mut Workspace, &XConn, &Screen, xcb::Window),
    _window_del_focused: fn(&mut Workspace, &XConn, &Screen) -> Option<xcb::Window>,
    _window_focus:       fn(&mut Workspace, &XConn, &Screen, xcb::Window),
    _window_focus_idx:   fn(&mut Workspace, &XConn, &Screen, usize),
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            windows: Windows::default(),
            active:  false,

            _activate: floating::activate,
            _deactivate: floating::deactivate,
            _window_add: floating::window_add,
            _window_del: floating::window_del,
            _window_del_focused: floating::window_del_focused,
            _window_focus: floating::window_focus,
            _window_focus_idx: floating::window_focus_idx,
        }
    }
}

impl Workspace {
    pub fn set_layout(&mut self, conn: &XConn, screen: &Screen, t: LayoutType) {
        match t {
            LayoutType::Floating => {
                outlog::debug!("Switching to layout: floating");
                self._activate = floating::activate;
                self._deactivate = floating::deactivate;
                self._window_add = floating::window_add;
                self._window_del = floating::window_del;
                self._window_del_focused = floating::window_del_focused;
                self._window_focus = floating::window_focus;
                self._window_focus_idx = floating::window_focus_idx;
            },

            LayoutType::Tiling => {
                outlog::debug!("Switching to layout: tiling");
                self._activate = tiling::activate;
                self._deactivate = tiling::deactivate;
                self._window_add = tiling::window_add;
                self._window_del = tiling::window_del;
                self._window_del_focused = tiling::window_del_focused;
                self._window_focus = tiling::window_focus;
                self._window_focus_idx = tiling::window_focus_idx;

                // Lay things out as they should be
                tiling::perform_layout(self, conn, screen);
            },
        }
    }
    pub fn activate(&mut self, conn: &XConn, screen: &Screen) {
        outlog::debug!("Activating workspace");
        (self._activate)(self, conn, screen);
        self.active = true;
    }

    pub fn deactivate(&mut self, conn: &XConn) {
        outlog::debug!("Deactivating workspace");
        (self._deactivate)(self, conn);
        self.active = false;
    }

    pub fn window_add(&mut self, conn: &XConn, screen: &Screen, window: xcb::Window) {
        outlog::debug!("Adding window to workspace");
       (self._window_add)(self, conn, screen, window);
    }

    pub fn window_del(&mut self, conn: &XConn, screen: &Screen, window: xcb::Window) {
        outlog::debug!("Deleting window from workspace");
        (self._window_del)(self, conn, screen, window);
    }

    pub fn window_del_focused(&mut self, conn: &XConn, screen: &Screen) -> Option<xcb::Window> {
        outlog::debug!("Deleting focused window from workspace");
        return (self._window_del_focused)(self, conn, screen);
    }

    pub fn window_focus(&mut self, conn: &XConn, screen: &Screen, window: xcb::Window) {
        outlog::debug!("Focusing window in workspace");
        (self._window_focus)(self, conn, screen, window);
    }

    pub fn window_focus_idx(&mut self, conn: &XConn, screen: &Screen, idx: usize) {
        outlog::debug!("Focusing window at index in workspace");
        (self._window_focus_idx)(self, conn, screen, idx);
    }

    pub fn is_active(&self) -> bool {
        return self.active;
    }

    pub fn window_focus_cycle(&mut self, conn: &XConn, screen: &Screen) {
        // Get length just the once
        let len = self.windows.len();

        // If length < 2 nothing to do
        if len < 2 {
            return;
        }

        // Focus last window
        self.window_focus_idx(conn, screen, len-1);
    }

    pub fn window_close_focused(&self, conn: &XConn) {
        // Log
        outlog::debug!("Closing focused window in workspace");

        // If there is a focused window, close
        if let Some(window) = self.windows.focused() {
            conn.window_close(window.id);
        }
    }
}