use crate::helper;
use crate::screen::Screen;
use crate::windows::Window;
use crate::workspace::Workspace;
use crate::x::{XConn, XWindowID};

pub fn activate(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // If empty, this is pointless
    if ws.windows.is_empty() {
        return;
    }

    // Iterate windows
    for window in ws.windows.iter_rev() {
        // Map the window to the display
        conn.map_window(window.xwindow.id);
    }

    // Tell X to focus our focused window
    conn.set_input_focus(ws.windows.focused().unwrap().xwindow.id);
}

pub fn deactivate(ws: &mut Workspace, conn: &XConn) {
    // Iterate windows
    for window in ws.windows.iter() {
        // Disable events before unmapping the window
        conn.change_window_attributes(window.xwindow.id, &helper::values_attributes_no_events());

        // Unmap the window
        conn.unmap_window(window.xwindow.id);

        // Enable events again
        conn.change_window_attributes(window.xwindow.id, &helper::values_attributes_child_events());
    }
}

pub fn window_add(ws: &mut Workspace, conn: &XConn, screen: &Screen, window: Window) {
    // Tell X to map and focus the window
    conn.map_window(window.xwindow.id);

    // Start tracking events for this window
    conn.change_window_attributes(window.xwindow.id, &helper::values_attributes_child_events());

    // Set window ontop
    conn.configure_window(window.xwindow.id, &helper::values_configure_stack_above());

    // Set focused
    conn.set_input_focus(window.xwindow.id);

    // Internally add
    ws.windows.add(window);
}

pub fn window_del(ws: &mut Workspace, conn: &XConn, screen: &Screen, idx: usize, window_id: XWindowID) -> Window {
    // Get window and own_
    let window = ws.windows.get(idx).unwrap().to_owned();

    // Internally remove window at position
    ws.windows.remove(idx);

    // Stop tracking events for this window
    conn.change_window_attributes(window_id, &helper::values_attributes_no_events());

    // Tell X to unmap the window
    conn.unmap_window(window_id);

    // If we just deleted the previously focused, try focus the next index 0
    if idx == 0 {
        if let Some(window) = ws.windows.get(0) { window_input_focus_set_ontop(conn, window.xwindow.id); }
    }

    // Return the Window
    return window;
}

pub fn window_focus(ws: &mut Workspace, conn: &XConn, screen: &Screen, window_id: XWindowID) {
    // Focus window (if there!)
    if let Some(idx) = ws.windows.index_of(window_id) {
        // Internally, move to front
        ws.windows.move_front(idx);

        // Focus input + set ontop
        window_input_focus_set_ontop(conn, window_id);
    }
}

pub fn window_focus_cycle(ws: &mut Workspace, conn: &XConn, screen: &Screen) {
    // Get length just the once
    let len = ws.windows.len();

    // If length < 2 nothing to do
    if len < 2 {
        return;
    }

    // Internally, move last window to front
    ws.windows.move_front(len-1);

    // Get window in question
    let window = ws.windows.get(len-1).unwrap();

    // Focus input + set ontop
    window_input_focus_set_ontop(conn, window.xwindow.id);
}

fn window_input_focus_set_ontop(conn: &XConn, window_id: XWindowID) {
    // Disable event tracking before making changes
    conn.change_window_attributes(window_id, &helper::values_attributes_no_events());

    // Set window ontop
    conn.configure_window(window_id, &helper::values_configure_stack_above());

    // Tell X to focus the window
    conn.set_input_focus(window_id);

    // Enable event tracking again
    conn.change_window_attributes(window_id, &helper::values_attributes_child_events());
}