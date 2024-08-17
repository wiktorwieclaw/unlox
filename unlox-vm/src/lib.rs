use unlox_bytecode::{Chunk, OpCode, Value};

const STACK_SIZE: usize = 256;

pub struct Vm {
    stack: Stack,
}

struct Stack {
    stack: [Value; STACK_SIZE],
    top: usize,
}

impl Stack {
    fn new() -> Self {
        Self {
            stack: [0.0; STACK_SIZE],
            top: 0,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack[self.top] = value;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        self.top -= 1;
        self.stack[self.top]
    }
}

#[derive(Debug)]
pub enum Error {
    Compile,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<()> {
        let mut ip = 0;
        let mut read_byte = || {
            let byte = chunk.code[ip];
            ip += 1;
            byte
        };
        loop {
            let byte = read_byte();
            let opcode = OpCode::parse(byte).unwrap();
            match opcode {
                OpCode::Constant => {
                    let constant = chunk.constants[usize::from(read_byte())];
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    let v = self.stack.pop();
                    self.stack.push(-v);
                }
                OpCode::Return => {
                    println!("{}", self.stack.pop());
                    return Ok(());
                }
            }
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
