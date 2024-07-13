use crate::{Error, Result};
use std::collections::HashMap;
use unlox_ast::{Lit, Token};

#[derive(Default)]
pub struct Environment(HashMap<String, Lit>);

impl Environment {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, name: Token, value: Lit) {
        self.0.insert(name.lexeme, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Lit> {
        self.0
            .get(&name.lexeme)
            .ok_or_else(|| Error::UndefinedVariable { name: name.clone() })
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, name: &Token) -> Result<&mut Lit> {
        self.0
            .get_mut(&name.lexeme)
            .ok_or_else(|| Error::UndefinedVariable { name: name.clone() })
    }
}
