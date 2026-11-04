use crate::hir::typed_ast::{
    TypedElseIf, TypedExpr, TypedExprKind, TypedMatchArm, TypedStmtKind, TypedStructLiteralField,
};
use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::statement::FuncParam;
use crate::parser::types::Type;
use crate::parser::{Expression, Statement};
use crate::typechecker::checker::{Checker, TypeError};
use crate::typechecker::error::BindingKind;
use crate::typechecker::types::Ty;
use std::collections::HashSet;

impl Checker {
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
            Expression::NumberedParam { index, span } => {
                let arg_name = format!("_arg_{}", index);

                let (ty, is_ref) = match self.env.lookup(&arg_name) {
                    Some(entity) => (entity.ty.clone(), false),
                    None => {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!(
                                "Numbered parameter `#{}` used outside of lambda block",
                                index
                            ),
                            span: *span,
                        });
                        (Ty::Error, false)
                    }
                };

                TypedExpr {
                    kind: TypedExprKind::Identifier {
                        name: arg_name,
                        is_ref,
                    },
                    ty,
                    span: *span,
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
                // First check: Enum constructor call (e.g. Status.Ok("done"))
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

                // Second check: Is it static?

                let mut found_sig = None;

                let mut is_static = false;

                let mut obj_ty = Ty::Error;

                // Is there struct name at left?
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

                // If it isn't a type, it's an object
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

                // Third check: checking receiver and args
                if let Some(sig) = found_sig {
                    if !is_static && !sig.is_dynamic_method {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!(
                                "Function: `{}` has no receiver and cannot be called as a method. Declare it as `func (obj_name: Self) {}` or call it ad `{}.{}()` pls",
                                name, name, self.type_to_prefix(&obj_ty), name
                            ),
                            span: *span,
                        });
                        return TypedExpr::error(
                            *span,
                            TypeError::Unsupported {
                                desc: format!(
                                    "Function: `{}` has no receiver and cannot be called as a method. Declare it as `func (obj_name: Self) {}` or call it ad `{}.{}()` pls",
                                    name,
                                    name,
                                    self.type_to_prefix(&obj_ty),
                                    name
                                ),
                                span: *span,
                            },
                        );
                    }

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
                            return TypedExpr::error(
                                *span,
                                TypeError::NoSuchMethod {
                                    ty: obj_ty,
                                    method: name.clone(),
                                    span: *span,
                                },
                            );
                        }
                        self.expect(&obj_ty, &sig.params[0].ty, object.span());

                        &sig.params[1..]
                    };

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
                    // Last check: Didn't find anything, error hints formatting
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
                            return TypedExpr::error(
                                *span,
                                TypeError::NoSuchMethod {
                                    ty: obj_ty,
                                    method: name.clone(),
                                    span: *span,
                                },
                            );
                        }
                        self.errors.push(TypeError::NoSuchMethod {
                            ty: obj_ty.clone(),
                            method: name.clone(),
                            span: *span,
                        });
                    }

                    TypedExpr::error(
                        *span,
                        TypeError::NoSuchMethod {
                            ty: obj_ty,
                            method: name.clone(),
                            span: *span,
                        },
                    )
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
                // 1. lambdas
                let numbered_params = self.collect_numbered_params(body);

                if !numbered_params.is_empty() {
                    let max_idx = *numbered_params.iter().max().unwrap();

                    let (expected_fn_params, expected_ret) = match expected {
                        Some(Ty::Fn { params, ret }) => (Some(params), Some(ret.as_ref())),
                        _ => (None, None),
                    };

                    let mut params = Vec::new();
                    let mut param_tys = Vec::new();

                    self.env.push();

                    for i in 0..=max_idx {
                        let mut ty = if let Some(expected_p) =
                            expected_fn_params.and_then(|ps| ps.get(i as usize))
                        {
                            expected_p.clone()
                        } else {
                            Ty::Infer
                        };

                        if ty == Ty::Infer {
                            self.errors.push(TypeError::Unsupported {
                                desc: "Cannot infer type for parameter (try to pass lambda directly to a typed function or add type annotation pls)".to_string(),
                                span: *span,
                            });
                            ty = Ty::Error;
                        }

                        let arg_name = format!("_arg_{}", i);
                        self.defined(arg_name.clone(), ty.clone(), BindingKind::Let, *span);

                        params.push(FuncParam {
                            name: arg_name,
                            annotated_name: String::new(),
                            param_type: Type::Infer,
                            span: *span,
                            kind: BindingKind::Param,
                            is_ref: false,
                        });
                        param_tys.push(ty);
                    }

                    let ret = if let Some(exp_ret) = expected_ret {
                        exp_ret.clone()
                    } else {
                        let explicit_ret = body.iter().find_map(|s| {
                            if let Statement::Return { value: Some(v), .. } = s {
                                Some(self.infer(v, None))
                            } else {
                                None
                            }
                        });

                        explicit_ret.unwrap_or(Ty::Infer)
                    };

                    let saved_ret = self.current_ret.clone();

                    self.current_ret = ret.clone();

                    let mut last_ty = Ty::Unit;
                    let mut typed_body = Vec::new();

                    for stmt in body {
                        let typed_stmt = self.check_stmt(stmt);

                        if let TypedStmtKind::Expression(ref expr) = typed_stmt.kind {
                            last_ty = expr.ty.clone();
                        }

                        typed_body.push(typed_stmt);
                    }

                    self.current_ret = saved_ret;

                    let final_ret = if ret == Ty::Infer { last_ty } else { ret };

                    self.env.pop();

                    return TypedExpr {
                        kind: TypedExprKind::Lambda {
                            params,
                            return_ty: Some(final_ret.clone()),
                            body: typed_body,
                        },
                        ty: Ty::Fn {
                            params: param_tys,
                            ret: Box::new(final_ret.clone()),
                        },
                        span: *span,
                    };
                }

                // 2. regular blocks
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

    #[allow(dead_code)]
    pub fn collect_expr_params(&self, expr: &Expression, set: &mut HashSet<u8>) {
        match expr {
            Expression::NumberedParam { index, .. } => {
                set.insert(*index);
            }
            Expression::Infix { left, right, .. } => {
                self.collect_expr_params(left.as_ref(), set);
                self.collect_expr_params(right.as_ref(), set);
            }
            Expression::Prefix { right, .. } | Expression::Postfix { left: right, .. } => {
                self.collect_expr_params(right, set);
            }
            Expression::ArrayLiteral { values, .. } => {
                for val in values {
                    self.collect_expr_params(val, set);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.collect_expr_params(field.field_val.as_ref(), set)
                }
            }
            Expression::Index { left, index, .. } => {
                self.collect_expr_params(left.as_ref(), set);
                self.collect_expr_params(index.as_ref(), set);
            }
            Expression::Call { function, args, .. } => {
                self.collect_expr_params(function.as_ref(), set);

                for arg in args {
                    self.collect_expr_params(&arg.value, set)
                }
            }
            Expression::MethodCall { object, args, .. } => {
                self.collect_expr_params(object.as_ref(), set);

                for arg in args {
                    self.collect_expr_params(&arg.value, set)
                }
            }
            Expression::Match { target, .. } => {
                self.collect_expr_params(target.as_ref(), set);
            }
            Expression::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_expr_params(s.as_ref(), set);
                }

                if let Some(e) = end {
                    self.collect_expr_params(e.as_ref(), set);
                }
            }
            Expression::If {
                condition,
                then_block,
                else_if,
                else_block,
                ..
            } => {
                self.collect_expr_params(condition.as_ref(), set);

                for s in then_block {
                    self.collect_stmt_params(s, set);
                }

                for elif in else_if {
                    self.collect_expr_params(&elif.condition, set);

                    for s in &elif.block {
                        self.collect_stmt_params(s, set);
                    }
                }

                for s in else_block {
                    self.collect_stmt_params(s, set);
                }
            }
            Expression::Block { .. } | Expression::Lambda { .. } => {}
            _ => {}
        }
    }
}
