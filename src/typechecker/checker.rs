use crate::hir::typed_ast::{TypedExpr, TypedProgram};
use crate::lexer::span::Span;
use crate::parser::expression::CallArg;
use crate::parser::program::Program;
use crate::parser::types::Type;
use crate::parser::{Expression, Statement};
use crate::typechecker::env::Env;
pub use crate::typechecker::error::TypeError;
use crate::typechecker::error::TypeError::ConflictingEntryPoint;
use crate::typechecker::error::{BindingKind, DefinitionKind};
use crate::typechecker::types::Ty;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct FnSig {
    pub generic_params: Vec<String>,
    pub params: Vec<ParamSig>,
    pub ret: Ty,
    pub span: Span,

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
    pub current_self_type: Option<Ty>,
    pub current_generics: Vec<String>,
    pub known_types: HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct ParamSig {
    pub label: String,
    pub ty: Ty,
}

#[derive(Clone, Debug)]
pub struct StructSig {
    pub generic_params: Vec<String>,
    pub fields: Vec<(String, Ty, Span)>,
    pub span: Span,
}

impl Checker {
    pub fn collect_signatures(&mut self, program: &[Statement]) {
        for stmt in program {
            if let Statement::Struct { name, .. } | Statement::Enum { name, .. } = stmt {
                self.known_types.insert(name.clone());
            }
        }

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
                    generic_params,
                    ..
                } => {
                    if name == "main" {
                        self.main_fn_span = Some(*span)
                    }

                    let saved_generics =
                        std::mem::replace(&mut self.current_generics, generic_params.clone());

                    let params: Vec<_> = params
                        .iter()
                        .map(|p| {
                            self.current_type_span = Some(p.span);
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

                    self.current_generics = saved_generics;

                    self.funcs.insert(
                        name.clone(),
                        FnSig {
                            generic_params: generic_params.clone(),
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
                        .map(|p| {
                            self.current_type_span = Some(p.span);
                            let ps = ParamSig {
                                label: p.annotated_name.clone(),
                                ty: self.resolve(&p.param_type),
                            };
                            self.current_type_span = None;

                            ps
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
                            generic_params: vec![],
                            params,
                            ret,
                            span: *span,
                            is_dynamic_method: true,
                        },
                    );
                }

                Statement::Struct {
                    name,
                    generic_params,
                    fields,
                    span,
                } => {
                    let saved_generics =
                        std::mem::replace(&mut self.current_generics, generic_params.clone());

                    let fields = fields
                        .iter()
                        .map(|f| {
                            self.current_type_span = Some(f.span);
                            let res = (f.name.clone(), self.resolve(&f.param_type), f.span);

                            self.current_type_span = None;

                            res
                        })
                        .collect::<Vec<(_, _, _)>>();

                    self.current_generics = saved_generics;

                    let struct_sign = StructSig {
                        generic_params: generic_params.clone(),
                        fields,
                        span: *span,
                    };

                    if self.structs.contains_key(name) {
                        self.errors.push(TypeError::DuplicateDefinition {
                            name: name.clone(),
                            first_span: match self.structs.get(name) {
                                Some(s) => s.span,
                                None => unreachable!(),
                            },
                            second_span: stmt.span(),
                            def_kind: DefinitionKind::Struct,
                        });

                        continue;
                    }

                    self.structs.insert(name.clone(), struct_sign);
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
                                .map(|p| {
                                    self.current_type_span = Some(p.span);
                                    let ps = ParamSig {
                                        label: p.annotated_name.clone(),
                                        ty: self.resolve(&p.param_type),
                                    };

                                    self.current_type_span = None;
                                    ps
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
                                    generic_params: vec![],
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
                                .map(|p| {
                                    self.current_type_span = Some(p.span);
                                    let ps = ParamSig {
                                        label: p.annotated_name.clone(),
                                        ty: self.resolve(&p.param_type),
                                    };
                                    self.current_type_span = None;

                                    ps
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
                                    generic_params: vec![],
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
                    _ => {
                        if self.current_generics.contains(name) {
                            Ty::Param(name.clone())
                        } else if self.known_types.contains(name) {
                            Ty::Struct(name.clone())
                        } else {
                            self.errors.push(TypeError::UnknownName {
                                name: name.clone(),
                                span: self.current_type_span.unwrap_or_default(),
                            });
                            Ty::Error
                        }
                    }
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

            Type::Generic { name, param } => Ty::Instantiated {
                name: name.clone(),
                args: vec![self.resolve(param)],
            },

            Type::Infer => Ty::Infer,
        }
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
                    Ty::Sized { signed: false },
                    Ty::Int {
                        bits: 8,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 16,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 32,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 64,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 8,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 16,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 32,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 64,
                        signed: true,
                    },
                    Ty::Float { bits: 64 },
                    Ty::Float { bits: 32 },
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
                    Ty::Sized { signed: false },
                    Ty::Int {
                        bits: 8,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 16,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 32,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 64,
                        signed: false,
                    },
                    Ty::Int {
                        bits: 8,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 16,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 32,
                        signed: true,
                    },
                    Ty::Int {
                        bits: 64,
                        signed: true,
                    },
                    Ty::Float { bits: 64 },
                    Ty::Float { bits: 32 },
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

        if expected == &Ty::Infer {
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
                expected: Box::new(expected.clone()),
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

    pub fn resolve_generics(&mut self, ty: Ty, resolved: &HashMap<String, Ty>) -> Ty {
        match &ty {
            Ty::Param(name) => resolved.get(name).unwrap_or(&ty).clone(),
            Ty::Instantiated { args, name } => {
                for arg in args {
                    self.resolve_generics(arg.clone(), resolved);
                }

                resolved.get(name).unwrap_or(&Ty::Unit).clone()
            }
            Ty::Fn { params, ret } => Ty::Fn {
                params: params
                    .iter()
                    .map(|param| self.resolve_generics(param.clone(), resolved))
                    .collect(),
                ret: Box::new(self.resolve_generics(ret.as_ref().clone(), resolved)),
            },
            Ty::Ref { inner, is_mut } => Ty::Ref {
                inner: Box::new(self.resolve_generics(inner.as_ref().clone(), resolved)),
                is_mut: *is_mut,
            },
            Ty::Array(inner) => Ty::Array(Box::new(
                self.resolve_generics(inner.as_ref().clone(), resolved),
            )),
            Ty::Tuple(inner) => Ty::Tuple(
                inner
                    .iter()
                    .map(|t| self.resolve_generics(t.clone(), resolved))
                    .collect(),
            ),
            Ty::Int { .. }
            | Ty::Float { .. }
            | Ty::Sized { .. }
            | Ty::Unit
            | Ty::String
            | Ty::Bool
            | Ty::Char
            | Ty::Struct(_)
            | Ty::Union(_)
            | Ty::Infer
            | Ty::Error => ty,
        }
    }

    pub fn unify_generic(&self, param_ty: &Ty, arg_ty: &Ty, resolved: &mut HashMap<String, Ty>) {
        match (param_ty, arg_ty) {
            (Ty::Param(name), actual) if !matches!(actual, Ty::Infer | Ty::Error) => {
                resolved
                    .entry(name.clone())
                    .or_insert_with(|| actual.clone());
            }
            (Ty::Array(p_inner), Ty::Array(a_inner)) => {
                self.unify_generic(p_inner.as_ref(), a_inner.as_ref(), resolved)
            }
            (Ty::Ref { inner: p_ty, .. }, Ty::Ref { inner: a_ty, .. }) => {
                self.unify_generic(p_ty.as_ref(), a_ty.as_ref(), resolved)
            }
            (Ty::Tuple(p_tys), Ty::Tuple(a_tys)) => {
                for (p, a) in p_tys.iter().zip(a_tys.iter()) {
                    self.unify_generic(p, a, resolved)
                }
            }
            (
                Ty::Fn {
                    params: p_tys,
                    ret: p_ret,
                },
                Ty::Fn {
                    params: a_tys,
                    ret: a_ret,
                },
            ) => {
                for (p, a) in p_tys.iter().zip(a_tys.iter()) {
                    self.unify_generic(p, a, resolved)
                }

                self.unify_generic(p_ret.as_ref(), a_ret.as_ref(), resolved)
            }
            _ => {}
        }
    }

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
