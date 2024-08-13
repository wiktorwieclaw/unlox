use std::{
    cell::Cell,
    cmp::Ordering,
    env, fs,
    io::{self, stderr, stdout, BufRead, Stderr, Stdout, Write},
    process,
};
use unlox_interpreter::{output::SplitOutput, Interpreter};
use unlox_lexer::Lexer;

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
    let mut interpreter = Interpreter::with_split_output(stdout(), stderr());
    run(&code, &mut interpreter);
    if HAD_ERROR.with(|e| e.get()) {
        process::exit(65);
    }
    if HAD_RUNTIME_ERROR.with(|e| e.get()) {
        process::exit(70);
    }
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut interpreter = Interpreter::with_split_output(stdout(), stderr());
    loop {
        print!("> ");
        io::stdout().flush()?;
        match lines.next() {
            Some(line) => {
                run(&line?, &mut interpreter);
                HAD_ERROR.with(|e| e.set(false))
            }
            None => break,
        }
    }
    Ok(())
}

fn run(code: &str, interpreter: &mut Interpreter<SplitOutput<Stdout, Stderr>>) {
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer, &mut std::io::stderr());
    interpreter.interpret(code, &ast);
}
