use std::ops::Range;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Range<usize>,
    pub line: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
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
    StringUnterminated(String),
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

    // Unexpected character
    Unknown,

    // end of input
    #[default]
    Eof,
}

pub trait TokenStream {
    fn next(&mut self) -> Token;
    fn peek(&mut self) -> &Token;
}

pub trait TokenStreamExt {
    fn match_next(&mut self, matcher: impl FnOnce(&TokenKind) -> bool) -> Result<Token, Token>;
    fn eof(&mut self) -> bool;
}

impl<T: TokenStream> TokenStreamExt for T {
    fn match_next(&mut self, matcher: impl FnOnce(&TokenKind) -> bool) -> Result<Token, Token> {
        let token = self.peek();
        if matcher(&token.kind) {
            Ok(self.next())
        } else {
            Err(token.clone())
        }
    }

    fn eof(&mut self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

pub mod matcher {
    use super::*;

    pub fn eq(kind: TokenKind) -> impl FnOnce(&TokenKind) -> bool {
        move |k| *k == kind
    }
}
