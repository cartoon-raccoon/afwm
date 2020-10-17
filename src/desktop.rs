use crate::config::WORKSPACES;
use crate::screen::Screen;
use crate::workspace::Workspace;
use crate::x::{XConn, XWindowID};

#[derive(Default)]
pub struct Desktop {
    // Internal workspace tracking
    workspaces: [Workspace; WORKSPACES],

    // Current workspace index
    idx: usize,
}

impl Desktop {
    pub fn index_next(&self) -> usize {
        if self.idx < WORKSPACES-1 {
            return self.idx + 1;
        } else {
            return 0;
        }
    }

    pub fn index_prev(&self) -> usize {
        if self.idx > 0 {
            return self.idx - 1;
        } else {
            return WORKSPACES - 1;
        }
    }

    pub fn goto(&mut self, conn: &XConn, screen: &Screen, idx: usize) {
        // Log
        debug!("Goto workspace: {}", idx);

        // Deactivate current selected workspace
        self.workspaces.get_mut(self.idx).unwrap().deactivate(conn);

        // Update index
        self.idx = idx;

        // Activate newly selected workspace
        self.workspaces.get_mut(self.idx).unwrap().activate(conn, screen);
    }

    pub fn current(&self) -> &Workspace {
        return self.workspaces.get(self.idx).unwrap();
    }

    pub fn current_mut(&mut self) -> &mut Workspace {
        return self.workspaces.get_mut(self.idx).unwrap();
    }

    pub fn get(&self, idx: usize) -> &Workspace {
        return self.workspaces.get(idx).unwrap();
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Workspace {
        return self.workspaces.get_mut(idx).unwrap();
    }

    pub fn contains(&self, window_id: XWindowID) -> Option<(&Workspace, usize)> {
        for ws in self.workspaces.iter() {
            if let Some(idx) = ws.windows.index_of(window_id) {
                return Some((ws, idx));
            }
        }
        return None;
    }

    pub fn contains_mut(&mut self, window_id: XWindowID) -> Option<(&mut Workspace, usize)> {
        for ws in self.workspaces.iter_mut() {
            if let Some(idx) = ws.windows.index_of(window_id) {
                return Some((ws, idx));
            }
        }
        return None;
    }
}
