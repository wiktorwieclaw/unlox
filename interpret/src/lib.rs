use ast::{Expr, Lit, Token, TokenKind};

#[derive(Debug, thiserror::Error)]
#[error("[Line {}]: {}", operator.line, kind)]
pub struct Error {
    pub operator: Token,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(operator: Token, kind: ErrorKind) -> Self {
        Self { operator, kind }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Operand must be a number.")]
    ExpectedNumber,
    #[error("Operands must be numbers.")]
    ExpectedNumbers,
    #[error("Operands must be two numbers or two strings.")]
    ExpectedNumbersOrStrings,
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn interpret(expr: Expr) -> Result<Lit> {
    let lit = match expr {
        Expr::Literal(value) => value,
        Expr::Grouping(expr) => interpret(*expr)?,
        Expr::Unary(operator, right) => {
            let right = interpret(*right)?;
            match (&operator.kind, right) {
                (TokenKind::Bang, right) => Lit::Bool(!right.is_truthy()),
                (TokenKind::Minus, Lit::Number(n)) => Lit::Number(-n),
                (TokenKind::Minus, _) => {
                    return Err(Error::new(operator, ErrorKind::ExpectedNumber));
                }
                _ => unreachable!(),
            }
        }
        Expr::Binary(operator, left, right) => {
            let left = interpret(*left)?;
            let right = interpret(*right)?;

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
                    return Err(Error::new(operator, ErrorKind::ExpectedNumbersOrStrings));
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
                    return Err(Error::new(operator, ErrorKind::ExpectedNumbers));
                }
                _ => unreachable!(),
            }
        }
    };
    Ok(lit)
}
