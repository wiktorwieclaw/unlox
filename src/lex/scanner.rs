use crate::error;

use super::{
    cursor::Cursor,
    token::{Token, TokenKind},
};

pub struct Scanner<'a> {
    inner: Inner<'a>,
    peeked: Option<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            inner: Inner {
                cursor: Cursor::new(source),
            },
            peeked: None,
        }
    }

    pub fn advance(&mut self) -> Token {
        match self.peeked.take() {
            Some(token) => token,
            None => self.inner.advance(),
        }
    }

    pub fn peek(&mut self) -> &Token {
        self.peeked.get_or_insert_with(|| self.inner.advance())
    }
}

struct Inner<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Inner<'a> {
    fn advance(&mut self) -> Token {
        loop {
            self.cursor.reset_position();
            match self.cursor.advance() {
                Some(' ' | '\r' | '\t' | '\n') => (),
                Some('(') => break self.token(TokenKind::LeftParen),
                Some(')') => break self.token(TokenKind::RightParen),
                Some('{') => break self.token(TokenKind::LeftBrace),
                Some('}') => break self.token(TokenKind::RightBrace),
                Some(',') => break self.token(TokenKind::Comma),
                Some('.') => break self.token(TokenKind::Dot),
                Some('-') => break self.token(TokenKind::Minus),
                Some('+') => break self.token(TokenKind::Plus),
                Some(';') => break self.token(TokenKind::Semicolon),
                Some('*') => break self.token(TokenKind::Star),
                Some('!') if self.cursor.match_current('=') => {
                    break self.token(TokenKind::BangEqual)
                }
                Some('!') => break self.token(TokenKind::Bang),
                Some('=') if self.cursor.match_current('=') => {
                    break self.token(TokenKind::EqualEqual)
                }
                Some('=') => break self.token(TokenKind::Equal),
                Some('<') if self.cursor.match_current('=') => {
                    break self.token(TokenKind::LessEqual)
                }
                Some('<') => break self.token(TokenKind::Less),
                Some('>') if self.cursor.match_current('=') => {
                    break self.token(TokenKind::GreaterEqual)
                }
                Some('>') => break self.token(TokenKind::Greater),
                Some('/') if self.cursor.match_current('/') => {
                    self.cursor.advance_while(|c| c != '\n')
                }
                Some('/') => break self.token(TokenKind::Slash),
                Some('"') => match self.string_token() {
                    Some(token) => break token,
                    None => error::error(self.cursor.line(), "Unterminated string."),
                },
                Some('0'..='9') => break self.number_token(),
                Some('A'..='Z' | 'a'..='z' | '_') => break self.ident_token(),
                None => break self.token(TokenKind::Eof),
                _ => error::error(self.cursor.line(), "Unexpected character."),
            }
        }
    }

    fn token(&mut self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.cursor.current_str().into(),
            line: self.cursor.line(),
        }
    }

    fn string_token(&mut self) -> Option<Token> {
        self.cursor.advance_while(|c| c != '"');

        if self.cursor.is_at_end() {
            return None;
        }

        self.cursor.advance();
        let trimmed = trim_bounds(self.cursor.current_str(), 1);
        let value = String::from(trimmed);
        Some(self.token(TokenKind::String(value)))
    }

    fn number_token(&mut self) -> Token {
        self.cursor.advance_while(|c| c.is_ascii_digit());

        if let (Some('.'), Some('0'..='9')) = (self.cursor.peek(), self.cursor.peek_next()) {
            self.cursor.advance();
            self.cursor.advance_while(|c| c.is_ascii_digit());
        };

        let value: f64 = self.cursor.current_str().parse().unwrap();
        self.token(TokenKind::Number(value))
    }

    fn ident_token(&mut self) -> Token {
        self.cursor
            .advance_while(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '_'));
        let text = self.cursor.current_str();
        let kind = match text {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };
        self.token(kind)
    }
}

fn trim_bounds(s: &str, bounds_len: usize) -> &str {
    &s[bounds_len..(s.len() - bounds_len)]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scans_parens() {
        let mut scanner = Scanner::new("()");
        assert_eq!(
            scanner.advance(),
            Token {
                kind: TokenKind::LeftParen,
                lexeme: "(".into(),
                line: 1
            }
        );
        assert_eq!(
            scanner.advance(),
            Token {
                kind: TokenKind::RightParen,
                lexeme: ")".into(),
                line: 1
            }
        )
    }

    #[test]
    fn scans_float() {
        let mut scanner = Scanner::new("12.345");
        assert_eq!(
            scanner.advance(),
            Token {
                kind: TokenKind::Number(12.345),
                lexeme: "12.345".into(),
                line: 1
            }
        )
    }

    #[test]
    fn scans_string() {
        let mut scanner = Scanner::new(r#""string""#);
        assert_eq!(
            scanner.advance(),
            Token {
                kind: TokenKind::String("string".into()),
                lexeme: r#""string""#.into(),
                line: 1
            }
        )
    }
}
