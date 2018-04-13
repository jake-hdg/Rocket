#[macro_export]
macro_rules! debug {
    ($($token:tt)*) => (
        if ::std::env::var_os("ROCKET_CLI_DEBUG").is_some() {
            print!("[{}:{}] ", file!(), line!());
            println!($($token)*);
        }
    )
}
