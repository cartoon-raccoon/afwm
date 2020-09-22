#[cfg(debug_assertions)]
macro_rules! debug {
    ($fmt:expr) => (println!(concat!("[DEBUG] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("[DEBUG] ", $fmt), $($arg)*));
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($fmt:expr) => ({});
    ($fmt:expr, $($arg:tt)*) => ({});
}

macro_rules! info {
    ($fmt:expr) => (println!(concat!("[INFO] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("[INFO] ", $fmt), $($arg)*));
}

macro_rules! warn {
    ($fmt:expr) => (println!(concat!("[WARN] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("[WARN] ", $fmt), $($arg)*));
}

macro_rules! error {
    ($fmt:expr) => (println!(concat!("[ERROR] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("[ERROR] ", $fmt), $($arg)*));
}

macro_rules! fatal {
    ($fmt:expr) => (panic!(concat!("[FATAL] ", $fmt)));
    ($fmt:expr, $($arg:tt)*) => (panic!(concat!("[FATAL] ", $fmt), $($arg)*));
}