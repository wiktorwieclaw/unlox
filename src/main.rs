use std::{
    cell::Cell,
    cmp::Ordering,
    env, fs,
    io::{self, stderr, stdout, BufRead, Write},
    process,
};
use unlox_interpreter::{output::SplitOutput, Ctx, Interpreter};
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
    let mut interpreter = Interpreter::new();
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
    let mut interpreter = Interpreter::new();
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

fn run(code: &str, interpreter: &mut Interpreter) {
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer, &mut std::io::stderr());
    let mut ctx = Ctx {
        src: code,
        out: SplitOutput::new(stdout(), stderr()),
    };
    interpreter.interpret(&mut ctx, &ast);
}
