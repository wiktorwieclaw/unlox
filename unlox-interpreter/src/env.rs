use crate::{Error, Result};
use std::collections::HashMap;
use unlox_ast::{Lit, Token};

#[derive(Default)]
pub struct EnvTree(unlox_tree::Tree<Env>);

pub type EnvNode = unlox_tree::Node<Env>;

#[derive(Default)]
pub struct Env {
    vars: HashMap<String, Lit>,
}

pub type EnvIndex = unlox_tree::Index;

impl EnvTree {
    /// Creates an empty environment tree.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_global(&mut self, env: Env) -> EnvIndex {
        self.0.add_root(env)
    }

    pub fn add_nested(&mut self, parent: EnvIndex, env: Env) -> EnvIndex {
        self.0.add_leaf(parent, env)
    }

    /// Assigns value to variable.
    pub fn assign(&mut self, env: EnvIndex, name: &Token, value: Lit) -> Result<&Lit> {
        let slot = self.var_mut(env, name)?;
        *slot = value;
        Ok(slot)
    }

    /// Returns a reference to the value of a variable.
    ///
    /// # Panics
    /// Panics if the environment pointed to by `env` has been already dropped.
    pub fn var(&self, env: EnvIndex, name: &Token) -> Result<&Lit> {
        let mut env = self.node(env);

        loop {
            if let Some(val) = env.get().vars.get(&name.lexeme) {
                break Ok(val);
            }

            if let Some(parent) = env.parent() {
                env = self.node(parent);
            } else {
                break Err(Error::UndefinedVariable { name: name.clone() });
            }
        }
    }

    /// Returns a mutable reference to the value of a variable.
    ///
    /// # Panics
    /// Panics if the environment pointed to by `env` has been already dropped.
    pub fn var_mut(&mut self, env: EnvIndex, name: &Token) -> Result<&mut Lit> {
        let mut env_idx = env;

        loop {
            // Current borrow checker implementation doesn't allow mutable borrows of a variable
            // in a loop if the function also returns a reference to the variable or it's part.
            // As a safe workaround, use non-mutable borrow in a loop and then reborrow it mutably.
            let env = self.node(env_idx);
            if env.get().vars.contains_key(&name.lexeme) {
                break;
            }

            if let Some(parent) = env.parent() {
                env_idx = parent;
            } else {
                return Err(Error::UndefinedVariable { name: name.clone() });
            }
        }

        let var = self
            .env_mut(env_idx)
            .vars
            .get_mut(&name.lexeme)
            .expect("Presence of variable should already be asserted");

        Ok(var)
    }

    pub fn env_mut(&mut self, env: EnvIndex) -> &mut Env {
        self.node_mut(env).get_mut()
    }

    fn node(&self, env: EnvIndex) -> &EnvNode {
        self.0
            .get(env)
            .expect("Env with provided index doesn't exist")
    }

    fn node_mut(&mut self, env: EnvIndex) -> &mut EnvNode {
        self.0
            .get_mut(env)
            .expect("Env with provided index doesn't exist")
    }
}

impl Env {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Default::default()
    }

    /// Defines new variable.
    pub fn define_var(&mut self, name: Token, value: Lit) {
        self.vars.insert(name.lexeme, value);
    }
}
