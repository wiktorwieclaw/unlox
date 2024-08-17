use unlox_bytecode::{Chunk, OpCode};

pub struct Vm;

#[derive(Debug)]
pub enum Error {
    Compile,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
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
                    println!("{}", chunk.constants[usize::from(read_byte())]);
                }
                OpCode::Return => return Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
