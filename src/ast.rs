use crate::lex;

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(lex::Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Lit),
    Unary(lex::Token, Box<Expr>),
}
