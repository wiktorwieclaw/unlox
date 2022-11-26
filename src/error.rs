use std::cell::Cell;

// TODO: do not use global variables
// make an ErrorReporter trait and pass its implementors
// to other components
thread_local! {
    pub static HAD_ERROR: Cell<bool>  = Cell::new(false);
}

pub fn error(line: u32, message: &str) {
    report(line, "", message);
}

pub fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
    HAD_ERROR.with(|e| e.set(true));
}
