use crate::screen::Screen;
use crate::windows::Window;
use crate::workspace::Workspace;
use crate::xconn::XConn;

pub fn activate(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // If empty, this is pointless
    if ws.windows.is_empty() {
        return;
    }

    // Map the windows in the workspace in reverse order and grab their geometry
    for window in ws.windows.iter_rev_mut() {
        conn.window_map(window.id);
        window.update_geometry(conn)
    }

    // Tell X to focus our focused window
    conn.window_focus(ws.windows.focused().unwrap().id);
}

pub fn deactivate(ws: &mut Workspace, conn: &XConn) {
    // Unmap all the windows
    for window in ws.windows.iter() {
        conn.window_unmap(window.id);
    }
}

pub fn window_add(ws: &mut Workspace, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
    // Internally add
    ws.windows.add(Window::from(window_id));

    // Ensure the window spawns at the visible screen start (not ontop of bars etc)
    conn.window_move(window_id, screen.x as u32 + 16, screen.y as u32 + 9);

    // Tell X to map and focus the window
    conn.window_map(window_id);
    conn.window_focus(window_id);

    // Update the window geometry
    ws.windows.focused_mut().unwrap().update_geometry(conn);
}

pub fn window_del(ws: &mut Workspace, conn: &XConn, screen: &Screen, window_id: xcb::Window) {
    // Delete window (if there!)
    if let Some(idx) = ws.windows.index_of(window_id) {
        // Internally remove at position
        ws.windows.remove(idx);

        // Tell X to unmap the window
        conn.window_unmap(window_id);

        // If we just deleted the previously focused, focus the next index 0
        if idx == 0 { window_focus_idx(ws, conn, screen, 0); }
    }
}

pub fn window_del_focused(ws: &mut Workspace, conn: &XConn, screen: &Screen) -> Option<xcb::Window> {
    // Delete focused window (if there!)
    if let Some(focused) = ws.windows.focused() {
        // Take ownership of focused window
        let focused = *focused;

        // Internally, remove the focused window
        ws.windows.remove(0);

        // Tell X to unmap the window
        conn.window_unmap(focused.id);

        // Focus the next
        window_focus_idx(ws, conn, screen, 0);

        // Return the old focused window
        return Some(focused.id);
    }

    // Return nothing
    return None;
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
        conn.window_ontop(window_id);

        // Tell X to focus the window
        conn.window_focus(window_id);
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
        conn.window_ontop(window.id);

        // Tell X to focus the window
        conn.window_focus(window.id);
    }
}


pub fn perform_layout(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // do nothing
}