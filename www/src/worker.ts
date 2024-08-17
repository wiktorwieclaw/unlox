import init, { Interpreter } from "unlox-wasm";

self.onmessage = async (event) => {
    await init();
    const interpreter = new Interpreter();
    interpreter.interpret(event.data)
    postMessage(interpreter.out())
}