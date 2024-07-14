use env::{Env, EnvCactus};
use unlox_ast::{Expr, Lit, Stmt, Token, TokenKind};

mod env;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("[Line {}]: Operand must be a number.", operator.line)]
    ExpectedNumber { operator: Token },
    #[error("[Line {}]: Operands must be numbers.", operator.line)]
    ExpectedNumbers { operator: Token },
    #[error("[Line {}]: Operands must be two numbers or two strings.", operator.line)]
    ExpectedNumbersOrStrings { operator: Token },
    #[error("[Line {}]: Undefined variable {}.", name.line, name.lexeme)]
    UndefinedVariable { name: Token },
    #[error("Parsing error.")]
    Parsing,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Interpreter {
    env_tree: EnvCactus,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            env_tree: EnvCactus::with_global(Env::new()),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(expr) => println!("{}", self.evaluate(expr)?),
            Stmt::VarDecl { name, init } => {
                let init = match init {
                    Some(init) => self.evaluate(init)?,
                    None => Lit::Nil,
                };
                self.env_tree.current_env_mut().define_var(name, init);
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Block(stmts) => self.execute_block(stmts)?,
            Stmt::ParseErr => return Err(Error::Parsing),
        }
        Ok(())
    }

    fn execute_block(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        self.env_tree.push(Env::new());
        let result = (|| {
            for stmt in stmts {
                self.execute(stmt)?;
            }
            Ok(())
        })();
        self.env_tree.pop();
        result
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Lit> {
        let lit = match expr {
            Expr::Literal(value) => value,
            Expr::Grouping(expr) => self.evaluate(*expr)?,
            Expr::Unary(operator, right) => {
                let right = self.evaluate(*right)?;
                match (&operator.kind, right) {
                    (TokenKind::Bang, right) => Lit::Bool(!right.is_truthy()),
                    (TokenKind::Minus, Lit::Number(n)) => Lit::Number(-n),
                    (TokenKind::Minus, _) => {
                        return Err(Error::ExpectedNumber { operator });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(operator, left, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match (&operator.kind, left, right) {
                    (TokenKind::Minus, Lit::Number(l), Lit::Number(r)) => Lit::Number(l - r),
                    (TokenKind::Slash, Lit::Number(l), Lit::Number(r)) => Lit::Number(l / r),
                    (TokenKind::Star, Lit::Number(l), Lit::Number(r)) => Lit::Number(l * r),
                    (TokenKind::Plus, Lit::Number(l), Lit::Number(r)) => Lit::Number(l + r),
                    (TokenKind::Plus, Lit::String(l), Lit::String(r)) => Lit::String(l + &r),
                    (TokenKind::Greater, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l > r),
                    (TokenKind::GreaterEqual, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l >= r),
                    (TokenKind::Less, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l < r),
                    (TokenKind::LessEqual, Lit::Number(l), Lit::Number(r)) => Lit::Bool(l <= r),
                    (TokenKind::BangEqual, l, r) => Lit::Bool(l != r),
                    (TokenKind::EqualEqual, l, r) => Lit::Bool(l == r),
                    (TokenKind::Plus, _, _) => {
                        return Err(Error::ExpectedNumbersOrStrings { operator });
                    }
                    (
                        TokenKind::Greater
                        | TokenKind::GreaterEqual
                        | TokenKind::Less
                        | TokenKind::LessEqual
                        | TokenKind::Minus
                        | TokenKind::Slash
                        | TokenKind::Star,
                        _,
                        _,
                    ) => {
                        return Err(Error::ExpectedNumbers { operator });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => self.env_tree.var(&name)?.clone(),
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.env_tree.assign_var(&name, value)?.clone()
            }
        };
        Ok(lit)
    }
}
