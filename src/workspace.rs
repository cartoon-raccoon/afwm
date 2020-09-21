use crate::layout::{floating, LayoutType};
use crate::screen::Screen;
use crate::windows::Windows;
use crate::x::{XConn, XWindow};

pub struct Workspace {
    // Internal window id tracking
    pub windows: Windows,

    // Track if Workspace active (on-screen)
    active:  bool,

    // Layout functions
    _activate:             fn(&mut Workspace, &XConn, &Screen),
    _deactivate:           fn(&mut Workspace, &XConn),
    _window_add:           fn(&mut Workspace, &XConn, &Screen, xcb::Window),
    _window_del:           fn(&mut Workspace, &XConn, &Screen, usize, xcb::Window),
    _window_focus:         fn(&mut Workspace, &XConn, &Screen, xcb::Window),
    _window_focus_idx:     fn(&mut Workspace, &XConn, &Screen, usize),
    _window_focus_cycle:   fn(&mut Workspace, &XConn, &Screen),
    _window_close_focused: fn(&mut Workspace, &XConn, &Screen),
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
            _window_focus: floating::window_focus,
            _window_focus_idx: floating::window_focus_idx,
            _window_focus_cycle: floating::window_focus_cycle,
            _window_close_focused: floating::window_close_focused,
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
                self._window_focus = floating::window_focus;
                self._window_focus_idx = floating::window_focus_idx;
                self._window_focus_cycle = floating::window_focus_cycle;
                self._window_close_focused = floating::window_close_focused;
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

    pub fn window_add(&mut self, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
        outlog::debug!("Adding window to workspace: {}", window_id);
       (self._window_add)(self, conn, screen, window_id);
    }

    pub fn window_del(&mut self, conn: &XConn, screen: &Screen, idx: usize, window_id: xcb::Window) {
        outlog::debug!("Deleting window at index {} from workspace: {}", idx, window_id);
        (self._window_del)(self, conn, screen, idx, window_id);
    }

    pub fn window_del_focused(&mut self, conn: &XConn, screen: &Screen) -> Option<xcb::Window> {
        if let Some(focused) = self.windows.focused() {
            // Take ownership
            let focused = *focused;

            // Remove from current workspace
            self.window_del(conn, screen, 0, focused.id());

            return Some(focused.id());
        }
        return None;
    }

    pub fn window_focus(&mut self, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
        outlog::debug!("Focusing window in workspace: {}", window_id);
        (self._window_focus)(self, conn, screen, window_id);
    }

    pub fn window_focus_idx(&mut self, conn: &XConn, screen: &Screen, idx: usize) {
        outlog::debug!("Focusing window at index in workspace: {}", idx);
        (self._window_focus_idx)(self, conn, screen, idx);
    }

    pub fn window_focus_cycle(&mut self, conn: &XConn, screen: &Screen) {
        outlog::debug!("Cycling focused window");
        (self._window_focus_cycle)(self, conn, screen);
    }

    pub fn window_close_focused(&mut self, conn: &XConn, screen: &Screen) {
        outlog::debug!("Closing focused window");
        (self._window_close_focused)(self, conn, screen);
    }

    pub fn is_active(&self) -> bool {
        return self.active;
    }
}