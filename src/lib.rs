#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(debug)]
        eprintln!($($arg)*);
    }};
}
