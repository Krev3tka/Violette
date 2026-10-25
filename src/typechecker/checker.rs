use crate::hir::typed_ast::{
    TypedElseIf, TypedExpr, TypedExprKind, TypedMatchArm, TypedProgram, TypedStmt, TypedStmtKind,
    TypedStructLiteralField,
};
use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::expression::CallArg;
use crate::parser::program::Program;
use crate::parser::types::Type;
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
    params: Vec<ParamSig>,
    ret: Ty,
    span: Span,
}

#[derive(Clone, Debug)]
pub struct ParamSig {
    pub label: String,
    pub ty: Ty,
}

pub type StructSig = Vec<(String, Ty, Span)>;

#[derive(Default)]
pub struct Checker {
    pub funcs: HashMap<String, FnSig>,
    pub structs: HashMap<String, StructSig>,
    pub variant_cases: HashMap<String, (String, Option<Ty>)>,
    pub env: Env,
    current_ret: Ty,
    pub errors: Vec<TypeError>,
    pub main_fn_span: Option<Span>,
    pub in_loop: bool,
}

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
                        Ty::Int => "int".to_string(),
                        Ty::Float => "float64".to_string(),
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
                                },
                            );
                        }
                    }
                }
                Statement::Variant { name, cases, .. } => {
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
                    "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16"
                    | "UInt32" | "UInt64" => Ty::Int,
                    "Float32" | "Float64" => Ty::Float,
                    "String" => Ty::String,
                    "Bool" => Ty::Bool,
                    "Char" => Ty::Char,
                    _ => Ty::Struct(name.clone()),
                }
            }

            Type::Array(inner) => Ty::Array(Box::new(self.resolve(inner.as_ref()))),

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

            self.defined(variable.clone(), Ty::Int, BindingKind::Var, stmt.span());

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
                name,
                value,
                annotated_type,
                type_span,
                span,
            } => {
                let expected = annotated_type.as_ref().map(|t| self.resolve(t));
                let typed_val = self.check_expr(value, expected.as_ref());

                let final_ty = if let Some(expected_ty) = expected {
                    let err_span = type_span.unwrap_or(value.span());

                    self.expect(&typed_val.ty, &expected_ty, err_span);

                    expected_ty
                } else {
                    typed_val.ty.clone()
                };

                self.defined(
                    name.clone(),
                    final_ty.clone(),
                    BindingKind::Var,
                    stmt.span(),
                );

                TypedStmt {
                    kind: TypedStmtKind::Var {
                        name: name.clone(),
                        ty: final_ty,
                        value: typed_val,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Let {
                name,
                value,
                annotated_type,
                span,
                ..
            } => {
                let expected = annotated_type.as_ref().map(|t| self.resolve(t));
                let typed_val = self.check_expr(value, expected.as_ref());

                let final_ty = if let Some(expected_ty) = expected {
                    self.expect(&typed_val.ty, &expected_ty, value.span());

                    expected_ty
                } else {
                    typed_val.ty.clone()
                };

                self.defined(
                    name.clone(),
                    final_ty.clone(),
                    BindingKind::Let,
                    stmt.span(),
                );

                TypedStmt {
                    kind: TypedStmtKind::Let {
                        name: name.clone(),
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

                let mut typed_methods = Vec::new();

                for method in methods {
                    if let Statement::Func { .. } = method {
                        typed_methods.push(self.check_fn(method));
                    }
                }

                TypedStmt {
                    kind: TypedStmtKind::Extend {
                        target,
                        methods: typed_methods,
                    },
                    ty: Ty::Unit,
                    span: *span,
                }
            }
            Statement::Variant { name, cases, span } => TypedStmt {
                kind: TypedStmtKind::Variant {
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

    #[allow(dead_code)]
    pub fn check_expr(&mut self, expr: &Expression, expected: Option<&Ty>) -> TypedExpr {
        match expr {
            Expression::IntLiteral { val, span } => TypedExpr {
                kind: TypedExprKind::IntLiteral { val: *val },
                ty: Ty::Int,
                span: *span,
            },
            Expression::FloatLiteral { val, span } => TypedExpr {
                kind: TypedExprKind::FloatLiteral { val: *val },
                ty: Ty::Float,
                span: *span,
            },
            Expression::BoolLiteral { val, span } => TypedExpr {
                kind: TypedExprKind::BoolLiteral { val: *val },
                ty: Ty::Bool,
                span: *span,
            },
            Expression::CharLiteral { val, span } => TypedExpr {
                kind: TypedExprKind::CharLiteral { val: *val },
                ty: Ty::Char,
                span: *span,
            },
            Expression::StringLiteral { val, span } => TypedExpr {
                kind: TypedExprKind::StringLiteral { val: val.clone() },
                ty: Ty::String,
                span: *span,
            },
            Expression::ArrayLiteral { values, span } => {
                let expected_elem_ty = match expected {
                    Some(Ty::Array(inner)) => Some(inner.as_ref()),
                    _ => None,
                };

                let target_elem_ty = if let Some(exp) = expected_elem_ty {
                    exp.clone()
                } else {
                    match values.first() {
                        Some(first_expr) => self.infer(first_expr, None),
                        None => {
                            self.errors.push(TypeError::Unsupported {
                                desc: "cannot infer type of an empty array, add type annotation like `let a: [Int] = []``".to_string(),
                                span: *span,
                            });
                            Ty::Error
                        }
                    }
                };

                let values: Vec<_> = values
                    .iter()
                    .map(|expr| {
                        let typed = self.check_expr(expr, expected);
                        self.expect(&typed.ty, &target_elem_ty, expr.span());
                        typed
                    })
                    .collect();

                TypedExpr {
                    kind: TypedExprKind::ArrayLiteral { values },
                    ty: Ty::Array(Box::new(target_elem_ty)),
                    span: *span,
                }
            }
            Expression::Identifier { name, span } => {
                if let Some((variant_name, None)) = self.variant_cases.get(name) {
                    return TypedExpr {
                        kind: TypedExprKind::Identifier {
                            name: name.clone(),
                            is_ref: false,
                        },
                        ty: Ty::Struct(variant_name.clone()),
                        span: *span,
                    };
                }

                let (ty, is_ref) = match self.env.lookup(name) {
                    Some(entity) => match &entity.ty {
                        Ty::Ref { inner, .. } => (*inner.clone(), true),
                        t => (t.clone(), false),
                    },
                    None => {
                        let err = TypeError::UnknownName {
                            name: name.clone(),
                            span: expr.span(),
                        };

                        if !self.errors.contains(&err) {
                            self.errors.push(err);
                        }

                        (Ty::Error, false)
                    }
                };

                TypedExpr {
                    kind: TypedExprKind::Identifier {
                        name: name.clone(),
                        is_ref,
                    },
                    ty,
                    span: *span,
                }
            }
            Expression::Infix {
                left,
                operator,
                right,
                span,
            } => {
                let typed_left = self.check_expr(left.as_ref(), expected);
                let typed_right = self.check_expr(right.as_ref(), expected);

                let left_ty = typed_left.clone().ty;
                let right_ty = typed_right.clone().ty;

                let ty = match operator {
                    Token::Add => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::Float, Ty::Float) => Ty::Float,
                        (Ty::String, Ty::String) => Ty::String,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            self.errors.push(TypeError::InvalidBinaryOperator {
                                operator: operator.clone(),
                                left: Box::new(left_ty.clone()),
                                right: Box::new(right_ty.clone()),
                                span: *span,
                            });
                            Ty::Error
                        }
                    },
                    Token::Subtract | Token::Multiply | Token::Divide | Token::Power => {
                        match (&left_ty, &right_ty) {
                            (Ty::Int, Ty::Int) => Ty::Int,
                            (Ty::Float, Ty::Float) => Ty::Float,
                            (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                            _ => {
                                self.errors.push(TypeError::InvalidBinaryOperator {
                                    operator: operator.clone(),
                                    left: Box::new(left_ty.clone()),
                                    right: Box::new(right_ty.clone()),
                                    span: *span,
                                });
                                Ty::Error
                            }
                        }
                    }
                    Token::Modulus => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            self.errors.push(TypeError::InvalidBinaryOperator {
                                operator: operator.clone(),
                                left: Box::new(left_ty.clone()),
                                right: Box::new(right_ty.clone()),
                                span: *span,
                            });
                            Ty::Error
                        }
                    },
                    Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => {
                        match (&left_ty, &right_ty) {
                            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) => Ty::Bool,
                            (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                            _ => {
                                self.errors.push(TypeError::InvalidBinaryOperator {
                                    operator: operator.clone(),
                                    left: Box::new(left_ty.clone()),
                                    right: Box::new(right_ty.clone()),
                                    span: *span,
                                });
                                Ty::Error
                            }
                        }
                    }
                    Token::Equals | Token::NotEquals => match (&left_ty, &right_ty) {
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            if left_ty != right_ty {
                                self.errors.push(TypeError::InvalidBinaryOperator {
                                    operator: operator.clone(),
                                    left: Box::new(left_ty.clone()),
                                    right: Box::new(right_ty.clone()),
                                    span: *span,
                                });
                                return TypedExpr::error(
                                    *span,
                                    TypeError::InvalidBinaryOperator {
                                        operator: operator.clone(),
                                        left: Box::new(left_ty.clone()),
                                        right: Box::new(right_ty.clone()),
                                        span: *span,
                                    },
                                );
                            }
                            Ty::Bool
                        }
                    },
                    Token::LogicAnd | Token::LogicOr => {
                        self.expect(&left_ty, &Ty::Bool, expr.span());
                        self.expect(&right_ty, &Ty::Bool, expr.span());
                        Ty::Bool
                    }
                    Token::BitAnd | Token::BitOr | Token::BitNot | Token::BitXOR => {
                        self.expect(&left_ty, &Ty::Int, expr.span());
                        self.expect(&right_ty, &Ty::Int, expr.span());
                        Ty::Int
                    }
                    _ => Ty::Error,
                };

                TypedExpr {
                    kind: TypedExprKind::Infix {
                        left: Box::new(typed_left),
                        operator: operator.clone(),
                        right: Box::new(typed_right),
                    },
                    ty,
                    span: *span,
                }
            }
            Expression::Prefix {
                operator,
                right,
                span,
            } => {
                let right_ty = self.infer(right.as_ref(), None);

                let ty = match operator {
                    Token::LogicNot => {
                        self.expect(&right_ty, &Ty::Bool, expr.span());

                        Ty::Bool
                    }
                    Token::Subtract => match right_ty {
                        Ty::Int => Ty::Int,
                        Ty::Float => Ty::Float,
                        _ => {
                            self.errors.push(TypeError::InvalidUnaryOperator {
                                operator: operator.clone(),
                                operand: right_ty,
                                span: *span,
                            });

                            Ty::Error
                        }
                    },
                    Token::BitNot => {
                        self.expect(&right_ty, &Ty::Int, expr.span());

                        Ty::Int
                    }
                    Token::BitAnd => match right.as_ref() {
                        Expression::Identifier { name, .. } => match self.env.lookup(name) {
                            Some(entity) => Ty::Ref {
                                inner: Box::new(entity.ty.clone()),
                                is_mut: entity.kind.is_mutable(),
                            },
                            None => {
                                self.errors.push(TypeError::UnknownName {
                                    name: name.clone(),
                                    span: *span,
                                });

                                Ty::Error
                            }
                        },
                        _ => {
                            self.errors.push(TypeError::Unsupported {
                                desc: "Cannot take reference from non-variable expression"
                                    .to_string(),
                                span: *span,
                            });
                            Ty::Error
                        }
                    },
                    _ => Ty::Error,
                };

                TypedExpr {
                    kind: TypedExprKind::Prefix {
                        operator: operator.clone(),
                        right: Box::new(self.check_expr(right, expected)),
                    },
                    ty,
                    span: *span,
                }
            }
            Expression::Postfix {
                left,
                operator,
                span,
            } => {
                let typed_left = self.check_expr(left.as_ref(), expected);

                if let Expression::Identifier { name, .. } = left.as_ref()
                    && let Some(entity) = self.env.lookup(name)
                    && !entity.kind.is_mutable()
                {
                    self.errors.push(TypeError::AssignmentToImmutable {
                        name: name.clone(),
                        kind: entity.kind,
                        decl_span: entity.span,
                        assign_span: *span,
                    });
                }

                let ty = match typed_left.ty {
                    Ty::Int => Ty::Int,
                    Ty::Float => Ty::Float,
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::InvalidUnaryOperator {
                            operator: operator.clone(),
                            operand: typed_left.ty.clone(),
                            span: *span,
                        });
                        Ty::Error
                    }
                };

                TypedExpr {
                    kind: TypedExprKind::Postfix {
                        left: Box::new(typed_left),
                        operator: operator.clone(),
                    },
                    ty,
                    span: *span,
                }
            }
            Expression::Range {
                start,
                end,
                range_kind,
                span,
            } => {
                let typed_start = start
                    .as_ref()
                    .map(|s| Box::new(self.check_expr(s.as_ref(), expected)));

                let typed_end = end
                    .as_ref()
                    .map(|e| Box::new(self.check_expr(e.as_ref(), expected)));

                let start_ty = typed_start.as_ref().map_or(Ty::Int, |s| s.ty.clone());
                let end_ty = typed_end.as_ref().map_or(Ty::Int, |e| e.ty.clone());

                self.expect(&start_ty, &Ty::Int, expr.span());
                self.expect(&end_ty, &Ty::Int, expr.span());

                TypedExpr {
                    kind: TypedExprKind::Range {
                        start: typed_start,
                        end: typed_end,
                        range_kind: range_kind.clone(),
                    },
                    ty: Ty::Generic {
                        name: "Range".to_string(),
                        param: Box::new(Ty::Int),
                    },
                    span: *span,
                }
            }
            Expression::Index { left, index, span } => {
                let typed_left = self.check_expr(left.as_ref(), expected);
                let typed_index = self.check_expr(index.as_ref(), None);

                let is_slice =
                    matches!(typed_index.ty, Ty::Generic { ref name, .. } if name == "Range");

                if !is_slice {
                    self.expect(&typed_index.ty, &Ty::Int, index.span());
                }

                let el_ty = match typed_left.ty {
                    Ty::Array(inner) => {
                        if is_slice {
                            Ty::Array(inner.clone())
                        } else {
                            *inner.clone()
                        }
                    }
                    Ty::String => {
                        if is_slice {
                            Ty::String
                        } else {
                            Ty::Char
                        }
                    }
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!("Type `{:?}` doesn't support indexing", typed_left.ty),
                            span: *span,
                        });

                        Ty::Error
                    }
                };

                TypedExpr {
                    kind: TypedExprKind::Index {
                        left: Box::new(self.check_expr(left.as_ref(), expected)),
                        index: Box::new(self.check_expr(index.as_ref(), expected)),
                    },
                    ty: el_ty,
                    span: *span,
                }
            }
            Expression::Call {
                function,
                args,
                span,
            } => {
                if let Expression::Identifier {
                    name,
                    span: fn_span,
                    ..
                } = function.as_ref()
                    && let Some((variant_name, Some(expected_payload))) =
                        self.variant_cases.get(name).cloned()
                {
                    if args.len() != 1 {
                        self.errors.push(TypeError::ArityMismatch {
                            name: name.clone(),
                            expected: 1,
                            found: args.len(),
                            span: *span,
                        });

                        let typed_arg = self.check_expr(&args[0].value, Some(&expected_payload));
                        self.expect(&typed_arg.ty, &expected_payload, args[0].span);

                        let typed_fn = TypedExpr {
                            kind: TypedExprKind::Identifier {
                                name: name.clone(),
                                is_ref: false,
                            },
                            ty: Ty::Fn {
                                params: vec![expected_payload.clone()],
                                ret: Box::new(Ty::Struct(variant_name.clone())),
                            },
                            span: *fn_span,
                        };

                        return TypedExpr {
                            kind: TypedExprKind::Call {
                                function: Box::new(typed_fn),
                                args: args
                                    .iter()
                                    .map(|e| self.check_expr(&e.value, None))
                                    .collect(),
                            },
                            ty: Ty::Struct(variant_name),
                            span: *span,
                        };
                    }

                    let arg_ty = self.infer(&args[0].value, Some(&expected_payload));
                    self.expect(&arg_ty, &expected_payload, args[0].span);

                    return TypedExpr {
                        kind: TypedExprKind::Call {
                            function: Box::new(self.check_expr(function.as_ref(), None)),
                            args: args
                                .iter()
                                .map(|e| self.check_expr(&e.value, None))
                                .collect(),
                        },
                        ty: Ty::Struct(variant_name),
                        span: *span,
                    };
                }

                if let Expression::Identifier { name, .. } = function.as_ref()
                    && self.env.lookup(name).is_none()
                    && !args.is_empty()
                {
                    let first_arg_ty = self.infer(&args[0].value, None);
                    if let Ty::Struct(ref s_name) = first_arg_ty {
                        let method_name = format!("{}.{}", s_name, name);
                        if let Some(sig) = self.funcs.get(&method_name).cloned() {
                            if args.len() != sig.params.len() {
                                self.errors.push(TypeError::ArityMismatch {
                                    name: name.clone(),
                                    expected: sig.params.len(),
                                    found: args.len(),
                                    span: *span,
                                });

                                return TypedExpr {
                                    kind: TypedExprKind::Call {
                                        function: Box::new(
                                            self.check_expr(function.as_ref(), None),
                                        ),
                                        args: args
                                            .iter()
                                            .zip(sig.params.iter())
                                            .map(|(e, param_ty)| {
                                                self.check_expr(&e.value, Some(&param_ty.ty))
                                            })
                                            .collect(),
                                    },
                                    ty: sig.ret,
                                    span: *span,
                                };
                            }

                            let typed_args =
                                self.resolve_args(name.clone(), &sig.params, args, *span);

                            return TypedExpr {
                                kind: TypedExprKind::Call {
                                    function: Box::new(self.check_expr(function.as_ref(), None)),
                                    args: typed_args,
                                },
                                ty: sig.ret,
                                span: *span,
                            };
                        }
                    }
                }

                let callee = if matches!(function.as_ref(), Expression::Lambda { .. }) {
                    let arg_types: Vec<Ty> =
                        args.iter().map(|a| self.infer(&a.value, None)).collect();

                    let expected_fn = Ty::Fn {
                        params: arg_types,
                        ret: Box::new(Ty::Infer),
                    };
                    self.infer(function, Some(&expected_fn))
                } else {
                    self.infer(function, None)
                };

                if let Expression::Identifier { name, .. } = function.as_ref()
                    && let Some(sig) = self.funcs.get(name).cloned()
                {
                    let typed_args = self.resolve_args(name.clone(), &sig.params, args, *span);

                    return TypedExpr {
                        kind: TypedExprKind::Call {
                            function: Box::new(self.check_expr(function.as_ref(), None)),
                            args: typed_args,
                        },
                        ty: sig.ret,
                        span: *span,
                    };
                }

                match callee {
                    Ty::Fn { params, ret } => {
                        if args.len() != params.len() {
                            self.errors.push(TypeError::ArityMismatch {
                                name: match function.as_ref() {
                                    Expression::Identifier { name: n, .. } => n.to_string(),
                                    _ => "<function value>".to_string(),
                                },
                                expected: params.len(),
                                found: args.len(),
                                span: *span,
                            });
                        }

                        let mut typed_args = Vec::new();

                        for (arg, param_ty) in args.iter().zip(params.iter()) {
                            let typed_arg = self.check_expr(&arg.value, Some(param_ty));

                            self.expect(&typed_arg.ty, param_ty, arg.span);

                            typed_args.push(typed_arg);
                        }

                        let typed_function =
                            if matches!(function.as_ref(), Expression::Lambda { .. }) {
                                let expected_fn = Ty::Fn {
                                    params: params.clone(),
                                    ret: ret.clone(),
                                };
                                self.check_expr(function.as_ref(), Some(&expected_fn))
                            } else {
                                self.check_expr(function.as_ref(), None)
                            };

                        TypedExpr {
                            kind: TypedExprKind::Call {
                                function: Box::new(typed_function),
                                args: args
                                    .iter()
                                    .zip(params.iter())
                                    .map(|(e, param_ty)| self.check_expr(&e.value, Some(param_ty)))
                                    .collect(),
                            },
                            ty: *ret,
                            span: *span,
                        }
                    }
                    Ty::Error => TypedExpr {
                        kind: TypedExprKind::Call {
                            function: Box::new(self.check_expr(function.as_ref(), expected)),
                            args: args
                                .iter()
                                .map(|e| self.check_expr(&e.value, None))
                                .collect(),
                        },
                        ty: Ty::Error,
                        span: *span,
                    },
                    _ => {
                        let callee_name = match function.as_ref() {
                            Expression::Identifier { name, .. } => name.clone(),
                            Expression::IntLiteral { val, .. } => val.to_string(),
                            Expression::FloatLiteral { val, .. } => val.to_string(),
                            Expression::StringLiteral { val, .. } => format!("\"{}\"", val),
                            _ => format!("{}", callee),
                        };

                        self.errors.push(TypeError::NotCallable {
                            name: callee_name,
                            ty: callee,
                            span: *span,
                        });

                        TypedExpr {
                            kind: TypedExprKind::Call {
                                function: Box::new(self.check_expr(function.as_ref(), expected)),
                                args: args
                                    .iter()
                                    .map(|e| self.check_expr(&e.value, None))
                                    .collect(),
                            },
                            ty: Ty::Error,
                            span: *span,
                        }
                    }
                }
            }
            Expression::Lambda {
                params,
                return_type,
                body,
                span,
            } => {
                let (expected_fn_params, expected_ret) = match expected {
                    Some(Ty::Fn { params, ret, .. }) => (Some(params), Some(ret.as_ref())),
                    _ => (None, None),
                };

                let mut param_tys = Vec::new();

                for (i, p) in params.iter().enumerate() {
                    let mut ty = self.resolve(&p.param_type);

                    if ty == Ty::Infer {
                        if let Some(expected_p) = expected_fn_params.and_then(|ep| ep.get(i)) {
                            ty = expected_p.clone();
                        } else {
                            self.errors.push(TypeError::Unsupported {
                                desc: "Cannot infer type for parameter (try to pass lambda directly to a typed function)".to_string(),
                                span: p.span,
                            });
                            ty = Ty::Error;
                        }
                    }
                    param_tys.push(ty)
                }

                self.env.push();

                for (p, ty) in params.iter().zip(param_tys.iter()) {
                    self.defined(p.name.clone(), ty.clone(), BindingKind::Var, p.span);
                }

                let ret = match return_type {
                    Some(t) => self.resolve(t),
                    None => {
                        if let Some(exp_r) = expected_ret
                            && *exp_r != Ty::Infer
                            && *exp_r != Ty::Error
                        {
                            exp_r.clone()
                        } else {
                            let explicit_ret = body.iter().find_map(|s| {
                                if let Statement::Return { value: Some(v), .. } = s {
                                    Some(self.infer(v, None))
                                } else {
                                    None
                                }
                            });

                            explicit_ret
                                .or_else(|| {
                                    body.last().and_then(|s| match s {
                                        Statement::Expression { expression, .. } => {
                                            let ty = self.infer(expression, None);
                                            if ty != Ty::Error && ty != Ty::Infer {
                                                Some(ty)
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    })
                                })
                                .unwrap_or(Ty::Unit)
                        }
                    }
                };

                let saved_ret = self.current_ret.clone();
                self.current_ret = ret.clone();

                let typed_body = body.iter().map(|s| self.check_stmt(s)).collect();

                self.env.pop();

                self.current_ret = saved_ret;

                TypedExpr {
                    kind: TypedExprKind::Lambda {
                        params: params.clone(),
                        return_ty: Some(ret.clone()),
                        body: typed_body,
                    },
                    ty: Ty::Fn {
                        params: param_tys,
                        ret: Box::new(ret.clone()),
                    },
                    span: *span,
                }
            }
            Expression::StructLiteral { name, fields, span } => {
                if !self.structs.contains_key(name) {
                    self.errors.push(TypeError::UnknownName {
                        name: name.clone(),
                        span: *span,
                    });
                    return TypedExpr {
                        kind: TypedExprKind::StructLiteral {
                            name: name.clone(),
                            fields: fields
                                .iter()
                                .map(|f| TypedStructLiteralField {
                                    field_name: f.field_name.clone(),
                                    field_val: Box::new(
                                        self.check_expr(f.field_val.as_ref(), None),
                                    ),
                                })
                                .collect(),
                        },
                        ty: Ty::Error,
                        span: *span,
                    };
                }

                if fields.is_empty() {
                    return TypedExpr {
                        kind: TypedExprKind::StructLiteral {
                            name: name.clone(),
                            fields: fields
                                .iter()
                                .map(|f| TypedStructLiteralField {
                                    field_name: f.field_name.clone(),
                                    field_val: Box::new(
                                        self.check_expr(f.field_val.as_ref(), None),
                                    ),
                                })
                                .collect(),
                        },
                        ty: Ty::Struct(name.clone()),
                        span: *span,
                    };
                }

                for f in fields {
                    match self.infer(f.field_val.as_ref(), None) {
                        Ty::Error => {
                            return TypedExpr {
                                kind: TypedExprKind::StructLiteral {
                                    name: name.clone(),
                                    fields: fields
                                        .iter()
                                        .map(|f| TypedStructLiteralField {
                                            field_name: f.field_name.clone(),
                                            field_val: Box::new(
                                                self.check_expr(f.field_val.as_ref(), None),
                                            ),
                                        })
                                        .collect(),
                                },
                                ty: Ty::Error,
                                span: *span,
                            };
                        }
                        _ => continue,
                    }
                }

                TypedExpr {
                    kind: TypedExprKind::StructLiteral {
                        name: name.clone(),
                        fields: fields
                            .iter()
                            .map(|f| TypedStructLiteralField {
                                field_name: f.field_name.clone(),
                                field_val: Box::new(self.check_expr(f.field_val.as_ref(), None)),
                            })
                            .collect(),
                    },
                    ty: Ty::Struct(name.clone()),
                    span: *span,
                }
            }
            Expression::Field { object, name, span } => {
                if let Expression::Identifier {
                    name: obj_name,
                    span: obj_span,
                } = object.as_ref()
                    && let Some((var_name, None)) = self.variant_cases.get(name)
                    && var_name == obj_name
                {
                    return TypedExpr {
                        kind: TypedExprKind::Field {
                            object: Box::new(TypedExpr {
                                kind: TypedExprKind::Identifier {
                                    name: obj_name.clone(),
                                    is_ref: false,
                                },
                                ty: Ty::Struct(var_name.clone()),
                                span: *obj_span,
                            }),
                            name: name.clone(),
                        },
                        ty: Ty::Struct(var_name.clone()),
                        span: *span,
                    };
                }

                let obj = self.check_expr(object.as_ref(), expected);

                if name == "len" && matches!(obj.ty, Ty::Array(_) | Ty::String) {
                    return TypedExpr {
                        kind: TypedExprKind::Field {
                            object: Box::new(obj),
                            name: name.clone(),
                        },
                        ty: Ty::Int,
                        span: *span,
                    };
                }

                if let Expression::Identifier { name: obj_name, .. } = object.as_ref()
                    && let Some((var_name, None)) = self.variant_cases.get(name)
                    && var_name == obj_name
                {
                    return TypedExpr {
                        kind: TypedExprKind::Field {
                            object: Box::new(obj),
                            name: name.clone(),
                        },
                        ty: Ty::Struct(var_name.clone()),
                        span: *span,
                    };
                }

                let res_ty = match obj.ty {
                    Ty::Struct(s) => match self.structs.get(&s) {
                        Some(struct_sig) => {
                            match struct_sig
                                .iter()
                                .find(|(curr_name, _, _)| curr_name == name)
                            {
                                Some(field) => field.1.clone(),
                                None => {
                                    self.errors.push(TypeError::UnknownField {
                                        struct_name: s,
                                        field: name.clone(),
                                        span: *span,
                                    });
                                    Ty::Error
                                }
                            }
                        }
                        None => {
                            self.errors.push(TypeError::UnknownName {
                                name: name.clone(),
                                span: *span,
                            });
                            Ty::Error
                        }
                    },
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::NoFields {
                            ty: obj.ty,
                            span: *span,
                        });
                        Ty::Error
                    }
                };

                TypedExpr {
                    kind: TypedExprKind::Field {
                        object: Box::new(self.check_expr(object.as_ref(), expected)),
                        name: name.clone(),
                    },
                    ty: res_ty,
                    span: *span,
                }
            }
            Expression::MethodCall {
                object,
                name,
                args,
                span,
            } => {
                if let Expression::Identifier {
                    name: obj_name,
                    span: obj_span,
                    ..
                } = object.as_ref()
                    && let Some((var_name, payload)) = self.variant_cases.clone().get(name)
                    && var_name == obj_name
                {
                    let mut typed_args = Vec::new();

                    if let Some(payload_ty) = payload {
                        if args.len() != 1 {
                            self.errors.push(TypeError::ArityMismatch {
                                name: var_name.clone(),
                                expected: 1,
                                found: args.len(),
                                span: *span,
                            });
                        }
                        if let Some(arg) = args.first() {
                            let a = self.check_expr(&arg.value, Some(payload_ty));

                            self.expect(&a.ty, payload_ty, arg.span);

                            typed_args.push(a);
                        }
                    }

                    let typed_obj = TypedExpr {
                        kind: TypedExprKind::Identifier {
                            name: obj_name.clone(),
                            is_ref: false,
                        },
                        ty: Ty::Struct(obj_name.clone()),
                        span: *obj_span,
                    };

                    return TypedExpr {
                        kind: TypedExprKind::MethodCall {
                            object: Box::new(typed_obj),
                            args: typed_args,
                            name: name.clone(),
                        },
                        ty: Ty::Struct(var_name.clone()),
                        span: *span,
                    };
                }

                let mut found_sig = None;

                let mut is_static = false;

                let mut obj_ty = Ty::Error;

                if let Expression::Identifier { name: obj_name, .. } = object.as_ref() {
                    if self.structs.contains_key(obj_name) {
                        let sig_name = format!("{}.{}", obj_name, name);
                        found_sig = self.funcs.get(&sig_name).cloned();
                        is_static = true;
                    } else if self.env.lookup(obj_name).is_none() {
                        found_sig = self.funcs.get(name).cloned();
                        if found_sig.is_some() {
                            is_static = true;
                        }
                    }
                }

                if !is_static {
                    obj_ty = self.infer(object.as_ref(), None);
                    let sig_name = match &obj_ty {
                        Ty::Struct(s) => format!("{}.{}", s, name),
                        Ty::Int => format!("int.{}", name),
                        Ty::Float => format!("float64.{}", name),
                        Ty::String => format!("string.{}", name),
                        Ty::Char => format!("char.{}", name),
                        Ty::Bool => format!("bool.{}", name),
                        Ty::Array(inner) => {
                            format!("Array_{}.{}", self.type_to_prefix(inner), name)
                        }
                        Ty::Error => {
                            return TypedExpr {
                                kind: TypedExprKind::MethodCall {
                                    object: Box::new(self.check_expr(object.as_ref(), expected)),
                                    args: args
                                        .iter()
                                        .map(|a| self.check_expr(&a.value, None))
                                        .collect(),
                                    name: name.clone(),
                                },
                                ty: Ty::Error,
                                span: *span,
                            };
                        }
                        _ => {
                            self.errors.push(TypeError::NoSuchMethod {
                                ty: obj_ty.clone(),
                                method: name.clone(),
                                span: *span,
                            });
                            return TypedExpr {
                                kind: TypedExprKind::MethodCall {
                                    object: Box::new(self.check_expr(object.as_ref(), expected)),
                                    args: args
                                        .iter()
                                        .map(|a| self.check_expr(&a.value, None))
                                        .collect(),
                                    name: name.clone(),
                                },
                                ty: Ty::Error,
                                span: *span,
                            };
                        }
                    };
                    found_sig = self.funcs.get(&sig_name).cloned();
                }

                if let Some(sig) = found_sig {
                    let expected_args = if is_static {
                        &sig.params[..]
                    } else {
                        if sig.params.is_empty() {
                            self.errors.push(TypeError::ArityMismatch {
                                name: name.clone(),
                                expected: 1,
                                found: 0,
                                span: *span,
                            });
                            return TypedExpr {
                                kind: TypedExprKind::MethodCall {
                                    object: Box::new(self.check_expr(object.as_ref(), expected)),
                                    args: args
                                        .iter()
                                        .map(|a| self.check_expr(&a.value, None))
                                        .collect(),
                                    name: name.clone(),
                                },
                                ty: Ty::Error,
                                span: *span,
                            };
                        }
                        self.expect(&obj_ty, &sig.params[0].ty, object.span());

                        &sig.params[1..]
                    };

                    if args.len() != expected_args.len() {
                        self.errors.push(TypeError::ArityMismatch {
                            name: name.clone(),
                            expected: expected_args.len(),
                            found: args.len(),
                            span: *span,
                        });
                        return TypedExpr {
                            kind: TypedExprKind::MethodCall {
                                object: Box::new(self.check_expr(object.as_ref(), expected)),
                                args: args
                                    .iter()
                                    .map(|a| self.check_expr(&a.value, None))
                                    .collect(),
                                name: name.clone(),
                            },
                            ty: sig.ret,
                            span: *span,
                        };
                    }

                    let typed_object = if is_static {
                        if let Expression::Identifier {
                            name: obj_name,
                            span: obj_span,
                        } = object.as_ref()
                        {
                            TypedExpr {
                                kind: TypedExprKind::Identifier {
                                    name: obj_name.clone(),
                                    is_ref: false,
                                },
                                ty: Ty::Struct(obj_name.clone()),
                                span: *obj_span,
                            }
                        } else {
                            self.check_expr(object.as_ref(), None)
                        }
                    } else {
                        self.check_expr(object.as_ref(), None)
                    };

                    let typed_args = self.resolve_args(name.clone(), expected_args, args, *span);

                    TypedExpr {
                        kind: TypedExprKind::MethodCall {
                            object: Box::new(typed_object),
                            args: typed_args,
                            name: name.clone(),
                        },
                        ty: sig.ret,
                        span: *span,
                    }
                } else {
                    if is_static {
                        self.errors.push(TypeError::UnknownName {
                            name: name.clone(),
                            span: *span,
                        });
                    } else {
                        if let Some(global_sig) = self.funcs.get(name)
                            && !global_sig.params.is_empty()
                            && global_sig.params[0].ty == obj_ty
                        {
                            self.errors.push(TypeError::MethodFoundAsGlobal {
                                ty: obj_ty.clone(),
                                method: name.clone(),
                                span: *span,
                                help: format!(
                                    "there is a global function `{}`, did you want to declare it in `extend {} {{ ... }}`?",
                                    name,
                                    match obj_ty {
                                        Ty::Struct(ref s_name) => s_name,
                                        _ => "Type"
                                    }),
                            });
                            return TypedExpr {
                                kind: TypedExprKind::MethodCall {
                                    object: Box::new(self.check_expr(object.as_ref(), expected)),
                                    args: args
                                        .iter()
                                        .map(|a| self.check_expr(&a.value, None))
                                        .collect(),
                                    name: name.clone(),
                                },
                                ty: Ty::Error,
                                span: *span,
                            };
                        }
                        self.errors.push(TypeError::NoSuchMethod {
                            ty: obj_ty,
                            method: name.clone(),
                            span: *span,
                        });
                    }

                    TypedExpr {
                        kind: TypedExprKind::MethodCall {
                            object: Box::new(self.check_expr(object.as_ref(), expected)),
                            args: args
                                .iter()
                                .map(|a| self.check_expr(&a.value, None))
                                .collect(),
                            name: name.clone(),
                        },
                        ty: Ty::Error,
                        span: *span,
                    }
                }
            }
            Expression::If {
                condition,
                then_block,
                else_if,
                else_block,
                span,
                closing_span,
            } => {
                let typed_cond = self.check_expr(condition.as_ref(), Some(&Ty::Bool));
                self.expect(&typed_cond.ty, &Ty::Bool, *span);

                let then_ty = if let Some((last, _)) = then_block.split_last()
                    && let Statement::Expression { expression, .. } = last
                {
                    self.infer(expression, expected)
                } else {
                    Ty::Unit
                };

                if !matches!(then_ty, Ty::Unit) && else_block.is_empty() {
                    self.errors.push(TypeError::IfNoElse {
                        if_span: *span,
                        closing_span: closing_span.unwrap_or(Span::default()),
                    });

                    return TypedExpr {
                        kind: TypedExprKind::Error {
                            error: TypeError::IfNoElse {
                                if_span: *span,
                                closing_span: closing_span.unwrap_or(Span::default()),
                            },
                        },
                        ty: Ty::Error,
                        span: *span,
                    };
                }

                TypedExpr {
                    kind: TypedExprKind::If {
                        condition: Box::new(typed_cond),
                        then_block: then_block.iter().map(|s| self.check_stmt(s)).collect(),
                        else_if: else_if
                            .iter()
                            .map(|elif| TypedElseIf {
                                condition: self.check_expr(&elif.condition, None),
                                block: self.check_block(&elif.block),
                            })
                            .collect(),
                        else_block: else_block.iter().map(|s| self.check_stmt(s)).collect(),
                        closing_span: *closing_span,
                    },
                    ty: then_ty,
                    span: *span,
                }
            }
            Expression::Match { target, arms, span } => {
                let typed_target = self.check_expr(target.as_ref(), None);
                let target_ty = self.infer(target.as_ref(), None);

                if target_ty == Ty::Error {
                    return TypedExpr {
                        kind: TypedExprKind::Match {
                            target: Box::new(typed_target),
                            arms: arms
                                .iter()
                                .map(|a| TypedMatchArm {
                                    pattern: self.check_expr(&a.pattern, None),
                                    body: self.check_expr(&a.body, None),
                                })
                                .collect(),
                        },
                        ty: match arms.first() {
                            Some(arm) => self.infer(&arm.body, None),
                            None => Ty::Unit,
                        },
                        span: *span,
                    };
                }

                let target_variant_name = match &target_ty {
                    Ty::Struct(name) => name.clone(),
                    _ => {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!("Cannot match on non-variant type {}", target_ty),
                            span: *span,
                        });
                        return TypedExpr::error(
                            *span,
                            TypeError::Unsupported {
                                desc: format!("Cannot match on non-variant type {}", target_ty),
                                span: *span,
                            },
                        );
                    }
                };

                let mut match_ret_ty = expected.cloned();

                let mut must_be_covered = HashSet::new();
                let mut has_been_covered: HashSet<String> = HashSet::new();

                let mut has_wildcard = false;

                for (case_name, variant) in self.variant_cases.clone() {
                    if variant.0 == target_variant_name {
                        must_be_covered.insert(case_name);
                    }
                }

                let mut typed_arms = Vec::new();

                for arm in arms {
                    self.env.push();

                    let pat_name: String;
                    let is_wild: bool;

                    (pat_name, is_wild) = self.extract_pattern_case(&arm.pattern);

                    has_wildcard |= is_wild;

                    if pat_name != "_" {
                        has_been_covered.insert(pat_name);
                    }

                    let mut payload_var_info = None;

                    match &arm.pattern {
                        Expression::Identifier {
                            name,
                            span: pat_span,
                        } => {
                            if let Some((var_name, expected_payload)) = self.variant_cases.get(name)
                            {
                                if var_name != &target_variant_name {
                                    self.errors.push(TypeError::Mismatch {
                                        expected: target_ty.clone(),
                                        found: Ty::Struct(var_name.clone()),
                                        span: *pat_span,
                                    });
                                } else if expected_payload.is_some() {
                                    self.errors.push(TypeError::Unsupported {
                                        desc: format!(
                                            "Variant case `{}` expecting a payload without it",
                                            name
                                        ),
                                        span: *pat_span,
                                    });
                                } else if name != "_" {
                                    self.errors.push(TypeError::UnknownName {
                                        name: name.clone(),
                                        span: *pat_span,
                                    });
                                }
                            }
                        }

                        Expression::Field {
                            object,
                            name,
                            span: pat_span,
                        } => {
                            if let Expression::Identifier { name: obj_name, .. } = object.as_ref() {
                                if let Some((var_name, expected_payload)) =
                                    self.variant_cases.get(name)
                                {
                                    if var_name != obj_name || var_name != &target_variant_name {
                                        self.errors.push(TypeError::Mismatch {
                                            expected: target_ty.clone(),
                                            found: Ty::Struct(var_name.clone()),
                                            span: *pat_span,
                                        });
                                    } else if expected_payload.is_some() {
                                        self.errors.push(TypeError::Unsupported {
                                            desc: format!(
                                                "Variant case `{}` expecting a payload without it",
                                                name
                                            ),
                                            span: *pat_span,
                                        });
                                    }
                                } else {
                                    self.errors.push(TypeError::UnknownName {
                                        name: name.clone(),
                                        span: *pat_span,
                                    });
                                }
                            } else {
                                self.errors.push(TypeError::Unsupported {
                                    desc: "Invalid pattern expression".to_string(),
                                    span: *pat_span,
                                });
                            }
                        }

                        Expression::Call {
                            function,
                            args,
                            span: pat_span,
                        } => {
                            if let Expression::Identifier { name, .. } = function.as_ref() {
                                if let Some((var_name, expected_payload)) =
                                    self.variant_cases.get(name).cloned()
                                {
                                    if var_name != target_variant_name {
                                        self.errors.push(TypeError::Mismatch {
                                            expected: target_ty.clone(),
                                            found: Ty::Struct(var_name),
                                            span: *pat_span,
                                        });
                                    } else if let Some(payload_ty) = expected_payload {
                                        if args.len() != 1 {
                                            self.errors.push(TypeError::ArityMismatch {
                                                name: name.clone(),
                                                expected: 1,
                                                found: args.len(),
                                                span: *pat_span,
                                            });
                                        } else if let Expression::Identifier {
                                            name: bind_name,
                                            span: bind_span,
                                        } = &args[0].value
                                        {
                                            self.defined(
                                                bind_name.clone(),
                                                payload_ty.clone(),
                                                BindingKind::Let,
                                                *bind_span,
                                            );

                                            payload_var_info = Some((
                                                name.clone(),
                                                bind_name.clone(),
                                                payload_ty,
                                                *bind_span,
                                                *pat_span,
                                            ));
                                        } else {
                                            self.errors.push(TypeError::Unsupported {
                                                desc: "Pattern payload not as identifier"
                                                    .to_string(),
                                                span: args[0].span,
                                            });
                                        }
                                    } else {
                                        self.errors.push(TypeError::Unsupported {
                                            desc: "Invalid pattern shape".to_string(),
                                            span: *pat_span,
                                        });
                                    }
                                } else {
                                    self.errors.push(TypeError::UnknownName {
                                        name: name.clone(),
                                        span: *pat_span,
                                    })
                                }
                            }
                        }

                        Expression::MethodCall {
                            object,
                            name,
                            args,
                            span: pat_span,
                        } => {
                            if let Expression::Identifier { name: obj_name, .. } = object.as_ref() {
                                if let Some((var_name, expected_payload)) =
                                    self.variant_cases.clone().get(name).cloned()
                                {
                                    if var_name != *obj_name || var_name != target_variant_name {
                                        self.errors.push(TypeError::Mismatch {
                                            expected: target_ty.clone(),
                                            found: Ty::Struct(var_name),
                                            span: *pat_span,
                                        });
                                    } else if let Some(payload_ty) = expected_payload {
                                        if args.len() != 1 {
                                            self.errors.push(TypeError::ArityMismatch {
                                                name: name.clone(),
                                                expected: 1,
                                                found: args.len(),
                                                span: *pat_span,
                                            });
                                        } else if let Expression::Identifier {
                                            name: bind_name,
                                            span: bind_span,
                                        } = &args[0].value
                                        {
                                            self.defined(
                                                bind_name.clone(),
                                                payload_ty,
                                                BindingKind::Let,
                                                *bind_span,
                                            )
                                        } else {
                                            self.errors.push(TypeError::Unsupported {
                                                desc: "Pattern payload not as identifier"
                                                    .to_string(),
                                                span: args[0].span,
                                            });
                                        }
                                    } else {
                                        self.errors.push(TypeError::Unsupported {
                                            desc: format!(
                                                "Variant case `{}` does not take a payload",
                                                name
                                            ),
                                            span: *pat_span,
                                        });
                                    }
                                } else {
                                    self.errors.push(TypeError::UnknownName {
                                        name: name.clone(),
                                        span: *pat_span,
                                    });
                                }
                            } else {
                                self.errors.push(TypeError::Unsupported {
                                    desc: "Invalid pattern shape".to_string(),
                                    span: *pat_span,
                                });
                            }
                        }

                        _ => self.errors.push(TypeError::Unsupported {
                            desc: "Invalid pattern expression".to_string(),
                            span: arm.pattern.span(),
                        }),
                    }

                    let typed_pattern = if let Some((case_name, bind_name, p_ty, b_span, p_span)) =
                        payload_var_info
                    {
                        TypedExpr {
                            kind: TypedExprKind::Call {
                                function: Box::new(TypedExpr {
                                    kind: TypedExprKind::Identifier {
                                        name: case_name,
                                        is_ref: false,
                                    },
                                    ty: Ty::Struct(target_variant_name.clone()),
                                    span: p_span,
                                }),
                                args: vec![TypedExpr {
                                    kind: TypedExprKind::Identifier {
                                        name: bind_name,
                                        is_ref: false,
                                    },
                                    ty: p_ty,
                                    span: b_span,
                                }],
                            },
                            ty: Ty::Struct(target_variant_name.clone()),
                            span: p_span,
                        }
                    } else {
                        TypedExpr {
                            kind: match &arm.pattern {
                                Expression::Identifier { name, .. } => TypedExprKind::Identifier {
                                    name: name.clone(),
                                    is_ref: false,
                                },
                                _ => TypedExprKind::Identifier {
                                    name: "_".to_string(),
                                    is_ref: false,
                                },
                            },
                            ty: target_ty.clone(),
                            span: arm.pattern.span(),
                        }
                    };

                    let typed_body = self.check_expr(&arm.body, match_ret_ty.as_ref());

                    if let Some(ref expected) = match_ret_ty {
                        self.expect(&typed_body.ty, expected, arm.body.span());
                    } else {
                        match_ret_ty = Some(typed_body.ty.clone());
                    }

                    typed_arms.push(TypedMatchArm {
                        pattern: typed_pattern,
                        body: typed_body,
                    });

                    self.env.pop()
                }

                if !has_wildcard {
                    let missing: Vec<&String> =
                        must_be_covered.difference(&has_been_covered).collect();

                    if !missing.is_empty() {
                        let missed_str = missing
                            .into_iter()
                            .cloned()
                            .collect::<Vec<String>>()
                            .join(", ");

                        self.errors.push(TypeError::NotFullMatch {
                            target_name: target_variant_name.clone(),
                            missed_patterns: missed_str,
                            span: target.span(),
                        })
                    }
                }

                TypedExpr {
                    kind: TypedExprKind::Match {
                        target: Box::new(typed_target),
                        arms: typed_arms,
                    },
                    ty: match_ret_ty.unwrap_or(Ty::Unit),
                    span: *span,
                }
            }
            Expression::Block { body, span } => {
                self.env.push();

                let mut last_ty = Ty::Unit;

                for stmt in body {
                    match stmt {
                        Statement::Expression { expression, .. } => {
                            last_ty = self.infer(expression, None);
                        }
                        _ => {
                            self.check_stmt(stmt);
                        }
                    }
                }

                self.env.pop();

                TypedExpr {
                    kind: TypedExprKind::Block {
                        body: body.iter().map(|s| self.check_stmt(s)).collect(),
                    },
                    ty: last_ty,
                    span: *span,
                }
            }
        }
    }

    pub fn define_builtins(&mut self) {
        self.defined(
            "print".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![
                    Ty::Int,
                    Ty::Float,
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
                    Ty::Int,
                    Ty::Float,
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
                | Statement::Break { .. }
                | Statement::Continue { .. }
                | Statement::ExternFunc { .. }
                | Statement::Extend { .. }
                | Statement::Variant { .. }
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

    fn extract_pattern_case(&mut self, pattern: &Expression) -> (String, bool) {
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

    fn type_to_prefix<'a>(&self, ty: &'a Ty) -> Cow<'a, str> {
        match ty {
            Ty::Int => Cow::from("int"),
            Ty::Float => Cow::from("float64"),
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
}
