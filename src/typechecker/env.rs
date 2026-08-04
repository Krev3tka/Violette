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
    pub fn define(&mut self, name: String, ty: Ty) {

        if self.scopes.is_empty() {
            self.push();
        }

        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<Ty> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }
}
