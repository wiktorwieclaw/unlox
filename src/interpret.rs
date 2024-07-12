use crate::{
    ast::{Expr, Lit},
    lex::TokenKind,
};

#[allow(clippy::boxed_local)]
pub fn interpret(expr: Box<Expr>) -> Lit {
    match *expr {
        Expr::Literal(value) => value,
        Expr::Grouping(expr) => interpret(expr),
        Expr::Unary(operator, right) => {
            let right = interpret(right);

            match (operator.kind, right) {
                (TokenKind::Bang, right) => Lit::Bool(!is_truthy(right)),
                (TokenKind::Minus, Lit::Number(r)) => Lit::Number(-r),
                (TokenKind::Minus, _) => unimplemented!("Err"),
                _ => unreachable!(),
            }
        }
        Expr::Binary(operator, left, right) => {
            let left = interpret(left);
            let right = interpret(right);

            match (operator.kind, left, right) {
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
                _ => unimplemented!(),
            }
        }
    }
}

fn is_truthy(lit: Lit) -> bool {
    !matches!(lit, Lit::Nil | Lit::Bool(false))
}
