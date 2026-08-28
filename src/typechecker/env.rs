use crate::lexer::span::Span;
use crate::typechecker::error::{BindingKind, DefinitionKind, TypeError};
use crate::typechecker::types::Ty;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub ty: Ty,
    pub kind: BindingKind,
    pub span: Span,
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
    pub fn define(
        &mut self,
        name: String,
        ty: Ty,
        kind: BindingKind,
        span: &Span,
    ) -> Result<(), TypeError> {
        if self.scopes.is_empty() {
            self.push();
        }

        if self.scopes.last_mut().unwrap().contains_key(&name) {
            return Err(TypeError::DuplicateDefinition {
                name: name.clone(),
                first_span: self.scopes.last_mut().unwrap().get(&name).unwrap().span,
                second_span: *span,
                def_kind: DefinitionKind::Var,
            });
        }

        self.scopes.last_mut().unwrap().insert(
            name,
            EntityInfo {
                ty,
                kind,
                span: *span,
            },
        );

        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<EntityInfo> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }
}
