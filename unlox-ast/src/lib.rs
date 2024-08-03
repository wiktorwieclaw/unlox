use std::fmt::{self, Display};
pub use tokens::{Token, TokenKind};
pub use unlox_tokens as tokens;

#[derive(Debug, Default, Clone)]
pub struct Ast {
    stmts: Vec<Stmt>,
    exprs: Vec<Expr>,
    roots: Vec<StmtIdx>,
}

impl Ast {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_root_stmt(&mut self, stmt: Stmt) -> StmtIdx {
        let idx = self.push_stmt(stmt);
        self.roots.push(idx);
        idx
    }

    pub fn push_stmt(&mut self, stmt: Stmt) -> StmtIdx {
        let len = self.stmts.len();
        self.stmts.push(stmt);
        StmtIdx(len)
    }

    pub fn stmt(&self, idx: StmtIdx) -> &Stmt {
        &self.stmts[idx.0]
    }

    pub fn stmt_mut(&mut self, idx: ExprIdx) -> &mut Stmt {
        &mut self.stmts[idx.0]
    }

    pub fn push_expr(&mut self, expr: Expr) -> ExprIdx {
        let len = self.exprs.len();
        self.exprs.push(expr);
        ExprIdx(len)
    }

    pub fn expr(&self, idx: ExprIdx) -> &Expr {
        &self.exprs[idx.0]
    }

    pub fn expr_mut(&mut self, idx: ExprIdx) -> &mut Expr {
        &mut self.exprs[idx.0]
    }

    pub fn roots(&self) -> &[StmtIdx] {
        &self.roots
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If {
        cond: ExprIdx,
        then_branch: StmtIdx,
        else_branch: Option<StmtIdx>,
    },
    While {
        cond: ExprIdx,
        body: StmtIdx,
    },
    Print(ExprIdx),
    VarDecl {
        name: Token,
        init: Option<ExprIdx>,
    },
    Expression(ExprIdx),
    Block(Vec<StmtIdx>),
    ParseErr,
}

#[derive(Debug, Clone, Copy)]
pub struct StmtIdx(usize);

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Token, ExprIdx, ExprIdx),
    Grouping(ExprIdx),
    Literal(Lit),
    Unary(Token, ExprIdx),
    Variable(Token),
    Assign {
        var: Token,
        value: ExprIdx,
    },
    Logical(Token, ExprIdx, ExprIdx),
    Call {
        callee: ExprIdx,
        paren: Token,
        args: Vec<ExprIdx>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ExprIdx(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Lit {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Lit::Nil | Lit::Bool(false))
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lit::String(s) => write!(f, "{s}"),
            Lit::Number(n) => write!(f, "{n}"),
            Lit::Bool(b) => write!(f, "{b}"),
            Lit::Nil => write!(f, "nil"),
        }
    }
}
