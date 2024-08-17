use std::io;

use crate::{Chunk, OpCode};

pub fn dissassemble(chunk: &Chunk, name: &str, out: &mut impl io::Write) -> io::Result<()> {
    writeln!(out, "== {name} ==")?;

    let mut bytecode = chunk.code.iter().cloned().enumerate();
    while let Some((offset, opcode)) = bytecode.next() {
        write!(out, "{:04} ", offset)?;

        if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
            write!(out, "   | ")?;
        } else {
            write!(out, "{:4} ", chunk.lines[offset])?;
        }

        let opcode = OpCode::parse(opcode).unwrap();
        match opcode {
            OpCode::Constant => {
                let name = "OP_CONSTANT";
                let (_, arg_idx) = bytecode.next().unwrap();
                let arg = &chunk.constants[usize::from(arg_idx)];
                writeln!(out, "{name:<16} {arg_idx:4} '{arg}'")?;
            }
            OpCode::Return => {
                writeln!(out, "OP_RETURN")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test() {
        let mut chunk = Chunk::new();
        let constant = chunk.add_constant(Value(1.2));
        chunk.write(OpCode::Constant as u8, 123);
        chunk.write(constant, 123);
        chunk.write(OpCode::Return as u8, 123);

        let mut out = Vec::new();
        dissassemble(&chunk, "test chunk", &mut out).unwrap();
        let out = std::str::from_utf8(&out).unwrap();
        let expected = "\
            == test chunk ==\n\
            0000  123 OP_CONSTANT         0 '1.2'\n\
            0002    | OP_RETURN\n\
        ";
        println!("{out}");
        println!("{expected}");
        assert_eq!(out, expected);
    }
}
