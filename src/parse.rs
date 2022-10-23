//! # Expression grammar:
//! expression     → equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

use crate::{
    ast::{Expr, Lit},
    error,
    lex::{Scanner, Token, TokenKind},
};

pub struct Error {
    token: Token,
    message: String,
}

impl Error {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

pub struct Parser<'source> {
    scanner: Scanner<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(scanner: Scanner<'source>) -> Self {
        Self { scanner }
    }

    pub fn parse(&mut self) -> Option<Box<Expr>> {
        match self.expression() {
            Ok(ast) => Some(ast),
            Err(Error { token, message }) => {
                error::error(token.line, &message);
                None
            }
        }
    }

    fn synchronize(&mut self) {
        let mut previous = self.scanner.advance();
        loop {
            let current = self.scanner.peek();

            if matches!(
                current.kind,
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

            if previous.kind == TokenKind::Semicolon {
                break;
            }

            previous = self.scanner.advance();
        }
    }

    fn expression(&mut self) -> Result<Box<Expr>, Error> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.comparison()?;
        while let TokenKind::BangEqual | TokenKind::EqualEqual = self.scanner.peek().kind {
            let previous = self.scanner.advance();
            expr = Box::new(Expr::Binary(previous, expr, self.comparison()?));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.term()?;
        while let TokenKind::Less
        | TokenKind::LessEqual
        | TokenKind::Greater
        | TokenKind::GreaterEqual = self.scanner.peek().kind
        {
            let previous = self.scanner.advance();
            expr = Box::new(Expr::Binary(previous, expr, self.term()?));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.factor()?;
        while let TokenKind::Minus | TokenKind::Plus = self.scanner.peek().kind {
            let previous = self.scanner.advance();
            expr = Box::new(Expr::Binary(previous, expr, self.factor()?));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.unary()?;
        while let TokenKind::Slash | TokenKind::Star = self.scanner.peek().kind {
            let previous = self.scanner.advance();
            expr = Box::new(Expr::Binary(previous, expr, self.unary()?));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>, Error> {
        match self.scanner.peek().kind {
            TokenKind::Bang | TokenKind::Minus => {
                let previous = self.scanner.advance();
                let expr = Expr::Unary(previous, self.unary()?);
                Ok(Box::new(expr))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, Error> {
        let token = self.scanner.peek();
        let expr = match token.kind {
            TokenKind::False => {
                self.scanner.advance();
                Expr::Literal(Lit::Bool(false))
            }
            TokenKind::True => {
                self.scanner.advance();
                Expr::Literal(Lit::Bool(true))
            }
            TokenKind::Nil => {
                self.scanner.advance();
                Expr::Literal(Lit::Nil)
            }
            TokenKind::Number(..) | TokenKind::String(..) => {
                let previous = self.scanner.advance();
                match previous.kind {
                    TokenKind::Number(value) => Expr::Literal(Lit::Number(value)),
                    TokenKind::String(value) => Expr::Literal(Lit::String(value)),
                    _ => unreachable!(),
                }
            }
            TokenKind::LeftParen => {
                self.scanner.advance();
                let expr = self.expression()?;
                let token = self.scanner.peek();
                match token.kind {
                    TokenKind::RightParen => {
                        self.scanner.advance();
                    }
                    _ => return Err(Error::new(token.clone(), "Expected expression.".into())),
                }
                Expr::Grouping(expr)
            }
            _ => {
                return Err(Error::new(token.clone(), "Expected expression".into()));
            }
        };
        Ok(Box::new(expr))
    }
}
