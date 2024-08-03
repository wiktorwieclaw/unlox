use std::fmt::{self, Display};
pub use tokens::{Token, TokenKind};
pub use unlox_tokens as tokens;

#[derive(Debug, Clone)]
pub enum Stmt {
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    Print(Expr),
    VarDecl {
        name: Token,
        init: Option<Expr>,
    },
    Expression(Expr),
    Block(Vec<Stmt>),
    ParseErr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Lit),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assign {
        var: Token,
        value: Box<Expr>,
    },
    Logical(Token, Box<Expr>, Box<Expr>),
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
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
