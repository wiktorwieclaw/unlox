use unlox_interpreter::Interpreter;
use unlox_lexer::Lexer;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
enum Error {
    Parse(unlox_parse::Error),
    Interpret(unlox_interpreter::Error),
}

fn interpret(code: &str) -> Result<String, Error> {
    let mut out = Vec::new();
    let lexer = Lexer::new(code);
    let ast = unlox_parse::parse(lexer).map_err(Error::Parse)?;
    let mut interpreter = Interpreter::new();
    interpreter
        .interpret(&ast, &mut out)
        .map_err(Error::Interpret)?;
    Ok(String::from_utf8(out).unwrap())
}

#[test]
fn empty() {
    assert_eq!(interpret("").unwrap(), "");
}

#[test]
fn math_expressions() {
    assert_eq!(interpret("print 2 + 2 * 2;").unwrap(), "6\n");
    assert_eq!(interpret("print (2 + 2) * 2;").unwrap(), "8\n");
}

#[test]
fn boolean_logic() {
    let code = r#"
        print "hi" or 2;
        print nil or "yes";
    "#;
    assert_eq!(interpret(code).unwrap(), "hi\nyes\n");
}

#[test]
fn if_statements() {
    let code = r#"
        if (true) {
            print true;
        } else {
            print false;
        }

        if (true) print true; else print false;
        
        if (false) {
            print true;
        } else {
            print false;
        }

        if (false) print true; else print false;
    "#;
    assert_eq!(interpret(code).unwrap(), "true\ntrue\nfalse\nfalse\n");
}
