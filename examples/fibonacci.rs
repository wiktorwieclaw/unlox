use std::io::stdout;

use unlox_interpreter::Interpreter;
use unlox_lexer::Lexer;

fn main() {
    let code = "
        fun fib_iterative(n) {
            var a = 0;
            var b = 1;

            for (var i = 0; i < n; i = i + 1) {
                var temp = a;
                a = b;
                b = temp + b;
            }
            return a;
        }

        fun fib_recursive(n) {
            if (n <= 1) return n;
            return fib_recursive(n - 2) + fib_recursive(n - 1);
        }

        fun bench(f, n) {
            var start = clock();
            var result = f(n);
            var time_secs = clock() - start;
            print result;
            print time_secs;
        }

        var n = 28;

        print \"iterative:\";
        bench(fib_iterative, n);
        print \"\nrecursive:\";
        bench(fib_recursive, n);
    ";
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer).unwrap();
    let mut interpreter = Interpreter::new(stdout());
    interpreter.interpret(code, &ast).unwrap();
}
