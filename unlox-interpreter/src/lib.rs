use env::{Env, EnvCactus};
use std::io::Write;
use unlox_ast::{Ast, Expr, ExprIdx, Stmt, StmtIdx, Token, TokenKind};
use val::{Callable, Val};

mod env;
mod val;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("[Line {}]: Operand must be a number.", operator.line)]
    ExpectedNumber { operator: Token },
    #[error("[Line {}]: Operands must be numbers.", operator.line)]
    ExpectedNumbers { operator: Token },
    #[error("[Line {}]: Operands must be two numbers or two strings.", operator.line)]
    ExpectedNumbersOrStrings { operator: Token },
    #[error("[Line {}]: Undefined variable {}.", token.line, name)]
    UndefinedVariable { name: String, token: Token },
    #[error("[Line] {}: Can only call functions and classes.", paren.line)]
    BadCall { paren: Token },
    #[error("[Line] {}: Expected {expected} arguments but got {got}.", paren.line)]
    WrongNumberOfArgs {
        paren: Token,
        expected: usize,
        got: usize,
    },
    #[error("Parsing error.")]
    Parsing,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Interpreter<Out> {
    env_tree: EnvCactus,
    out: Out,
}

impl<Out> Interpreter<Out> {
    pub fn new(out: Out) -> Self {
        let mut global = Env::new();
        global.define_var("clock".to_owned(), Val::Callable(Callable::Clock));
        Self {
            env_tree: EnvCactus::with_global(global),
            out,
        }
    }
}

impl<Out> Interpreter<Out>
where
    Out: Write,
{
    pub fn interpret(&mut self, source: &str, ast: &Ast) -> Result<()> {
        for stmt in ast.roots() {
            self.execute(source, ast, *stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, source: &str, ast: &Ast, stmt: StmtIdx) -> Result<()> {
        match ast.stmt(stmt) {
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(source, ast, *cond)?.is_truthy() {
                    self.execute(source, ast, *then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(source, ast, *else_branch)?;
                }
            }
            Stmt::While { cond, body } => {
                while self.evaluate(source, ast, *cond)?.is_truthy() {
                    self.execute(source, ast, *body)?;
                }
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(source, ast, *expr)?;
                writeln!(self.out, "{val}").unwrap()
            }
            Stmt::VarDecl { name, init } => {
                let init = match init {
                    Some(init) => self.evaluate(source, ast, *init)?,
                    None => Val::Nil,
                };
                self.env_tree
                    .current_env_mut()
                    .define_var(source[name.lexeme.clone()].to_owned(), init);
            }
            Stmt::Expression(expr) => {
                self.evaluate(source, ast, *expr)?;
            }
            Stmt::Block(stmts) => self.execute_block(source, ast, stmts)?,
            Stmt::ParseErr => return Err(Error::Parsing),
        }
        Ok(())
    }

    fn execute_block(&mut self, source: &str, ast: &Ast, stmts: &[StmtIdx]) -> Result<()> {
        self.env_tree.push(Env::new());
        let result = (|| {
            for stmt in stmts {
                self.execute(source, ast, *stmt)?;
            }
            Ok(())
        })();
        self.env_tree.pop();
        result
    }

    fn evaluate(&mut self, source: &str, ast: &Ast, expr: ExprIdx) -> Result<Val> {
        let lit = match ast.expr(expr) {
            Expr::Literal(value) => value.clone().into(),
            Expr::Grouping(expr) => self.evaluate(source, ast, *expr)?,
            Expr::Unary(operator, right) => {
                let right = self.evaluate(source, ast, *right)?;
                match (&operator.kind, right) {
                    (TokenKind::Bang, right) => Val::Bool(!right.is_truthy()),
                    (TokenKind::Minus, Val::Number(n)) => Val::Number(-n),
                    (TokenKind::Minus, _) => {
                        return Err(Error::ExpectedNumber {
                            operator: operator.clone(),
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(operator, left, right) => {
                let left = self.evaluate(source, ast, *left)?;
                let right = self.evaluate(source, ast, *right)?;

                match (&operator.kind, left, right) {
                    (TokenKind::Minus, Val::Number(l), Val::Number(r)) => Val::Number(l - r),
                    (TokenKind::Slash, Val::Number(l), Val::Number(r)) => Val::Number(l / r),
                    (TokenKind::Star, Val::Number(l), Val::Number(r)) => Val::Number(l * r),
                    (TokenKind::Plus, Val::Number(l), Val::Number(r)) => Val::Number(l + r),
                    (TokenKind::Plus, Val::String(l), Val::String(r)) => Val::String(l + &r),
                    (TokenKind::Greater, Val::Number(l), Val::Number(r)) => Val::Bool(l > r),
                    (TokenKind::GreaterEqual, Val::Number(l), Val::Number(r)) => Val::Bool(l >= r),
                    (TokenKind::Less, Val::Number(l), Val::Number(r)) => Val::Bool(l < r),
                    (TokenKind::LessEqual, Val::Number(l), Val::Number(r)) => Val::Bool(l <= r),
                    (TokenKind::BangEqual, l, r) => Val::Bool(l != r),
                    (TokenKind::EqualEqual, l, r) => Val::Bool(l == r),
                    (TokenKind::Plus, _, _) => {
                        return Err(Error::ExpectedNumbersOrStrings {
                            operator: operator.clone(),
                        });
                    }
                    (
                        TokenKind::Greater
                        | TokenKind::GreaterEqual
                        | TokenKind::Less
                        | TokenKind::LessEqual
                        | TokenKind::Minus
                        | TokenKind::Slash
                        | TokenKind::Star,
                        _,
                        _,
                    ) => {
                        return Err(Error::ExpectedNumbers {
                            operator: operator.clone(),
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(var) => {
                let name = &source[var.lexeme.clone()];
                self.env_tree
                    .var(name)
                    .ok_or_else(|| Error::UndefinedVariable {
                        name: name.to_owned(),
                        token: var.clone(),
                    })?
                    .clone()
            }
            Expr::Assign { var, value } => {
                let value = self.evaluate(source, ast, *value)?;
                let name = &source[var.lexeme.clone()];
                self.env_tree
                    .assign_var(name, value)
                    .ok_or_else(|| Error::UndefinedVariable {
                        name: name.to_owned(),
                        token: var.clone(),
                    })?
                    .clone()
            }
            Expr::Logical(operator, left, right) => {
                let left = self.evaluate(source, ast, *left)?;
                match (&operator.kind, left.is_truthy()) {
                    (TokenKind::Or, true) => left,
                    (TokenKind::Or, false) => self.evaluate(source, ast, *right)?,
                    (_, false) => left,
                    _ => self.evaluate(source, ast, *right)?,
                }
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let callee = self.evaluate(source, ast, *callee)?;
                let Val::Callable(callable) = callee else {
                    return Err(Error::BadCall {
                        paren: paren.clone(),
                    });
                };
                let args: Result<Vec<_>> = args
                    .iter()
                    .map(|arg| self.evaluate(source, ast, *arg))
                    .collect();
                let args = args?;
                if args.len() != callable.arity() {
                    return Err(Error::WrongNumberOfArgs {
                        paren: paren.clone(),
                        expected: callable.arity(),
                        got: args.len(),
                    });
                }
                callable.call(self, args)
            }
        };
        Ok(lit)
    }
}
