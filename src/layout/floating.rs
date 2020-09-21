use crate::helper;
use crate::screen::Screen;
use crate::windows::Window;
use crate::workspace::Workspace;
use crate::x::{XConn, XWindow};

pub fn activate(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // If empty, this is pointless
    if ws.windows.is_empty() {
        return;
    }

    // Map the windows in the workspace in reverse order and grab their geometry
    for window in ws.windows.iter_rev_mut() {
        conn.map_window(window.id());
        conn.update_geometry(window);
    }

    // Tell X to focus our focused window
    conn.set_input_focus(ws.windows.focused().unwrap().id());
}

pub fn deactivate(ws: &mut Workspace, conn: &XConn) {
    // Unmap all the windows
    for window in ws.windows.iter() {
        conn.unmap_window(window.id());
    }
}

pub fn window_add(ws: &mut Workspace, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
    // Internally add
    ws.windows.add(Window::from(window_id));

    // Tell X to map and focus the window
    conn.map_window(window_id);
    conn.set_input_focus(window_id);

    // Set child window mask
    conn.change_window_attributes(window_id, &helper::values_attributes_child_events());

    // Update the window geometry
    conn.update_geometry(ws.windows.focused_mut().unwrap());
}

pub fn window_del(ws: &mut Workspace, conn: &XConn, screen: &Screen, idx: usize, window_id: xcb::Window) {
    // Internally remove window at position
    ws.windows.remove(idx);

    // Tell X to unmap the window
    conn.unmap_window(window_id);

    // If we just deleted the previously focused, focus the next index 0
    if idx == 0 { window_focus_idx(ws, conn, screen, 0); }
}

pub fn window_focus(ws: &mut Workspace, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
    // Focus window (if there!)
    if let Some(idx) = ws.windows.index_of(window_id) {
        // Get window at idx
        let window = *ws.windows.get(idx).unwrap();

        // Internally, remove old position and readd (to front)
        ws.windows.remove(idx);
        ws.windows.add(window);

        // Set window ontop
        conn.configure_window(window_id, &helper::values_configure_stack_above());

        // Tell X to focus the window
        conn.set_input_focus(window_id);
    }
}

pub fn window_focus_idx(ws: &mut Workspace, conn: &XConn, screen: &Screen, idx: usize) {
    // Focus window (if there!)
    if let Some(window) = ws.windows.get(idx) {
        // Get actual Window (not just reference)
        let window = *window;
    
        // Internally remove old position
        ws.windows.remove(idx);
        ws.windows.add(window);

        // Set window ontop
        conn.configure_window(window.id(), &helper::values_configure_stack_above());

        // Tell X to focus the window
        conn.set_input_focus(window.id());
    }
}

pub fn window_focus_cycle(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // Get length just the once
    let len = ws.windows.len();

    // If length < 2 nothing to do
    if len < 2 {
        return;
    }

    // Focus last window
    ws.window_focus_idx(conn, screen, len-1);
}

pub fn window_close_focused(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // Log
    outlog::debug!("Closing focused window in workspace");

    // If there is a focused window, close
    if let Some(window) = ws.windows.focused() {
        conn.destroy_window(window.id());
    }
}
