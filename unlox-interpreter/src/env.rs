use crate::{Error, Result};
use std::collections::HashMap;
use unlox_ast::{Lit, Token};

pub struct EnvCactus(unlox_cactus::Cactus<Env>);

#[derive(Default)]
pub struct Env {
    vars: HashMap<String, Lit>,
}

pub type EnvIndex = unlox_cactus::Index;

impl EnvCactus {
    /// Creates a new environment cactus stack with the `global` env used for it's root.
    pub fn with_global(global: Env) -> Self {
        let mut tree = unlox_cactus::Cactus::new();
        tree.push(global);
        Self(tree)
    }

    /// Pushes a new environment onto the current one.
    pub fn push(&mut self, env: Env) -> EnvIndex {
        self.0.push(env)
    }

    /// Pops current environemnt.
    ///
    /// Returns `None` if on attempt to pop the global environment.
    pub fn pop(&mut self) -> Option<Env> {
        if self.0.len() == 1 {
            return None;
        }
        self.0.pop()
    }

    pub fn current_env_mut(&mut self) -> &mut Env {
        self.0
            .current_node_mut()
            .map(|node| node.data_mut())
            .unwrap()
    }

    /// Assigns value to variable.
    pub fn assign_var(&mut self, name: &Token, value: Lit) -> Result<&Lit> {
        let slot = self.var_mut(name)?;
        *slot = value;
        Ok(slot)
    }

    /// Returns a reference to the value of a variable from the current environment.
    pub fn var(&self, name: &Token) -> Result<&Lit> {
        let mut env_node = self.0.node(self.0.current().unwrap()).unwrap();

        loop {
            if let Some(data) = env_node.data().vars.get(&name.lexeme) {
                break Ok(data);
            }

            if let Some(parent) = env_node.parent() {
                env_node = self.0.node(parent).unwrap();
            } else {
                break Err(Error::UndefinedVariable { name: name.clone() });
            }
        }
    }

    /// Returns a mutable reference to the value of a variable from the current environment.
    pub fn var_mut(&mut self, name: &Token) -> Result<&mut Lit> {
        let mut env_idx = self.0.current().unwrap();

        loop {
            // Current borrow checker implementation doesn't allow mutable borrows of a variable
            // in a loop if the function also returns a reference to the variable or it's part.
            // As a safe workaround, use non-mutable borrow in a loop and then reborrow it mutably.
            let env_node = self.0.node(env_idx).unwrap();
            if env_node.data().vars.contains_key(&name.lexeme) {
                break;
            }

            if let Some(parent_idx) = env_node.parent() {
                env_idx = parent_idx;
            } else {
                return Err(Error::UndefinedVariable { name: name.clone() });
            }
        }

        let var = self
            .0
            .node_data_mut(env_idx)
            .unwrap()
            .vars
            .get_mut(&name.lexeme)
            .expect("Presence of variable should already be asserted");

        Ok(var)
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
