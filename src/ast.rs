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
    Binary {
        operator: lex::Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Lit),
    Unary {
        operator: lex::Token,
        right: Box<Expr>,
    },
}
