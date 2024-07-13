pub use tokens::{Token, TokenKind};
pub use unlox_tokens as tokens;

use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    VarDecl { name: Token, init: Option<Expr> },
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Lit),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assign { name: Token, value: Box<Expr> }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Lit {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Lit::Nil | Lit::Bool(false))
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lit::String(s) => write!(f, "{s}"),
            Lit::Number(n) => write!(f, "{n}"),
            Lit::Bool(b) => write!(f, "{b}"),
            Lit::Nil => write!(f, "nil"),
        }
    }
}
