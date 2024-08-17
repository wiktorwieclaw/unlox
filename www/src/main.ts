import * as monaco from 'monaco-editor';
// @ts-ignore  
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
// @ts-ignore  
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
// @ts-ignore  
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
// @ts-ignore  
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
// @ts-ignore  
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

// @ts-ignore
self.MonacoEnvironment = {
    getWorker(_: any, label: string) {
        if (label === 'json') {
            return new jsonWorker();
        }
        if (label === 'css' || label === 'scss' || label === 'less') {
            return new cssWorker();
        }
        if (label === 'html' || label === 'handlebars' || label === 'razor') {
            return new htmlWorker();
        }
        if (label === 'typescript' || label === 'javascript') {
            return new tsWorker();
        }
        return new editorWorker();
    }
};

monaco.languages.register({ id: "lox" });
monaco.languages.setMonarchTokensProvider('lox', {
    keywords: [
        "and",
        "class",
        "else",
        "false",
        "fun",
        "for",
        "if",
        "nil",
        "or",
        "print",
        "return",
        "super",
        "this",
        "true",
        "var",
        "while"
    ],
    tokenizer: {
        root: [
            [/@?[a-zA-Z][\w$]*/, {
                cases: {
                    '@keywords': 'keyword',
                    '@default': 'variable',
                }
            }],
            [/".*?"/, 'string'],
            [/\/\//, 'comment']
        ]
    }
});
const editor = monaco.editor.create(document.getElementById("code-editor")!, {
    value: [
        'fun fib(n) {',
        '    if (n <= 1) return n;',
        '    return fib(n - 2) + fib(n - 1);',
        '}\n',
        'print fib(15);'
    ].join('\n'),
    language: 'lox',
    automaticLayout: true
});
monaco.editor.setTheme("vs-dark");

const output = document.getElementById("output")!;

let worker: Worker | null;
let indicatorInterval: number | null;
document.getElementById("run")?.addEventListener("click", () => {
    output.textContent = "";

    if (worker) {
        if (indicatorInterval) {
            clearInterval(indicatorInterval);
        }
        worker.terminate();
        worker = null;
    }

    indicatorInterval = setInterval(() => {
        if (output.textContent?.length === 3) {
            output.textContent = ""
        } else {
            output.textContent += '•';
        }
    }, 500)

    worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    worker.onmessage = (event) => {
        clearInterval(indicatorInterval);
        output.textContent = event.data;
    };
    worker.postMessage(editor.getValue());
});