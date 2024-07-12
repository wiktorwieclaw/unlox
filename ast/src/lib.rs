pub mod token;
pub use token::{Token, TokenKind};

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

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Lit),
    Unary(Token, Box<Expr>),
}
