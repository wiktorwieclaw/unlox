use crate::{
    selection::Selection,
    token::{Token, TokenKind},
};

pub struct Scanner<'src> {
    inner: InnerScanner<'src>,
    peeked: Option<Token>,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Scanner {
            inner: InnerScanner {
                cursor: Selection::new(source),
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

struct InnerScanner<'src> {
    cursor: Selection<'src>,
}

impl InnerScanner<'_> {
    fn advance(&mut self) -> Token {
        loop {
            self.cursor.clear();
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
                Some('!') if self.cursor.advance_match('=') => {
                    break self.token(TokenKind::BangEqual)
                }
                Some('!') => break self.token(TokenKind::Bang),
                Some('=') if self.cursor.advance_match('=') => {
                    break self.token(TokenKind::EqualEqual)
                }
                Some('=') => break self.token(TokenKind::Equal),
                Some('<') if self.cursor.advance_match('=') => {
                    break self.token(TokenKind::LessEqual)
                }
                Some('<') => break self.token(TokenKind::Less),
                Some('>') if self.cursor.advance_match('=') => {
                    break self.token(TokenKind::GreaterEqual)
                }
                Some('>') => break self.token(TokenKind::Greater),
                Some('/') if self.cursor.advance_match('/') => {
                    self.cursor.advance_while(|c| c != '\n')
                }
                Some('/') => break self.token(TokenKind::Slash),
                Some('"') => break self.string_token(),
                Some('0'..='9') => break self.number_token(),
                Some('A'..='Z' | 'a'..='z' | '_') => break self.ident_token(),
                None => break self.token(TokenKind::Eof),
                _ => break self.token(TokenKind::Unknown),
            }
        }
    }

    fn token(&mut self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.cursor.str().into(),
            line: self.cursor.line(),
        }
    }

    fn string_token(&mut self) -> Token {
        self.cursor.advance_while(|c| c != '"');
        let is_terminated = !self.cursor.is_at_end();
        let value = if is_terminated {
            self.cursor.advance();
            trim_bounds(self.cursor.str(), 1).to_string()
        } else {
            let s = self.cursor.str();
            s[1..].to_string()
        };
        self.token(TokenKind::String {
            value,
            is_terminated,
        })
    }

    fn number_token(&mut self) -> Token {
        self.cursor.advance_while(|c| c.is_ascii_digit());

        if let (Some('.'), Some('0'..='9')) = (self.cursor.peek(), self.cursor.peek_2()) {
            self.cursor.advance();
            self.cursor.advance_while(|c| c.is_ascii_digit());
        };

        let value: f64 = self.cursor.str().parse().unwrap();
        self.token(TokenKind::Number(value))
    }

    fn ident_token(&mut self) -> Token {
        self.cursor
            .advance_while(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '_'));
        let text = self.cursor.str();
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
                kind: TokenKind::String {
                    value: "string".into(),
                    is_terminated: true
                },
                lexeme: r#""string""#.into(),
                line: 1
            }
        )
    }
}
