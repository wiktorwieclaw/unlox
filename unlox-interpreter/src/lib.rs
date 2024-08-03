use env::{Env, EnvCactus};
use std::io::Write;
use unlox_ast::{Expr, Stmt, Token, TokenKind};
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

pub struct Interpreter {
    env_tree: EnvCactus,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut global = Env::new();
        global.define_var("clock".to_owned(), Val::Callable(Callable::Clock));
        Self {
            env_tree: EnvCactus::with_global(global),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, source: &str, stmts: &[Stmt], out: &mut impl Write) -> Result<()> {
        for stmt in stmts {
            self.execute(source, stmt, out)?;
        }
        Ok(())
    }

    fn execute(&mut self, source: &str, stmt: &Stmt, out: &mut impl Write) -> Result<()> {
        match stmt {
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(source, cond)?.is_truthy() {
                    self.execute(source, then_branch, out)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(source, else_branch, out)?;
                }
            }
            Stmt::While { cond, body } => {
                while self.evaluate(source, cond)?.is_truthy() {
                    self.execute(source, body, out)?;
                }
            }
            Stmt::Print(expr) => writeln!(out, "{}", self.evaluate(source, expr)?).unwrap(),
            Stmt::VarDecl { name, init } => {
                let init = match init {
                    Some(init) => self.evaluate(source, init)?,
                    None => Val::Nil,
                };
                self.env_tree
                    .current_env_mut()
                    .define_var(source[name.lexeme.clone()].to_owned(), init);
            }
            Stmt::Expression(expr) => {
                self.evaluate(source, expr)?;
            }
            Stmt::Block(stmts) => self.execute_block(source, stmts, out)?,
            Stmt::ParseErr => return Err(Error::Parsing),
        }
        Ok(())
    }

    fn execute_block(&mut self, source: &str, stmts: &[Stmt], out: &mut impl Write) -> Result<()> {
        self.env_tree.push(Env::new());
        let result = (|| {
            for stmt in stmts {
                self.execute(source, stmt, out)?;
            }
            Ok(())
        })();
        self.env_tree.pop();
        result
    }

    fn evaluate(&mut self, source: &str, expr: &Expr) -> Result<Val> {
        let lit = match expr {
            Expr::Literal(value) => value.clone().into(),
            Expr::Grouping(expr) => self.evaluate(source, expr)?,
            Expr::Unary(operator, right) => {
                let right = self.evaluate(source, right)?;
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
                let left = self.evaluate(source, left)?;
                let right = self.evaluate(source, right)?;

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
                let value = self.evaluate(source, value)?;
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
                let left = self.evaluate(source, left)?;
                match (&operator.kind, left.is_truthy()) {
                    (TokenKind::Or, true) => left,
                    (TokenKind::Or, false) => self.evaluate(source, right)?,
                    (_, false) => left,
                    _ => self.evaluate(source, right)?,
                }
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let callee = self.evaluate(source, callee)?;
                let Val::Callable(callable) = callee else {
                    return Err(Error::BadCall {
                        paren: paren.clone(),
                    });
                };
                let args: Result<Vec<_>> =
                    args.iter().map(|arg| self.evaluate(source, arg)).collect();
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
