use crate::error;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // single character
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String(String),
    Number(f64),

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // eof
    Eof,
}

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
}

static KEYWORDS: Lazy<HashMap<&'static str, TokenKind>> = Lazy::new(|| {
    HashMap::from([
        ("and", TokenKind::And),
        ("class", TokenKind::Class),
        ("else", TokenKind::Else),
        ("false", TokenKind::False),
        ("for", TokenKind::For),
        ("fun", TokenKind::Fun),
        ("if", TokenKind::If),
        ("nil", TokenKind::Nil),
        ("or", TokenKind::Or),
        ("print", TokenKind::Print),
        ("return", TokenKind::Return),
        ("super", TokenKind::Super),
        ("this", TokenKind::This),
        ("true", TokenKind::True),
        ("var", TokenKind::Var),
        ("while", TokenKind::While),
    ])
});

impl Iterator for Scanner<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.start = self.current;
            match self.advance() {
                Some(' ' | '\r' | '\t') => {}
                Some('\n') => self.line += 1,
                Some('(') => return Some(self.token(TokenKind::LeftParen)),
                Some(')') => return Some(self.token(TokenKind::RightParen)),
                Some('{') => return Some(self.token(TokenKind::LeftBrace)),
                Some('}') => return Some(self.token(TokenKind::RightBrace)),
                Some(',') => return Some(self.token(TokenKind::Comma)),
                Some('.') => return Some(self.token(TokenKind::Dot)),
                Some('-') => return Some(self.token(TokenKind::Minus)),
                Some('+') => return Some(self.token(TokenKind::Plus)),
                Some(';') => return Some(self.token(TokenKind::Semicolon)),
                Some('*') => return Some(self.token(TokenKind::Star)),
                Some('!') if self.match_current('=') => {
                    return Some(self.token(TokenKind::BangEqual))
                }
                Some('!') => return Some(self.token(TokenKind::Bang)),
                Some('=') if self.match_current('=') => {
                    return Some(self.token(TokenKind::EqualEqual))
                }
                Some('=') => return Some(self.token(TokenKind::Equal)),
                Some('<') if self.match_current('=') => {
                    return Some(self.token(TokenKind::LessEqual))
                }
                Some('<') => return Some(self.token(TokenKind::Less)),
                Some('>') if self.match_current('=') => {
                    return Some(self.token(TokenKind::GreaterEqual))
                }
                Some('>') => return Some(self.token(TokenKind::Greater)),
                Some('/') if self.match_current('/') => self.advance_until_newline(),
                Some('/') => return Some(self.token(TokenKind::Slash)),
                Some('"') => match self.string_token() {
                    Some(t) => return Some(t),
                    None => error::error(self.line, "Unterminated string"),
                },
                Some('0'..='9') => return Some(self.number_token()),
                Some('A'..='Z' | 'a'..='z' | '_') => return Some(self.ident_token()),
                Some(_) => error::error(self.line, "Unexpected character."),
                None => return None,
            }
        }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current_char();
        self.current += 1;
        c
    }

    fn current_char(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn next_char(&self) -> Option<char> {
        self.source[(self.current + 1)..].chars().next()
    }

    fn current_string(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn match_current(&mut self, expected: char) -> bool {
        match self.current_char() {
            Some(c) if c == expected => {
                self.current += 1;
                true
            }
            _ => false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.current_char()
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        self.next_char()
    }

    fn token(&mut self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.current_string().into(),
            line: self.line,
        }
    }

    fn string_token(&mut self) -> Option<Token> {
        self.advance_until_string_end();

        if self.is_at_end() {
            return None;
        }

        self.advance();
        let trimmed = trim_bounds(self.current_string(), 1);
        let value = String::from(trimmed);
        Some(self.token(TokenKind::String(value)))
    }

    fn number_token(&mut self) -> Token {
        self.advance_while_digit(10);

        if let (Some('.'), Some('0'..='9')) = (self.peek(), self.peek_next()) {
            self.advance();
            self.advance_while_digit(10);
        };

        let value: f64 = self.current_string().parse().unwrap();
        self.token(TokenKind::Number(value))
    }

    fn ident_token(&mut self) -> Token {
        self.advance_while_alphanumeric();
        let text = self.current_string();
        let kind = KEYWORDS.get(text).cloned().unwrap_or(TokenKind::Identifier);
        self.token(kind)
    }

    fn advance_until_newline(&mut self) {
        loop {
            match self.peek() {
                Some('\n') | None => break,
                _ => self.advance(),
            };
        }
    }

    fn advance_until_string_end(&mut self) {
        loop {
            match self.peek() {
                Some('"') | None => break,
                Some(c) => {
                    if c == '\n' {
                        self.line += 1;
                    }
                    self.advance();
                }
            };
        }
    }

    fn advance_while_digit(&mut self, radix: u32) {
        loop {
            match self.peek() {
                Some(c) if c.is_digit(radix) => self.advance(),
                _ => break,
            };
        }
    }

    fn advance_while_alphanumeric(&mut self) {
        while let Some('A'..='Z' | 'a'..='z' | '_') = self.peek() {
            self.advance();
        }
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
            scanner.next(),
            Some(Token {
                kind: TokenKind::LeftParen,
                lexeme: "(".into(),
                line: 1
            })
        );
        assert_eq!(
            scanner.next(),
            Some(Token {
                kind: TokenKind::RightParen,
                lexeme: ")".into(),
                line: 1
            })
        )
    }

    #[test]
    fn scans_float() {
        let mut scanner = Scanner::new("12.345");
        assert_eq!(
            scanner.next(),
            Some(Token {
                kind: TokenKind::Number(12.345),
                lexeme: "12.345".into(),
                line: 1
            })
        )
    }

    #[test]
    fn scans_string() {
        let mut scanner = Scanner::new(r#""string""#);
        assert_eq!(
            scanner.next(),
            Some(Token {
                kind: TokenKind::String("string".into()),
                lexeme: r#""string""#.into(),
                line: 1
            })
        )
    }
}
