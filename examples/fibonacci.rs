use std::io::stdout;

use unlox_interpreter::Interpreter;
use unlox_lexer::Lexer;

fn main() {
    let code = r#"
        var start = clock();
        var n = 50;

        var a = 0;
        var b = 1;

        for (var i = 0; i < n; i = i + 1) {
            var temp = a;
            a = b;
            b = temp + b;
        }

        var end = clock();
        print a;
        print end - start;
    "#;
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer).unwrap();
    let mut interpreter = Interpreter::new(stdout());
    interpreter.interpret(code, &ast).unwrap();
}
