use crate::lex;

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        operator: lex::Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary {
        operator: lex::Token,
        right: Box<Expr>,
    },
}