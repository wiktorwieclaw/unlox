//! # Expression grammar:
//! ```text
//! program        → declaration* EOF ;
//!
//! declaration    → var_decl | statement ;
//!
//! statement      → expr_stmt | for_stmt | if_stmt | print_stmt | while_stmt | block ;
//!
//! expr_stmt      → expression ";" ;
//! for_stmt       → "for" "(" (var_decl | expr_stmt | ";" ) expression? ";" expression? ")" statement;
//! if_stmt        → "if" "(" epxression ")" statement ( "else" statement)? ;
//! print_stmt     → "print" expression ";" ;
//! while_stmt     → "while" "(" expression ")" statement ;
//! block          → "{" declaration* "}" ;
//!
//! var_decl       → "var" IDENTIFIER ( "=" expression )? ";" ;
//! expression     → assignment ;
//! assignment     → IDENTIFIER "=" assignment | logic_or ;
//! logic_or       → logic_and ( "or" logic_and )* ;
//! logic_and      → equality ( "and" equality )* ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! call           → primary ( "(" arguments? ")" )*  ;
//! arguments      → expression ( "," expression )* ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER;
//! ```

use std::fmt::Display;

use unlox_ast::{
    tokens::{matcher, TokenStream, TokenStreamExt},
    Expr, Lit, Stmt, Token, TokenKind,
};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    pub token: Token,
    pub message: String,
}

impl Error {
    fn new(token: Token, message: impl Display) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub fn parse(mut stream: impl TokenStream) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    while !stream.eof() {
        stmts.push(declaration(&mut stream));
    }
    Ok(stmts)
}

fn declaration(stream: &mut impl TokenStream) -> Stmt {
    let token = stream.peek();
    let result = match &token.kind {
        TokenKind::Var => {
            stream.next();
            var_decl(stream)
        }
        _ => statement(stream),
    };
    result
        .inspect_err(|e| eprintln!("{e}"))
        .ok()
        .unwrap_or_else(|| {
            synchronize(stream);
            Stmt::ParseErr
        })
}

fn statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let token = stream.peek();
    match &token.kind {
        TokenKind::For => {
            stream.next();
            for_statement(stream)
        }
        TokenKind::If => {
            stream.next();
            if_statement(stream)
        }
        TokenKind::Print => {
            stream.next();
            print_statement(stream)
        }
        TokenKind::While => {
            stream.next();
            while_statement(stream)
        }
        TokenKind::LeftBrace => {
            stream.next();
            Ok(Stmt::Block(block(stream)?))
        }
        _ => expression_statement(stream),
    }
}

fn for_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    stream
        .match_next(matcher::eq(TokenKind::LeftParen))
        .map_err(|t| Error::new(t, "Expected '(' after 'for'."))?;
    let init = match stream.peek().kind {
        TokenKind::Semicolon => {
            stream.next();
            None
        }
        TokenKind::Var => {
            stream.next();
            Some(var_decl(stream)?)
        }
        _ => Some(expression_statement(stream)?),
    };

    let cond = if stream.peek().kind != TokenKind::Semicolon {
        Some(expression(stream)?)
    } else {
        None
    };

    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after loop condition."))?;

    let inc = if stream.peek().kind != TokenKind::RightParen {
        Some(expression(stream)?)
    } else {
        None
    };

    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after for clauses."))?;

    let mut body = statement(stream)?;
    if let Some(inc) = inc {
        body = Stmt::Block(vec![body, Stmt::Expression(inc)])
    }
    let cond = cond.unwrap_or(Expr::Literal(Lit::Bool(true)));
    let while_stmt = Stmt::While {
        cond,
        body: Box::new(body),
    };
    let for_stmt = if let Some(init) = init {
        Stmt::Block(vec![init, while_stmt])
    } else {
        while_stmt
    };
    Ok(for_stmt)
}

fn if_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    stream
        .match_next(matcher::eq(TokenKind::LeftParen))
        .map_err(|t| Error::new(t, "Expected '(' after 'if'."))?;
    let cond = expression(stream)?;
    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after if condition."))?;
    let then_branch = statement(stream)?;
    let else_branch = stream
        .match_next(matcher::eq(TokenKind::Else))
        .ok()
        .map(|_| statement(stream))
        .transpose()?;
    Ok(Stmt::If {
        cond,
        then_branch: Box::new(then_branch),
        else_branch: else_branch.map(Box::new),
    })
}

fn while_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    stream
        .match_next(matcher::eq(TokenKind::LeftParen))
        .map_err(|t| Error::new(t, "Expected '(' after 'while'."))?;
    let cond = expression(stream)?;
    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after condition."))?;
    let body = statement(stream)?;
    Ok(Stmt::While {
        cond,
        body: Box::new(body),
    })
}

fn print_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let expr = expression(stream)?;
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after value."))?;
    Ok(Stmt::Print(expr))
}

fn expression_statement(stream: &mut impl TokenStream) -> Result<Stmt> {
    let expr = expression(stream)?;
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after expression."))?;
    Ok(Stmt::Expression(expr))
}

fn block(stream: &mut impl TokenStream) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];

    while stream.peek().kind != TokenKind::RightBrace && !stream.eof() {
        stmts.push(declaration(stream));
    }

    stream
        .match_next(matcher::eq(TokenKind::RightBrace))
        .map_err(|t| Error::new(t, "Expect '}' after block."))?;
    Ok(stmts)
}

fn var_decl(stream: &mut impl TokenStream) -> Result<Stmt> {
    let name = stream
        .match_next(matcher::eq(TokenKind::Identifier))
        .map_err(|t| Error::new(t, "Expected variable name."))?;
    let token = stream.peek();
    let init = if token.kind == TokenKind::Equal {
        stream.next();
        Some(expression(stream)?)
    } else {
        None
    };
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after variable declaration."))?;
    Ok(Stmt::VarDecl { name, init })
}

fn expression(stream: &mut impl TokenStream) -> Result<Expr> {
    assignment(stream)
}

fn assignment(stream: &mut impl TokenStream) -> Result<Expr> {
    let expr = or(stream)?;

    if let Ok(equals) = stream.match_next(matcher::eq(TokenKind::Equal)) {
        let value = assignment(stream)?;
        if let Expr::Variable(name) = expr {
            Ok(Expr::Assign {
                var: name,
                value: Box::new(value),
            })
        } else {
            Err(Error::new(equals, "Invalid assignment target."))
        }
    } else {
        Ok(expr)
    }
}

fn or(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = and(stream)?;

    while let TokenKind::Or = stream.peek().kind {
        let operator = stream.next();
        let right = and(stream)?;
        expr = Expr::Logical(operator, Box::new(expr), Box::new(right));
    }

    Ok(expr)
}

fn and(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = equality(stream)?;

    while let TokenKind::And = stream.peek().kind {
        let operator = stream.next();
        let right = equality(stream)?;
        expr = Expr::Logical(operator, Box::new(expr), Box::new(right));
    }

    Ok(expr)
}

fn equality(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = comparison(stream)?;
    while let TokenKind::BangEqual | TokenKind::EqualEqual = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(comparison(stream)?));
    }
    Ok(expr)
}

fn comparison(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = term(stream)?;
    while let TokenKind::Less
    | TokenKind::LessEqual
    | TokenKind::Greater
    | TokenKind::GreaterEqual = stream.peek().kind
    {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(term(stream)?));
    }
    Ok(expr)
}

fn term(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = factor(stream)?;
    while let TokenKind::Minus | TokenKind::Plus = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(factor(stream)?));
    }
    Ok(expr)
}

fn factor(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = unary(stream)?;
    while let TokenKind::Slash | TokenKind::Star = stream.peek().kind {
        let token = stream.next();
        expr = Expr::Binary(token, Box::new(expr), Box::new(unary(stream)?));
    }
    Ok(expr)
}

fn unary(stream: &mut impl TokenStream) -> Result<Expr> {
    match stream.peek().kind {
        TokenKind::Bang | TokenKind::Minus => {
            let token = stream.next();
            let expr = Expr::Unary(token, Box::new(unary(stream)?));
            Ok(expr)
        }
        _ => call(stream),
    }
}

fn call(stream: &mut impl TokenStream) -> Result<Expr> {
    let mut expr = primary(stream)?;
    while let TokenKind::LeftParen = stream.peek().kind {
        stream.next();

        let mut args = vec![];
        if stream.peek().kind != TokenKind::RightParen {
            loop {
                if args.len() >= 255 {
                    return Err(Error::new(
                        stream.next(),
                        "Can't have more than 255 arguments",
                    ));
                }
                args.push(expression(stream)?);
                if stream.match_next(matcher::eq(TokenKind::Comma)).is_err() {
                    break;
                }
            }
        }

        let paren = stream
            .match_next(matcher::eq(TokenKind::RightParen))
            .map_err(|t| Error::new(t, "Expect ')' after arguments."))?;
        expr = Expr::Call {
            callee: Box::new(expr),
            paren,
            args,
        };
    }
    Ok(expr)
}

fn primary(stream: &mut impl TokenStream) -> Result<Expr> {
    let token = stream.peek();
    let expr = match &token.kind {
        TokenKind::False => Expr::Literal(Lit::Bool(false)),
        TokenKind::True => Expr::Literal(Lit::Bool(true)),
        TokenKind::Nil => Expr::Literal(Lit::Nil),
        TokenKind::Number(n) => Expr::Literal(Lit::Number(*n)),
        TokenKind::String(value)=> Expr::Literal(Lit::String(value.clone())),
        TokenKind::StringUnterminated(_) => {
            return Err(Error::new(token.clone(), "Unterminated string."));
        }
        TokenKind::LeftParen => {
            stream.next();
            let expr = expression(stream)?;
            let token = stream.peek();
            if token.kind != TokenKind::RightParen {
                return Err(Error::new(
                    token.clone(),
                    r#"Expected ")" after expression."#,
                ));
            }
            Expr::Grouping(Box::new(expr))
        }
        TokenKind::Identifier => Expr::Variable(token.clone()),
        TokenKind::Eof => {
            return Err(Error::new(
                token.clone(),
                "Unexpected end of file.".to_owned(),
            ));
        }
        _ => {
            return Err(Error::new(token.clone(), "Expected expression.".to_owned()));
        }
    };
    stream.next();
    Ok(expr)
}

fn synchronize(stream: &mut impl TokenStream) {
    let mut current = stream.next();
    loop {
        if current.kind == TokenKind::Semicolon {
            break;
        }

        let next = stream.peek();

        if matches!(
            next.kind,
            TokenKind::Eof
                | TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return
        ) {
            break;
        }

        current = stream.next();
    }
}
