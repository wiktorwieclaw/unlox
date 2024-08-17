pub mod dissassemble;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, v: Value) -> u8 {
        let idx = self.constants.len();
        self.constants.push(v);
        idx as u8
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl OpCode {
    pub fn parse(raw: u8) -> Option<Self> {
        match raw {
            0x00 => Some(OpCode::Constant),
            0x01 => Some(OpCode::Add),
            0x02 => Some(OpCode::Subtract),
            0x03 => Some(OpCode::Multiply),
            0x04 => Some(OpCode::Divide),
            0x05 => Some(OpCode::Negate),
            0x06 => Some(OpCode::Return),
            _ => None,
        }
    }
}

pub type Value = f64;
