use crate::layout::{floating, LayoutType};
use crate::screen::Screen;
use crate::windows::{Window, Windows};
use crate::x::{XConn, XWindowID};

pub struct Workspace {
    // Internal window id tracking
    pub windows: Windows,

    // Track if Workspace active (on-screen)
    pub active:  bool,

    // Layout functions
    // You're probably asking yourself, why are we doing it like this? Instead of say,
    // holding onto an object that implements a Layout trait? Well doing it that way
    // significantly complicated things in other ways, and doing it this way allows us
    // to abstract away these methods and changing them on the fly with no performance
    // penalty unlike other methods e.g.
    // match self.layout {
    //     Layout::Tiling => {},
    //     Layout::Floating => {},
    //     Layout::SomeOther => {},
    // }
    //
    // And as it turns out, having these functions implemented as separate modules under
    // src/layout/__.rs is still surprisingly neat :)
    //
    // The other alternative would be having only one interchangeable function e.g.
    // _layout_windows(). Which would simplify the implementation, but reduces performance
    // as it means performing a full re-layout on every single window change.
    //
    // If you're reading this and you have more ideas please do let me know, I'm open to them :p
    _activate:             fn(&mut Workspace, &XConn, &Screen),
    _deactivate:           fn(&mut Workspace, &XConn),
    _window_add:           fn(&mut Workspace, &XConn, &Screen, Window),
    _window_del:           fn(&mut Workspace, &XConn, &Screen, usize, XWindowID) -> Window,
    _window_focus:         fn(&mut Workspace, &XConn, &Screen, XWindowID),
    _window_focus_cycle:   fn(&mut Workspace, &XConn, &Screen),
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
            _window_focus_cycle: floating::window_focus_cycle,
        }
    }
}

impl Workspace {
    pub fn set_layout(&mut self, conn: &XConn, screen: &Screen, t: LayoutType) {
        match t {
            LayoutType::Floating => {
                debug!("Switching to layout: floating");
                self._activate = floating::activate;
                self._deactivate = floating::deactivate;
                self._window_add = floating::window_add;
                self._window_del = floating::window_del;
                self._window_focus = floating::window_focus;
                self._window_focus_cycle = floating::window_focus_cycle;
            },
        }
    }

    pub fn activate(&mut self, conn: &XConn, screen: &Screen) {
        debug!("Activating workspace");
        (self._activate)(self, conn, screen);
        self.active = true;
    }

    pub fn deactivate(&mut self, conn: &XConn) {
        debug!("Deactivating workspace");
        (self._deactivate)(self, conn);
        self.active = false;
    }

    pub fn window_add(&mut self, conn: &XConn, screen: &Screen, window: Window) {
        debug!("Adding window to workspace: {}", window.xwindow.id);
       (self._window_add)(self, conn, screen, window);
    }

    pub fn window_del(&mut self, conn: &XConn, screen: &Screen, idx: usize, window_id: XWindowID) -> Window {
        debug!("Deleting window at index {} from workspace: {}", idx, window_id);
        return (self._window_del)(self, conn, screen, idx, window_id);
    }

    pub fn window_del_focused(&mut self, conn: &XConn, screen: &Screen) -> Option<Window> {
        if let Some(focused) = self.windows.focused() {
            // Take ownership
            let focused = focused.to_owned();

            // Remove from current workspace
            self.window_del(conn, screen, 0, focused.xwindow.id);

            // Return id of previously focused window
            return Some(focused);
        }
        return None;
    }

    pub fn window_focus(&mut self, conn: &XConn, screen: &Screen, window_id: XWindowID) {
        debug!("Focusing window in workspace: {}", window_id);
        (self._window_focus)(self, conn, screen, window_id);
    }

    pub fn window_focus_cycle(&mut self, conn: &XConn, screen: &Screen) {
        debug!("Cycling focused window");
        (self._window_focus_cycle)(self, conn, screen);
    }
}