use crate::layout::LayoutType;
use crate::windows::Window;
use crate::wm::WM;

use std::process::Command;
use std::thread;

use xcb::xproto;
use x11::keysym;

// Geometry
pub const BAR_SIZE: u16 = 20;
pub const GAP: u16 = 10;

// Number of workspaces to have
pub const WORKSPACES: usize = 9;

// Modifier key for keybinds
pub const MODKEY: u32 = xproto::MOD_MASK_4;

// Key binds of tuple: (mask, key, function)
pub const KEYBINDS: &[(xcb::ModMask, xcb::Keysym, fn(&mut WM))] = &[
    // Dmenu
    (MODKEY, keysym::XK_p, |_|{ run(&["dmenu_run"]) }),

    // Backlight keys
    (0, keysym::XF86XK_MonBrightnessUp,   |_|{ run(&["xbacklight", "-inc", "5"]) }),
    (0, keysym::XF86XK_MonBrightnessDown, |_|{ run(&["xbacklight", "-dec", "5"]) }),

    // Volume control
    (0, keysym::XF86XK_AudioRaiseVolume, |_|{ run(&["amixer", "sset", "Master", "5%+"]) }),
    (0, keysym::XF86XK_AudioLowerVolume, |_|{ run(&["amixer", "sset", "Master", "5%-"]) }),
    (0, keysym::XF86XK_AudioMute,        |_|{ run(&["amixer", "sset", "Master", "1+", "toggle"]) }),
    (0, keysym::XF86XK_AudioMicMute,     |_|{ run(&["amixer", "sset", "Capture", "1+", "toggle"]) }),

    // Launch terminal
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_Return, |_|{ run(&["urxvt-launch"]) }),

    // Close focused window
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_c, |wm|{ wm.desktop.current_mut().window_close_focused(&wm.conn, &wm.screen) }),

    // Kill window manager
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_q, |wm|{ wm.kill() }),

    // Switch focused window
    (MODKEY, keysym::XK_Tab, |wm| { wm.desktop.current_mut().window_focus_cycle(&wm.conn, &wm.screen) }),

    // Workspace switching
    (MODKEY, keysym::XK_1, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 0) }),
    (MODKEY, keysym::XK_2, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 1) }),
    (MODKEY, keysym::XK_3, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 2) }),
    (MODKEY, keysym::XK_4, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 3) }),
    (MODKEY, keysym::XK_5, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 4) }),
    (MODKEY, keysym::XK_6, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 5) }),
    (MODKEY, keysym::XK_7, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 6) }),
    (MODKEY, keysym::XK_8, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 7) }),
    (MODKEY, keysym::XK_9, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, 8) }),
    (MODKEY, keysym::XK_Left,  |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, wm.desktop.index_prev()) }),
    (MODKEY, keysym::XK_Right, |wm|{ wm.desktop.goto(&wm.conn, &wm.screen, wm.desktop.index_next()) }),

    // Sending windows to workspaces
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_1, |wm|{ send_window_from_workspace_to(wm, 0) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_2, |wm|{ send_window_from_workspace_to(wm, 1) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_3, |wm|{ send_window_from_workspace_to(wm, 2) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_4, |wm|{ send_window_from_workspace_to(wm, 3) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_5, |wm|{ send_window_from_workspace_to(wm, 4) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_6, |wm|{ send_window_from_workspace_to(wm, 5) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_7, |wm|{ send_window_from_workspace_to(wm, 6) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_8, |wm|{ send_window_from_workspace_to(wm, 7) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_9, |wm|{ send_window_from_workspace_to(wm, 8) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_Left,  |wm|{ send_window_from_workspace_to(wm, wm.desktop.index_prev()) } ),
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_Right, |wm|{ send_window_from_workspace_to(wm, wm.desktop.index_next()) } ),

    // Set current workspace window layout
    (MODKEY|xproto::MOD_MASK_SHIFT, keysym::XK_f, |wm|{ wm.desktop.current_mut().set_layout(&wm.conn, &wm.screen, LayoutType::Floating) } ),
];

// If there is a currently focused window, sends from current workspace to workspace at index
fn send_window_from_workspace_to(wm: &mut WM, idx: usize) {
    if let Some(focused) = wm.desktop.current_mut().window_del_focused(&wm.conn, &wm.screen) {
        // Remove this window from current workspace
        wm.desktop.get_mut(idx).windows.add(Window::from(focused));
    }
}

// Executes an argument array in a new thread waiting for exit status
fn run(args: &'static [&str]) {
    thread::spawn(move || {
        execute(args);
    });
}

// Execute an argument array as child
fn execute(args: &[&str]) {
    // Log
    outlog::debug!("Running command: {:?}", args);

    // Create new Command object
    let mut cmd = Command::new(args[0]);

    // Set arguments
    cmd.args(args.iter().skip(1));

    // Execute!
    match cmd.status() {
        // Executed and returned exit status. Log returned status
        Ok(status) => outlog::debug!("{:?}: exited with {}", args, status),

        // Did not execute. Log returned error
        Err(err) => outlog::warn!("{:?}: {}", args, err),
    }
}