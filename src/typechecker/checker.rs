use crate::hir::typed_ast::{TypedExpr, TypedExprKind, TypedProgram, TypedStmt, TypedStmtKind};
use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::expression::CallArg;
use crate::parser::program::Program;
use crate::parser::statement::FuncParam;
use crate::parser::types::{BindingPattern, Type};
use crate::parser::{Expression, Statement};
use crate::typechecker::env::Env;
pub use crate::typechecker::error::TypeError;
use crate::typechecker::error::TypeError::ConflictingEntryPoint;
use crate::typechecker::error::{BindingKind, DefinitionKind, LoopControlKind};
use crate::typechecker::types::Ty;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct FnSig {
    pub params: Vec<ParamSig>,
    pub ret: Ty,
    span: Span,

    /// Is it a method and isn't it static? (not static => dynamic, don't try to find any logic here)
    pub is_dynamic_method: bool,
}

#[derive(Default)]
pub struct Checker {
    pub funcs: HashMap<String, FnSig>,
    pub structs: HashMap<String, StructSig>,
    pub variant_cases: HashMap<String, (String, Option<Ty>)>,
    pub env: Env,
    pub current_ret: Ty,
    pub errors: Vec<TypeError>,
    pub main_fn_span: Option<Span>,
    pub in_loop: bool,
    pub current_type_span: Option<Span>,
    current_self_type: Option<Ty>,
}

#[derive(Clone, Debug)]
pub struct ParamSig {
    pub label: String,
    pub ty: Ty,
}

pub type StructSig = Vec<(String, Ty, Span)>;

impl Checker {
    pub fn collect_signatures(&mut self, program: &[Statement]) {
        for stmt in program {
            match stmt {
                Statement::Const {
                    name, value, span, ..
                } => {
                    let ty = self.infer(value, None);
                    self.defined(name.clone(), ty, BindingKind::Const, *span);
                }
                Statement::Func {
                    name,
                    params,
                    return_type,
                    span,
                    ..
                } => {
                    if name == "main" {
                        self.main_fn_span = Some(*span)
                    }

                    let params: Vec<_> = params
                        .iter()
                        .map(|p| {
                            let ty = self.resolve(&p.param_type);
                            if p.is_ref {
                                ParamSig {
                                    label: p.annotated_name.clone(),
                                    ty: Ty::Ref {
                                        inner: Box::new(ty),
                                        is_mut: p.kind.is_mutable(),
                                    },
                                }
                            } else {
                                ParamSig {
                                    label: p.annotated_name.clone(),
                                    ty,
                                }
                            }
                        })
                        .collect();

                    let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                    let f = Ty::Fn {
                        params: params
                            .iter()
                            .map(|p| p.ty.clone())
                            .collect::<Vec<_>>()
                            .clone(),
                        ret: Box::new(return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t))),
                    };

                    if self.funcs.contains_key(name) {
                        self.errors.push(TypeError::DuplicateDefinition {
                            name: name.clone(),
                            first_span: match self.funcs.get(name) {
                                Some(f) => f.span,
                                None => unreachable!(),
                            },
                            second_span: stmt.span(),
                            def_kind: DefinitionKind::Fun,
                        });
                        continue;
                    }

                    self.defined(name.clone(), f, BindingKind::Var, *span);
                    self.funcs.insert(
                        name.clone(),
                        FnSig {
                            params,
                            ret,
                            span: *span,
                            is_dynamic_method: false,
                        },
                    );
                }

                Statement::ExternFunc {
                    name,
                    params,
                    return_type,
                    span,
                } => {
                    let params: Vec<_> = params
                        .iter()
                        .map(|p| ParamSig {
                            label: p.annotated_name.clone(),
                            ty: self.resolve(&p.param_type),
                        })
                        .collect();

                    let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                    self.defined(
                        name.clone(),
                        Ty::Fn {
                            params: params
                                .iter()
                                .map(|p| p.ty.clone())
                                .collect::<Vec<_>>()
                                .clone(),
                            ret: Box::new(ret.clone()),
                        },
                        BindingKind::Var,
                        *span,
                    );

                    self.funcs.insert(
                        name.clone(),
                        FnSig {
                            params,
                            ret,
                            span: *span,
                            is_dynamic_method: true,
                        },
                    );
                }

                Statement::Struct { name, fields, .. } => {
                    let fields = fields
                        .iter()
                        .map(|f| (f.name.clone(), self.resolve(&f.param_type), f.span))
                        .collect();
                    if self.structs.contains_key(name) {
                        self.errors.push(TypeError::DuplicateDefinition {
                            name: name.clone(),
                            first_span: match self.structs.get(name) {
                                Some(s) => s[0].2,
                                None => unreachable!(),
                            },
                            second_span: stmt.span(),
                            def_kind: DefinitionKind::Struct,
                        });
                        continue;
                    }
                    self.structs.insert(name.clone(), fields);
                }
                Statement::Extend {
                    target,
                    methods,
                    span,
                } => {
                    let target_ty = self.resolve(target);

                    self.current_self_type = Some(target_ty.clone());

                    let prefix = match &target_ty {
                        Ty::Struct(struct_name) => {
                            if !self.structs.contains_key(struct_name) {
                                self.errors.push(TypeError::UnknownName {
                                    name: struct_name.clone(),
                                    span: *span,
                                });
                                continue;
                            }
                            struct_name.clone()
                        }
                        Ty::Sized { signed: true } => "int".to_string(),
                        Ty::Sized { signed: false } => "uint".to_string(),
                        Ty::Int {
                            signed: false,
                            bits,
                        } => format!("int{bits}"),
                        Ty::Int { signed: true, bits } => format!("uint{bits}"),
                        Ty::Float { bits } => format!("float{bits}"),
                        Ty::String => "string".to_string(),
                        Ty::Char => "char".to_string(),
                        Ty::Bool => "bool".to_string(),
                        Ty::Array(inner) => {
                            format!("Array_{}", self.type_to_prefix(inner.as_ref()))
                        }
                        _ => {
                            self.errors.push(TypeError::Unsupported {
                                desc: format!(
                                    "Cannot extend non-struct or non-primitive type {:?}",
                                    target_ty
                                ),
                                span: *span,
                            });
                            continue;
                        }
                    };

                    for method in methods {
                        if let Statement::Func {
                            name,
                            params,
                            return_type,
                            span: func_span,
                            ..
                        } = method
                        {
                            let full_name = format!("{}.{}", prefix, name);

                            let params_ty: Vec<_> = params
                                .iter()
                                .map(|p| ParamSig {
                                    label: p.annotated_name.clone(),
                                    ty: self.resolve(&p.param_type),
                                })
                                .collect();

                            let ret_ty = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                            if self.funcs.contains_key(&full_name) {
                                self.errors.push(TypeError::DuplicateDefinition {
                                    name: full_name.clone(),
                                    first_span: self.funcs.get(&full_name).unwrap().span,
                                    second_span: *func_span,
                                    def_kind: DefinitionKind::Fun,
                                });
                                continue;
                            }

                            self.funcs.insert(
                                full_name,
                                FnSig {
                                    params: params_ty,
                                    ret: ret_ty,
                                    span: *func_span,
                                    is_dynamic_method: false,
                                },
                            );
                        }
                        if let Statement::Method {
                            obj_name,
                            name,
                            params,
                            return_type,
                            span: func_span,
                            ..
                        } = method
                        {
                            let full_name = format!("{}.{}", prefix, name);

                            let mut params_ty: Vec<_> = params
                                .iter()
                                .map(|p| ParamSig {
                                    label: p.annotated_name.clone(),
                                    ty: self.resolve(&p.param_type),
                                })
                                .collect();

                            params_ty.insert(
                                0,
                                ParamSig {
                                    label: obj_name.clone(),
                                    ty: target_ty.clone(),
                                },
                            );

                            let ret_ty = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                            if self.funcs.contains_key(&full_name) {
                                self.errors.push(TypeError::DuplicateDefinition {
                                    name: full_name.clone(),
                                    first_span: self.funcs.get(&full_name).unwrap().span,
                                    second_span: *func_span,
                                    def_kind: DefinitionKind::Fun,
                                });
                                continue;
                            }

                            self.funcs.insert(
                                full_name,
                                FnSig {
                                    params: params_ty,
                                    ret: ret_ty,
                                    span: *func_span,
                                    is_dynamic_method: true,
                                },
                            );
                        }
                    }

                    self.current_self_type = None;
                }
                Statement::Enum { name, cases, .. } => {
                    for case in cases {
                        let ty = case.payload.clone().map(|ty| self.resolve(&ty));
                        self.variant_cases
                            .insert(case.name.clone(), (name.clone(), ty));
                    }
                }
                _ => {}
            };
        }
    }

    pub fn resolve(&mut self, t: &Type) -> Ty {
        match t {
            Type::Named(path) => {
                let name = path.segments.last().unwrap();
                match name.as_str() {
                    "Int" => Ty::Sized { signed: true },
                    "Int8" => Ty::Int {
                        bits: 8,
                        signed: true,
                    },
                    "Int16" => Ty::Int {
                        bits: 16,
                        signed: true,
                    },
                    "Int32" => Ty::Int {
                        bits: 32,
                        signed: true,
                    },
                    "Int64" => Ty::Int {
                        bits: 64,
                        signed: true,
                    },
                    "Int128" => Ty::Int {
                        bits: 128,
                        signed: true,
                    },

                    "UInt" => Ty::Sized { signed: false },
                    "UInt8" => Ty::Int {
                        bits: 8,
                        signed: false,
                    },
                    "UInt16" => Ty::Int {
                        bits: 16,
                        signed: false,
                    },
                    "UInt32" => Ty::Int {
                        bits: 32,
                        signed: false,
                    },
                    "UInt64" => Ty::Int {
                        bits: 64,
                        signed: false,
                    },
                    "UInt128" => Ty::Int {
                        bits: 128,
                        signed: false,
                    },

                    "Float32" => Ty::Float { bits: 32 },
                    "Float64" => Ty::Float { bits: 64 },
                    "String" => Ty::String,
                    "Bool" => Ty::Bool,
                    "Char" => Ty::Char,
                    _ => Ty::Struct(name.clone()),
                }
            }

            Type::SelfTy => {
                if let Some(ref self_ty) = self.current_self_type {
                    self_ty.clone()
                } else {
                    self.errors.push(TypeError::Unsupported {
                        desc: "`Self` can only be used inside `extend` blocks".to_string(),
                        span: Span::default(),
                    });
                    Ty::Error
                }
            }

            Type::Array(inner) => Ty::Array(Box::new(self.resolve(inner.as_ref()))),

            Type::Tuple(inner) => Ty::Tuple(inner.iter().map(|t| self.resolve(t)).collect()),

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

            Type::Infer => Ty::Infer,
        }
    }

    pub fn check_fn(&mut self, stmt: &Statement) -> TypedStmt {
        self.env.push();

        if let Statement::Func {
            name,
            params,
            return_type,
            body,
            span,
            ..
        } = stmt
        {
            self.current_ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));
            for p in params {
                let mut ty = self.resolve(&p.param_type);

                if p.is_ref {
                    ty = Ty::Ref {
                        inner: Box::new(ty),
                        is_mut: p.kind.is_mutable(),
                    };
                }

                if let Err(e) = self.env.define(p.name.clone(), ty, p.kind, &p.span) {
                    self.errors.push(e);
                }
            }

            let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

            let body = self.check_block(body);

            self.check_returns(stmt);
            self.env.pop();

            TypedStmt {
                kind: TypedStmtKind::Func {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: ret,
                    body,
                },
                ty: Ty::Unit,
                span: *span,
            }
        } else if let Statement::Method {
            obj_name,
            obj_type,
            name,
            is_ref_obj,
            params,
            return_type,
            body,
            span,
            ..
        } = stmt
        {
            self.current_ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

            let mut params = params.clone();

            params.insert(
                0,
                FuncParam {
                    name: obj_name.clone(),
                    annotated_name: obj_name.clone(),
                    param_type: obj_type.clone(),
                    span: *span,
                    kind: BindingKind::Param,
                    is_ref: *is_ref_obj,
                },
            );

            for p in &params {
                let mut ty = self.resolve(&p.param_type);

                if p.is_ref {
                    ty = Ty::Ref {
                        inner: Box::new(ty),
                        is_mut: p.kind.is_mutable(),
                    };
                }

                if let Err(e) = self.env.define(p.name.clone(), ty, p.kind, &p.span) {
                    self.errors.push(e);
                }
            }

            let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

            let body = self.check_block(body);

            self.check_returns(stmt);
            self.env.pop();

            TypedStmt {
                kind: TypedStmtKind::Func {
                    name: name.clone(),
                    params,
                    return_type: ret,
                    body,
                },
                ty: Ty::Unit,
                span: *span,
            }
        } else {
            self.env.pop();

            TypedStmt::error(stmt.span(), TypeError::Unreachable { span: stmt.span() })
        }
    }

    pub fn check_for_stmt(&mut self, stmt: &Statement) -> TypedStmt {
        self.env.push();

        self.in_loop = true;

        let res = if let Statement::While {
            condition,
            body,
            span,
        } = stmt
        {
            let cond = self.check_expr(condition, None);

            self.expect(&cond.ty, &Ty::Bool, stmt.span());

            let body = self.check_block(body);

            TypedStmt {
                kind: TypedStmtKind::While {
                    condition: cond,
                    body,
                },
                ty: Ty::Unit,
                span: *span,
            }
        } else if let Statement::ForRange {
            variable,
            iterable,
            body,
            span,
        } = stmt
        {
            let iter = self.check_expr(iterable, None);

            if let Expression::Range { end, .. } = iterable
                && end.is_none()
            {
                self.errors.push(TypeError::Unsupported {
                    desc: "Cannot iterate over an open-ended range in for-loop".to_string(),
                    span: *span,
                });
            }

            self.defined(
                variable.clone(),
                Ty::Sized { signed: true },
                BindingKind::Var,
                stmt.span(),
            );

            let body = self.check_block(body);

            TypedStmt {
                kind: TypedStmtKind::ForRange {
                    variable: variable.clone(),
                    iterable: iter,
                    body,
                },
                ty: Ty::Unit,
                span: *span,
            }
        } else {
            unreachable!()
        };

        self.env.pop();

        self.in_loop = false;

        res
    }

    pub fn check_stmt(&mut self, stmt: &Statement) -> TypedStmt {
        match stmt {
            Statement::Var {
                pattern,
                value,
                annotated_type,
                type_span,
                span,
            } => {
                self.current_type_span = *type_span;

                let expected = annotated_type.as_ref().map(|t| self.resolve(t));
                let typed_val = self.check_expr(value, expected.as_ref());

                if let BindingPattern::Tuple(_) = pattern
                    && !matches!(typed_val.ty, Ty::Tuple(_))
                {
                    self.errors.push(TypeError::Unsupported {
                        desc: "Destructuring non-tuple type".to_string(),
                        span: *span,
                    });
                    return TypedStmt::error(
                        *span,
                        TypeError::Unsupported {
                            desc: "Destructuring non-tuple type".to_string(),
                            span: *span,
                        },
                    );
                }

                let final_ty = if let Some(ref expected_ty) = expected {
                    let err_span = type_span.unwrap_or(value.span());

                    self.expect(&typed_val.ty, expected_ty, err_span);

                    expected_ty
                } else {
                    &typed_val.ty
                };

                match pattern {
                    BindingPattern::Ident(name) => self.defined(
                        name.clone(),
                        final_ty.clone(),
                        BindingKind::Var,
                        stmt.span(),
                    ),
                    BindingPattern::Tuple(names) => {
                        if let Ty::Tuple(elem_tys) = &final_ty {
                            if names.len() != elem_tys.len() {
                                self.errors.push(TypeError::Unsupported {
                                    desc: format!(
                                        "Destructuring tuple of length {} into {} variables",
                                        elem_tys.len(),
                                        names.len()
                                    ),
                                    span: *span,
                                })
                            } else {
                                for (name, elem_ty) in names.iter().zip(elem_tys.iter()) {
                                    self.defined(
                                        name.clone(),
                                        elem_ty.clone(),
                                        BindingKind::Var,
                                        stmt.span(),
                                    )
                                }
                            }
                        }
                    }
                }

                self.current_type_span = None;

                TypedStmt {
                    kind: TypedStmtKind::Var {
                        pattern: pattern.clone(),
                        ty: final_ty.clone(),
                        value: self.check_expr(value, expected.as_ref()),
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Let {
                pattern,
                value,
                annotated_type,
                span,
                type_span,
            } => {
                self.current_type_span = *type_span;

                let expected = annotated_type.as_ref().map(|t| self.resolve(t));
                let typed_val = self.check_expr(value, expected.as_ref());

                if let BindingPattern::Tuple(_) = pattern
                    && !matches!(typed_val.ty, Ty::Tuple(_))
                {
                    self.errors.push(TypeError::Unsupported {
                        desc: "Destructuring non-tuple type".to_string(),
                        span: *span,
                    });
                    return TypedStmt::error(
                        *span,
                        TypeError::Unsupported {
                            desc: "Destructuring non-tuple type".to_string(),
                            span: *span,
                        },
                    );
                }

                let final_ty = if let Some(expected_ty) = expected {
                    self.expect(&typed_val.ty, &expected_ty, value.span());

                    expected_ty
                } else {
                    typed_val.ty.clone()
                };

                match pattern {
                    BindingPattern::Ident(name) => self.defined(
                        name.clone(),
                        final_ty.clone(),
                        BindingKind::Var,
                        stmt.span(),
                    ),
                    BindingPattern::Tuple(names) => {
                        if let Ty::Tuple(elem_tys) = &final_ty {
                            if names.len() != elem_tys.len() {
                                self.errors.push(TypeError::Unsupported {
                                    desc: format!(
                                        "Destructuring tuple of length {} into {} variables",
                                        elem_tys.len(),
                                        names.len()
                                    ),
                                    span: *span,
                                })
                            } else {
                                for (name, elem_ty) in names.iter().zip(elem_tys.iter()) {
                                    self.defined(
                                        name.clone(),
                                        elem_ty.clone(),
                                        BindingKind::Var,
                                        stmt.span(),
                                    )
                                }
                            }
                        }
                    }
                }

                self.current_type_span = None;

                TypedStmt {
                    kind: TypedStmtKind::Let {
                        pattern: pattern.clone(),
                        ty: final_ty.clone(),
                        value: typed_val,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Const {
                name,
                value,
                annotated_type,
                type_span,
                span,
            } => {
                let expected = self.resolve(annotated_type);
                let typed_val = self.check_expr(value, Some(&expected));

                let err_span = type_span;
                self.expect(&typed_val.ty, &expected, *err_span);

                self.defined(name.clone(), expected, BindingKind::Const, stmt.span());

                TypedStmt {
                    kind: TypedStmtKind::Const {
                        name: name.clone(),
                        ty: typed_val.ty.clone(),
                        value: typed_val,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::While { .. } | Statement::ForRange { .. } => self.check_for_stmt(stmt),
            Statement::Break { .. } | Statement::Continue { .. } => {
                if !self.in_loop {
                    self.errors.push(TypeError::OutsideLoop {
                        kind: match stmt {
                            Statement::Break { .. } => LoopControlKind::Break,
                            Statement::Continue { .. } => LoopControlKind::Continue,
                            _ => unreachable!(),
                        },
                        span: stmt.span(),
                    })
                }

                match stmt {
                    Statement::Break { .. } => TypedStmt {
                        kind: TypedStmtKind::Break,
                        ty: Ty::Unit,
                        span: stmt.span(),
                    },
                    Statement::Continue { .. } => TypedStmt {
                        kind: TypedStmtKind::Continue,
                        ty: Ty::Unit,
                        span: stmt.span(),
                    },
                    _ => unreachable!(),
                }
            }
            Statement::Return { value, span } => {
                let ty = match value {
                    Some(v) => self.infer(v, None),
                    None => Ty::Unit,
                };

                let cur_ref = &self.current_ret.clone();

                self.expect(&ty, cur_ref, stmt.span());

                let val = value.as_ref().map(|expr| self.check_expr(expr, None));

                TypedStmt {
                    kind: TypedStmtKind::Return { value: val },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::ExternFunc {
                name,
                params,
                span,
                return_type,
            } => {
                let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));
                let sig = FnSig {
                    params: params
                        .iter()
                        .map(|p| ParamSig {
                            label: p.annotated_name.clone(),
                            ty: self.resolve(&p.param_type),
                        })
                        .collect(),
                    ret,
                    span: *span,
                    is_dynamic_method: false,
                };

                self.funcs.insert(name.clone(), sig.clone());

                TypedStmt {
                    kind: TypedStmtKind::ExternFunc {
                        name: name.clone(),
                        params: params.clone(),
                        return_type: sig.ret,
                        span: *span,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Expression { expression, span } => {
                if let Expression::Infix {
                    left,
                    operator,
                    right,
                    ..
                } = expression
                    && matches!(
                        operator,
                        Token::Assign
                            | Token::AddAndAssign
                            | Token::SubAndAssign
                            | Token::MulAndAssign
                            | Token::DivAndAssign
                            | Token::ModAndAssign
                    )
                {
                    let typed_left = self.check_expr(left.as_ref(), None);
                    let typed_right = self.check_expr(right.as_ref(), Some(&typed_left.ty));

                    match left.as_ref() {
                        Expression::Identifier { name, .. } => {
                            self.check_assignment(name.as_str(), right.as_ref());
                        }
                        Expression::Field {
                            object,
                            name: field_name,
                            span,
                        } => {
                            self.check_field_assignment(
                                object.as_ref(),
                                field_name,
                                right.as_ref(),
                                *span,
                            );
                        }
                        _ => {
                            self.errors.push(TypeError::Unsupported {
                                desc: "Invalid assignment target (non as l-value)".to_string(),
                                span: left.span(),
                            });
                        }
                    }
                    let assign_expr = TypedExpr {
                        kind: TypedExprKind::Infix {
                            left: Box::new(typed_left),
                            operator: operator.clone(),
                            right: Box::new(typed_right),
                        },
                        ty: Ty::Unit,
                        span: *span,
                    };

                    TypedStmt {
                        kind: TypedStmtKind::Expression(assign_expr),
                        ty: Ty::Unit,
                        span: *span,
                    }
                } else {
                    let typed_expr = self.check_expr(expression, None);

                    TypedStmt {
                        kind: TypedStmtKind::Expression(typed_expr),
                        ty: Ty::Unit,
                        span: *span,
                    }
                }
            }
            Statement::Struct { .. } => self.check_struct(stmt),
            Statement::Extend {
                target,
                methods,
                span,
            } => {
                let target = self.resolve(target);

                self.current_self_type = Some(target.clone());

                let mut typed_methods = Vec::new();

                for method in methods {
                    if matches!(method, Statement::Func { .. }) {
                        typed_methods.push(self.check_fn(method));
                    } else if let Statement::Method {
                        obj_type, obj_span, ..
                    } = method
                    {
                        let receiver = self.resolve(obj_type);

                        if receiver != target {
                            self.errors.push(TypeError::Mismatch {
                                expected: target.clone(),
                                found: receiver,
                                span: *obj_span,
                                annotation_span: Box::new(self.current_type_span),
                            });
                        }

                        typed_methods.push(self.check_fn(method));
                    }
                }

                self.current_self_type = None;

                TypedStmt {
                    kind: TypedStmtKind::Extend {
                        target,
                        methods: typed_methods,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Enum { name, cases, span } => TypedStmt {
                kind: TypedStmtKind::Enum {
                    name: name.clone(),
                    cases: cases.clone(),
                },
                ty: Ty::Unit,
                span: *span,
            },
            Statement::Defer { callee, span } => {
                let typed_callee = self.check_stmt(callee.as_ref());

                TypedStmt {
                    kind: TypedStmtKind::Defer {
                        callee: Box::new(typed_callee),
                        span: *span,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            _ => TypedStmt::error(
                stmt.span(),
                TypeError::Unsupported {
                    desc: format!("this statement: {:?}", stmt),
                    span: stmt.span(),
                },
            ),
        }
    }

    pub fn check_struct(&mut self, stmt: &Statement) -> TypedStmt {
        if let Statement::Struct { name, fields, .. } = stmt {
            for f in fields {
                let ty = self.resolve(&f.param_type);
                self.defined(f.name.clone(), ty, BindingKind::Var, stmt.span())
            }

            TypedStmt {
                kind: TypedStmtKind::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                },
                ty: Ty::Unit,
                span: stmt.span(),
            }
        } else {
            TypedStmt::error(stmt.span(), TypeError::Unreachable { span: stmt.span() })
        }
    }

    pub fn check_assignment(&mut self, name: &str, value_expr: &Expression) {
        let entity = match self.env.lookup(name) {
            Some(e) => e,
            None => {
                self.errors.push(TypeError::UnknownName {
                    name: name.to_string(),
                    span: value_expr.span(),
                });
                return;
            }
        };

        if !entity.kind.is_mutable() {
            self.errors.push(TypeError::AssignmentToImmutable {
                name: name.to_string(),
                kind: entity.kind,
                assign_span: value_expr.span(),
                decl_span: entity.span,
            });
            return;
        }

        let expected_ty = match &entity.ty {
            Ty::Ref { inner, .. } => inner.as_ref(),
            t => t,
        };

        let value_ty = self.infer(value_expr, None);

        self.expect(&value_ty, expected_ty, value_expr.span())
    }

    pub fn check_field_assignment(
        &mut self,
        object: &Expression,
        field_name: &str,
        value_expr: &Expression,
        span: Span,
    ) {
        if let Expression::Identifier { name: obj_name, .. } = object
            && let Some(entity) = self.env.lookup(obj_name)
            && !entity.kind.is_mutable()
        {
            self.errors.push(TypeError::AssignmentToImmutable {
                name: format!("{}.{}", obj_name, field_name),
                kind: entity.kind,
                decl_span: entity.span,
                assign_span: span,
            });
        }

        let obj_ty = self.infer(object, None);

        match obj_ty {
            Ty::Struct(struct_name) => {
                let field_ty = self
                    .structs
                    .get(&struct_name)
                    .and_then(|fields| fields.iter().find(|(n, _, _)| n == field_name))
                    .map(|(_, ty, _)| ty.clone());

                if let Some(field_ty) = field_ty {
                    let val_ty = self.infer(value_expr, None);

                    self.expect(&val_ty, &field_ty, value_expr.span());
                } else {
                    self.errors.push(TypeError::UnknownField {
                        struct_name,
                        field: field_name.to_string(),
                        span,
                    })
                }
            }
            Ty::Error => {}
            _ => self.errors.push(TypeError::NoFields { ty: obj_ty, span }),
        }
    }

    pub fn check_block(&mut self, block: &[Statement]) -> Vec<TypedStmt> {
        self.env.push();

        let mut stmts = Vec::new();

        for s in block {
            stmts.push(self.check_stmt(s));
        }
        self.env.pop();

        stmts
    }

    pub fn check_program(&mut self, program: &Program) -> TypedProgram {
        self.env.push();
        self.define_builtins();
        self.collect_signatures(&program.declarations);

        let mut typed_declarations = Vec::new();

        for stmt in &program.declarations {
            match stmt {
                Statement::Func { .. } => typed_declarations.push(self.check_fn(stmt)),
                Statement::Struct { .. } => typed_declarations.push(self.check_struct(stmt)),
                _ => {
                    typed_declarations.push(self.check_stmt(stmt));
                }
            }
        }

        if let Some(first_top_level_stmt) = program.main.first()
            && let Some(fn_span) = self.main_fn_span
        {
            self.errors.push(ConflictingEntryPoint {
                first_decl_span: first_top_level_stmt.span(),
                second_decl_span: fn_span,
            });
        }

        self.current_ret = Ty::Unit;
        let mut typed_main = Vec::new();

        for stmt in &program.main {
            typed_main.push(self.check_stmt(stmt));
        }
        self.env.pop();

        TypedProgram {
            package: program.package.clone(),
            declarations: typed_declarations,
            main: typed_main,
        }
    }

    pub fn infer(&mut self, expr: &Expression, expected_ty: Option<&Ty>) -> Ty {
        self.check_expr(expr, expected_ty).ty
    }

    pub fn resolve_args(
        &mut self,
        name: String,
        params: &[ParamSig],
        args: &[CallArg],
        call_span: Span,
    ) -> Vec<TypedExpr> {
        let mut resolved = vec![None; params.len()];
        let mut positional_idx = 0;
        let mut has_specific_error = false;
        let mut seen_named = false;
        let mut reported_pos_after_named = false;

        for arg in args {
            if let Some(ref label) = arg.label {
                if let Some(idx) = params.iter().position(|p| p.label == *label) {
                    seen_named = true;
                    if resolved[idx].is_some() {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!("Argument `{}` already provided", label),
                            span: arg.span,
                        });
                        has_specific_error = true;
                    } else {
                        let typed_arg = self.check_expr(&arg.value, Some(&params[idx].ty));

                        self.expect(&typed_arg.ty, &params[idx].ty, arg.span);

                        resolved[idx] = Some(typed_arg);
                    }
                } else {
                    self.errors.push(TypeError::Unsupported {
                        desc: format!("Unknown argument label `{}`", label),
                        span: arg.span,
                    });
                    has_specific_error = true;
                }
            } else {
                if seen_named {
                    if !reported_pos_after_named {
                        self.errors.push(TypeError::Unsupported {
                            desc: "Positional argument cannot follow named argument".to_string(),
                            span: arg.span,
                        });
                        reported_pos_after_named = true;
                    }
                    has_specific_error = true;
                }

                while positional_idx < resolved.len() && resolved[positional_idx].is_some() {
                    positional_idx += 1;
                }

                if positional_idx < params.len() {
                    let typed_arg = self.check_expr(&arg.value, Some(&params[positional_idx].ty));

                    self.expect(&typed_arg.ty, &params[positional_idx].ty, arg.span);

                    resolved[positional_idx] = Some(typed_arg);

                    positional_idx += 1;
                } else {
                    self.errors.push(TypeError::ArityMismatch {
                        name: name.clone(),
                        expected: params.len(),
                        found: args.len(),
                        span: call_span,
                    });

                    has_specific_error = true;
                }
            }
        }

        if args.len() != params.len() && !has_specific_error {
            self.errors.push(TypeError::ArityMismatch {
                name: name.clone(),
                expected: params.len(),
                found: args.len(),
                span: call_span,
            })
        }

        if !has_specific_error {
            for (idx, slot) in resolved.iter().enumerate() {
                if slot.is_none() {
                    self.errors.push(TypeError::Unsupported {
                        desc: format!("Missing argument for parameter `{}`", params[idx].label),
                        span: call_span,
                    });
                }
            }
        }

        resolved
            .into_iter()
            .map(|opt| {
                opt.unwrap_or_else(|| {
                    TypedExpr::error(
                        call_span,
                        TypeError::Unsupported {
                            desc: "Missing argument".to_string(),
                            span: call_span,
                        },
                    )
                })
            })
            .collect()
    }

    pub fn define_builtins(&mut self) {
        self.defined(
            "print".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![
                    Ty::Sized { signed: true },
                    Ty::Float { bits: 64 },
                    Ty::String,
                    Ty::Bool,
                    Ty::Char,
                ])],
                ret: Box::new(Ty::Unit),
            },
            BindingKind::Let,
            Span::default(),
        );
        self.defined(
            "println".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![
                    Ty::Sized { signed: true },
                    Ty::Float { bits: 64 },
                    Ty::String,
                    Ty::Bool,
                    Ty::Char,
                ])],
                ret: Box::new(Ty::Unit),
            },
            BindingKind::Let,
            Span::default(),
        );
        self.defined(
            "scanln".to_string(),
            Ty::Fn {
                params: vec![],
                ret: Box::new(Ty::String),
            },
            BindingKind::Var,
            Span::default(),
        )
    }

    pub fn defined(&mut self, name: String, ty: Ty, kind: BindingKind, span: Span) {
        if let Err(e) = self.env.define(name.clone(), ty, kind, &span)
            && !self.errors.contains(&e)
        {
            self.errors.push(e);
        }
    }

    pub fn expect(&mut self, actual: &Ty, expected: &Ty, span: Span) {
        if let (
            Ty::Ref {
                inner: exp_in,
                is_mut: false,
            },
            Ty::Ref {
                inner: act_in,
                is_mut: true,
            },
        ) = (expected, actual)
            && exp_in == act_in
        {
            return;
        }

        if let (
            Ty::Ref {
                inner: exp_in,
                is_mut: true,
            },
            Ty::Ref {
                inner: act_in,
                is_mut: false,
            },
        ) = (expected, actual)
            && exp_in == act_in
        {
            self.errors.push(TypeError::Unsupported {
                desc: "Cannot pass immutable reference to mutable parameter (variable must be declared with `var`)".to_string(),
                span,
            });
            return;
        }

        if let Ty::Union(types) = expected
            && types.iter().any(|ty| ty == actual)
        {
            return;
        }

        if !matches!(actual, Ty::Error) && !matches!(expected, Ty::Error) && actual != expected {
            self.errors.push(TypeError::Mismatch {
                expected: expected.clone(),
                found: actual.clone(),
                span,
                annotation_span: Box::new(self.current_type_span),
            })
        }
    }

    pub fn check_returns(&mut self, stmt: &Statement) {
        if let Statement::Func {
            name,
            body,
            span,
            ending_span,
            ..
        } = stmt
        {
            if self.ret_type_is_unit(stmt) {
                return;
            }

            let guarantee_returns = self.does_block_return(body);

            if !guarantee_returns {
                self.errors.push(TypeError::MissingReturn {
                    name: name.clone(),
                    func_span: *span,
                    close_brace_span: *ending_span,
                })
            }
        }
    }

    #[allow(dead_code)]
    pub fn collect_numbered_params(&mut self, body: &[Statement]) -> HashSet<u8> {
        let mut set = HashSet::new();

        for stmt in body {
            self.collect_stmt_params(stmt, &mut set);
        }

        set
    }

    pub fn collect_stmt_params(&self, stmt: &Statement, set: &mut HashSet<u8>) {
        match stmt {
            Statement::Expression { expression, .. } => {
                self.collect_expr_params(expression, set);
            }
            Statement::Let { value, .. } | Statement::Var { value, .. } => {
                self.collect_expr_params(value, set);
            }
            Statement::Return {
                value: Some(val), ..
            } => {
                self.collect_expr_params(val, set);
            }
            Statement::While {
                condition, body, ..
            } => {
                self.collect_expr_params(condition, set);
                for s in body {
                    self.collect_stmt_params(s, set);
                }
            }
            _ => {}
        }
    }

    fn does_block_return(&mut self, body: &[Statement]) -> bool {
        for stmt in body {
            match stmt {
                Statement::Var { .. }
                | Statement::Let { .. }
                | Statement::Const { .. }
                | Statement::While { .. }
                | Statement::ForRange { .. }
                | Statement::Func { .. }
                | Statement::Struct { .. }
                | Statement::Method { .. }
                | Statement::Break { .. }
                | Statement::Continue { .. }
                | Statement::ExternFunc { .. }
                | Statement::Extend { .. }
                | Statement::Enum { .. }
                | Statement::Defer { .. } => continue,
                Statement::Expression { expression, .. } => {
                    if let Expression::If {
                        then_block,
                        else_if,
                        else_block,
                        ..
                    } = expression
                    {
                        let does_return_if = self.does_block_return(then_block);
                        let mut do_return_elifs = Vec::new();

                        for elif in else_if {
                            do_return_elifs.push(self.does_block_return(&elif.block))
                        }

                        let does_return_else = self.does_block_return(else_block);

                        if does_return_if
                            && do_return_elifs.iter().all(|elif| *elif)
                            && does_return_else
                        {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
                Statement::Return { .. } => return true,
            }
        }

        false
    }

    fn ret_type_is_unit(&mut self, stmt: &Statement) -> bool {
        if let Statement::Func { return_type, .. } = stmt {
            if return_type.is_none() {
                return true;
            }
            return false;
        }

        true
    }

    pub fn extract_pattern_case(&mut self, pattern: &Expression) -> (String, bool) {
        let mut has_wildcard = false;

        let name = match pattern {
            Expression::Identifier { name, .. } => {
                if name == "_" {
                    has_wildcard = true;
                }

                name
            }
            Expression::Call { function, .. } => {
                if let Expression::Identifier { name, .. } = function.as_ref() {
                    name
                } else {
                    unreachable!()
                }
            }
            Expression::Field { name, .. } | Expression::MethodCall { name, .. } => name,
            _ => unreachable!(),
        };

        (name.clone(), has_wildcard)
    }

    pub fn type_to_prefix<'a>(&self, ty: &'a Ty) -> Cow<'a, str> {
        match ty {
            Ty::Sized { signed: true } => Cow::from("int"),
            Ty::Sized { signed: false } => Cow::from("uint"),
            Ty::Int { signed: true, bits } => Cow::from(format!("int{bits}")),
            Ty::Int {
                signed: false,
                bits,
            } => Cow::from(format!("uint{bits}")),
            Ty::Float { bits } => Cow::from(format!("float{bits}")),
            Ty::String => Cow::from("string"),
            Ty::Char => Cow::from("char"),
            Ty::Bool => Cow::from("bool"),
            Ty::Struct(name) => Cow::from(name),
            Ty::Array(inner) => {
                Cow::Owned(format!("array_{}", self.type_to_prefix(inner.as_ref())))
            }
            _ => Cow::from("unknown"),
        }
    }

    pub fn unify_numeric_types(&mut self, left: &Ty, right: &Ty) -> Option<Ty> {
        // 1. identical types
        if left == right && (matches!(left, Ty::Int { .. } | Ty::Float { .. })) {
            return Some(left.clone());
        }

        // 2. One of operands is a literal, second is integer
        if let (Ty::Int { bits, signed }, Ty::Sized { .. })
        | (Ty::Sized { .. }, Ty::Int { bits, signed }) = (left, right)
        {
            return Some(Ty::Int {
                bits: *bits,
                signed: *signed,
            });
        }

        // 3. One of operands is a literal, second is floating
        if let (Ty::Float { bits }, Ty::Sized { .. }) | (Ty::Sized { .. }, Ty::Float { bits }) =
            (left, right)
        {
            return Some(Ty::Float { bits: *bits });
        }

        // 4. Two literals
        if let (Ty::Sized { signed: s1 }, Ty::Sized { signed: s2 }) = (left, right) {
            return Some(Ty::Sized { signed: *s1 && *s2 });
        }

        None
    }
}
