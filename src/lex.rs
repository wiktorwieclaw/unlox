use crate::error;
use lazy_static::lazy_static;
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
    token: Option<Token>,
}

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = HashMap::from([
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
        ("while", TokenKind::While)
    ]);
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
            token: None,
        }
    }

    pub fn next_token(&mut self) -> Token {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
            if let Some(token) = self.token.take() {
                return token;
            }
        }
        Token {
            kind: TokenKind::Eof,
            lexeme: "".into(),
            line: self.line,
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            ';' => self.make_token(TokenKind::Semicolon),
            '*' => self.make_token(TokenKind::Star),
            '!' if self.match_current('=') => self.make_token(TokenKind::BangEqual),
            '!' => self.make_token(TokenKind::Bang),
            '=' if self.match_current('=') => self.make_token(TokenKind::EqualEqual),
            '=' => self.make_token(TokenKind::Equal),
            '<' if self.match_current('=') => self.make_token(TokenKind::LessEqual),
            '<' => self.make_token(TokenKind::Less),
            '>' if self.match_current('=') => self.make_token(TokenKind::GreaterEqual),
            '>' => self.make_token(TokenKind::Greater),
            '/' if self.match_current('/') => self.advance_until_newline(),
            '/' => self.make_token(TokenKind::Slash),
            '"' => self.make_string_token(),
            '0'..='9' => self.make_number_token(),
            c if is_alphabetic(c) => self.make_identifier_token(),
            _ => error::error(self.line, "Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += 1;
        c as char
    }

    fn char_at(&self, idx: usize) -> char {
        self.source.as_bytes()[idx] as char // todo: handle utf8 properly
    }

    fn current_char(&self) -> char {
        self.char_at(self.current)
    }

    fn next_char(&self) -> char {
        self.char_at(self.current + 1)
    }

    fn current_string(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn match_current(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.current_char()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.next_char()
    }

    fn make_token(&mut self, kind: TokenKind) {
        self.token = Some(Token {
            kind,
            lexeme: self.current_string().into(),
            line: self.line,
        });
    }

    fn make_string_token(&mut self) {
        self.advance_until_string_end();

        if self.is_at_end() {
            error::error(self.line, "Unterminated string");
            return;
        }

        self.advance();
        let trimmed = trim_bounds(self.current_string(), 1);
        let value = String::from(trimmed);
        self.make_token(TokenKind::String(value))
    }

    fn make_number_token(&mut self) {
        self.advance_while_digit(10);

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            self.advance_while_digit(10);
        }

        let value: f64 = self.current_string().parse().unwrap();
        self.make_token(TokenKind::Number(value));
    }

    fn make_identifier_token(&mut self) {
        self.advance_while_alphanumeric();
        let text = self.current_string();
        let kind = KEYWORDS.get(text).cloned().unwrap_or(TokenKind::Identifier);
        self.make_token(kind);
    }

    fn advance_until_newline(&mut self) {
        while self.peek() != '\n' && self.is_at_end() {
            self.advance();
        }
    }

    fn advance_until_string_end(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
    }

    fn advance_while_digit(&mut self, radix: u32) {
        while self.peek().is_digit(radix) {
            self.advance();
        }
    }

    fn advance_while_alphanumeric(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
    }
}

fn trim_bounds(s: &str, bounds_len: usize) -> &str {
    &s[bounds_len..(s.len() - bounds_len)]
}

fn is_alphabetic(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '_')
}

fn is_alphanumeric(c: char) -> bool {
    is_alphabetic(c) || c.is_digit(10)
}

#[cfg(test)]
mod test {
    use super::*;

    fn expect_tokens(code: &str, expected_tokens: &[Token]) {
        let mut expected_tokens = expected_tokens.iter();
        let mut scanner = Scanner::new(code);
        let mut token = scanner.next_token();
        while token.kind != TokenKind::Eof {
            assert_eq!(&token, expected_tokens.next().unwrap());
            token = scanner.next_token();
        }
    }

    #[test]
    fn string_variable() {
        let code = r#"var text = "hello";"#;
        let expected_tokens = [
            Token {
                kind: TokenKind::Var,
                lexeme: "var".into(),
                line: 1,
            },
            Token {
                kind: TokenKind::Identifier,
                lexeme: "text".into(),
                line: 1,
            },
            Token {
                kind: TokenKind::Equal,
                lexeme: "=".into(),
                line: 1,
            },
            Token {
                kind: TokenKind::String("hello".into()),
                lexeme: r#""hello""#.into(),
                line: 1,
            },
            Token {
                kind: TokenKind::Semicolon,
                lexeme: ";".into(),
                line: 1,
            },
            Token {
                kind: TokenKind::Eof,
                lexeme: "".into(),
                line: 1,
            },
        ];
        expect_tokens(code, &expected_tokens);
    }
}
