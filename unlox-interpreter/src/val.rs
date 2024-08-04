use unlox_ast::{Lit, StmtIdx, Token};

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
    Function {
        name: String,
        params: Vec<Token>,
        body: Vec<StmtIdx>,
    },
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
            Val::Number(v) => write!(f, "{}", v),
            Val::String(v) => write!(f, "{}", v),
            Val::Bool(v) => write!(f, "{}", v),
            Val::Nil => write!(f, "nil"),
            Val::Callable(v) => write!(f, "{}", v),
        }
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Clock => write!(f, "<native fn>"),
            Callable::Function { name, .. } => write!(f, "<fn {name}>"),
        }
    }
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Callable::Clock => 0,
            Callable::Function { params, .. } => params.len(),
        }
    }
}
