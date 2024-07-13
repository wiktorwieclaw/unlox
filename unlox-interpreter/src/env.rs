use crate::{Error, Result};
use std::collections::HashMap;
use unlox_ast::{Lit, Token};

// Invariants:
// - `nodes.len >= 1`
// - `nodes[0].enclosing.is_none()`
pub struct EnvTree {
    nodes: Vec<Option<Env>>,
}

pub struct Env {
    vars: HashMap<String, Lit>,
    enclosing: Option<EnvIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvIndex(usize);

impl Default for EnvTree {
    fn default() -> Self {
        Self {
            nodes: vec![Some(Env::global())],
        }
    }
}

impl EnvTree {
    pub fn create(&mut self, enclosing: EnvIndex) -> EnvIndex {
        let idx = self
            .nodes
            .iter_mut()
            .skip(1)
            .position(|slot| slot.is_none());
        if let Some(idx) = idx {
            self.nodes[idx] = Some(Env::nested(enclosing));
            EnvIndex(idx)
        } else {
            let idx = self.nodes.len();
            self.nodes.push(Some(Env::nested(enclosing)));
            EnvIndex(idx)
        }
    }

    /// Assign value to exiting variable.
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
        let mut env = self.env(env);

        loop {
            if let Some(val) = env.vars.get(&name.lexeme) {
                break Ok(val);
            }

            if let Some(enclosing) = env.enclosing {
                env = self.env(enclosing);
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
            // Current borrow checker implementation doesn't allow mutable borrows of a env
            // in a loop if the function also returns a reference to the env or it's part.
            // As a safe workaround, use non-mutable borrow in a loop and then reborrow it mutably.
            let env = self.env(env_idx);
            if env.vars.contains_key(&name.lexeme) {
                break;
            }

            if let Some(enclosing) = env.enclosing {
                env_idx = enclosing;
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

    pub fn env(&self, env: EnvIndex) -> &Env {
        self.nodes
            .get(env.as_usize())
            .expect("EnvIndex is out of bounds")
            .as_ref()
            .expect("Env has been removed")
    }

    pub fn env_mut(&mut self, env: EnvIndex) -> &mut Env {
        self.nodes
            .get_mut(env.as_usize())
            .expect("EnvIndex is out of bounds")
            .as_mut()
            .expect("Env has been removed")
    }
}

impl Env {
    pub fn global() -> Self {
        Self {
            vars: Default::default(),
            enclosing: None,
        }
    }

    pub fn nested(enclosing: EnvIndex) -> Self {
        Self {
            vars: Default::default(),
            enclosing: Some(enclosing),
        }
    }

    /// Define new variable.
    pub fn define(&mut self, name: Token, value: Lit) {
        self.vars.insert(name.lexeme, value);
    }
}

impl EnvIndex {
    pub fn global() -> EnvIndex {
        EnvIndex(0)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}
