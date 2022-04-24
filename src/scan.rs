use std::any::Any;

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

struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
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

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += 1;
        c as char
    }

    fn current_char(&self) -> char {
        self.source.as_bytes()[self.current] as char
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        let lexeme = String::from(&self.source[self.start..self.current]);
        Token {
            kind,
            lexeme,
            literal: None,
            line: self.line,
        }
    }

    fn match_c(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_at_end() {
            let c = self.advance();
            let token = match c {
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
                '!' => {
                    let kind = if self.match_c('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    };
                    self.make_token(kind)
                }
                '=' => {
                    let kind = if self.match_c('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    self.make_token(kind)
                }
                '<' => {
                    let kind = if self.match_c('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    self.make_token(kind)
                }
                '>' => {
                    let kind = if self.match_c('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    self.make_token(kind)
                }
                _ => {
                    crate::error::error(self.line, "Unexpected character.");
                    continue;
                }
            };
            return Some(token);
        }
        None
    }
}
