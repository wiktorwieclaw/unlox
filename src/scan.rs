use std::any::Any;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Box<dyn Any>>,
    pub line: u32,
}

#[derive(Debug, Clone, Copy)]
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
    String,
    Number,

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

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
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
            _ => crate::error::error(self.line, "Unexpected character."),
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

    fn make_token_with_value(&mut self, kind: TokenKind, value: Option<Box<dyn Any>>) {
        let lexeme = String::from(&self.source[self.start..self.current]);
        self.token = Some(Token {
            kind,
            lexeme,
            literal: value,
            line: self.line,
        });
    }

    fn make_token(&mut self, kind: TokenKind) {
        self.make_token_with_value(kind, None);
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
        while self.peek() != '"' && self.is_at_end() {
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
        self.make_token_with_value(TokenKind::String, Some(Box::new(value)));
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
        self.make_token_with_value(TokenKind::Number, Some(Box::new(value)));
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
                literal: None,
                line: self.line,
            });
        }
        None
    }
}
