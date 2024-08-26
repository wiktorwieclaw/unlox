use js_sys::Reflect;
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
    pub fn interpret(&mut self, src: &str, writer: JsValue) -> Result<(), JsError> {
        let mut writer = JsWriter::new(writer)?;
        let lexer = unlox_lexer::Lexer::new(src);
        let ast = unlox_parse::parse(lexer, &mut self.out);
        let mut ctx = unlox_interpreter::Ctx {
            src,
            out: SingleOutput::new(&mut writer),
        };
        self.interpreter.interpret(&mut ctx, &ast);
        Ok(())
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

#[derive(Debug, Clone, Copy, thiserror::Error)]
enum JsWriterError {
    #[error("Passed `writer` is not an object.")]
    NotAnObject,
    #[error("Passed `writer` doesn't have a `write` method.")]
    MissingWrite,
    #[error("Passed `writer` has a `write` method with wrong number of arguments - {0}.")]
    WrongLengthWrite(u32),
    #[error("Passed `writer` doesn't have a `flush` method.")]
    MissingFlush,
    #[error("Passed `writer` has a `flush` method with wrong number of arguments - {0}.")]
    WrongLengthFlush(u32),
}

struct JsWriter {
    writer: JsValue,
    write: js_sys::Function,
    flush: js_sys::Function,
}

impl JsWriter {
    pub fn new(writer: JsValue) -> Result<Self, JsWriterError> {
        let write = Reflect::get(&writer, &JsValue::from_str("write"))
            .map_err(|_| JsWriterError::NotAnObject)?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsWriterError::MissingWrite)?;
        let write_len = write.length();
        if write_len != 1 {
            return Err(JsWriterError::WrongLengthWrite(write_len));
        }

        let flush = Reflect::get(&writer, &JsValue::from_str("flush"))
            .map_err(|_| JsWriterError::NotAnObject)?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| JsWriterError::MissingFlush)?;
        let flush_len = flush.length();
        if flush_len != 0 {
            return Err(JsWriterError::WrongLengthFlush(flush_len));
        }

        Ok(Self {
            writer,
            write,
            flush,
        })
    }
}

impl std::io::Write for JsWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf = std::str::from_utf8(buf).unwrap();
        let buf = JsValue::from_str(buf);
        let nwritten = self.write.call1(&self.writer, &buf).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected exception caught from JsWriter",
            )
        })?;
        let nwritten = nwritten.as_f64().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Expected JsWriter.write to return number of bytes written",
            )
        })?;
        Ok(nwritten as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush.call0(&self.writer).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unexpected exception caught from JsWriter",
            )
        })?;
        Ok(())
    }
}
