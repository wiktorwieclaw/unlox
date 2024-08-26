import init, { Interpreter } from "unlox-wasm";

class Writer {
    write(output: string) {
        postMessage({ type: "output", output });
        return output.length
    }

    flush() {
        // no-op
    }
}

self.onmessage = async (event: any) => {
    await init({});
    const writer = new Writer();
    const interpreter = new Interpreter();
    interpreter.interpret(event.data, writer);
    postMessage({ type: "end" })
}