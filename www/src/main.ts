import * as monaco from 'monaco-editor';
import init, { Interpreter } from "../pkg";
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

init().then(() => {
    monaco.languages.register({ id: "lox" });
    monaco.languages.setMonarchTokensProvider('lox', {
        keywords: ["fun", "if", "else", "for", "print", "class", "return"],
        tokenizer: {
            root: [
                [ /@?[a-zA-Z][\w$]*/, {
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
            'fun fib(n){',
            '    if (n <= 1) return n;',
            '    return fib(n - 2) + fib(n - 1);',
            '}\n',
            'print fib(28);'
        ].join('\n'),
        language: 'lox',
        automaticLayout: true
    });
    monaco.editor.setTheme("vs-dark");

    const interpreter = new Interpreter();
    const output = document.getElementById("output")!;

    document.getElementById("run")?.addEventListener("click", (_e) => {
        interpreter.clear();
        interpreter.interpret(editor.getValue());
        output.textContent = interpreter.out();
    });
})