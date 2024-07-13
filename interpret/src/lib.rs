use crate::env::Environment;
use ast::{Expr, Lit, Stmt, Token, TokenKind};

mod env;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operand must be a number.")]
    ExpectedNumber { operator: Token },
    #[error("Operands must be numbers.")]
    ExpectedNumbers { operator: Token },
    #[error("Operands must be two numbers or two strings.")]
    ExpectedNumbersOrStrings { operator: Token },
    #[error("Undefined variable {}", name.lexeme)]
    UndefinedVariable { name: Token },
}

impl Error {
    pub fn line(&self) -> u32 {
        match self {
            Error::ExpectedNumber { operator } => operator.line,
            Error::ExpectedNumbers { operator } => operator.line,
            Error::ExpectedNumbersOrStrings { operator } => operator.line,
            Error::UndefinedVariable { name } => name.line,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::Print(expr) => println!("{}", self.evaluate(expr)?),
                Stmt::VarDecl { name, init } => {
                    let init = match init {
                        Some(init) => self.evaluate(init)?,
                        None => Lit::Nil,
                    };
                    self.env.define(name, init);
                }
                Stmt::Expression(expr) => {
                    self.evaluate(expr)?;
                }
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, expr: Expr) -> Result<Lit> {
        let lit = match expr {
            Expr::Literal(value) => value,
            Expr::Grouping(expr) => self.evaluate(*expr)?,
            Expr::Unary(operator, right) => {
                let right = self.evaluate(*right)?;
                match (&operator.kind, right) {
                    (TokenKind::Bang, right) => Lit::Bool(!right.is_truthy()),
                    (TokenKind::Minus, Lit::Number(n)) => Lit::Number(-n),
                    (TokenKind::Minus, _) => {
                        return Err(Error::ExpectedNumber { operator });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(operator, left, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match (&operator.kind, left, right) {
                    (TokenKind::Minus, Lit::Number(l), Lit::Number(r)) => Lit::Number(l - r),
                    (TokenKind::Slash, Lit::Number(l), Lit::Number(r)) => Lit::Number(l / r),
                    (TokenKind::Star, Lit::Number(l), Lit::Number(r)) => Lit::Number(l * r),
                    (TokenKind::Plus, Lit::Number(l), Lit::Number(r)) => Lit::Number(l + r),
                    (TokenKind::Plus, Lit::String(l), Lit::String(r)) => Lit::String(l + &r),
                    (TokenKind::Greater, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l > r),
                    (TokenKind::GreaterEqual, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l >= r),
                    (TokenKind::Less, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l < r),
                    (TokenKind::LessEqual, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l <= r),
                    (TokenKind::BangEqual, l, r) => Lit::Bool(l != r),
                    (TokenKind::EqualEqual, l, r) => Lit::Bool(l == r),
                    (TokenKind::Plus, _, _) => {
                        return Err(Error::ExpectedNumbersOrStrings { operator });
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
                        return Err(Error::ExpectedNumbers { operator });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => self.env.get(&name)?.clone(),
        };
        Ok(lit)
    }
}
