#[macro_use]
mod log;

mod config;
mod desktop;
mod event;
mod helper;
mod layout;
mod screen;
mod windows;
mod wm;
mod workspace;
mod x;

use wm::WM;

use std::env;
use std::process;

fn print_version() {
    println!(
        "{}-{}.{}.{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
    )
}

fn print_usage() {
    println!(
        "Usage: {} [-h|--help] [-v|--version] [-y|--why]",
        env!("CARGO_PKG_NAME"),
    )
}

fn main() {
    // Get arguments
    let args: Vec<String> = env::args().collect();

    // If arguments provided, either show version or help
    if args.len() >= 2 {
        match args.get(1).unwrap().as_str() {
            "-v"|"--version" => {
                print_version();
                process::exit(0);
            },

            "-h"|"--help" => {
                print_usage();
                process::exit(0);
            },

            "-y"|"--why" => {
                println!("Captain Kirk is climbing a mountain, why is he climbing a mountain?");
                process::exit(69);
            },

            _ => {
                print_usage();
                process::exit(1);
            },
        }
    }

    // Register OS signals
    unsafe { signal_hook::register(signal_hook::SIGINT|signal_hook::SIGTERM, || { panic!("OS Signal received!") }).expect("Failed to register OS signal receiver"); }
    debug!("Registered OS signal hook");

    // Try connect to xserver
    let (conn, screen_idx) = xcb::Connection::connect(None).expect("Failed to connect to xserver");
    debug!("Connected to X server");

    // Create new window manager object
    let mut wm = WM::register(&conn, screen_idx);

    // Run window manager!
    wm.run();
}
