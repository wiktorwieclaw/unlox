import init, { Interpreter } from "unlox-wasm";

self.onmessage = async (event) => {
    await init();
    const interpreter = new Interpreter();
    interpreter.interpret(event.data, (output: string) => {
        postMessage({ type: "output", output });
    })
    postMessage({ type: "end" })

}