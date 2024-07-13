use selection::Selection;
use unlox_tokens::{Token, TokenKind, TokenStream};

mod selection;

pub struct Lexer<'src> {
    inner: LexerInner<'src>,
    peeked: Option<Token>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Lexer {
            inner: LexerInner {
                selection: Selection::new(source),
            },
            peeked: None,
        }
    }
}

impl TokenStream for Lexer<'_> {
    fn next(&mut self) -> Token {
        match self.peeked.take() {
            Some(token) => token,
            None => self.inner.advance(),
        }
    }

    fn peek(&mut self) -> &Token {
        self.peeked.get_or_insert_with(|| self.inner.advance())
    }
}

struct LexerInner<'src> {
    selection: Selection<'src>,
}

impl LexerInner<'_> {
    fn advance(&mut self) -> Token {
        loop {
            self.selection.clear();
            match self.selection.advance() {
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
                Some('!') if self.selection.try_match('=').is_some() => {
                    break self.token(TokenKind::BangEqual)
                }
                Some('!') => break self.token(TokenKind::Bang),
                Some('=') if self.selection.try_match('=').is_some() => {
                    break self.token(TokenKind::EqualEqual)
                }
                Some('=') => break self.token(TokenKind::Equal),
                Some('<') if self.selection.try_match('=').is_some() => {
                    break self.token(TokenKind::LessEqual)
                }
                Some('<') => break self.token(TokenKind::Less),
                Some('>') if self.selection.try_match('=').is_some() => {
                    break self.token(TokenKind::GreaterEqual)
                }
                Some('>') => break self.token(TokenKind::Greater),
                Some('/') if self.selection.try_match('/').is_some() => {
                    self.selection.advance_while(|c| c != '\n')
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
            lexeme: self.selection.str().into(),
            line: self.selection.line(),
        }
    }

    fn string_token(&mut self) -> Token {
        self.selection.advance_while(|c| c != '"');
        let is_terminated = !self.selection.eof();
        let str = if is_terminated {
            self.selection.advance();
            let str = self.selection.str();
            str[1..str.len() - 1].to_owned()
        } else {
            let str = self.selection.str();
            str[1..].to_owned()
        };
        self.token(TokenKind::String {
            value: str,
            is_terminated,
        })
    }

    fn number_token(&mut self) -> Token {
        self.selection.advance_while(|c| c.is_ascii_digit());

        if let Some(('.', '0'..='9')) = self.selection.peek().zip(self.selection.peek_second()) {
            self.selection.advance();
            self.selection.advance_while(|c| c.is_ascii_digit());
        };

        let value: f64 = self.selection.str().parse().unwrap();
        self.token(TokenKind::Number(value))
    }

    fn ident_token(&mut self) -> Token {
        self.selection
            .advance_while(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '_'));
        let text = self.selection.str();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scans_parens() {
        let mut lexer = Lexer::new("()");
        assert_eq!(
            lexer.next(),
            Token {
                kind: TokenKind::LeftParen,
                lexeme: "(".into(),
                line: 1
            }
        );
        assert_eq!(
            lexer.next(),
            Token {
                kind: TokenKind::RightParen,
                lexeme: ")".into(),
                line: 1
            }
        )
    }

    #[test]
    fn scans_float() {
        let mut lexer = Lexer::new("12.345");
        assert_eq!(
            lexer.next(),
            Token {
                kind: TokenKind::Number(12.345),
                lexeme: "12.345".into(),
                line: 1
            }
        )
    }

    #[test]
    fn scans_string() {
        let mut lexer = Lexer::new(r#""string""#);
        assert_eq!(
            lexer.next(),
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
