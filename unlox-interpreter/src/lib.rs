use env::{Env, EnvCactus, EnvIndex};
use output::{Output, SingleOutput, SplitOutput};
use std::{
    io::Write,
    ops::ControlFlow,
    time::{SystemTime, UNIX_EPOCH},
};
use unlox_ast::{Ast, Expr, ExprIdx, Stmt, StmtIdx, Token, TokenKind};
use val::{Callable, Val};

mod env;
pub mod output;
mod val;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("[Line {}]: Operand must be a number.", operator.line)]
    ExpectedNumber { operator: Token },
    #[error("[Line {}]: Operands must be numbers.", operator.line)]
    ExpectedNumbers { operator: Token },
    #[error("[Line {}]: Operands must be two numbers or two strings.", operator.line)]
    ExpectedNumbersOrStrings { operator: Token },
    #[error("[Line {}]: Undefined variable {}.", token.line, name)]
    UndefinedVariable { name: String, token: Token },
    #[error("[Line] {}: Can only call functions and classes.", paren.line)]
    BadCall { paren: Token },
    #[error("[Line] {}: Expected {expected} arguments but got {got}.", paren.line)]
    WrongNumberOfArgs {
        paren: Token,
        expected: usize,
        got: usize,
    },
    #[error("The program terminated due to a syntax error.")]
    Parsing,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Interpreter<Out> {
    env_tree: EnvCactus,
    out: Out,
}

impl<Out> Interpreter<SingleOutput<Out>> {
    pub fn new(out: Out) -> Self {
        Self {
            env_tree: EnvCactus::with_global(new_global_env()),
            out: SingleOutput(out),
        }
    }
}

impl<Out, Err> Interpreter<SplitOutput<Out, Err>> {
    pub fn with_split_output(out: Out, err: Err) -> Self {
        Self {
            env_tree: EnvCactus::with_global(new_global_env()),
            out: SplitOutput(out, err),
        }
    }
}

fn new_global_env() -> Env {
    let mut global = Env::new();
    global.define_var("clock".to_owned(), Val::Callable(Callable::Clock));
    global
}

impl<Out> Interpreter<Out>
where
    Out: Output,
{
    pub fn interpret(&mut self, source: &str, ast: &Ast) {
        for stmt in ast.roots() {
            if let Err(error) = self.execute(source, ast, *stmt) {
                writeln!(self.out.err(), "{error}").unwrap();
                return;
            }
        }
    }

    fn execute(&mut self, source: &str, ast: &Ast, stmt: StmtIdx) -> Result<ControlFlow<Val>> {
        match ast.stmt(stmt) {
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(source, ast, *cond)?.is_truthy() {
                    self.execute(source, ast, *then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(source, ast, *else_branch)
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            }
            Stmt::While { cond, body } => {
                while self.evaluate(source, ast, *cond)?.is_truthy() {
                    let control_flow = self.execute(source, ast, *body)?;
                    if control_flow.is_break() {
                        return Ok(control_flow);
                    }
                }
                Ok(ControlFlow::Continue(()))
            }
            Stmt::Print(expr) => {
                let val = self.evaluate(source, ast, *expr)?;
                writeln!(self.out.out(), "{val}").unwrap();
                Ok(ControlFlow::Continue(()))
            }
            Stmt::Return(_, expr) => {
                let val = expr
                    .map(|e| self.evaluate(source, ast, e))
                    .transpose()?
                    .unwrap_or_default();
                Ok(ControlFlow::Break(val))
            }
            Stmt::VarDecl { name, init } => {
                let init = match init {
                    Some(init) => self.evaluate(source, ast, *init)?,
                    None => Val::Nil,
                };
                self.env_tree
                    .current_env_mut()
                    .define_var(source[name.lexeme.clone()].to_owned(), init);
                Ok(ControlFlow::Continue(()))
            }
            Stmt::Expression(expr) => {
                self.evaluate(source, ast, *expr)?;
                Ok(ControlFlow::Continue(()))
            }
            Stmt::Block(stmts) => {
                self.execute_block(source, ast, stmts, Env::new(), self.env_tree.current())
            }
            Stmt::Function { name, params, body } => {
                let callable = Callable::Function {
                    name: source[name.lexeme.clone()].to_owned(),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env_tree.current_env_mut().define_var(
                    source[name.lexeme.clone()].to_owned(),
                    Val::Callable(callable),
                );
                Ok(ControlFlow::Continue(()))
            }
            Stmt::ParseErr => Err(Error::Parsing),
        }
    }

    fn execute_block(
        &mut self,
        source: &str,
        ast: &Ast,
        stmts: &[StmtIdx],
        env: Env,
        env_parent: EnvIndex,
    ) -> Result<ControlFlow<Val>> {
        self.env_tree.push_at(env_parent, env);
        let result = (|| {
            for stmt in stmts {
                let control_flow = self.execute(source, ast, *stmt)?;
                if control_flow.is_break() {
                    return Ok(control_flow);
                }
            }
            Ok(ControlFlow::Continue(()))
        })();
        self.env_tree.pop();
        result
    }

    fn evaluate(&mut self, source: &str, ast: &Ast, expr: ExprIdx) -> Result<Val> {
        let lit = match ast.expr(expr) {
            Expr::Literal(value) => value.clone().into(),
            Expr::Grouping(expr) => self.evaluate(source, ast, *expr)?,
            Expr::Unary(operator, right) => {
                let right = self.evaluate(source, ast, *right)?;
                match (&operator.kind, right) {
                    (TokenKind::Bang, right) => Val::Bool(!right.is_truthy()),
                    (TokenKind::Minus, Val::Number(n)) => Val::Number(-n),
                    (TokenKind::Minus, _) => {
                        return Err(Error::ExpectedNumber {
                            operator: operator.clone(),
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(operator, left, right) => {
                let left = self.evaluate(source, ast, *left)?;
                let right = self.evaluate(source, ast, *right)?;

                match (&operator.kind, left, right) {
                    (TokenKind::Minus, Val::Number(l), Val::Number(r)) => Val::Number(l - r),
                    (TokenKind::Slash, Val::Number(l), Val::Number(r)) => Val::Number(l / r),
                    (TokenKind::Star, Val::Number(l), Val::Number(r)) => Val::Number(l * r),
                    (TokenKind::Plus, Val::Number(l), Val::Number(r)) => Val::Number(l + r),
                    (TokenKind::Plus, Val::String(l), Val::String(r)) => Val::String(l + &r),
                    (TokenKind::Greater, Val::Number(l), Val::Number(r)) => Val::Bool(l > r),
                    (TokenKind::GreaterEqual, Val::Number(l), Val::Number(r)) => Val::Bool(l >= r),
                    (TokenKind::Less, Val::Number(l), Val::Number(r)) => Val::Bool(l < r),
                    (TokenKind::LessEqual, Val::Number(l), Val::Number(r)) => Val::Bool(l <= r),
                    (TokenKind::BangEqual, l, r) => Val::Bool(l != r),
                    (TokenKind::EqualEqual, l, r) => Val::Bool(l == r),
                    (TokenKind::Plus, _, _) => {
                        return Err(Error::ExpectedNumbersOrStrings {
                            operator: operator.clone(),
                        });
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
                        return Err(Error::ExpectedNumbers {
                            operator: operator.clone(),
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(var) => {
                let name = &source[var.lexeme.clone()];
                self.env_tree
                    .var(name)
                    .ok_or_else(|| Error::UndefinedVariable {
                        name: name.to_owned(),
                        token: var.clone(),
                    })?
                    .clone()
            }
            Expr::Assign { var, value } => {
                let value = self.evaluate(source, ast, *value)?;
                let name = &source[var.lexeme.clone()];
                self.env_tree
                    .assign_var(name, value)
                    .ok_or_else(|| Error::UndefinedVariable {
                        name: name.to_owned(),
                        token: var.clone(),
                    })?
                    .clone()
            }
            Expr::Logical(operator, left, right) => {
                let left = self.evaluate(source, ast, *left)?;
                match (&operator.kind, left.is_truthy()) {
                    (TokenKind::Or, true) => left,
                    (TokenKind::Or, false) => self.evaluate(source, ast, *right)?,
                    (_, false) => left,
                    _ => self.evaluate(source, ast, *right)?,
                }
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let callee = self.evaluate(source, ast, *callee)?;
                let Val::Callable(callable) = callee else {
                    return Err(Error::BadCall {
                        paren: paren.clone(),
                    });
                };
                let args: Result<Vec<_>> = args
                    .iter()
                    .map(|arg| self.evaluate(source, ast, *arg))
                    .collect();
                let args = args?;
                if args.len() != callable.arity() {
                    return Err(Error::WrongNumberOfArgs {
                        paren: paren.clone(),
                        expected: callable.arity(),
                        got: args.len(),
                    });
                }
                self.call(source, ast, callable, args)?
            }
        };
        Ok(lit)
    }

    fn call(&mut self, source: &str, ast: &Ast, callable: Callable, args: Vec<Val>) -> Result<Val> {
        match callable {
            Callable::Clock => Ok(Val::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            )),
            Callable::Function { params, body, .. } => {
                let mut env = Env::new();
                for (param, arg) in params.iter().zip(args) {
                    let name = &source[param.lexeme.clone()];
                    env.define_var(name.to_owned(), arg);
                }
                let control_flow =
                    self.execute_block(source, ast, &body, env, self.env_tree.global())?;
                match control_flow {
                    ControlFlow::Continue(()) => Ok(Val::Nil),
                    ControlFlow::Break(val) => Ok(val),
                }
            }
        }
    }
}
