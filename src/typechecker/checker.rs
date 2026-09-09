use crate::lexer::span::Span;
use crate::lexer::token::{PrimitiveType, Token};
use crate::parser::program::Program;
use crate::parser::statement::IfStatement;
use crate::parser::types::Type;
use crate::parser::{Expression, Statement};
use crate::typechecker::env::Env;
pub use crate::typechecker::error::TypeError;
use crate::typechecker::error::TypeError::ConflictingEntryPoint;
use crate::typechecker::error::{BindingKind, DefinitionKind, LoopControlKind};
use crate::typechecker::types::Ty;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FnSig {
    params: Vec<Ty>,
    ret: Ty,
    span: Span,
}

pub type StructSig = Vec<(String, Ty, Span)>;

#[derive(Default)]
pub struct Checker {
    pub funcs: HashMap<String, FnSig>,
    pub structs: HashMap<String, StructSig>,
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

                    let params: Vec<_> =
                        params.iter().map(|p| self.resolve(&p.param_type)).collect();

                    let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                    let f = Ty::Fn {
                        params: params.clone(),
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
                    let params: Vec<_> =
                        params.iter().map(|p| self.resolve(&p.param_type)).collect();
                    let ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                    self.defined(
                        name.clone(),
                        Ty::Fn {
                            params: params.clone(),
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

                    match target_ty {
                        Ty::Struct(struct_name) => {
                            if !self.structs.contains_key(&struct_name) {
                                self.errors.push(TypeError::UnknownName {
                                    name: struct_name.clone(),
                                    span: *span,
                                });
                                continue;
                            }

                            for method in methods {
                                if let Statement::Func {
                                    name,
                                    params,
                                    return_type,
                                    span,
                                    ..
                                } = method
                                {
                                    let full_name = format!("{}.{}", struct_name, name);

                                    let params_ty: Vec<_> = params
                                        .iter()
                                        .map(|p| self.resolve(&p.param_type))
                                        .collect();

                                    let ret =
                                        return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));

                                    if self.funcs.contains_key(&full_name) {
                                        self.errors.push(TypeError::DuplicateDefinition {
                                            name: full_name.clone(),
                                            first_span: match self.funcs.get(name) {
                                                Some(f) => f.span,
                                                None => unreachable!(),
                                            },
                                            second_span: stmt.span(),
                                            def_kind: DefinitionKind::Fun,
                                        });
                                        continue;
                                    }

                                    self.funcs.insert(
                                        full_name.clone(),
                                        FnSig {
                                            params: params_ty,
                                            ret,
                                            span: *span,
                                        },
                                    );
                                }
                            }
                        }
                        _ => {
                            self.errors.push(TypeError::Unsupported {
                                desc: format!("Cannot extend non-struct type {:?}", target_ty),
                                span: *span,
                            });
                        }
                    }
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
            Type::Infer => Ty::Infer
        }
    }

    pub fn check_fn(&mut self, stmt: &Statement) {
        self.env.push();

        if let Statement::Func {
            params,
            return_type,
            body,
            ..
        } = stmt
        {
            self.current_ret = return_type.as_ref().map_or(Ty::Unit, |t| self.resolve(t));
            for p in params {
                let ty = self.resolve(&p.param_type);
                if let Err(e) = self
                    .env
                    .define(p.name.clone(), ty, BindingKind::Param, &p.span)
                {
                    self.errors.push(e);
                }
            }

            self.check_block(body);

            self.check_returns(stmt);
        }

        self.env.pop();
    }

    pub fn check_for_stmt(&mut self, stmt: &Statement) {
        self.env.push();

        self.in_loop = true;

        if let Statement::While {
            condition: cond,
            body,
            ..
        } = stmt
        {
            let cond_ty = self.infer(cond, None);

            self.expect(&cond_ty, &Ty::Bool, stmt.span());

            self.check_block(body);
        } else if let Statement::ForRange {
            variable,
            iterable,
            body,
            span,
        } = stmt
        {
            let _iter_ty = self.infer(iterable, None);

            if let Expression::Range { end, .. } = iterable
                && end.is_none()
            {
                self.errors.push(TypeError::Unsupported {
                    desc: "Cannot iterate over an open-ended range in for-loop".to_string(),
                    span: *span,
                });
            }

            self.defined(variable.clone(), Ty::Int, BindingKind::Var, stmt.span());

            self.check_block(body);
        } else if let Statement::ForCounter {
            init,
            condition: cond,
            post,
            body,
            ..
        } = stmt
        {
            self.check_statement(init.as_ref());

            let cond_ty = self.infer(cond, None);

            self.expect(&cond_ty, &Ty::Bool, stmt.span());

            let _post_ty = self.infer(post, None);

            self.check_block(body);
        }
        self.env.pop();

        self.in_loop = false;
    }

    pub fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Var { name, value, .. } => {
                let ty = self.infer(value, None);

                self.defined(name.clone(), ty, BindingKind::Var, stmt.span())
            }
            Statement::Let { name, value, .. } => {
                let ty = self.infer(value, None);

                self.defined(name.clone(), ty, BindingKind::Let, stmt.span())
            }
            Statement::Const { name, value, .. } => {
                let ty = self.infer(value, None);

                self.defined(name.clone(), ty, BindingKind::Const, stmt.span())
            }
            Statement::If(if_stmt) => {
                let cond_ty = self.infer(&if_stmt.condition, None);

                self.expect(&cond_ty, &Ty::Bool, stmt.span());

                self.check_block(&if_stmt.then_block);

                for s in &if_stmt.else_if {
                    let cond_ty = self.infer(&s.condition, None);

                    self.expect(&cond_ty, &Ty::Bool, stmt.span());

                    self.check_block(&s.block);
                }

                if !if_stmt.else_block.is_empty() {
                    self.check_block(&if_stmt.else_block);
                }
            }
            Statement::While { .. } | Statement::ForCounter { .. } | Statement::ForRange { .. } => {
                self.check_for_stmt(stmt)
            }
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
            }
            Statement::Return { value, .. } => {
                let ty = match value {
                    Some(v) => self.infer(v, None),
                    None => Ty::Unit,
                };

                let cur_ref = &self.current_ret.clone();

                self.expect(&ty, cur_ref, stmt.span())
            }
            Statement::ExternFunc {
                name, params, span, ..
            } => {
                let sig = FnSig {
                    params: params.iter().map(|p| self.resolve(&p.param_type)).collect(),
                    ret: Ty::Unit,
                    span: *span,
                };

                self.funcs.insert(name.clone(), sig);
            }
            Statement::Expression { expression, .. } => {
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
                    match left.as_ref() {
                        Expression::Identifier { name, .. } => {
                            self.check_assignment(name.as_str(), right.as_ref());
                        }
                        Expression::Field {
                            object,
                            name: field_name,
                            span,
                        } => self.check_field_assignment(
                            object.as_ref(),
                            field_name,
                            right.as_ref(),
                            *span,
                        ),
                        _ => self.errors.push(TypeError::Unsupported {
                            desc: "Invalid assignment target (non as l-value)".to_string(),
                            span: left.span(),
                        }),
                    }
                } else {
                    self.infer(expression, None);
                }
            }
            Statement::Struct { .. } => self.check_struct(stmt),
            Statement::Extend { methods, .. } => {
                for method in methods {
                    if let Statement::Func { .. } = method {
                        self.check_fn(method);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn check_struct(&mut self, stmt: &Statement) {
        if let Statement::Struct {
            name: _, fields, ..
        } = stmt
        {
            for f in fields {
                let ty = self.resolve(&f.param_type);
                self.defined(f.name.clone(), ty, BindingKind::Var, stmt.span())
            }
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

        let value_ty = self.infer(value_expr, None);

        self.expect(&value_ty, &entity.ty, value_expr.span())
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
            if let Statement::Func { .. } = stmt {
                self.check_fn(stmt);
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
        for stmt in &program.main {
            self.check_statement(stmt);
        }
        self.env.pop()
    }

    pub fn infer(&mut self, expr: &Expression, expected_ty: Option<&Ty>) -> Ty {
        match expr {
            Expression::IntLiteral { .. } => Ty::Int,
            Expression::BoolLiteral { .. } => Ty::Bool,
            Expression::StringLiteral { .. } => Ty::String,
            Expression::FloatLiteral { .. } => Ty::Float,
            Expression::Identifier { name, .. } => match self.env.lookup(name) {
                Some(entity) => entity.ty,
                None => {
                    self.errors.push(TypeError::UnknownName {
                        name: name.clone(),
                        span: expr.span(),
                    });
                    Ty::Error
                }
            },
            Expression::Infix {
                left,
                operator,
                right,
                span,
            } => {
                let left_ty = self.infer(left, None);
                let right_ty = self.infer(right, None);

                match operator {
                    Token::Add => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::Float, Ty::Float) => Ty::Float,
                        (Ty::String, Ty::String) => Ty::String,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            self.errors.push(TypeError::InvalidBinaryOperator {
                                operator: operator.clone(),
                                left: Box::new(left_ty),
                                right: Box::new(right_ty),
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
                                    left: Box::new(left_ty),
                                    right: Box::new(right_ty),
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
                                left: Box::new(left_ty),
                                right: Box::new(right_ty),
                                span: *span,
                            });
                            Ty::Error
                        }
                    },
                    Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => {
                        match (&left_ty, &right_ty) {
                            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) => {}
                            (Ty::Error, _) | (_, Ty::Error) => return Ty::Error,
                            _ => self.errors.push(TypeError::InvalidBinaryOperator {
                                operator: operator.clone(),
                                left: Box::new(left_ty),
                                right: Box::new(right_ty),
                                span: *span,
                            }),
                        };
                        Ty::Bool
                    }
                    Token::Equals | Token::NotEquals => match (&left_ty, &right_ty) {
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            if left_ty != right_ty {
                                self.errors.push(TypeError::InvalidBinaryOperator {
                                    operator: operator.clone(),
                                    left: Box::new(left_ty),
                                    right: Box::new(right_ty),
                                    span: *span,
                                });
                                return Ty::Error;
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
                }
            }
            Expression::Prefix {
                operator,
                right,
                span,
            } => {
                let right_ty = self.infer(right.as_ref(), None);

                match operator {
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
                    _ => Ty::Error,
                }
            }
            Expression::Range { start, end, .. } => {
                let start_ty = start.as_ref().map_or(Ty::Int, |s| self.infer(s, None));
                let end_ty = end.as_ref().map_or(Ty::Int, |e| self.infer(e, None));

                self.expect(&start_ty, &Ty::Int, expr.span());
                self.expect(&end_ty, &Ty::Int, expr.span());

                Ty::Generic {
                    name: "Range".to_string(),
                    param: Box::new(Ty::Int),
                }
            }
            Expression::Index { left, index, span } => {
                let left_ty = self.infer(left.as_ref(), None);
                let index_ty = self.infer(index.as_ref(), None);

                self.expect(&index_ty, &Ty::Int, *span);

                match left_ty {
                    Ty::String => Ty::Int,
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::Unsupported {
                            desc: format!("Type `{:?}` doesn't support indexing", left_ty),
                            span: *span,
                        });

                        Ty::Error
                    }
                }
            }
            Expression::Call {
                function,
                args,
                span,
            } => {
                if let Expression::Identifier { name, .. } = function.as_ref()
                    && self.env.lookup(name).is_none()
                    && !args.is_empty()
                {
                    let first_arg_ty = self.infer(&args[0], None);
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
                                return sig.ret;
                            }

                            for (arg, param) in args.iter().zip(sig.params.iter()) {
                                let a_ty = self.infer(arg, Some(param));
                                self.expect(&a_ty, param, arg.span());
                            }
                            return sig.ret;
                        }
                    }
                }

                let callee = self.infer(function, None);
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
                            return *ret;
                        }

                        for (arg, param) in args.iter().zip(params.iter()) {
                            let a_ty = self.infer(arg, Some(param));
                            self.expect(&a_ty, param, arg.span());
                        }
                        *ret
                    }
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError::NotCallable {
                            ty: callee,
                            span: *span,
                        });
                        Ty::Error
                    }
                }
            }
            Expression::Lambda {
                params,
                return_type,
                body,
                ..
            } => {
                let (expected_fn_params, expected_fn_ret) = match expected_ty {
                    Some(Ty::Fn { params, ret, .. }) => (Some(params), Some(ret.as_ref())),
                    _ => (None, None)
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

                let ret = match return_type {
                    Some(t) => self.resolve(t),
                    None => expected_fn_ret.cloned().unwrap_or(Ty::Unit)
                };

                let saved_ret = self.current_ret.clone();
                self.current_ret = ret.clone();

                self.env.push();

                for (p, ty) in params.iter().zip(param_tys.iter()) {
                    self.defined(p.name.clone(), ty.clone(), BindingKind::Var, p.span);
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
            Expression::StructLiteral { name, fields, span } => {
                if !self.structs.contains_key(name) {
                    self.errors.push(TypeError::UnknownName {
                        name: name.clone(),
                        span: *span,
                    });
                    return Ty::Error;
                }

                if fields.is_empty() {
                    return Ty::Struct(name.clone());
                }

                for f in fields {
                    match self.infer(f.field_val.as_ref(), None) {
                        Ty::Error => return Ty::Error,
                        _ => continue,
                    }
                }

                Ty::Struct(name.clone())
            }
            Expression::Field { object, name, span } => {
                let obj_ty = self.infer(object.as_ref(), None);

                match obj_ty {
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
                            ty: obj_ty,
                            span: *span,
                        });
                        Ty::Error
                    }
                }
            }
            Expression::MethodCall {
                object,
                name,
                args,
                span,
            } => {
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
                        Ty::String | Ty::Int | Ty::Float | Ty::Bool => name.clone(),
                        Ty::Error => return Ty::Error,
                        _ => {
                            self.errors.push(TypeError::NoSuchMethod {
                                ty: obj_ty.clone(),
                                method: name.clone(),
                                span: *span,
                            });
                            return Ty::Error;
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
                            return Ty::Error;
                        }
                        self.expect(&obj_ty, &sig.params[0], object.span());
                        &sig.params[1..]
                    };

                    if args.len() != expected_args.len() {
                        self.errors.push(TypeError::ArityMismatch {
                            name: name.clone(),
                            expected: expected_args.len(),
                            found: args.len(),
                            span: *span,
                        });
                        return sig.ret;
                    }

                    for (arg, param_ty) in args.iter().zip(expected_args.iter()) {
                        let a_ty = self.infer(arg, Some(param_ty));
                        self.expect(&a_ty, param_ty, arg.span());
                    }

                    sig.ret
                } else {
                    if is_static {
                        self.errors.push(TypeError::UnknownName {
                            name: name.clone(),
                            span: *span,
                        });
                    } else {
                        if let Some(global_sig) = self.funcs.get(name)
                            && !global_sig.params.is_empty()
                            && global_sig.params[0] == obj_ty
                        {
                            self.errors.push(TypeError::MethodFoundAsGlobal {
                                ty: obj_ty.clone(),
                                method: name.clone(),
                                span: *span,
                                help: format!(
                                    "there is a global function `{}`, did you want to declare it to `extend {} {{ ... }}`?",
                                    name,
                                    match obj_ty {
                                        Ty::Struct(ref s_name) => s_name,
                                        _ => "Type"
                                    }),
                            });
                            return Ty::Error;
                        }
                        self.errors.push(TypeError::NoSuchMethod {
                            ty: obj_ty,
                            method: name.clone(),
                            span: *span,
                        });
                    }
                    Ty::Error
                }
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
            BindingKind::Let,
            Span::default(),
        );
        self.defined(
            "println".to_string(),
            Ty::Fn {
                params: vec![Ty::Union(vec![Ty::Int, Ty::Float, Ty::String, Ty::Bool])],
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
        if let Err(e) = self.env.define(name.clone(), ty, kind, &span) {
            self.errors.push(e);
        }
    }

    pub fn expect(&mut self, actual: &Ty, expected: &Ty, span: Span) {
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
                | Statement::Expression { .. }
                | Statement::While { .. }
                | Statement::ForRange { .. }
                | Statement::ForCounter { .. }
                | Statement::Func { .. }
                | Statement::Struct { .. }
                | Statement::Break { .. }
                | Statement::Continue { .. }
                | Statement::ExternFunc { .. }
                | Statement::Extend { .. } => continue,
                Statement::If(IfStatement {
                    then_block,
                    else_if,
                    else_block,
                    ..
                }) => {
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
}
