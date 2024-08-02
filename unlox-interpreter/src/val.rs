use std::time::{SystemTime, UNIX_EPOCH};

use unlox_ast::Lit;

use crate::Interpreter;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Callable(Callable),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Clock,
}

impl Val {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Self::Nil | Self::Bool(false))
    }
}

impl From<Lit> for Val {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::String(v) => Self::String(v),
            Lit::Number(v) => Self::Number(v),
            Lit::Bool(v) => Self::Bool(v),
            Lit::Nil => Self::Nil,
        }
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Number(v) => writeln!(f, "{}", v),
            Val::String(v) => writeln!(f, "{}", v),
            Val::Bool(v) => writeln!(f, "{}", v),
            Val::Nil => writeln!(f, "nil"),
            Val::Callable(v) => writeln!(f, "{}", v),
        }
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<native fn>")
    }
}

impl Callable {
    pub fn call(&self, _interpreter: &Interpreter, _args: Vec<Val>) -> Val {
        match self {
            Callable::Clock => Val::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::Clock => 0,
        }
    }
}
