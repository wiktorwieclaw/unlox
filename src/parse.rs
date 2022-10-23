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

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Option<Token>,
    previous: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        let mut parser = Self {
            scanner,
            current: None,
            previous: None,
        };
        parser.advance();
        parser
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

    fn advance(&mut self) {
        self.previous = self.current.take();
        self.current = self.scanner.next();
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn previous(&mut self) -> Token {
        self.previous.take().unwrap()
    }

    #[allow(dead_code)]
    #[allow(clippy::while_let_loop)]
    fn synchronize(&mut self) {
        self.advance();
        loop {
            match self.peek().cloned() {
                Some(token) => {
                    if self.previous().kind == TokenKind::Semicolon {
                        break;
                    }

                    if matches!(
                        token.kind,
                        TokenKind::Class
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

                    self.advance()
                }
                None => break,
            }
        }
    }

    fn expression(&mut self) -> Result<Box<Expr>, Error> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.comparison()?;
        while let Some(TokenKind::BangEqual | TokenKind::EqualEqual) = self.peek().map(|t| &t.kind)
        {
            self.advance();
            expr = Box::new(Expr::Binary {
                operator: self.previous(),
                left: expr,
                right: self.comparison()?,
            });
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.term()?;
        while let Some(
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual,
        ) = self.peek().map(|t| &t.kind)
        {
            self.advance();
            expr = Box::new(Expr::Binary {
                operator: self.previous(),
                left: expr,
                right: self.term()?,
            });
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.factor()?;
        while let Some(TokenKind::Minus | TokenKind::Plus) = self.peek().map(|t| &t.kind) {
            self.advance();
            expr = Box::new(Expr::Binary {
                operator: self.previous(),
                left: expr,
                right: self.factor()?,
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.unary()?;
        while let Some(TokenKind::Slash | TokenKind::Star) = self.peek().map(|t| &t.kind) {
            self.advance();
            expr = Box::new(Expr::Binary {
                operator: self.previous(),
                left: expr,
                right: self.unary()?,
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>, Error> {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Bang | TokenKind::Minus) => {
                self.advance();
                let expr = Expr::Unary {
                    operator: self.previous(),
                    right: self.unary()?,
                };
                Ok(Box::new(expr))
            }
            _ => self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, Error> {
        let expr = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::False) => {
                self.advance();
                Expr::Literal(Lit::Bool(false))
            }
            Some(TokenKind::True) => {
                self.advance();
                Expr::Literal(Lit::Bool(true))
            }
            Some(TokenKind::Nil) => {
                self.advance();
                Expr::Literal(Lit::Nil)
            }
            Some(TokenKind::Number(..) | TokenKind::String(..)) => {
                self.advance();
                match self.previous().kind {
                    TokenKind::Number(value) => Expr::Literal(Lit::Number(value)),
                    TokenKind::String(value) => Expr::Literal(Lit::String(value)),
                    _ => unreachable!(),
                }    
            }
            Some(TokenKind::LeftParen) => {
                self.advance();
                let expr = self.expression()?;
                match self.peek().map(|t| &t.kind) {
                    Some(TokenKind::RightParen) => self.advance(),
                    _ => return Err(Error::new(
                        Token {
                            kind: TokenKind::Eof,
                            lexeme: "".into(),
                            line: self.scanner.line()
                        },
                        "Expected expression.".into()
                    ))
                }
                Expr::Grouping(expr)
            }
            _ => {
                return Err(Error::new(
                    Token {
                        kind: TokenKind::Eof,
                        lexeme: "".into(),
                        line: self.scanner.line(),
                    },
                    "Expected expression".into(),
                ));
            }
        };
        Ok(Box::new(expr))
    }
}
