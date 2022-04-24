use std::{
    cmp::Ordering,
    env, fs,
    io::{self, BufRead, Write},
    process,
};

// TODO: do not use global variables
// make an ErrorReporter trait and pass its implementors
// to other components
static mut HAD_ERROR: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: jlox [script]");
            process::exit(64);
        }
        Ordering::Equal => run_file(&args[1]).unwrap(),
        Ordering::Less => run_prompt().unwrap(),
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let code = fs::read_to_string(path)?;
    run(&code);
    if unsafe { HAD_ERROR } {
        process::exit(65);
    }
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let stdout = io::stdin();
    let mut lines = stdout.lock().lines();
    loop {
        print!("> ");
        io::stdout().flush()?;
        match lines.next() {
            Some(line) => {
                run(&line?);
                unsafe { HAD_ERROR = false };
            }
            None => break,
        }
    }
    Ok(())
}

fn run(code: &str) {
    // TODO
    // let tokens = scan_tokens();

    // for token in tokens {
    //     println!("{token}");
    // }
}

fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, location: &str, message: &str) {
    eprintln!("[line {line}] Error {location}: {message}");
    unsafe { HAD_ERROR = true };
}
