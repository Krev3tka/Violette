use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::Unexpected;
use crate::lexer::token::Token;
use crate::parser::program::Program;
use crate::parser::statement::{FunParam, IfStatement, StructParam};
use crate::parser::{Expression, Statement};
use crate::parser::expression::RangeKind;
use crate::typechecker::checker::Checker;
use crate::typechecker::error::BindingKind;
use crate::typechecker::types::Ty;

pub struct Codegen {
    checker: Checker,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            checker: Checker::default(),
        }
    }

    pub fn c_type(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Int => "int64_t".to_string(),
            Ty::Float => "double".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::String => "char*".to_string(),
            Ty::Struct(s) => s.to_string(),
            Ty::Unit => "void".to_string(),
            Ty::Fn { .. } => todo!(),
            _ => "unknown".to_string(),
        }
    }

    pub fn emit_program(&mut self, prg: Program) -> Result<String, CodegenError> {
        let mut lines: Vec<String> = vec![
            "#include <stdio.h>".to_string(),
            "#include <math.h>".to_string(),
            "#include <stdbool.h>".to_string(),
            "#include <stdint.h>".to_string(),
            "#include <string.h>".to_string(),
            "#include <stdlib.h>\n".to_string(),
        ];

        let mut global_defines: Vec<String> = Vec::new();

        self.checker.collect_signatures(&prg.declarations);

        self.checker.env.push();

        for s in &prg.declarations {
            if let Statement::Const { name, value } = s {
                let val_str = self.emit_expression(value)?;
                let ty = self.checker.infer(value);

                self.checker.defined(name.clone(), ty, BindingKind::Const);

                global_defines.push(format!("#define {} {}", name, val_str))
            }
            if let Statement::Fun {
                name,
                params,
                return_type,
                ..
            } = s
            {
                let p: Vec<Ty> = params
                    .iter()
                    .map(|p| self.checker.resolve(&p.param_type))
                    .collect();

                let ret = return_type
                    .as_ref()
                    .map_or(Ty::Unit, |t| self.checker.resolve(t));

                let fn_ty = Ty::Fn {
                    params: p,
                    ret: Box::new(ret),
                };

                self.checker
                    .defined(name.clone(), fn_ty.clone(), BindingKind::Var);
            }
        }

        if !global_defines.is_empty() {
            lines.extend(global_defines);
            lines.push("\n".to_string());
        }

        lines.extend(vec![
            "static void vio_print_int(int64_t x)  { printf(\"%lld\", (long long)x); }".to_string(),
            "static void vio_print_float(double x) { printf(\"%g\", x); }".to_string(),
            "static void vio_print_string(char* s)    { printf(\"%s\", s); }".to_string(),
            "static void vio_print_bool(bool b)    { printf(\"%s\", b ? \"true\" : \"false\"); }".to_string(),
            "static void vio_println_int(int64_t x)  { printf(\"%lld\\n\", (long long)x); }".to_string(),
            "static void vio_println_float(double x) { printf(\"%g\\n\", x); }".to_string(),
            "static void vio_println_string(char* s) { printf(\"%s\\n\", s); }".to_string(),
            "static void vio_println_bool(bool b)    { printf(\"%s\\n\", b ? \"true\" : \"false\"); }\n".to_string(),
            "const char* vio_str_concat(const char* s1, const char* s2) {
    size_t len1 = strlen(s1);
    size_t len2 = strlen(s2);
    char* result = malloc(len1 + len2 + 1);

    if (result == NULL) {
        return NULL;
    }

    strcpy(result, s1);
    strcat(result, s2);

    return result;
}\n".to_string()
        ]);

        for s in &prg.declarations {
            if let Statement::Const { .. } = s {
                continue;
            }
            if let Statement::Fun { name, body, .. } = s
                && name == "main"
            {
                lines.push("int main(void) {".to_string());

                lines.push(self.emit_block(body)?);

                lines.push("}".to_string());

                continue;
            }
            let stmt = self.emit_statement(s)?;
            for line in stmt.lines() {
                lines.push(line.to_string());
            }
        }

        if !prg.main.is_empty() {
            lines.push("int main(void) {".to_string());
            lines.push(self.emit_block(&prg.main)?);
            lines.push("}".to_string());
        }

        let mut res = lines.join("\n");

        res.push('\n');

        Ok(res)
    }

    pub fn emit_expression(&mut self, expr: &Expression) -> Result<String, CodegenError> {
        Ok(match expr.clone() {
            Expression::IntLiteral(i) => i.to_string(),
            Expression::FloatLiteral(f) => {
                if f.fract() == 0.0 {
                    format!("{f:.1}")
                } else {
                    f.to_string()
                }
            }
            Expression::BoolLiteral(b) => b.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::StructLiteral { name, fields } => {
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
            Expression::Prefix { operator, right } => {
                format!(
                    "{}{}",
                    self.correlate_operator(&operator)?,
                    self.emit_expression(right.as_ref())?
                )
            }
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                if matches!(operator, Token::Add)
                    && matches!(self.checker.infer(left.as_ref()), Ty::String)
                    && matches!(self.checker.infer(right.as_ref()), Ty::String)
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
            Expression::Postfix { left, operator } => {
                format!(
                    "{}{}",
                    self.emit_expression(left.as_ref())?,
                    self.correlate_operator(&operator)?
                )
            }
            Expression::Identifier(ident) => ident,
            Expression::Call { function, args } => {
                if let Expression::Identifier(name) = function.as_ref()
                    && (name == "print" || name == "println")
                    && args.len() == 1
                {
                    let arg_ty = self.checker.infer(&args[0]);
                    let suffix = match arg_ty {
                        Ty::Int => "int",
                        Ty::Float => "float",
                        Ty::Bool => "bool",
                        Ty::String => "string",
                        _ => {
                            return Err(CodegenError::Unsupported(
                                "print for this type".to_string(),
                            ));
                        }
                    };

                    let a = self.emit_expression(&args[0])?;

                    return Ok(format!("vio_{name}_{suffix}({a})"));
                }

                let f = self.emit_expression(function.as_ref())?;
                let a = args
                    .iter()
                    .map(|arg| self.emit_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                format!("{}({})", f, a)
            }
            Expression::Field { object, name } => {
                format!("{}.{}", self.emit_expression(object.as_ref())?, name)
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
            Statement::Let { name, value } | Statement::Const { name, value } => {
                let val_str = self.emit_expression(value)?;

                let ty = self.checker.infer(value);

                let mut res = String::new();

                if matches!(stmt, Statement::Const { .. }) {
                    res.push_str("const ");
                    self.checker
                        .defined(name.clone(), ty.clone(), BindingKind::Const);
                } else {
                    self.checker
                        .defined(name.clone(), ty.clone(), BindingKind::Let);
                }

                res.push_str(format!("{} {} = {};", self.c_type(&ty), name, val_str).as_str());

                res
            }
            Statement::If(IfStatement {
                condition,
                then_block,
                else_if,
                else_block,
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
            Statement::Return { value } => {
                let mut val_str = String::new();
                if let Some(expr) = value {
                    val_str = format!(" {}", self.emit_expression(expr)?);
                }

                format!("return {};", val_str)
            }
            Statement::Expression { expression } => {
                format!("{};", self.emit_expression(expression)?)
            }
            Statement::ForCondition { .. } | Statement::ForCounter { .. } | Statement::ForRange { .. }  => self.emit_for(stmt)?,
            Statement::Fun { .. } => self.emit_function(stmt)?,
            Statement::Struct { .. } => self.emit_struct(stmt)?,
            _ => return Err(CodegenError::Unsupported(format!("{:?}", stmt))),
        })
    }

    pub fn emit_for(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::ForCondition { condition, body } = stmt {
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
            body
        } = stmt {
            self.checker.env.push();

            let (start, end, cmp) = match iterable {
                Expression::Range {
                    start,
                    end,
                    range_kind
                } => (
                    start.as_ref().map_or(Ok("0".to_string()), |s| self.emit_expression(s))?,
                    end.as_ref().map_or(Err(CodegenError::Unsupported("Open ranges".to_string())), |e| self.emit_expression(e))?,
                    if matches!(range_kind, RangeKind::Inclusive) { "<=" } else { "<" }
                    ),
                _ => return Err(CodegenError::Unsupported("For-loop expects a range".to_string()))
            };

            self.checker.defined(variable.clone(), Ty::Int, BindingKind::Var);
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
        if let Statement::Fun {
            name,
            params,
            return_type,
            body,
        } = stmt
        {
            self.checker.env.push();

            let param_tys: Vec<Ty> = params
                .iter()
                .map(|p| self.checker.resolve(&p.param_type))
                .collect();

            for (p, param_ty) in params.iter().zip(param_tys.iter()) {
                self.checker
                    .defined(p.name.clone(), param_ty.clone(), BindingKind::Var)
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
                .map(|FunParam { name, param_type }| {
                    let ty = self.checker.resolve(param_type);
                    format!("{} {}", self.c_type(&ty), name.clone())
                })
                .collect::<Vec<String>>()
                .join(", ");

            let body_str = self.emit_block(body)?;

            self.checker.env.pop();

            if !params.is_empty() {
                Ok(format!("{ret} {name}({parameters}) {{\n{body_str}\n}}"))
            } else {
                Ok(format!("{ret} {name}(void) {{\n{body_str}\n}}"))
            }
        } else {
            Err(Unexpected(format!("{:?}", stmt)))
        }
    }

    pub fn emit_struct(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::Struct { name, fields } = stmt {
            self.checker.env.push();

            let field_tys: Vec<Ty> = fields
                .iter()
                .map(|p| self.checker.resolve(&p.param_type))
                .collect();

            for (f, field_ty) in fields.iter().zip(field_tys.iter()) {
                self.checker
                    .defined(f.name.clone(), field_ty.clone(), BindingKind::Var)
            }

            let mut c_fields = fields
                .iter()
                .map(
                    |StructParam {
                         name: n,
                         param_type,
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

        for s in body {
            let stmt = self.emit_statement(s)?;
            for line in stmt.lines() {
                lines.push(format!("    {line}"));
            }
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

            Token::BitAnd => "&",
            Token::BitOr => "|",
            Token::BitXOR => "^",
            Token::BitNot => "~",

            _ => return Err(Unexpected("Not an operator".to_string())),
        }))
    }
}
