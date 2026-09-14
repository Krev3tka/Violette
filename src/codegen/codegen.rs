use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::Unexpected;
use crate::lexer::token::Token;
use crate::parser::expression::RangeKind;
use crate::parser::program::Program;
use crate::parser::statement::{FuncParam, IfStatement, StructParam};
use crate::parser::{Expression, Statement};
use crate::typechecker::checker::Checker;
use crate::typechecker::error::BindingKind;
use crate::typechecker::types::Ty;

pub struct Codegen {
    checker: Checker,
    extern_funcs: std::collections::HashSet<String>,
    typedefs: Vec<String>,
    fn_type_names: std::collections::HashMap<String, String>,
    lambda_prototypes: Vec<String>,
    lifted_lambdas: Vec<String>,
    lambda_count: usize,
}

impl Codegen {
    pub fn new() -> Self {
        let mut checker = Checker::default();
        checker.define_builtins();
        Codegen {
            checker,
            extern_funcs: std::collections::HashSet::new(),
            typedefs: Vec::new(),
            fn_type_names: std::collections::HashMap::new(),
            lambda_prototypes: Vec::new(),
            lifted_lambdas: Vec::new(),
            lambda_count: 0,
        }
    }

    pub fn c_type(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Int => "int64_t".to_string(),
            Ty::Float => "double".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Char => "uint32_t".to_string(),
            Ty::String => "VioString".to_string(),
            Ty::Struct(s) => s.to_string(),
            Ty::Unit => "void".to_string(),
            Ty::Fn { params, ret } => {
                let ret_c = self.c_type(ret);
                let params_c: Vec<String> = params.iter().map(|p| self.c_type(p)).collect();
                let params_str = if params_c.is_empty() {
                    "void".to_string()
                } else {
                    params_c.join(", ")
                };

                let sig_key = format!("{ret_c}({params_str})");
                if let Some(name) = self.fn_type_names.get(&sig_key) {
                    return name.clone();
                }

                let type_name = format!("vio_fn_type_{}", self.fn_type_names.len());
                let typedef_def = format!("typedef {ret_c} (*{type_name})({params_str});");
                self.fn_type_names.insert(sig_key, type_name.clone());
                self.typedefs.push(typedef_def);

                type_name
            }
            Ty::Ref { inner, .. } => format!("{}*", self.c_type(inner)),
            Ty::Infer => "int64_t".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub fn emit_program(&mut self, prg: Program) -> Result<String, CodegenError> {
        // 1. Extern-functions

        for s in &prg.declarations {
            if let Statement::ExternFunc { name, .. } = s {
                self.extern_funcs.insert(name.clone());
            }
        }

        let mut lines: Vec<String> = vec!["#include \"vio_runtime.h\"\n".to_string()];

        let mut global_defines: Vec<String> = Vec::new();

        self.checker.collect_signatures(&prg.declarations);

        self.checker.env.push();

        // 2. Constant variables (#define)

        for s in &prg.declarations {
            if let Statement::Const {
                name, value, span, ..
            } = s
            {
                let val_str = self.emit_expression(value)?;
                let ty = self.checker.infer(value, None);

                self.checker
                    .defined(name.clone(), ty, BindingKind::Const, *span);

                global_defines.push(format!("#define {} {}", name, val_str))
            }
        }

        if !global_defines.is_empty() {
            lines.extend(global_defines);
            lines.push("\n".to_string())
        }

        // 3. Structs

        for s in &prg.declarations {
            if let Statement::Struct { .. } = s {
                let struct_str = self.emit_struct(s)?;

                lines.push(struct_str)
            } else if let Statement::Variant { .. } = s {
                let variant_str = self.emit_statement(s)?;
                lines.push(variant_str);
            }
        }

        // 4. First pass over functions && extend-blocks

        for s in &prg.declarations {
            if let Statement::Func {
                name,
                params,
                return_type,
                ..
            } = s
                && name != "main"
            {
                let c_name = if name.starts_with("vio_") {
                    name.clone()
                } else {
                    format!("vio_user_{}", name)
                };

                let ret_ty = return_type
                    .as_ref()
                    .map_or(Ty::Unit, |t| self.checker.resolve(t));
                let ret_str = self.c_type(&ret_ty);

                let params_str = params
                    .iter()
                    .map(|p| {
                        let mut ty = self.checker.resolve(&p.param_type);

                        if p.is_ref {
                            ty = Ty::Ref {
                                inner: Box::new(ty),
                                is_mut: p.kind.is_mutable(),
                            }
                        }

                        format!("{} {}", self.c_type(&ty), p.name)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                let params_str = if params_str.is_empty() {
                    "void".to_string()
                } else {
                    params_str
                };
                lines.push(format!("{ret_str} {c_name}({params_str});"))
            } else if let Statement::Extend {
                target, methods, ..
            } = s
            {
                let target_ty = self.checker.resolve(target);
                let target_name = match target_ty {
                    Ty::Struct(name) => name,
                    _ => format!("{:?}", target_ty),
                };

                for method in methods {
                    if let Statement::Func {
                        name,
                        params,
                        return_type,
                        ..
                    } = method
                    {
                        let c_name = format!("vio_user_{}_{}", target_name, name);

                        let ret_ty = return_type
                            .as_ref()
                            .map_or(Ty::Unit, |t| self.checker.resolve(t));

                        let ret_str = self.c_type(&ret_ty);

                        let params_str = params
                            .iter()
                            .map(|p| {
                                let ty = self.checker.resolve(&p.param_type);
                                format!("{} {}", self.c_type(&ty), p.name)
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let params_str = if params_str.is_empty() {
                            "void".to_string()
                        } else {
                            params_str
                        };
                        lines.push(format!("{ret_str} {c_name}({params_str});"));
                    }
                }
            }
        }

        lines.push("\n".to_string());

        // 5. Second pass over functions (pls don't make forward declaration in your PLs)

        for s in &prg.declarations {
            if let Statement::Const { .. } = s {
                continue;
            }
            if let Statement::Func {
                name,
                params,
                return_type,
                span,
                ..
            } = s
            {
                let p: Vec<Ty> = params
                    .iter()
                    .map(|p| {
                        let mut ty = self.checker.resolve(&p.param_type);
                        if p.is_ref {
                            ty = Ty::Ref {
                                inner: Box::new(ty),
                                is_mut: p.kind.is_mutable(),
                            }
                        }
                        ty
                    })
                    .collect();

                let ret = return_type
                    .as_ref()
                    .map_or(Ty::Unit, |t| self.checker.resolve(t));

                let fn_ty = Ty::Fn {
                    params: p,
                    ret: Box::new(ret),
                };

                self.checker
                    .defined(name.clone(), fn_ty.clone(), BindingKind::Var, *span);
            }
        }

        // 6. Main function

        for s in &prg.declarations {
            if let Statement::Const { .. } = s {
                continue;
            }
            if let Statement::Func { name, body, .. } = s
                && name == "main"
            {
                lines.push("int main(void) {".to_string());

                lines.push(self.emit_block(body)?);

                lines.push("    return 0;".to_string());

                lines.push("}".to_string());

                continue;
            }
            let stmt = match s {
                Statement::Struct { .. } | Statement::Variant { .. } => String::new(),
                _ => self.emit_statement(s)?,
            };
            for line in stmt.lines() {
                lines.push(line.to_string());
            }
        }

        if !prg.main.is_empty() {
            lines.push("int main(void) {".to_string());

            self.checker.env.push();
            lines.push(self.emit_block(&prg.main)?);
            self.checker.env.pop();

            lines.push("}".to_string());
        } else if prg.main.is_empty() && prg.declarations.is_empty() {
            lines.push("int main(void) {\n\t\n}".to_string())
        }

        let mut final_lines = Vec::new();
        final_lines.push("#include \"vio_runtime.h\"\n".to_string());

        if !self.typedefs.is_empty() {
            final_lines.extend(self.typedefs.clone());
            final_lines.push("\n".to_string());
        }

        if !self.lambda_prototypes.is_empty() {
            final_lines.extend(self.lambda_prototypes.clone());
            final_lines.push("\n".to_string());
        }

        final_lines.extend(lines.into_iter().skip(1));

        if !self.lifted_lambdas.is_empty() {
            final_lines.extend(self.lifted_lambdas.clone());
            final_lines.push("\n".to_string());
        }

        let mut res = final_lines.join("\n");

        res.push('\n');

        Ok(res)
    }

    pub fn emit_expression(&mut self, expr: &Expression) -> Result<String, CodegenError> {
        self.emit_expression_with_expected(expr, None)
    }

    pub fn emit_expression_with_expected(
        &mut self,
        expr: &Expression,
        expected_ty: Option<&Ty>,
    ) -> Result<String, CodegenError> {
        Ok(match expr {
            Expression::IntLiteral { val: i, .. } => i.to_string(),
            Expression::FloatLiteral { val: f, .. } => {
                if f.fract() == 0.0 {
                    format!("{f:.1}")
                } else {
                    f.to_string()
                }
            }
            Expression::BoolLiteral { val: b, .. } => b.to_string(),
            Expression::StringLiteral { val: s, .. } => {
                let escaped = s.escape_default().to_string();

                let byte_len = s.len();

                format!("vio_str_from_literal(\"{}\", {})", escaped, byte_len)
            }
            Expression::CharLiteral { val, .. } => {
                format!("((uint32_t){:#X}U)", *val as u32)
            }
            Expression::StructLiteral { name, fields, .. } => {
                if fields.is_empty() {
                    return Ok(format!("({}){{0}}", name));
                }

                let c_fields = fields
                    .iter()
                    .map(|f| {
                        let f_val = self.emit_expression(f.field_val.as_ref()).unwrap();

                        Ok(format!("{} = {}", f.field_name, f_val))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", .");

                format!("({}){{ .{} }}", name.clone(), c_fields)
            }
            Expression::Prefix {
                operator, right, ..
            } => {
                if matches!(operator, Token::BitAnd) {
                    return Ok(format!("&{}", self.emit_expression(right.as_ref())?));
                }
                format!(
                    "{}{}",
                    self.correlate_operator(operator)?,
                    self.emit_expression(right.as_ref())?
                )
            }
            Expression::Infix {
                left,
                operator,
                right,
                ..
            } => {
                if matches!(operator, Token::Add)
                    && matches!(self.checker.infer(left.as_ref(), None), Ty::String)
                    && matches!(self.checker.infer(right.as_ref(), None), Ty::String)
                {
                    return Ok(format!(
                        "vio_str_concat({}, {})",
                        self.emit_expression(left.as_ref())?,
                        self.emit_expression(right.as_ref())?
                    ));
                }

                if matches!(operator, Token::Power) {
                    return Ok(format!(
                        "pow({}, {})",
                        self.emit_expression(left.as_ref())?,
                        self.emit_expression(right.as_ref())?
                    ));
                }

                format!(
                    "({} {} {})",
                    self.emit_expression(left.as_ref())?,
                    self.correlate_operator(operator)?,
                    self.emit_expression(right.as_ref())?
                )
            }
            Expression::Postfix { left, operator, .. } => {
                format!(
                    "{}{}",
                    self.emit_expression(left.as_ref())?,
                    self.correlate_operator(operator)?
                )
            }
            Expression::Index { left, index, .. } => {
                let left_str = self.emit_expression(left.as_ref())?;

                let index_str = self.emit_expression(index.as_ref())?;

                format!("vio_str_get({}, {})", left_str, index_str)
            }
            Expression::Identifier { name: ident, .. } => {
                if let Some((variant_name, None)) = self.checker.variant_cases.get(ident) {
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{} }}",
                        variant_name, variant_name, ident
                    ));
                }

                if let Some(entity) = self.checker.env.lookup(ident)
                    && matches!(entity.ty, Ty::Ref { .. })
                {
                    return Ok(format!("(*{})", ident.replace("$", "_arg_")));
                }

                ident.replace("$", "_arg_")
            }
            Expression::Call { function, args, .. } => {
                if let Expression::Identifier { name, .. } = function.as_ref()
                    && let Some((variant_name, Some(_))) =
                        self.checker.variant_cases.get(name).cloned()
                {
                    let val_str = self.emit_expression(&args[0])?;
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{}, .data = {{ .{} = {} }} }}",
                        variant_name, variant_name, name, name, val_str
                    ));
                }

                if let Expression::Identifier { name, .. } = function.as_ref()
                    && (name == "print" || name == "println")
                    && args.len() == 1
                {
                    let arg_ty = self.checker.infer(&args[0], None);
                    let suffix = match arg_ty {
                        Ty::Int => "int",
                        Ty::Float => "float",
                        Ty::Bool => "bool",
                        Ty::String => "string",
                        Ty::Char => "char",
                        _ => {
                            return Err(CodegenError::Unsupported(format!(
                                "print for this type: {:?}",
                                arg_ty
                            )));
                        }
                    };

                    let a = self.emit_expression(&args[0])?;

                    return Ok(format!("vio_{name}_{suffix}({a})"));
                }

                if let Expression::Identifier { name, .. } = function.as_ref()
                    && name == "scanln"
                    && args.is_empty()
                {
                    return Ok(format!("vio_{name}()"));
                }

                let callee_ty = self.checker.infer(function.as_ref(), None);
                let expected_params = match &callee_ty {
                    Ty::Fn { params, .. } => Some(params.clone()),
                    _ => None,
                };

                let f = self.emit_expression(function.as_ref())?;

                let c_fn_name =
                    if f == "main" || f.starts_with("vio_") || self.extern_funcs.contains(&f) {
                        f.clone()
                    } else if self.checker.funcs.contains_key(&f) {
                        format!("vio_user_{}", f)
                    } else {
                        f.clone()
                    };

                let a = args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        let exp = expected_params.as_ref().and_then(|p| p.get(i));
                        self.emit_expression_with_expected(arg, exp)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                format!("{}({})", c_fn_name, a)
            }
            Expression::MethodCall {
                object, name, args, ..
            } => {
                if let Expression::Identifier { name: obj_name, .. } = object.as_ref()
                    && let Some((variant_name, Some(_))) = self.checker.variant_cases.get(name).cloned()
                    && variant_name == *obj_name
                {
                    let val_str = self.emit_expression(&args[0])?;
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{}, .data = {{ .{} = {} }} }}",
                        variant_name, variant_name, name, name, val_str
                    ));
                }

                let (c_fn_name, is_static_or_module) =
                    if let Expression::Identifier { name: obj_name, .. } = object.as_ref() {
                        if self.checker.structs.contains_key(obj_name) {
                            (format!("vio_user_{}_{}", obj_name, name), true)
                        } else if name == "main"
                            || name.starts_with("vio_")
                            || self.extern_funcs.contains(name)
                        {
                            (name.clone(), false)
                        } else if self.checker.env.lookup(obj_name).is_none() {
                            (format!("vio_user_{}", name), true)
                        } else {
                            let obj_ty = self.checker.infer(object.as_ref(), None);
                            match obj_ty {
                                Ty::Struct(s) => (format!("vio_user_{}_{}", s, name), false),
                                _ => (format!("vio_user_{}", name), false),
                            }
                        }
                    } else {
                        let obj_ty = self.checker.infer(object.as_ref(), None);
                        match obj_ty {
                            Ty::Struct(s) => (format!("vio_user_{}_{}", s, name), false),
                            _ => (format!("vio_user_{}", name), false),
                        }
                    };

                let mut all_args = Vec::new();

                if !is_static_or_module {
                    all_args.push(self.emit_expression(object.as_ref())?);
                }

                for a in args {
                    all_args.push(self.emit_expression(a)?)
                }

                format!("{}({})", c_fn_name, all_args.join(", "))
            }
            Expression::Field { object, name, .. } => {
                if let Expression::Identifier { name: obj_name, .. } = object.as_ref()
                    && let Some((var_name, None)) = self.checker.variant_cases.get(name)
                    && var_name == obj_name
                {
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{} }}",
                        var_name, var_name, name
                    ));
                }

                format!("{}.{}", self.emit_expression(object.as_ref())?, name)
            }
            Expression::Lambda {
                params,
                return_type,
                body,
                ..
            } => {
                let lambda_name = format!("vio_lambda_{}", self.lambda_count);
                self.lambda_count += 1;

                let (expected_params, expected_ret) = match expected_ty {
                    Some(Ty::Fn { params, ret, .. }) => (Some(params), Some(ret.as_ref())),
                    _ => (None, None),
                };

                self.checker.env.push();

                let mut param_strs = Vec::new();
                for (i, p) in params.iter().enumerate() {
                    let clean_name = p.name.replace("$", "_arg_");

                    let mut ty = self.checker.resolve(&p.param_type);
                    if ty == Ty::Infer || ty == Ty::Error {
                        if let Some(exp_p) = expected_params.and_then(|ep| ep.get(i)) {
                            ty = exp_p.clone();
                        } else {
                            ty = Ty::Int
                        }
                    }

                    self.checker
                        .defined(p.name.clone(), ty.clone(), BindingKind::Let, p.span);
                    param_strs.push(format!("{} {}", self.c_type(&ty), clean_name))
                }

                let ret_ty = match return_type {
                    Some(t) => self.checker.resolve(t),
                    None => {
                        if let Some(exp_r) = expected_ret
                            && *exp_r != Ty::Infer
                            && *exp_r != Ty::Error
                        {
                            exp_r.clone()
                        } else {
                            let inferred = body.iter().find_map(|s| {
                                if let Statement::Return { value: Some(v), .. } = s {
                                    let t = self.checker.infer(v, None);
                                    if t != Ty::Error && t != Ty::Infer {
                                        Some(t)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });
                            inferred.unwrap_or(Ty::Unit)
                        }
                    }
                };

                let ret_c = self.c_type(&ret_ty);

                let param_list = if param_strs.is_empty() {
                    "void".to_string()
                } else {
                    param_strs.join(", ")
                };

                let body_c = self.emit_block(body)?;

                self.checker.env.pop();

                self.lambda_prototypes
                    .push(format!("static {ret_c} {lambda_name}({param_list});"));
                self.lifted_lambdas.push(format!(
                    "static {ret_c} {lambda_name}({param_list}) {{\n{body_c}\n}}\n"
                ));

                lambda_name
            }
            Expression::Match { target, arms, .. } => {
                let target_ty = self.checker.infer(target.as_ref(), None);
                let variant_name = match &target_ty {
                    Ty::Struct(name) => name.clone(),
                    _ => return Err(CodegenError::Unsupported("Non-struct match target".to_string()))
                };

                let res_ty = match expected_ty {
                    Some(t) => t.clone(),
                    None => self.checker.infer(expr, None)
                };

                let res_c_type = self.c_type(&res_ty);

                let target_var = format!("_vio_match_target_{}", self.lambda_count);

                let res_var = format!("_vio_match_res_{}", self.lambda_count);
                self.lambda_count += 1;

                let target_val_str = self.emit_expression(target.as_ref())?;

                let mut cases_c = Vec::new();

                for arm in arms {
                    let (case_name, bind_var) = match &arm.pattern {
                        Expression::Identifier { name, .. }
                        | Expression::Field { name, .. } => (name.clone(), None),
                        Expression::Call { function, args, .. } => {
                            let name = match function.as_ref() {
                                Expression::Identifier { name, ..} => name.clone(),
                                _ => unreachable!()
                            };

                            let bind = match &args[0] {
                                Expression::Identifier { name: b, ..} => b.clone(),
                                _ => unreachable!()
                            };
                            (name.clone(), Some(bind))
                        }
                        Expression::MethodCall { name, args, .. } => {
                            let bind = match &args[0] {
                                Expression::Identifier { name: b, ..} => b.clone(),
                                _ => unreachable!()
                            };
                            (name.clone(), Some(bind))
                        }
                        _ => return Err(CodegenError::Unsupported("Pattern shape in codegen".to_string()))
                    };

                    let tag_name = if case_name != "_".to_string() {
                        format!("VIO_TAG_{}_{}", variant_name, case_name)
                    } else {
                        "default".to_string()
                    };

                    self.checker.env.push();

                    let mut arm_lines = Vec::new();

                    if let Some(var) = bind_var
                        && let Some((_, Some(payload_ty))) = self.checker.variant_cases.get(&case_name).cloned() {
                        let payload_c_ty = self.c_type(&payload_ty);

                        arm_lines.push(format!(
                            "    {} {} = {}.data.{};", payload_c_ty, var, target_var, case_name)
                        );

                        self.checker.defined(var, payload_ty, BindingKind::Let, arm.pattern.span());
                    }

                    match &arm.body {
                        Expression::Block { body, .. } => {
                            if let Some((last, init)) = body.split_last() {
                                for stmt in init {
                                    arm_lines.push(format!("    {}", self.emit_statement(stmt)?));
                                }

                                match last {
                                    Statement::Expression { expression, .. } => {
                                        let val = self.emit_expression(expression)?;
                                        arm_lines.push(format!("    {} = {};", res_var, val));
                                    }
                                    _ => arm_lines.push(format!("    {}", self.emit_statement(last)?))
                                }
                            }
                        }
                        _ => {
                            let val = self.emit_expression(&arm.body)?;
                            arm_lines.push(format!("   {} = {};", res_var, val));
                        }
                    }

                    self.checker.env.pop();

                    arm_lines.push("    break;".to_string());

                    cases_c.push( if tag_name != "default" {
                        format!("case {}: {{\n{}\n}}", tag_name, arm_lines.join("\n"))
                    } else {
                        format!("{}: {{\n{}\n}}", tag_name, arm_lines.join("\n"))
                    });
                }

                format!(
                    "({{\n{} {} = {};\n{} {};\nswitch ({}.tag) {{\n{}\n}}\n{};\n}})",
                    self.c_type(&target_ty),
                    target_var,
                    target_val_str,
                    res_c_type,
                    res_var,
                    target_var,
                    cases_c.join("\n"),
                    res_var
                )
            }
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "this expression: {:?}",
                    expr
                )));
            }
        })
    }

    pub fn emit_statement(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        Ok(match stmt {
            Statement::Let {
                name, value, span, ..
            }
            | Statement::Const {
                name, value, span, ..
            }
            | Statement::Var {
                name, value, span, ..
            } => {
                let val_str = self.emit_expression(value)?;

                let ty = self.checker.infer(value, None);

                self.checker.defined(
                    name.clone(),
                    ty.clone(),
                    if matches!(stmt, Statement::Const { .. }) {
                        BindingKind::Const
                    } else {
                        BindingKind::Var
                    },
                    *span,
                );

                let mut res = String::new();

                res.push_str(format!("{} {} = {};", self.c_type(&ty), name, val_str).as_str());

                res
            }
            Statement::If(IfStatement {
                condition,
                then_block,
                else_if,
                else_block,
                ..
            }) => {
                self.checker.env.push();

                let mut res = String::new();

                let cond = self.emit_expression(condition)?;

                let first_block = self.emit_block(then_block)?;

                res.push_str(format!("if ({}) {{\n{}\n}}\n", cond, first_block).as_str());

                if !else_if.is_empty() {
                    for if_s in else_if {
                        let cond = self.emit_expression(&if_s.condition)?;

                        let some_block = self.emit_block(&if_s.block)?;

                        res.push_str(
                            format!("else if ({}) {{\n{}\n}}\n", cond, some_block).as_str(),
                        );
                    }
                }

                if !else_block.is_empty() {
                    let el_block = self.emit_block(else_block)?;

                    res.push_str(format!("else {{\n{}\n}}\n", el_block).as_str())
                }

                self.checker.env.pop();

                res
            }
            Statement::Return { value, .. } => {
                let mut val_str = String::new();
                if let Some(expr) = value {
                    val_str = format!(" {}", self.emit_expression(expr)?);
                }

                format!("return{};", val_str)
            }
            Statement::ExternFunc {
                name,
                params,
                return_type,
                ..
            } => {
                let mut ret = Ty::Unit;

                if let Some(ty) = return_type {
                    ret = self.checker.resolve(ty);
                }

                let parameters = params
                    .iter()
                    .map(
                        |FuncParam {
                             name, param_type, ..
                         }| {
                            let ty = self.checker.resolve(param_type);
                            format!("{} {}", self.c_type(&ty), name.clone())
                        },
                    )
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("extern {} {}({});", self.c_type(&ret), name, parameters)
            }
            Statement::Expression { expression, .. } => {
                format!("{};", self.emit_expression(expression)?)
            }
            Statement::While { .. } | Statement::ForCounter { .. } | Statement::ForRange { .. } => {
                self.emit_for(stmt)?
            }
            Statement::Break { .. } => "break;".to_string(),
            Statement::Continue { .. } => "continue;".to_string(),
            Statement::Func { .. } => self.emit_function(stmt)?,
            Statement::Struct { .. } => self.emit_struct(stmt)?,
            Statement::Extend {
                target, methods, ..
            } => {
                let target_ty = self.checker.resolve(target);

                let target_name = match target_ty {
                    Ty::Struct(name) => name,
                    _ => format!("{:?}", target_ty),
                };

                let mut lines = Vec::new();

                for method in methods {
                    if let Statement::Func { name, .. } = method {
                        let c_name = format!("vio_user_{}_{}", target_name, name);
                        lines.push(self.emit_function_custom(method, &c_name)?);
                    }
                }

                lines.join("\n")
            }

            Statement::Variant { name, cases, .. } => {
                let mut c_cases = Vec::new();

                for case in cases {
                    let case_c = format!("VIO_TAG_{}_{}", name, case.name);

                    c_cases.push(case_c);
                }

                let tag_name = format!("vio_tag_{}", name);

                let mut union_states = Vec::new();

                for case in cases {
                    let state_c_ty = match &case.payload {
                        Some(ty) => {
                            let c_ty = &self.checker.resolve(ty);

                            self.c_type(c_ty)
                        }
                        None => continue,
                    };

                    union_states.push(format!("{} {}", state_c_ty, case.name))
                }

                let mut res = Vec::new();
                res.push(format!(
                    "typedef enum {{\n    {}\n}} {};\n",
                    c_cases.join(",\n    "),
                    tag_name
                ));

                if union_states.is_empty() {
                    res.push(format!(
                        "typedef struct {{\n    {} tag\n}} {};\n",
                        tag_name, name
                    ));
                } else {
                    res.push(format!(
                        "typedef struct {{\n    {} tag;\n    union {{\n        {};\n    }} data;\n}} {};\n",
                        tag_name,
                        union_states.join(";\n        "),
                        name
                    ));
                }

                res.join("\n")
            }
        })
    }

    pub fn emit_for(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::While {
            condition, body, ..
        } = stmt
        {
            self.checker.env.push();

            let cond = self.emit_expression(condition)?;

            let body = self.emit_block(body)?;

            self.checker.env.pop();

            Ok(format!("while ({}) {{\n{}\n}}", cond, body))
        } else if let Statement::ForCounter {
            init,
            condition,
            post,
            body,
            ..
        } = stmt
        {
            self.checker.env.push();

            let initial = self.emit_statement(init.as_ref())?;

            let cond = self.emit_expression(condition)?;

            let postfix = self.emit_expression(post)?;

            let body = self.emit_block(body)?;

            self.checker.env.pop();

            Ok(format!(
                "for ({} {}; {}) {{\n{}\n}}",
                initial, cond, postfix, body
            ))
        } else if let Statement::ForRange {
            variable,
            iterable,
            body,
            span,
        } = stmt
        {
            self.checker.env.push();

            let (start, end, cmp) = match iterable {
                Expression::Range {
                    start,
                    end,
                    range_kind,
                    ..
                } => (
                    start
                        .as_ref()
                        .map_or(Ok("0".to_string()), |s| self.emit_expression(s))?,
                    end.as_ref().map_or(
                        Err(CodegenError::Unsupported("Open ranges".to_string())),
                        |e| self.emit_expression(e),
                    )?,
                    if matches!(range_kind, RangeKind::Inclusive) {
                        "<="
                    } else {
                        "<"
                    },
                ),
                _ => {
                    return Err(CodegenError::Unsupported(
                        "For-loop expects a range".to_string(),
                    ));
                }
            };

            self.checker
                .defined(variable.clone(), Ty::Int, BindingKind::Var, *span);
            let body_str = self.emit_block(body)?;

            self.checker.env.pop();

            Ok(format!(
                "for (int64_t {} = {}; {} {} {}; {}++) {{\n{}\n}}",
                variable, start, variable, cmp, end, variable, body_str
            ))
        } else {
            unreachable!()
        }
    }

    pub fn emit_function(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::Func { name, .. } = stmt {
            let c_name =
                if name == "main" || name.starts_with("vio_") || self.extern_funcs.contains(name) {
                    name.clone()
                } else {
                    format!("vio_user_{}", name)
                };

            self.emit_function_custom(stmt, &c_name)
        } else {
            Err(Unexpected(format!("{:?}", stmt)))
        }
    }

    pub fn emit_function_custom(
        &mut self,
        stmt: &Statement,
        c_name: &str,
    ) -> Result<String, CodegenError> {
        if let Statement::Func {
            name,
            params,
            return_type,
            body,
            span,
            ..
        } = stmt
        {
            self.checker.env.push();

            for p in params {
                let mut param_ty = self.checker.resolve(&p.param_type);
                if p.is_ref {
                    param_ty = Ty::Ref {
                        inner: Box::new(param_ty),
                        is_mut: p.kind.is_mutable(),
                    };
                }
                self.checker
                    .defined(p.name.clone(), param_ty, p.kind, *span);
            }

            let mut ret = if name == "main" {
                "int".to_string()
            } else {
                String::from("void")
            };

            if let Some(ty) = return_type {
                let ty = self.checker.resolve(ty);
                ret = self.c_type(&ty)
            }

            let parameters = params
                .iter()
                .map(|p| {
                    let mut ty = self.checker.resolve(&p.param_type);
                    if p.is_ref {
                        ty = Ty::Ref {
                            inner: Box::new(ty),
                            is_mut: p.kind.is_mutable(),
                        };
                    }
                    format!("{} {}", self.c_type(&ty), p.name.clone())
                })
                .collect::<Vec<String>>()
                .join(", ");

            let parameters = if parameters.is_empty() && c_name == "main" {
                "void".to_string()
            } else {
                parameters
            };

            let body_str = self.emit_block(body)?;

            self.checker.env.pop();

            Ok(format!("{ret} {c_name}({parameters}) {{\n{body_str}\n}}\n"))
        } else {
            Err(Unexpected(format!("{:?}", stmt)))
        }
    }

    pub fn emit_struct(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::Struct { name, fields, span } = stmt {
            self.checker.env.push();

            let field_tys: Vec<Ty> = fields
                .iter()
                .map(|p| self.checker.resolve(&p.param_type))
                .collect();

            for (f, field_ty) in fields.iter().zip(field_tys.iter()) {
                self.checker
                    .defined(f.name.clone(), field_ty.clone(), BindingKind::Var, *span)
            }

            let mut c_fields = fields
                .iter()
                .map(
                    |StructParam {
                         name: n,
                         param_type,
                         ..
                     }| {
                        let ty = self.checker.resolve(param_type);
                        format!("\t{} {}", self.c_type(&ty), n.clone())
                    },
                )
                .collect::<Vec<String>>()
                .join(";\n");

            c_fields.push_str(";\n");

            self.checker.env.pop();

            Ok(format!(
                "typedef struct {{\n{}}} {};",
                c_fields,
                name.clone()
            ))
        } else {
            Err(Unexpected(format!("{:?}", stmt)))
        }
    }

    pub fn emit_block(&mut self, body: &[Statement]) -> Result<String, CodegenError> {
        let mut lines = Vec::new();
        let mut string_vars: Vec<String> = Vec::new();

        for s in body {
            if let Statement::Let { name, value, .. }
            | Statement::Const { name, value, .. }
            | Statement::Var { name, value, .. } = s
            {
                let ty = self.checker.infer(value, None);

                if matches!(ty, Ty::String) {
                    string_vars.push(name.clone());
                }
            }
            let stmt = self.emit_statement(s)?;
            for line in stmt.lines() {
                lines.push(format!("    {line}"));
            }
        }

        for name in string_vars.iter().rev() {
            lines.push(format!("    vio_str_release({});", name))
        }

        Ok(lines.join("\n"))
    }

    fn correlate_operator(&mut self, op: &Token) -> Result<String, CodegenError> {
        Ok(String::from(match *op {
            Token::Assign => "=",
            Token::Equals => "==",
            Token::NotEquals => "!=",
            Token::Less => "<",
            Token::Greater => ">",
            Token::LessOrEquals => "<=",
            Token::GreaterOrEquals => ">=",

            Token::Add => "+",
            Token::Subtract => "-",
            Token::Multiply => "*",
            Token::Divide => "/",
            Token::Modulus => "%",
            Token::Increment => "++",
            Token::Decrement => "--",

            Token::AddAndAssign => "+=",
            Token::SubAndAssign => "-=",
            Token::MulAndAssign => "*=",
            Token::DivAndAssign => "/=",
            Token::ModAndAssign => "%=",

            Token::LogicAnd => "&&",
            Token::LogicOr => "||",
            Token::LogicNot => "!",

            Token::BitAnd => "&",
            Token::BitOr => "|",
            Token::BitXOR => "^",
            Token::BitNot => "~",

            _ => return Err(Unexpected("Not an operator".to_string())),
        }))
    }
}
