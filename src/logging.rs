use std::{
    process,
    fmt::Debug,
};

pub fn info(a: &str)                 {println!("[INFO] {}", a)}
pub fn warn<T: Debug>(a: &str, b: T) {println!("[WARNING] {}: {:?}", a, b)}
pub fn err<T: Debug>(a: &str, b: T)  {println!("[ERROR] {}: {:?}", a, b)}

pub fn shit_yourself_and_die<T: Debug>(a: &str, b: T) -> ! {
    err(a, b);
    process::exit(1);
}
