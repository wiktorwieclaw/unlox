//! # Expression grammar:
//! expression     → equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

use crate::{
    ast::{Expr, Literal},
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

macro_rules! check_token {
    ($self:expr, $( $pattern:pat_param )|+) => {
        !$self.is_at_end() && matches!($self.peek().kind, $( $pattern )|+)
    }
}

macro_rules! match_token {
    ($self:expr, $( $pattern:pat_param )|+) => {
        if check_token!($self, $( $pattern )|+) {
            $self.advance();
            true
        } else {
            false
        }
    }
}

macro_rules! consume_token {
    ($self:expr, $( $pattern:pat_param )|+, $message:expr) => {
        if check_token!($self, $( $pattern )|+) {
            Ok($self.advance())
        } else {
            Err(Error::new($self.peek().clone(), $message))
        }
    }
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
        self.current = Some(self.scanner.next_token());
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        self.current.as_ref().unwrap()
    }

    fn previous(&mut self) -> Token {
        self.previous.take().unwrap()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            if matches!(
                self.peek().kind,
                TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return
            ) {
                return;
            }

            self.advance()
        }
    }

    fn expression(&mut self) -> Result<Box<Expr>, Error> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, Error> {
        let mut expr = self.comparison()?;
        while match_token!(self, TokenKind::BangEqual | TokenKind::EqualEqual) {
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
        while match_token!(
            self,
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual
        ) {
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
        while match_token!(self, TokenKind::Minus | TokenKind::Plus) {
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
        while match_token!(self, TokenKind::Slash | TokenKind::Star) {
            expr = Box::new(Expr::Binary {
                operator: self.previous(),
                left: expr,
                right: self.unary()?,
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>, Error> {
        if match_token!(self, TokenKind::Bang | TokenKind::Minus) {
            let expr = Expr::Unary {
                operator: self.previous(),
                right: self.unary()?,
            };
            Ok(Box::new(expr))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, Error> {
        let expr = if match_token!(self, TokenKind::False) {
            Expr::Literal(Literal::Bool(false))
        } else if match_token!(self, TokenKind::True) {
            Expr::Literal(Literal::Bool(true))
        } else if match_token!(self, TokenKind::Nil) {
            Expr::Literal(Literal::Nil)
        } else if match_token!(self, TokenKind::Number(..) | TokenKind::String(..)) {
            match self.previous().kind {
                TokenKind::Number(value) => Expr::Literal(Literal::Number(value)),
                TokenKind::String(value) => Expr::Literal(Literal::String(value)),
                _ => unreachable!(),
            }
        } else if match_token!(self, TokenKind::LeftParen) {
            let expr = self.expression()?;
            consume_token!(self, TokenKind::RightParen, "Expected expression.".into())?;
            Expr::Grouping(expr)
        } else {
            return Err(Error::new(
                self.peek().clone(),
                "Expected expression".into(),
            ));
        };
        Ok(Box::new(expr))
    }
}
