use unlox_interpreter::output::SingleOutput;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Interpreter {
    out: Vec<u8>,
    interpreter: unlox_interpreter::Interpreter,
}

#[wasm_bindgen]
impl Interpreter {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            out: Vec::new(),
            interpreter: unlox_interpreter::Interpreter::new(),
        }
    }

    #[wasm_bindgen]
    pub fn interpret(&mut self, src: &str) {
        let lexer = unlox_lexer::Lexer::new(src);
        let ast = unlox_parse::parse(lexer, &mut self.out);
        let mut ctx = unlox_interpreter::Ctx {
            src,
            out: SingleOutput::new(&mut self.out),
        };
        self.interpreter.interpret(&mut ctx, &ast);
    }

    #[wasm_bindgen]
    pub fn out(&self) -> Result<String, String> {
        String::from_utf8(self.out.clone()).map_err(|_| "UTF-8 encoding error".to_string())
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.out.clear()
    }
}
