use crate::typechecker::error::{BindingKind, TypeError};
use crate::typechecker::types::Ty;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub ty: Ty,
    pub kind: BindingKind,
}
#[derive(Default)]
pub struct Env {
    pub scopes: Vec<HashMap<String, EntityInfo>>,
}

impl Env {
    pub fn push(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn pop(&mut self) {
        self.scopes.pop();
    }
    pub fn define(&mut self, name: String, ty: Ty, kind: BindingKind) -> Result<(), TypeError> {
        if self.scopes.is_empty() {
            self.push();
        }

        if self.scopes.last_mut().unwrap().contains_key(&name) {
            return Err(TypeError::AlreadyDefined(name.clone()));
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name, EntityInfo { ty, kind });

        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<EntityInfo> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }
}
