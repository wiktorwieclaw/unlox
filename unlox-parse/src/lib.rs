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
    Ast, Expr, Lit, Stmt, Token, TokenKind,
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

pub fn parse(mut stream: impl TokenStream) -> Result<Ast> {
    let mut ast = Ast::new();
    while !stream.eof() {
        let stmt = declaration(&mut stream, &mut ast);
        ast.push_root_stmt(stmt);
    }
    Ok(ast)
}

fn declaration(stream: &mut impl TokenStream, ast: &mut Ast) -> Stmt {
    let token = stream.peek();
    let result = match &token.kind {
        TokenKind::Var => {
            stream.next();
            var_decl(stream, ast)
        }
        _ => statement(stream, ast),
    };
    result
        .inspect_err(|e| eprintln!("{e}"))
        .ok()
        .unwrap_or_else(|| {
            synchronize(stream);
            Stmt::ParseErr
        })
}

fn statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    let token = stream.peek();
    let stmt = match &token.kind {
        TokenKind::For => {
            stream.next();
            for_statement(stream, ast)
        }
        TokenKind::If => {
            stream.next();
            if_statement(stream, ast)
        }
        TokenKind::Print => {
            stream.next();
            print_statement(stream, ast)
        }
        TokenKind::While => {
            stream.next();
            while_statement(stream, ast)
        }
        TokenKind::LeftBrace => {
            stream.next();
            let stmt_indices = block(stream, ast)?
                .into_iter()
                .map(|stmt| ast.push_stmt(stmt))
                .collect();
            Ok(Stmt::Block(stmt_indices))
        }
        _ => expression_statement(stream, ast),
    }?;
    Ok(stmt)
}

fn for_statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
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
            Some(var_decl(stream, ast)?)
        }
        _ => Some(expression_statement(stream, ast)?),
    };

    let cond = if stream.peek().kind != TokenKind::Semicolon {
        Some(expression(stream, ast)?)
    } else {
        None
    };

    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after loop condition."))?;

    let inc = if stream.peek().kind != TokenKind::RightParen {
        Some(expression(stream, ast)?)
    } else {
        None
    };

    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after for clauses."))?;

    let mut body = statement(stream, ast)?;
    if let Some(inc) = inc {
        let inc = ast.push_expr(inc);
        body = Stmt::Block(vec![
            ast.push_stmt(body),
            ast.push_stmt(Stmt::Expression(inc)),
        ]);
    }
    let cond = cond.unwrap_or(Expr::Literal(Lit::Bool(true)));
    let while_stmt = Stmt::While {
        cond: ast.push_expr(cond),
        body: ast.push_stmt(body),
    };
    let for_stmt = if let Some(init) = init {
        Stmt::Block(vec![ast.push_stmt(init), ast.push_stmt(while_stmt)])
    } else {
        while_stmt
    };
    Ok(for_stmt)
}

fn if_statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    stream
        .match_next(matcher::eq(TokenKind::LeftParen))
        .map_err(|t| Error::new(t, "Expected '(' after 'if'."))?;
    let cond = expression(stream, ast)?;
    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after if condition."))?;
    let then_branch = statement(stream, ast)?;
    let else_branch = stream
        .match_next(matcher::eq(TokenKind::Else))
        .ok()
        .map(|_| statement(stream, ast))
        .transpose()?;
    Ok(Stmt::If {
        cond: ast.push_expr(cond),
        then_branch: ast.push_stmt(then_branch),
        else_branch: else_branch.map(|stmt| ast.push_stmt(stmt)),
    })
}

fn while_statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    stream
        .match_next(matcher::eq(TokenKind::LeftParen))
        .map_err(|t| Error::new(t, "Expected '(' after 'while'."))?;
    let cond = expression(stream, ast)?;
    stream
        .match_next(matcher::eq(TokenKind::RightParen))
        .map_err(|t| Error::new(t, "Expected ')' after condition."))?;
    let body = statement(stream, ast)?;
    Ok(Stmt::While {
        cond: ast.push_expr(cond),
        body: ast.push_stmt(body),
    })
}

fn print_statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    let expr = expression(stream, ast)?;
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after value."))?;
    Ok(Stmt::Print(ast.push_expr(expr)))
}

fn expression_statement(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    let expr = expression(stream, ast)?;
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after expression."))?;
    Ok(Stmt::Expression(ast.push_expr(expr)))
}

fn block(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];

    while stream.peek().kind != TokenKind::RightBrace && !stream.eof() {
        stmts.push(declaration(stream, ast));
    }

    stream
        .match_next(matcher::eq(TokenKind::RightBrace))
        .map_err(|t| Error::new(t, "Expect '}' after block."))?;
    Ok(stmts)
}

fn var_decl(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Stmt> {
    let name = stream
        .match_next(matcher::eq(TokenKind::Identifier))
        .map_err(|t| Error::new(t, "Expected variable name."))?;
    let token = stream.peek();
    let init = if token.kind == TokenKind::Equal {
        stream.next();
        Some(expression(stream, ast)?)
    } else {
        None
    };
    stream
        .match_next(matcher::eq(TokenKind::Semicolon))
        .map_err(|t| Error::new(t, "Expected ';' after variable declaration."))?;
    Ok(Stmt::VarDecl {
        name,
        init: init.map(|init| ast.push_expr(init)),
    })
}

fn expression(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    assignment(stream, ast)
}

fn assignment(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = or(stream, ast)?;

    if let Ok(equals) = stream.match_next(matcher::eq(TokenKind::Equal)) {
        let value = assignment(stream, ast)?;
        if let Expr::Variable(name) = expr {
            expr = Expr::Assign {
                var: name,
                value: ast.push_expr(value),
            };
            Ok(expr)
        } else {
            Err(Error::new(equals, "Invalid assignment target."))
        }
    } else {
        Ok(expr)
    }
}

fn or(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = and(stream, ast)?;

    while let TokenKind::Or = stream.peek().kind {
        let operator = stream.next();
        let right = and(stream, ast)?;
        expr = Expr::Logical(operator, ast.push_expr(expr), ast.push_expr(right));
    }

    Ok(expr)
}

fn and(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = equality(stream, ast)?;

    while let TokenKind::And = stream.peek().kind {
        let operator = stream.next();
        let right = equality(stream, ast)?;
        expr = Expr::Logical(operator, ast.push_expr(expr), ast.push_expr(right));
    }

    Ok(expr)
}

fn equality(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = comparison(stream, ast)?;
    while let TokenKind::BangEqual | TokenKind::EqualEqual = stream.peek().kind {
        let token = stream.next();
        let right = comparison(stream, ast)?;
        expr = Expr::Binary(token, ast.push_expr(expr), ast.push_expr(right));
    }
    Ok(expr)
}

fn comparison(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = term(stream, ast)?;
    while let TokenKind::Less
    | TokenKind::LessEqual
    | TokenKind::Greater
    | TokenKind::GreaterEqual = stream.peek().kind
    {
        let token = stream.next();
        let right = term(stream, ast)?;
        expr = Expr::Binary(token, ast.push_expr(expr), ast.push_expr(right));
    }
    Ok(expr)
}

fn term(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = factor(stream, ast)?;
    while let TokenKind::Minus | TokenKind::Plus = stream.peek().kind {
        let token = stream.next();
        let right = factor(stream, ast)?;
        expr = Expr::Binary(token, ast.push_expr(expr), ast.push_expr(right));
    }
    Ok(expr)
}

fn factor(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = unary(stream, ast)?;
    while let TokenKind::Slash | TokenKind::Star = stream.peek().kind {
        let token = stream.next();
        let right = unary(stream, ast)?;
        expr = Expr::Binary(token, ast.push_expr(expr), ast.push_expr(right));
    }
    Ok(expr)
}

fn unary(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    match stream.peek().kind {
        TokenKind::Bang | TokenKind::Minus => {
            let token = stream.next();
            let expr = unary(stream, ast)?;
            let expr = Expr::Unary(token, ast.push_expr(expr));
            Ok(expr)
        }
        _ => call(stream, ast),
    }
}

fn call(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let mut expr = primary(stream, ast)?;
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
                let arg = expression(stream, ast)?;
                args.push(arg);
                if stream.match_next(matcher::eq(TokenKind::Comma)).is_err() {
                    break;
                }
            }
        }

        let paren = stream
            .match_next(matcher::eq(TokenKind::RightParen))
            .map_err(|t| Error::new(t, "Expect ')' after arguments."))?;
        expr = Expr::Call {
            callee: ast.push_expr(expr),
            paren,
            args: args.into_iter().map(|arg| ast.push_expr(arg)).collect(),
        };
    }
    Ok(expr)
}

fn primary(stream: &mut impl TokenStream, ast: &mut Ast) -> Result<Expr> {
    let token = stream.peek();
    let expr = match &token.kind {
        TokenKind::False => Expr::Literal(Lit::Bool(false)),
        TokenKind::True => Expr::Literal(Lit::Bool(true)),
        TokenKind::Nil => Expr::Literal(Lit::Nil),
        TokenKind::Number(n) => Expr::Literal(Lit::Number(*n)),
        TokenKind::String(value) => Expr::Literal(Lit::String(value.clone())),
        TokenKind::StringUnterminated(_) => {
            return Err(Error::new(token.clone(), "Unterminated string."));
        }
        TokenKind::LeftParen => {
            stream.next();
            let expr = expression(stream, ast)?;
            let token = stream.peek();
            if token.kind != TokenKind::RightParen {
                return Err(Error::new(
                    token.clone(),
                    r#"Expected ")" after expression."#,
                ));
            }
            Expr::Grouping(ast.push_expr(expr))
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
