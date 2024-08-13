#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use unlox_interpreter::Interpreter;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

fn run(input: &str, output: &mut Vec<u8>) {
    output.clear();
    let lexer = unlox_lexer::Lexer::new(input);
    let ast = unlox_parse::parse(lexer, output);
    let mut interpreter = Interpreter::new(output);
    interpreter.interpret(input, &ast)
}

struct OutputBuffer(Vec<u8>);

impl std::fmt::Display for OutputBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Encoding error: {e}"),
        }
    }
}

#[component]
fn App() -> Element {
    let mut input = use_signal(|| "print \"Hello, World!\";".to_string());
    let mut output = use_signal(|| OutputBuffer(vec![]));
    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        div {
            id: "header",
            p {
                id: "title",
                "unlox"
            }
            button {
                id: "run-button",
                onclick: move |_| run(&input.read(), &mut output.write().0),
                "Run"
            }
        }
        div {
            id: "container",
            textarea {
                id: "code-editor",
                value: "{input}",
                oninput: move |event| input.set(event.value()),
                spellcheck: false,
                resize: "none",
            }
            textarea {
                id: "output",
                readonly: true,
                value: "{output}",
                resize: "none",
            }
        }
    }
}
