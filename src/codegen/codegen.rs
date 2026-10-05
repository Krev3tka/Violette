use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::Unexpected;
use crate::hir::typed_ast::{TypedExpr, TypedExprKind, TypedProgram, TypedStmt, TypedStmtKind};
use crate::lexer::token::Token;
use crate::parser::expression::RangeKind;
use crate::parser::statement::FuncParam;
use crate::typechecker::checker::Checker;
use crate::typechecker::types::Ty;
use std::collections::{HashMap, HashSet};
use crate::parser::types::Type;

pub struct Codegen {
    extern_funcs: HashSet<String>,
    typedefs: Vec<String>,
    fn_type_names: HashMap<String, String>,
    lambda_prototypes: Vec<String>,
    lifted_lambdas: Vec<String>,
    lambda_count: usize,
    is_main: bool,
    scope_defers: Vec<Vec<String>>,
    struct_names: HashSet<String>,
    variant_cases: HashMap<String, (String, bool)>,
    user_funcs: HashSet<String>
}

impl Codegen {
    pub fn new() -> Self {
        let mut checker = Checker::default();
        checker.define_builtins();
        Codegen {
            extern_funcs: HashSet::new(),
            typedefs: Vec::new(),
            fn_type_names: HashMap::new(),
            lambda_prototypes: Vec::new(),
            lifted_lambdas: Vec::new(),
            lambda_count: 0,
            is_main: false,
            scope_defers: Vec::new(),
            struct_names: HashSet::new(),
            variant_cases: HashMap::new(),
            user_funcs: HashSet::new(),
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

    pub fn emit_program(&mut self, prg: TypedProgram) -> Result<String, CodegenError> {
        // 1. Extern-functions

        for s in &prg.declarations {
            if let TypedStmtKind::Func { name, .. } = &s.kind {
                self.user_funcs.insert(name.clone());
            }

            match &s.kind {
                TypedStmtKind::ExternFunc { name, .. } => {
                    self.extern_funcs.insert(name.clone());
                }
                TypedStmtKind::Struct { name, .. } => {
                    self.struct_names.insert(name.clone());
                }
                TypedStmtKind::Variant { name, cases } => {
                    for c in cases {
                        self.variant_cases
                            .insert(c.name.clone(), (name.clone(), c.payload.is_some()));
                    }
                }
                _ => {}
            }
        }

        let mut lines: Vec<String> = vec!["#include \"vio_runtime.h\"\n".to_string()];
        let mut global_defines: Vec<String> = Vec::new();

        // 2. Constant variables (#define)

        for s in &prg.declarations {
            if let TypedStmtKind::Const { name, value, .. } = s.kind.clone() {
                let val_str = self.emit_expression(&value)?;

                global_defines.push(format!("#define {} {}", name, val_str))
            }
        }

        if !global_defines.is_empty() {
            lines.extend(global_defines);
            lines.push("\n".to_string())
        }

        // 3. Structs and Variants

        for s in &prg.declarations {
            match s.kind {
                TypedStmtKind::Struct { .. } | TypedStmtKind::Variant { .. } => {
                    lines.push(self.emit_statement(s)?)
                }
                _ => {}
            }
        }

        // 4. First pass over functions and extend-blocks

        for s in &prg.declarations {
            match &s.kind {
                TypedStmtKind::Func {
                    name,
                    params,
                    return_type,
                    ..
                } if name != "main" => {
                    let c_name = if name.starts_with("vio_") {
                        name.clone()
                    } else {
                        format!("vio_user_{}", name)
                    };
                    let ret_str = self.c_type(return_type);
                    let params_str = self.format_params(params);
                    lines.push(format!("{ret_str} {c_name}({params_str});"));
                }
                TypedStmtKind::Extend { target, methods } => {
                    for method in methods {
                        if let TypedStmtKind::Func {
                            name,
                            params,
                            return_type,
                            ..
                        } = &method.kind
                        {
                            let c_name =
                                format!("vio_user_{}_{}", self.type_to_prefix(target), name);
                            let ret_str = self.c_type(return_type);
                            let params_str = self.format_params(params);
                            lines.push(format!("{ret_str} {c_name}({params_str});"));
                        }
                    }
                }
                _ => {}
            }
        }

        lines.push("\n".to_string());

        // 5. Second pass over functions (pls don't make forward declaration in your PLs)

        for s in &prg.declarations {
            match &s.kind {
                TypedStmtKind::Const { .. }
                | TypedStmtKind::Struct { .. }
                | TypedStmtKind::Variant { .. } => continue,
                TypedStmtKind::Func { name, body, .. } if name == "main" => {
                    self.is_main = true;
                    lines.push("int main(void) {".to_string());
                    lines.push(self.emit_block(body)?);
                    lines.push("    return 0;".to_string());
                    lines.push("}".to_string());
                    self.is_main = false;
                }
                _ => {
                    let stmt = self.emit_statement(s)?;
                    for line in stmt.lines() {
                        lines.push(line.to_string());
                    }
                }
            }
        }

        // 6. Main function

        if !prg.main.is_empty() {
            lines.push("int main(void) {".to_string());
            lines.push(self.emit_block(&prg.main)?);
            lines.push("    return 0;".to_string());
            lines.push("}".to_string());
        } else if prg.main.is_empty()
            && !prg
                .declarations
                .iter()
                .any(|d| matches!(&d.kind, TypedStmtKind::Func { name, .. } if name == "main"))
        {
            lines.push("int main(void) {\n    return 0;\n}".to_string());
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

    pub fn emit_expression(&mut self, expr: &TypedExpr) -> Result<String, CodegenError> {
        Ok(match &expr.kind {
            TypedExprKind::IntLiteral { val } => val.to_string(),
            TypedExprKind::FloatLiteral { val } => {
                if val.fract() == 0.0 {
                    format!("{val:.1}")
                } else {
                    val.to_string()
                }
            }
            TypedExprKind::BoolLiteral { val } => val.to_string(),
            TypedExprKind::StringLiteral { val } => {
                let escaped = val.escape_default().to_string();
                let byte_len = val.len();
                format!("vio_str_from_literal(\"{}\", {})", escaped, byte_len)
            }
            TypedExprKind::CharLiteral { val } => {
                format!("((uint32_t){:#X}U)", val.clone() as u32)
            }
            TypedExprKind::StructLiteral { name, fields } => {
                if fields.is_empty() {
                    return Ok(format!("({}){{0}}", name));
                }
                let mut c_fields = Vec::new();
                for f in fields {
                    let f_val = self.emit_expression(&f.field_val)?;
                    c_fields.push(format!(".{} = {}", f.field_name, f_val));
                }
                format!("({}){{ {} }}", name, c_fields.join(", "))
            }
            TypedExprKind::Prefix { operator, right } => {
                if matches!(operator, Token::BitAnd) {
                    return Ok(format!("&{}", self.emit_expression(right.as_ref())?));
                }
                format!(
                    "{}{}",
                    self.correlate_operator(&operator)?,
                    self.emit_expression(right.as_ref())?
                )
            }
            TypedExprKind::Infix {
                left,
                operator,
                right,
            } => {
                if matches!(operator, Token::Add) && left.ty == Ty::String && right.ty == Ty::String
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
                    self.correlate_operator(&operator)?,
                    self.emit_expression(right.as_ref())?
                )
            }
            TypedExprKind::Postfix { left, operator } => {
                format!(
                    "{}{}",
                    self.emit_expression(left.as_ref())?,
                    self.correlate_operator(&operator)?
                )
            }
            TypedExprKind::Index { left, index } => {
                let left_str = self.emit_expression(left.as_ref())?;
                let index_str = self.emit_expression(index.as_ref())?;
                format!("vio_str_get({}, {})", left_str, index_str)
            }
            TypedExprKind::Identifier { name, is_ref } => {
                if let Some((variant_name, false)) = self.variant_cases.get(name) {
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{} }}",
                        variant_name, variant_name, name
                    ));
                }

                let clean = name.replace("$", "_arg_");

                if *is_ref {
                    format!("(*{})", clean)
                } else {
                    clean
                }
            }
            TypedExprKind::Call { function, args } => {
                if let TypedExprKind::Identifier { name, .. } = &function.kind
                    && let Some((variant_name, true)) = self.variant_cases.get(name).cloned()
                {
                    let val_str = self.emit_expression(&args[0])?;
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{}, .data = {{ .{} = {} }} }}",
                        variant_name, variant_name, name, name, val_str
                    ));
                }

                if let TypedExprKind::Identifier { name, .. } = &function.kind
                    && (name == "print" || name == "println")
                    && args.len() == 1
                {
                    let suffix = match &args[0].ty {
                        Ty::Int => "int",
                        Ty::Float => "float",
                        Ty::Bool => "bool",
                        Ty::String => "string",
                        Ty::Char => "char",
                        _ => {
                            return Err(CodegenError::Unsupported(format!(
                                "print for {:?}",
                                args[0].ty
                            )));
                        }
                    };
                    let a = self.emit_expression(&args[0])?;
                    return Ok(format!("vio_{name}_{suffix}({a})"));
                }

                if let TypedExprKind::Identifier { name, .. } = &function.kind
                    && name == "scanln"
                    && args.is_empty()
                {
                    return Ok("vio_scanln()".to_string());
                }

                let f = self.emit_expression(function.as_ref())?;
                let c_fn_name =
                    if self.user_funcs.contains(&f) {
                        format!("vio_user_{}", f)
                    } else {
                        f
                    };

                let a = args
                    .iter()
                    .map(|arg| self.emit_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!("{}({})", c_fn_name, a)
            }
            TypedExprKind::MethodCall { object, name, args } => {
                if let TypedExprKind::Identifier { name: obj_name, .. } = &object.kind
                    && let Some((variant_name, true)) = self.variant_cases.get(name).cloned()
                    && variant_name == *obj_name
                {
                    let val_str = self.emit_expression(&args[0])?;
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{}, .data = {{ .{} = {} }} }}",
                        variant_name, variant_name, name, name, val_str
                    ));
                }

                let (c_fn_name, is_static) =
                    if let TypedExprKind::Identifier { name: obj_name, .. } = &object.kind {
                        if self.struct_names.contains(obj_name) {
                            (format!("vio_user_{}_{}", obj_name, name), true)
                        } else {
                            (
                                format!("vio_user_{}_{}", self.type_to_prefix(&object.ty), name),
                                false,
                            )
                        }
                    } else {
                        (
                            format!("vio_user_{}_{}", self.type_to_prefix(&object.ty), name),
                            false,
                        )
                    };

                let mut all_args = Vec::new();
                if !is_static {
                    all_args.push(self.emit_expression(object.as_ref())?);
                }
                for a in args {
                    all_args.push(self.emit_expression(&a)?);
                }

                format!("{}({})", c_fn_name, all_args.join(", "))
            }
            TypedExprKind::Field { object, name } => {
                if let TypedExprKind::Identifier { name: obj_name, .. } = &object.kind
                    && let Some((var_name, false)) = self.variant_cases.get(name)
                    && var_name == obj_name
                {
                    return Ok(format!(
                        "({}){{ .tag = VIO_TAG_{}_{} }}",
                        var_name, var_name, name
                    ));
                }
                format!("{}.{}", self.emit_expression(&object)?, name)
            }
            TypedExprKind::Lambda {
                params,
                return_ty,
                body,
            } => {
                let lambda_name = format!("vio_lambda_{}", self.lambda_count);
                self.lambda_count += 1;

                let ret_ty = return_ty.clone().unwrap_or(Ty::Unit);
                let ret_c = self.c_type(&ret_ty);

                let param_tys = match &expr.ty {
                    Ty::Fn { params, .. } => params.clone(),
                    _ => vec![Ty::Int; params.len()],
                };

                let mut param_strs = Vec::new();
                for (p, ty) in params.iter().zip(param_tys.iter()) {
                    param_strs.push(format!("{} {}", self.c_type(ty), p.name.replace("$", "_arg_")));
                }
                let param_list = if param_strs.is_empty() {
                    "void".to_string()
                } else {
                    param_strs.join(", ")
                };

                let body_c = if ret_ty != Ty::Unit {
                    let has_explicit_return = Self::does_block_return(body);

                    if !has_explicit_return && let Some((last, init)) = body.split_last() {
                        let mut lines = Vec::new();
                        for s in init {
                            lines.push(format!("    {}", self.emit_statement(s)?));
                        }
                        if let TypedStmtKind::Expression(expr) = &last.kind {
                            lines.push(format!("    return {};", self.emit_expression(expr)?));
                        } else {
                            lines.push(format!("    {}", self.emit_statement(last)?));
                        }
                        lines.join("\n")
                    } else {
                        self.emit_block(&body)?
                    }
                } else {
                    self.emit_block(&body)?
                };

                self.lambda_prototypes
                    .push(format!("static {ret_c} {lambda_name}({param_list});"));
                self.lifted_lambdas.push(format!(
                    "static {ret_c} {lambda_name}({param_list}) {{\n{body_c}\n}}\n"
                ));

                lambda_name
            }
            TypedExprKind::If {
                condition,
                then_block,
                else_if,
                else_block,
            } => {

                let cond_c = self.emit_expression(condition)?;

                if expr.ty == Ty::Unit {
                    let then_c = self.emit_block(then_block)?;
                    let mut elifs_c = Vec::new();
                    for elif in else_if {
                        let cond = self.emit_expression(&elif.condition)?;
                        let body = self.emit_block(&elif.block)?;
                        elifs_c.push(format!("else if ({}) {{\n{}\n}}", cond, body));
                    }
                    let else_c = if !else_block.is_empty() {
                        format!("else {{\n{}\n}}", self.emit_block(else_block)?)
                    } else {
                        String::new()
                    };
                    return Ok(format!("({{\n    if ({}) {{\n{}\n    }} {} {}\n}})", cond_c, then_c, elifs_c.join(" "), else_c));
                }

                let res_c_type = self.c_type(&expr.ty);
                let res_var = format!("_vio_res_{}", self.lambda_count);
                self.lambda_count += 1;

                let cond_c = self.emit_expression(&condition)?;
                let if_branch_c = format!(
                    "if ({}) {{\n        {}\n    }}",
                    cond_c,
                    self.emit_assigning_block(&then_block, &res_var)?
                );

                let mut elifes_c = Vec::new();
                for elif in else_if {
                    let cond = self.emit_expression(&elif.condition)?;

                    let body = self.emit_assigning_block(&elif.block, &res_var)?;

                    elifes_c.push(format!("else if ({}) {{\n        {}\n    }}", cond, body));
                }

                let mut else_branch = String::new();
                if !else_block.is_empty() {
                    else_branch = format!(
                        "else {{\n        {}\n    }}\n",
                        self.emit_assigning_block(&else_block, &res_var)?
                    );
                }

                format!(
                    "({{\n    {} {};\n\n    {} {} {}\n    {};\n}})",
                    res_c_type,
                    res_var,
                    if_branch_c,
                    elifes_c.join("\n"),
                    else_branch,
                    res_var
                )
            }
            TypedExprKind::Match { target, arms } => {
                let variant_name = match &target.ty {
                    Ty::Struct(name) => name.clone(),
                    _ => {
                        return Err(CodegenError::Unsupported(
                            "Non-struct match target".to_string(),
                        ));
                    }
                };

                let res_c_type = self.c_type(&expr.ty);

                let target_var = format!("_vio_match_target_{}", self.lambda_count);
                let res_var = format!("_vio_match_res_{}", self.lambda_count);

                self.lambda_count += 1;

                let target_val_str = self.emit_expression(target.as_ref())?;
                let mut cases_c = Vec::new();

                for arm in arms {
                    let (case_name, bind_var) = match &arm.pattern.kind {
                        TypedExprKind::Identifier { name, .. } | TypedExprKind::Field { name, .. } => {
                            (name.clone(), None)
                        }
                        TypedExprKind::Call { function, args } => {
                            let name = match &function.kind {
                                TypedExprKind::Identifier { name, .. } => name.clone(),
                                _ => unreachable!(),
                            };
                            let (bind, ty) = match &args[0] {
                                TypedExpr {
                                    kind: TypedExprKind::Identifier { name: b, .. },
                                    ty,
                                    ..
                                } => (b.clone(), ty),
                                _ => unreachable!(),
                            };
                            (name, Some((bind, ty)))
                        }
                        TypedExprKind::MethodCall { name, args, .. } => {
                            let (bind, ty) = match &args[0] {
                                TypedExpr {
                                    kind: TypedExprKind::Identifier { name: b, .. },
                                    ty,
                                    ..
                                } => (b.clone(), ty),
                                _ => unreachable!(),
                            };
                            (name.clone(), Some((bind, ty)))
                        }
                        _ => {
                            return Err(CodegenError::Unsupported(
                                "Pattern shape in codegen".to_string(),
                            ));
                        }
                    };

                    let tag_name = if case_name != "_" {
                        format!("VIO_TAG_{}_{}", variant_name, case_name)
                    } else {
                        "default".to_string()
                    };

                    let mut arm_lines = Vec::new();

                    if let Some((var, p_ty)) = bind_var {
                        let payload_c_ty = self.c_type(p_ty);
                        arm_lines.push(format!(
                            "    {} {} = {}.data.{};",
                            payload_c_ty, var, target_var, case_name
                        ));
                    }

                    if let TypedExprKind::Block { body } = &arm.body.kind {
                        if let Some((last, init)) = body.split_last() {
                            for stmt in init {
                                arm_lines.push(format!("    {}", self.emit_statement(stmt)?));
                            }
                            if let TypedStmtKind::Expression(expr) = &last.kind {
                                let val = self.emit_expression(expr)?;
                                arm_lines.push(format!("    {} = {};", res_var, val));
                            } else {
                                arm_lines.push(format!("    {}", self.emit_statement(last)?));
                            }
                        }
                    } else {
                        let val = self.emit_expression(&arm.body)?;
                        arm_lines.push(format!("        {} = {};", res_var, val));
                    }

                    arm_lines.push("        break;".to_string());
                    cases_c.push(if tag_name != "default" {
                        format!("case {}: {{\n{}\n    }}", tag_name, arm_lines.join("\n"))
                    } else {
                        format!("    {}: {{\n{}\n    }}", tag_name, arm_lines.join("\n"))
                    });
                }

                format!(
                    "({{\n{} {} = {};\n{} {};\nswitch ({}.tag) {{\n    {}\n}}\n{};\n}})",
                    self.c_type(&target.ty),
                    target_var,
                    target_val_str,
                    res_c_type,
                    res_var,
                    target_var,
                    cases_c.join("    \n    "),
                    res_var
                )
            }
            TypedExprKind::Block { body } => {
                let res_var = format!("_vio_block_{}", self.lambda_count);
                self.lambda_count += 1;
                let res_c_ty = self.c_type(&expr.ty);

                let body_c = self.emit_assigning_block(&body, &res_var)?;
                format!(
                    "({{\n    {} {};\n{}\n    {};\n}})",
                    res_c_ty, res_var, body_c, res_var
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

    pub fn emit_statement(&mut self, stmt: &TypedStmt) -> Result<String, CodegenError> {
        Ok(match &stmt.kind {
            TypedStmtKind::Let { name, ty, value }
            | TypedStmtKind::Const { name, ty, value }
            | TypedStmtKind::Var { name, ty, value } => {
                let val_str = self.emit_expression(value)?;
                format!("{} {} = {};", self.c_type(ty), name, val_str)
            }
            TypedStmtKind::Return { value } => {
                let mut val_str = String::new();
                if let Some(expr) = value {
                    val_str = format!(" {}", self.emit_expression(expr)?);
                }
                if self.is_main && value.is_none() {
                    "return 0;".to_string()
                } else {
                    format!("return{};", val_str)
                }
            }
            TypedStmtKind::Defer { .. } => String::new(),
            TypedStmtKind::ExternFunc {
                name,
                params,
                return_type,
                ..
            } => {
                let parameters = self.format_params(params);
                format!(
                    "extern {} {}({});",
                    self.c_type(return_type),
                    name,
                    parameters
                )
            }
            TypedStmtKind::Expression(expr) => {
                if let TypedExprKind::If {
                    condition,
                    then_block,
                    else_if,
                    else_block,
                } = &expr.kind {
                    let cond_c = self.emit_expression(condition.as_ref())?;

                    let then_c= self.emit_block(then_block)?;

                    let mut res = format!("if ({}) {{\n{}\n}}", cond_c, then_c);

                    for elif in else_if {
                        let elif_cond = self.emit_expression(&elif.condition)?;
                        let elif_block = self.emit_block(&elif.block)?;

                        res.push_str(&format!(" else if ({}) {{\n{}\n}}", elif_cond, elif_block));
                    }

                    if !else_block.is_empty() {
                        let else_body = self.emit_block(else_block)?;

                        res.push_str(&format!(" else {{\n{}\n}}", else_body))
                    }

                    res
                } else {
                    format!("{};", self.emit_expression(expr)?)
                }
            }
            TypedStmtKind::While { condition, body } => {
                let cond = self.emit_expression(condition)?;
                let body = self.emit_block(body)?;
                format!("while ({}) {{\n{}\n}}", cond, body)
            }
            TypedStmtKind::ForCounter {
                init,
                condition,
                post,
                body,
            } => {
                let initial = self.emit_statement(init)?;
                let cond = self.emit_expression(condition)?;
                let postfix = self.emit_expression(post)?;
                let body = self.emit_block(body)?;
                format!("for ({} {}; {}) {{\n{}\n}}", initial, cond, postfix, body)
            }
            TypedStmtKind::ForRange {
                variable,
                iterable,
                body,
            } => {
                let (start, end, cmp) = match &iterable.kind {
                    TypedExprKind::Range {
                        start,
                        end,
                        range_kind,
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
                let body_str = self.emit_block(body)?;
                format!(
                    "for (int64_t {} = {}; {} {} {}; {}++) {{\n{}\n}}",
                    variable, start, variable, cmp, end, variable, body_str
                )
            }
            TypedStmtKind::Break => "break;".to_string(),
            TypedStmtKind::Continue => "continue;".to_string(),
            TypedStmtKind::Func {
                name,
                params,
                return_type,
                body,
            } => {
                let c_name = if name == "main"
                    || name.starts_with("vio_")
                    || self.extern_funcs.contains(name)
                {
                    name.clone()
                } else {
                    format!("vio_user_{}", name)
                };
                self.emit_function_custom(c_name, params, return_type, body)?
            }
            TypedStmtKind::Struct { name, fields } => {
                let mut c_fields = Vec::new();
                for f in fields {
                    let ty = self.c_type(&self.type_param_to_ty(&f.param_type));
                    c_fields.push(format!("\t{} {}", ty, f.name));
                }
                format!("typedef struct {{\n{};\n}} {};", c_fields.join(";\n"), name)
            }
            TypedStmtKind::Extend { target, methods } => {
                let mut lines = Vec::new();
                for method in methods {
                    if let TypedStmtKind::Func {
                        name,
                        params,
                        return_type,
                        body,
                    } = &method.kind
                    {
                        let c_name = format!("vio_user_{}_{}", self.type_to_prefix(target), name);
                        lines.push(self.emit_function_custom(c_name, params, return_type, body)?);
                    }
                }
                lines.join("\n")
            }
            TypedStmtKind::Variant { name, cases } => {
                let mut c_cases = Vec::new();
                for case in cases {
                    c_cases.push(format!("VIO_TAG_{}_{}", name, case.name));
                }
                let tag_name = format!("vio_tag_{}", name);

                let mut union_states = Vec::new();
                for case in cases {
                    if let Some(ty) = &case.payload {
                        let c_ty = self.c_type(&self.type_param_to_ty(ty));
                        union_states.push(format!("{} {}", c_ty, case.name));
                    }
                }

                let mut res = Vec::new();
                res.push(format!(
                    "typedef enum {{\n    {}\n}} {};\n",
                    c_cases.join(",\n    "),
                    tag_name
                ));
                if union_states.is_empty() {
                    res.push(format!(
                        "typedef struct {{\n    {} tag;\n}} {};\n",
                        tag_name, name
                    ));
                } else {
                    res.push(format!(
                        "typedef struct {{\n    {} tag;\n    union {{\n        {};\n    }} data;\n}} {};\n",
                        tag_name, union_states.join(";\n        "), name
                    ));
                }
                res.join("\n")
            }
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "this statement: {:?}",
                    stmt.kind
                )));
            }
        })
    }

    fn emit_function_custom(
        &mut self,
        c_name: String,
        params: &[FuncParam],
        return_type: &Ty,
        body: &[TypedStmt],
    ) -> Result<String, CodegenError> {
        let ret = if c_name == "main" {
            "int".to_string()
        } else {
            self.c_type(return_type)
        };
        let parameters = self.format_params(params);
        let parameters = if parameters.is_empty() && c_name == "main" {
            "void".to_string()
        } else {
            parameters
        };

        let body_str = if ret != "void" {
            let has_explicit_return = Self::does_block_return(body);

            if !has_explicit_return && let Some((last, init)) = body.split_last() {
                let mut lines = Vec::new();

                for s in init {
                    lines.push(format!("    {}", self.emit_statement(s)?));
                }

                if let TypedStmtKind::Expression(expr) = &last.kind {
                    lines.push(format!("    return {};", self.emit_expression(expr)?));
                } else {
                    lines.push(format!("    {}", self.emit_statement(last)?));
                }

                lines.join("\n")
            } else {
                self.emit_block(body)?
            }
        } else {
            self.emit_block(body)?
        };

        Ok(format!("{ret} {c_name}({parameters}) {{\n{body_str}\n}}\n"))
    }

    pub fn emit_block(&mut self, body: &[TypedStmt]) -> Result<String, CodegenError> {
        let mut lines = Vec::new();
        let mut string_vars: Vec<String> = Vec::new();
        self.scope_defers.push(Vec::new());

        for s in body {
            if let TypedStmtKind::Let { name, ty, .. } | TypedStmtKind::Var { name, ty, .. } =
                &s.kind
            {
                if matches!(ty, Ty::String) {
                    string_vars.push(name.clone());
                }
            } else if let TypedStmtKind::Defer { callee, .. } = &s.kind {
                let c_callee = self.emit_statement(callee)?;
                self.scope_defers.last_mut().unwrap().push(c_callee);
            } else if let TypedStmtKind::Return { .. } = &s.kind {
                for deferred_stmts in self.scope_defers.iter().rev() {
                    for stmt in deferred_stmts.iter().rev() {
                        lines.push(format!("    {}", stmt));
                    }
                }
            }

            let stmt = self.emit_statement(s)?;
            for line in stmt.lines() {
                lines.push(format!("    {line}"));
            }
        }

        let defers = self.scope_defers.pop().unwrap();
        for stmt in defers.iter().rev() {
            lines.push(format!("    {}", stmt));
        }

        for name in string_vars.iter().rev() {
            lines.push(format!("    vio_str_release({});", name));
        }

        Ok(lines.join("\n"))
    }

    fn emit_assigning_block(
        &mut self,
        stmts: &[TypedStmt],
        res_var: &str,
    ) -> Result<String, CodegenError> {
        let mut stmts_c = Vec::new();
        if let Some((last, init)) = stmts.split_last() {
            for s in init {
                stmts_c.push(self.emit_statement(s)?);
            }
            if let TypedStmtKind::Expression(expr) = &last.kind {
                stmts_c.push(format!("{} = {};", res_var, self.emit_expression(expr)?));
            } else {
                stmts_c.push(self.emit_statement(last)?);
            }
            Ok(stmts_c.join("\n"))
        } else {
            Err(CodegenError::Unsupported("empty block".to_string()))
        }
    }

    fn format_params(&mut self, params: &[FuncParam]) -> String {
        let mut p_strs = Vec::new();
        for p in params {
            let ty = self.type_param_to_ty(&p.param_type);
            let c_ty = if p.is_ref {
                format!("{}*", self.c_type(&ty))
            } else {
                self.c_type(&ty)
            };
            p_strs.push(format!("{} {}", c_ty, p.name.replace("$", "_arg_")));
        }
        if p_strs.is_empty() {
            "void".to_string()
        } else {
            p_strs.join(", ")
        }
    }

    fn type_param_to_ty(&self, t: &Type) -> Ty {
        match t {
            Type::Named(path) => {
                let name = path.segments.last().unwrap();
                match name.as_str() {
                    "Int" => Ty::Int,
                    "Float32" | "Float64" => Ty::Float,
                    "String" => Ty::String,
                    "Bool" => Ty::Bool,
                    "Char" => Ty::Char,
                    _ => Ty::Struct(name.clone()),
                }
            }
            Type::Fn { params, ret } => Ty::Fn {
                params: params
                    .iter()
                    .map(|p| self.type_param_to_ty(p))
                    .collect(),
                ret: Box::new(
                    ret
                        .as_ref()
                        .map_or(Ty::Unit, |r|
                            self.type_param_to_ty(r.as_ref())
                        )
                )
            },
            _ => Ty::Unit,
        }
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

    fn type_to_prefix<'a>(&self, ty: &'a Ty) -> &'a str {
        match ty {
            Ty::Int => "int",
            Ty::Float => "float64",
            Ty::String => "string",
            Ty::Char => "char",
            Ty::Bool => "bool",
            Ty::Struct(name) => name.as_str(),
            _ => "unknown",
        }
    }

    fn does_block_return(body: &[TypedStmt]) -> bool {
        for s in body {
            match &s.kind {
                TypedStmtKind::Return { .. } => return true,
                TypedStmtKind::Expression(expr) => {
                    if let TypedExprKind::If {
                        then_block,
                        else_if,
                        else_block,
                        ..
                    } = &expr.kind
                    {
                        let does_return_if = Self::does_block_return(then_block);
                        let do_return_elifs = else_if.iter().all(|elif| Self::does_block_return(&elif.block));
                        let does_return_else = Self::does_block_return(else_block);

                        if does_return_if && do_return_elifs && does_return_else && !else_block.is_empty() {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
}
