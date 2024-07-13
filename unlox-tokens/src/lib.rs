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
    String { value: String, is_terminated: bool },
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
    Eof,
}

pub trait TokenStream {
    fn next(&mut self) -> Token;
    fn peek(&mut self) -> &Token;
}

pub trait TokenStreamExt {
    fn try_match(&mut self, pred: impl FnOnce(&TokenKind) -> bool) -> Option<Token>;
    fn eof(&mut self) -> bool;
}

impl<T: TokenStream> TokenStreamExt for T {
    fn try_match(&mut self, pred: impl FnOnce(&TokenKind) -> bool) -> Option<Token> {
        if pred(&self.peek().kind) {
            Some(self.next())
        } else {
            None
        }
    }

    fn eof(&mut self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}
