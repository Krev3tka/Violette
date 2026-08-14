use crate::lexer::token::{PrimitiveType, Token};
use crate::parser::program::Program;
use crate::parser::types::Type;
use crate::parser::{Expression, Statement};
use crate::typechecker::env::Env;
pub(crate) use crate::typechecker::error::TypeError;
use crate::typechecker::types::Ty;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FnSig {
    params: Vec<Ty>,
    ret: Ty,
}

pub type StructSig = Vec<(String, Ty)>;

#[derive(Default)]
pub struct Checker {
    pub funcs: HashMap<String, FnSig>,
    pub structs: HashMap<String, StructSig>,
    pub(crate) env: Env,
    current_ret: Ty,
    pub errors: Vec<TypeError>,
}

impl Checker {
    pub fn collect_signatures(&mut self, program: &[Statement]) {
        for stmt in program {
            match stmt {
                Statement::Fun {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    let params: Vec<_> =
                        params.iter().map(|p| self.resolve(&p.param_type)).collect();

                    let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                    let f = Ty::Fn {
                        params: params.clone(),
                        ret: Box::new(return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t))),
                    };

                    if self.funcs.contains_key(name) {
                        self.errors
                            .push(TypeError::DuplicateDefinition(name.clone()));
                        continue;
                    }

                    self.env.define(name.clone(), f);
                    self.funcs.insert(name.clone(), FnSig { params, ret });
                }

                Statement::Struct { name, fields } => {
                    let fields = fields
                        .iter()
                        .map(|f| (f.name.clone(), self.resolve(&f.param_type)))
                        .collect();
                    if self.structs.contains_key(name) {
                        self.errors
                            .push(TypeError::DuplicateDefinition(name.clone()));
                        continue;
                    }
                    self.structs.insert(name.clone(), fields);
                }
                _ => {}
            };
        }
    }

    pub fn resolve(&mut self, t: &Type) -> Ty {
        match t {
            Type::Primitive(PrimitiveType::Int)
            | Type::Primitive(PrimitiveType::Int8)
            | Type::Primitive(PrimitiveType::Int16)
            | Type::Primitive(PrimitiveType::Int32)
            | Type::Primitive(PrimitiveType::Int64)
            | Type::Primitive(PrimitiveType::Uint)
            | Type::Primitive(PrimitiveType::Uint8)
            | Type::Primitive(PrimitiveType::Uint16)
            | Type::Primitive(PrimitiveType::Uint32)
            | Type::Primitive(PrimitiveType::Uint64) => Ty::Int,
            Type::Primitive(PrimitiveType::Float32) | Type::Primitive(PrimitiveType::Float64) => {
                Ty::Float
            }
            Type::Primitive(PrimitiveType::String) => Ty::String,
            Type::Primitive(PrimitiveType::Bool) => Ty::Bool,

            Type::Named(path) => Ty::Struct(path.segments[path.segments.len() - 1].clone()),
            Type::Fn { params, ret } => Ty::Fn {
                params: params.iter().map(|t| self.resolve(t)).collect(),
                ret: Box::new(ret.as_ref().map_or(Ty::Unit, |t| self.resolve(t))),
            },
            Type::Union(types) => {
                let resolved = types.iter().map(|v| self.resolve(v)).collect();

                Ty::Union(resolved)
            }
            Type::Generic { name, param } => Ty::Generic {
                name: name.clone(),
                param: Box::new(self.resolve(param)),
            },
        }
    }

    pub fn check_fn(&mut self, stmt: &Statement) {
        self.env.push();

        if let Statement::Fun {
            params,
            return_type,
            body,
            ..
        } = stmt
        {
            self.current_ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));
            for p in params {
                let ty = self.resolve(&p.param_type);
                if let Err(e) = self.env.define(p.name.clone(), ty) {
                    self.errors.push(e);
                }
            }

            self.check_block(body);
        }

        self.env.pop();
    }

    pub fn check_for_stmt(&mut self, stmt: &Statement) {
        self.env.push();
        if let Statement::ForCondition {
            condition: cond,
            body,
        } = stmt
        {
            let cond_ty = self.infer(cond);

            self.expect(&cond_ty, &Ty::Bool);

            self.check_block(body);
        } else if let Statement::ForRange {
            variable,
            iterable,
            body,
        } = stmt
        {
            todo!();
        } else if let Statement::ForCounter {
            init,
            condition: cond,
            post,
            body,
        } = stmt
        {
            self.check_statement(init.as_ref());

            let cond_ty = self.infer(cond);

            self.expect(&cond_ty, &Ty::Bool);

            let post_ty = self.infer(post);

            self.check_block(body);
        }
        self.env.pop()
    }

    pub fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value } | Statement::Const { name, value } => {
                let ty = self.infer(value);

                self.defined(name.clone(), ty)
            }
            Statement::If(if_stmt) => {
                let cond_ty = self.infer(&if_stmt.condition);

                self.expect(&cond_ty, &Ty::Bool);

                self.check_block(&if_stmt.then_block);

                for s in &if_stmt.else_if {
                    let cond_ty = self.infer(&s.condition);

                    self.expect(&cond_ty, &Ty::Bool);

                    self.check_block(&s.block);
                }

                if !if_stmt.else_block.is_empty() {
                    self.check_block(&if_stmt.else_block);
                }
            }
            Statement::ForCondition { .. }
            | Statement::ForCounter { .. }
            | Statement::ForRange { .. } => self.check_for_stmt(stmt),
            Statement::Return { value } => {
                let ty = match value {
                    Some(v) => self.infer(v),
                    None => Ty::Unit,
                };

                let cur_ref = &self.current_ret.clone();

                self.expect(&ty, cur_ref)
            }
            Statement::Expression { expression } => {
                self.infer(expression);
            }
            _ => {}
        }
    }

    pub fn check_struct(&mut self, stmt: &Statement) {
        if let Statement::Struct { name, fields } = stmt {
            self.env.push();
            for f in fields {
                let ty = self.resolve(&f.param_type);
                self.defined(f.name.clone(), ty)
            }
            self.env.pop();
        }
    }

    pub fn check_block(&mut self, block: &[Statement]) {
        self.env.push();
        for s in block {
            self.check_statement(s);
        }
        self.env.pop();
    }

    pub fn check_program(&mut self, program: &Program) {
        self.env.push();
        self.define_builtins();
        self.collect_signatures(&program.declarations);
        for stmt in &program.declarations {
            if let Statement::Fun { .. } = stmt {
                self.check_fn(stmt);
            }
        }
        if !program.main.is_empty() && self.funcs.contains_key("main") {
            self.errors.push(TypeError::ConflictingEntryPoint);
        }

        self.current_ret = Ty::Unit;
        for stmt in &program.main {
            self.check_statement(stmt);
        }
        self.env.pop()
    }

    pub fn infer(&mut self, expr: &Expression) -> Ty {
        match expr {
            Expression::IntLiteral(_) => Ty::Int,
            Expression::BoolLiteral(_) => Ty::Bool,
            Expression::StringLiteral(_) => Ty::String,
            Expression::FloatLiteral(_) => Ty::Float,
            Expression::Identifier(name) => match self.env.lookup(name) {
                Some(ty) => ty,
                None => {
                    self.errors.push(TypeError::UnknownName(name.clone()));
                    Ty::Error
                }
            },
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left_ty = self.infer(left);
                let right_ty = self.infer(right);

                match operator {
                    Token::Add => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::Float, Ty::Float) => Ty::Float,
                        (Ty::String, Ty::String) => Ty::String,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            self.errors.push(TypeError::InvalidOperator {
                                operator: operator.clone(),
                                left: left_ty,
                                right: right_ty,
                            });
                            Ty::Error
                        }
                    },
                    Token::Subtract | Token::Multiply | Token::Divide => {
                        match (&left_ty, &right_ty) {
                            (Ty::Int, Ty::Int) => Ty::Int,
                            (Ty::Float, Ty::Float) => Ty::Float,
                            (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                            _ => {
                                self.errors.push(TypeError::InvalidOperator {
                                    operator: operator.clone(),
                                    left: left_ty,
                                    right: right_ty,
                                });
                                Ty::Error
                            }
                        }
                    }
                    Token::Modulus => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            self.errors.push(TypeError::InvalidOperator {
                                operator: operator.clone(),
                                left: left_ty,
                                right: right_ty,
                            });
                            Ty::Error
                        }
                    },
                    Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => {
                        match (&left_ty, &right_ty) {
                            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) => {}
                            (Ty::Error, _) | (_, Ty::Error) => return Ty::Error,
                            _ => self.errors.push(TypeError::InvalidOperator {
                                operator: operator.clone(),
                                left: left_ty,
                                right: right_ty,
                            }),
                        };
                        Ty::Bool
                    }
                    Token::Equals | Token::NotEquals => match (&left_ty, &right_ty) {
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            if left_ty != right_ty {
                                self.errors.push(TypeError::InvalidOperator {
                                    operator: operator.clone(),
                                    left: left_ty,
                                    right: right_ty,
                                });
                                return Ty::Error;
                            }
                            Ty::Bool
                        }
                    },
                    Token::LogicAnd | Token::LogicOr => {
                        self.expect(&left_ty, &Ty::Bool);
                        self.expect(&right_ty, &Ty::Bool);
                        Ty::Bool
                    }
                    _ => Ty::Error,
                }
            }
            Expression::Index { .. } => {
                self.errors
                    .push(TypeError::Unsupported("Indexing".to_string()));
                Ty::Error
            }
            Expression::Call { function, args } => {
                let callee = self.infer(function);
                match callee {
                    Ty::Fn { params, ret } => {
                        if args.len() != params.len() {
                            self.errors.push(TypeError::ArityMismatch {
                                name: match function.as_ref() {
                                    Expression::Identifier(n) => n.to_string(),
                                    _ => "<function value>".to_string(),
                                },
                                expected: params.len(),
                                found: args.len(),
                            });
                            return *ret;
                        }

                        for (arg, param) in args.iter().zip(params.iter()) {
                            let a = self.infer(arg);
                            self.expect(&a, param);
                        }
                        *ret
                    }
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::NotCallable);
                        Ty::Error
                    }
                }
            }
            Expression::Lambda {
                params,
                return_type,
                body,
            } => {
                let param_tys: Vec<Ty> =
                    params.iter().map(|p| self.resolve(&p.param_type)).collect();
                let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                let saved_ret = self.current_ret.clone();
                self.current_ret = ret.clone();

                self.env.push();

                for (p, ty) in params.iter().zip(param_tys.iter()) {
                    self.env.define(p.name.clone(), ty.clone());
                }

                for s in body {
                    self.check_statement(s);
                }
                self.env.pop();

                self.current_ret = saved_ret;

                Ty::Fn {
                    params: param_tys,
                    ret: Box::new(ret),
                }
            }
            Expression::StructLiteral { name, fields } => {
                if self.structs.get(name).is_none() {
                    self.errors.push(TypeError::UnknownName(name.clone()));
                    return Ty::Error;
                }

                for f in fields {
                    match self.infer(f.field_val.as_ref()) {
                        Ty::Error => return Ty::Error,
                        _ => continue,
                    }
                }

                Ty::Struct(name.clone())
            }
            Expression::Field { object, name } => {
                let obj_ty = self.infer(object.as_ref());

                match obj_ty {
                    Ty::Struct(s) => match self.structs.get(&s) {
                        Some(struct_sig) => {
                            match struct_sig.iter().find(|(curr_name, _)| curr_name == name) {
                                Some(field) => field.1.clone(),
                                None => {
                                    self.errors.push(TypeError::UnknownField {
                                        struct_name: s,
                                        field: name.clone(),
                                    });
                                    Ty::Error
                                }
                            }
                        }
                        None => {
                            self.errors.push(TypeError::UnknownName(name.clone()));
                            Ty::Error
                        }
                    },
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::NoFields(obj_ty));
                        Ty::Error
                    }
                }
            }
            Expression::MethodCall { .. } => {
                self.errors
                    .push(TypeError::Unsupported("Method calls".to_string()));
                Ty::Error
            }
            _ => Ty::Error,
        }
    }

    pub fn define_builtins(&mut self) {
        self.defined(
            "print".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![Ty::Int, Ty::Float, Ty::String, Ty::Bool])],
                ret: Box::new(Ty::Unit),
            },
        );
        self.defined(
            "println".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![Ty::Int, Ty::Float, Ty::String, Ty::Bool])],
                ret: Box::new(Ty::Unit),
            },
        );
    }

    pub fn defined(&mut self, name: String, ty: Ty) {
        if let Err(e) = self.env.define(name.clone(), ty) {
            self.errors.push(e);
        }
    }

    pub fn expect(&mut self, actual: &Ty, expected: &Ty) {
        if let Ty::Union(types) = expected
            && types.iter().any(|ty| ty == actual)
        {
            return;
        }

        if !matches!(actual, Ty::Error) && !matches!(expected, Ty::Error) && actual != expected {
            self.errors.push(TypeError::Mismatch {
                expected: expected.clone(),
                found: actual.clone(),
            })
        }
    }
}
