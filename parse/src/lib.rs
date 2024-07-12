//! # Expression grammar:
//! expression     → equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

use ast::{Expr, Lit, Token, TokenKind};
use lexer::Scanner;

pub struct Error {
    pub token: Token,
    pub message: String,
}

impl Error {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub fn parse(mut scanner: Scanner) -> Result<Expr> {
    expression(&mut scanner)
}

#[allow(dead_code)]
fn synchronize(scanner: &mut Scanner) {
    let mut current = scanner.advance();
    loop {
        if current.kind == TokenKind::Semicolon {
            break;
        }

        let next = scanner.peek();

        if matches!(
            next.kind,
            TokenKind::Eof
                | TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return
        ) {
            break;
        }

        current = scanner.advance();
    }
}

fn expression(scanner: &mut Scanner) -> Result<Expr> {
    equality(scanner)
}

fn equality(scanner: &mut Scanner) -> Result<Expr> {
    let mut expr = comparison(scanner)?;
    while let TokenKind::BangEqual | TokenKind::EqualEqual = scanner.peek().kind {
        let token = scanner.advance();
        expr = Expr::Binary(token, Box::new(expr), Box::new(comparison(scanner)?));
    }
    Ok(expr)
}

fn comparison(scanner: &mut Scanner) -> Result<Expr> {
    let mut expr = term(scanner)?;
    while let TokenKind::Less
    | TokenKind::LessEqual
    | TokenKind::Greater
    | TokenKind::GreaterEqual = scanner.peek().kind
    {
        let token = scanner.advance();
        expr = Expr::Binary(token, Box::new(expr), Box::new(term(scanner)?));
    }
    Ok(expr)
}

fn term(scanner: &mut Scanner) -> Result<Expr> {
    let mut expr = factor(scanner)?;
    while let TokenKind::Minus | TokenKind::Plus = scanner.peek().kind {
        let token = scanner.advance();
        expr = Expr::Binary(token, Box::new(expr), Box::new(factor(scanner)?));
    }
    Ok(expr)
}

fn factor(scanner: &mut Scanner) -> Result<Expr> {
    let mut expr = unary(scanner)?;
    while let TokenKind::Slash | TokenKind::Star = scanner.peek().kind {
        let token = scanner.advance();
        expr = Expr::Binary(token, Box::new(expr), Box::new(unary(scanner)?));
    }
    Ok(expr)
}

fn unary(scanner: &mut Scanner) -> Result<Expr> {
    match scanner.peek().kind {
        TokenKind::Bang | TokenKind::Minus => {
            let token = scanner.advance();
            let expr = Expr::Unary(token, Box::new(unary(scanner)?));
            Ok(expr)
        }
        _ => primary(scanner),
    }
}

fn primary(scanner: &mut Scanner) -> Result<Expr> {
    let token = scanner.peek();
    let expr = match &token.kind {
        TokenKind::False => Expr::Literal(Lit::Bool(false)),
        TokenKind::True => Expr::Literal(Lit::Bool(true)),
        TokenKind::Nil => Expr::Literal(Lit::Nil),
        TokenKind::Number(n) => Expr::Literal(Lit::Number(*n)),
        TokenKind::String {
            value,
            is_terminated: true,
        } => Expr::Literal(Lit::String(value.clone())),
        TokenKind::String {
            is_terminated: false,
            ..
        } => {
            return Err(Error::new(token.clone(), "Unterminated string.".into()));
        }
        TokenKind::LeftParen => {
            scanner.advance();
            let expr = expression(scanner)?;
            let token = scanner.peek();
            if token.kind != TokenKind::RightParen {
                return Err(Error::new(
                    token.clone(),
                    r#"Expected ")" after expression."#.into(),
                ));
            }
            Expr::Grouping(Box::new(expr))
        }
        _ => {
            return Err(Error::new(
                token.clone(),
                "Expected a literal or grouping".into(),
            ));
        }
    };
    scanner.advance();
    Ok(expr)
}
