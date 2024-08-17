use unlox_bytecode::{Chunk, OpCode, Value};
use unlox_vm::Vm;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value(1.2));
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant, 123);
    chunk.write(OpCode::Return as u8, 123);

    let mut vm = Vm;
    vm.interpret(&chunk).unwrap();
}
