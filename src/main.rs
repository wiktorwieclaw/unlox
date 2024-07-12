use std::{
    cmp::Ordering,
    env, fs,
    io::{self, BufRead, Write},
    process,
};

use std::cell::Cell;

thread_local! {
    pub static HAD_ERROR: Cell<bool>  = const { Cell::new(false) };
    pub static HAD_RUNTIME_ERROR: Cell<bool>  = const { Cell::new(false) };
}

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
    if HAD_ERROR.with(|e| e.get()) {
        process::exit(65);
    }
    if HAD_RUNTIME_ERROR.with(|e| e.get()) {
        process::exit(70);
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
                HAD_ERROR.with(|e| e.set(false))
            }
            None => break,
        }
    }
    Ok(())
}

fn run(code: &str) {
    let scanner = lexer::Scanner::new(code);
    let ast = match parse::parse(scanner) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    if let Err(e) = interpret::interpret(ast) {
        eprintln!("{e}");
    }
}
