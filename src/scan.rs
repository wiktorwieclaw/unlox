use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::error;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
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
    had_eof: bool,
}

pub fn tokens(source: &str) -> Scanner {
    Scanner::new(source)
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
    fn new(source: &'a str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
            token: None,
            had_eof: false,
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
            '/' if self.match_current('/') => {
                while self.peek() != '\n' && self.is_at_end() {
                    self.advance();
                }
            }
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
        self.source.as_bytes()[idx] as char
    }

    fn current_char(&self) -> char {
        self.char_at(self.current)
    }

    fn make_token(&mut self, kind: TokenKind) {
        let lexeme = String::from(&self.source[self.start..self.current]);
        self.token = Some(Token {
            kind,
            lexeme,
            line: self.line,
        });
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
        self.char_at(self.current + 1)
    }

    fn make_string_token(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            crate::error::error(self.line, "Unterminated string");
            return;
        }

        self.advance();
        let value = String::from(&self.source[(self.start + 1)..(self.current - 1)]);
        self.make_token(TokenKind::String(value));
    }

    fn make_number_token(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();
        self.make_token(TokenKind::Number(value));
    }

    fn make_identifier_token(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let kind = KEYWORDS.get(text).cloned().unwrap_or(TokenKind::Identifier);
        self.make_token(kind);
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
            if self.token.is_some() {
                return self.token.take();
            }
        }
        if !self.had_eof {
            self.had_eof = true;
            return Some(Token {
                kind: TokenKind::Eof,
                lexeme: String::default(),
                line: self.line,
            });
        }
        None
    }
}

fn is_alphabetic(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '_')
}

fn is_alphanumeric(c: char) -> bool {
    is_alphabetic(c) || c.is_digit(10)
}
