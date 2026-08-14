use crate::typechecker::error::TypeError;
use crate::typechecker::types::Ty;
use std::collections::HashMap;

#[derive(Default)]
pub struct Env {
    pub scopes: Vec<HashMap<String, Ty>>,
}

impl Env {
    pub fn push(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn pop(&mut self) {
        self.scopes.pop();
    }
    pub fn define(&mut self, name: String, ty: Ty) -> Result<(), TypeError> {
        if self.scopes.is_empty() {
            self.push();
        }

        if self.scopes.last_mut().unwrap().contains_key(&name) {
            return Err(TypeError::AlreadyDefined(name.clone()));
        }

        self.scopes.last_mut().unwrap().insert(name, ty);

        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<Ty> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }
}
