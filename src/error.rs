
// TODO: do not use global variables
// make an ErrorReporter trait and pass its implementors
// to other components
pub static mut HAD_ERROR: bool = false;

pub fn error(line: u32, message: &str) {
    report(line, "", message);
}

pub fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
    unsafe { HAD_ERROR = true };
}
