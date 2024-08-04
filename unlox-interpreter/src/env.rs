use unlox_cactus::Index;

use crate::Val;
use std::collections::HashMap;

pub struct EnvCactus {
    cactus: unlox_cactus::Cactus<Env>,
    global: EnvIndex,
}

#[derive(Default)]
pub struct Env {
    vars: HashMap<String, Val>,
}

pub type EnvIndex = unlox_cactus::Index;

impl EnvCactus {
    /// Creates a new environment cactus stack with the `global` env used for it's root.
    pub fn with_global(global: Env) -> Self {
        let mut cactus = unlox_cactus::Cactus::new();
        let global = cactus.push(global);
        Self { cactus, global }
    }

    pub fn global(&self) -> EnvIndex {
        self.global
    }

    /// Pushes a new environment onto the current one.
    pub fn push(&mut self, env: Env) -> EnvIndex {
        self.cactus.push(env)
    }

    pub fn push_at(&mut self, parent: EnvIndex, env: Env) -> EnvIndex {
        self.cactus.push_at(parent, env)
    }

    /// Pops current environemnt.
    ///
    /// Returns `None` if on attempt to pop the global environment.
    pub fn pop(&mut self) -> Option<Env> {
        if self.cactus.len() == 1 {
            return None;
        }
        self.cactus.pop()
    }

    pub fn current(&self) -> Index {
        self.cactus.current().unwrap()
    }

    pub fn current_env_mut(&mut self) -> &mut Env {
        self.cactus
            .current()
            .and_then(|idx| self.cactus.node_data_mut(idx))
            .expect("Should always have at least global env")
    }

    /// Assigns value to variable.
    pub fn assign_var(&mut self, name: &str, value: Val) -> Option<&Val> {
        let slot = self.var_mut(name)?;
        *slot = value;
        Some(slot)
    }

    /// Returns a reference to the value of a variable from the current environment.
    pub fn var(&self, name: &str) -> Option<&Val> {
        let mut env_idx = self.cactus.current().unwrap();

        loop {
            let env = self.cactus.node_data(env_idx).unwrap();
            if let Some(val) = env.vars.get(name) {
                break Some(val);
            }

            if let Some(parent) = self.cactus.parent(env_idx) {
                env_idx = parent;
            } else {
                break None;
            }
        }
    }

    /// Returns a mutable reference to the value of a Val from the current environment.
    pub fn var_mut(&mut self, name: &str) -> Option<&mut Val> {
        let mut env_idx = self.cactus.current().unwrap();

        loop {
            // Current borrow checker implementation doesn't allow mutable borrows of a variable
            // in a loop if the function also returns a reference to the variable or it's part.
            // As a safe workaround, use non-mutable borrow in a loop and then reborrow it mutably.
            let env = self.cactus.node_data(env_idx).unwrap();
            if env.vars.contains_key(name) {
                break;
            }

            if let Some(parent_idx) = self.cactus.parent(env_idx) {
                env_idx = parent_idx;
            } else {
                return None;
            }
        }

        let var = self
            .cactus
            .node_data_mut(env_idx)
            .unwrap()
            .vars
            .get_mut(name)
            .expect("Presence of variable should already be asserted");

        Some(var)
    }
}

impl Env {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Default::default()
    }

    /// Defines new variable.
    pub fn define_var(&mut self, name: String, value: Val) {
        self.vars.insert(name, value);
    }
}
