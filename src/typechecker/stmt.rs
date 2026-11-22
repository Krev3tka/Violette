use crate::hir::typed_ast::{TypedExpr, TypedExprKind, TypedStmt, TypedStmtKind};
use crate::lexer::token::Token;
use crate::parser::statement::FuncParam;
use crate::parser::types::BindingPattern;
use crate::parser::{Expression, Statement};
use crate::typechecker::checker::{Checker, FnSig, ParamSig, TypeError};
use crate::typechecker::error::{BindingKind, LoopControlKind};
use crate::typechecker::types::Ty;

impl Checker {
    pub fn check_fn(&mut self, stmt: &Statement) -> TypedStmt {
        self.env.push();

        if let Statement::Func {
            name,
            generic_params,
            params,
            return_type,
            body,
            span,
            ..
        } = stmt
        {
            let saved_generics =
                std::mem::replace(&mut self.current_generics, generic_params.clone());

            self.current_ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));
            for p in params {
                self.current_type_span = Some(p.span);

                let mut ty = self.resolve(&p.param_type);

                self.current_type_span = None;

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

            self.current_generics = saved_generics;

            TypedStmt {
                kind: TypedStmtKind::Func {
                    name: name.clone(),
                    generic_params: generic_params.clone(),
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
                self.current_type_span = Some(p.span);

                let mut ty = self.resolve(&p.param_type);

                self.current_type_span = None;

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
                    generic_params: vec![],
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

        let prev_loop = std::mem::replace(&mut self.in_loop, true);

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

        self.in_loop = prev_loop;

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
                        value: typed_val,
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
                        BindingKind::Let,
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
                                        BindingKind::Let,
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
                let val = value.as_ref().map(|expr| self.check_expr(expr, None));

                let cur_ref = &self.current_ret.clone();

                self.expect(
                    &if let Some(v) = val.clone() {
                        v.ty
                    } else {
                        Ty::Unit
                    },
                    cur_ref,
                    stmt.span(),
                );

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
                    generic_params: vec![],
                    params: params
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
                        Expression::Index { left: arr_expr, .. } => {
                            if let Expression::Identifier { name, .. } = arr_expr.as_ref()
                                && let Some(entity) = self.env.lookup(name)
                                && !entity.kind.is_mutable()
                            {
                                self.errors.push(TypeError::AssignmentToImmutable {
                                    name: name.clone(),
                                    kind: entity.kind,
                                    assign_span: left.span(),
                                    decl_span: entity.span,
                                });
                            }

                            self.expect(&typed_right.ty, &typed_left.ty, right.span())
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
                                expected: Box::new(target.clone()),
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
        if let Statement::Struct {
            name,
            generic_params,
            fields,
            ..
        } = stmt
        {
            let saved_generics =
                std::mem::replace(&mut self.current_generics, generic_params.clone());

            self.env.push();

            for f in fields {
                self.current_type_span = Some(f.span);

                let ty = self.resolve(&f.param_type);

                self.current_type_span = None;

                self.defined(f.name.clone(), ty, BindingKind::Var, stmt.span())
            }

            self.env.pop();

            self.current_generics = saved_generics;

            TypedStmt {
                kind: TypedStmtKind::Struct {
                    name: name.clone(),
                    generic_params: generic_params.clone(),
                    fields: fields.clone(),
                },
                ty: Ty::Unit,
                span: stmt.span(),
            }
        } else {
            TypedStmt::error(stmt.span(), TypeError::Unreachable { span: stmt.span() })
        }
    }
}
