use std::{any::Any, collections::HashMap};

use ast::{Expr, Lit, Stmt, Token, TokenKind};

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
struct Environment {
    values: HashMap<String, Box<dyn Any>>,
}

impl Environment {
    fn define(&mut self, name: &Token, value: Box<dyn Any>) {
        self.values.insert(name.lexeme.clone(), value);
    }

    fn get(&self, name: &Token) -> Result<&dyn Any> {
        self.values
            .get(&name.lexeme)
            .map(|v| v.as_ref())
            .ok_or_else(|| Error::UndefinedVariable { name: name.clone() })
    }

    fn get_mut(&mut self, name: &Token) -> Result<&mut dyn Any> {
        self.values
            .get_mut(&name.lexeme)
            .map(|v| v.as_mut())
            .ok_or_else(|| Error::UndefinedVariable { name: name.clone() })
    }
}

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
                Stmt::VarDecl { .. } => todo!(),
                Stmt::Expression(expr) => {
                    self.evaluate(expr)?;
                }
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Lit> {
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
            Expr::Variable(_) => todo!("variables are not implemented yet"),
        };
        Ok(lit)
    }
}
