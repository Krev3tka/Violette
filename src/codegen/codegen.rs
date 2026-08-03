use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::Unexpected;
use crate::lexer::token::Token;
use crate::parser::program::Program;
use crate::parser::statement::FunParam;
use crate::parser::{Expression, Statement};
use crate::typechecker::checker::Checker;
use crate::typechecker::env::Env;
use crate::typechecker::types::Ty;

pub struct Codegen {
    checker: Checker,
    env: Env,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            checker: Checker::default(),
            env: Env::default(),
        }
    }

    pub fn c_type(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Int => "int64_t".to_string(),
            Ty::Float => "double".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::String => "char*".to_string(),
            Ty::Struct(s) => format!("struct {}", s),
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
            "#include <stdint.h>\n".to_string(),

            "static void print_int(int64_t x)  { printf(\"%lld\", (long long)x); }".to_string(),
            "static void print_float(double x) { printf(\"%g\", x); }".to_string(),
            "static void print_string(char* s)    { printf(\"%s\", s); }".to_string(),
            "static void print_bool(bool b)    { printf(\"%s\", b ? \"true\" : \"false\"); }".to_string(),
            "static void println_int(int64_t x)  { printf(\"%lld\\n\", (long long)x); }".to_string(),
            "static void println_float(double x) { printf(\"%g\\n\", x); }".to_string(),
            "static void println_string(char* s) { printf(\"%s\\n\", s); }".to_string(),
            "static void println_bool(bool b)    { printf(\"%s\\n\", b ? \"true\" : \"false\"); }\n".to_string(),
        ];

        self.env.push();

        for s in &prg.declarations {
            if let Statement::Fun { name, params, return_type, .. } = s {
                let p: Vec<Ty> = params
                    .iter()
                    .map(|p| self.checker.resolve(&p.param_type))
                    .collect();
                let ret = return_type
                    .as_ref()
                    .map_or(Ty::Unit, |t| self.checker.resolve(t));
                self.env.define(
                    name.clone(),
                    Ty::Fn { params: p, ret: Box::new(ret) },
                );
            }
        }

        for s in &prg.declarations {
            if let Statement::Fun { name,
                body, ..
            } = s
                && name == "main"
            {
                lines.push("int main(void) {".to_string());

                lines.push(self.emit_block(body)?);

                lines.push("}".to_string());

                continue
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
                if f.fract() == 0.0 { format!("{f:.1}") } else { f.to_string() }
            },
            Expression::BoolLiteral(b) => b.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
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
                if operator == Token::Power {
                    return Ok(format!(
                        "pow({}, {})",
                        self.emit_expression(left.as_ref())?,
                        self.emit_expression(right.as_ref())?
                    ));
                }

                format!(
                    "{} {} {}",
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
                    && (name == "print" || name == "println") && args.len() == 1 {
                    let arg_ty = self.infer_expr(&args[0]);
                    let suffix = match arg_ty {
                        Ty::Int => "int",
                        Ty::Float => "float",
                        Ty::Bool => "bool",
                        Ty::String => "string",
                        _ => return Err(CodegenError::Unsupported("print for this type".to_string()))
                    };

                    let a = self.emit_expression(&args[0])?;

                    return Ok(format!("{name}_{suffix}({a})"))
                }

                let f = self.emit_expression(function.as_ref())?;
                let a = args
                    .iter()
                    .map(|arg| self.emit_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                format!("{}({})", f, a)
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

                let ty = self.infer_expr(value);

                self.env.define(name.clone(), ty.clone());

                format!("{} {} = {};", self.c_type(&ty), name, val_str)
            }
            // Statement::If(if_stmt) => {
            //     format("if ({}) {\n{}\n}",
            //            self.emit_expression(if_stmt.condition)?,
            //     self.emit_statement(if_stmt.then_block))
            // }
            // finish up later
            Statement::Return { value } => {
                let mut val_str = String::new();
                if let Some(expr) = value {
                    val_str = format!(" {}", self.emit_expression(expr)?);
                }

                format!("return{};", val_str)
            }
            Statement::Expression { expression } => {
                format!("{};", self.emit_expression(expression)?)
            }
            Statement::Fun { .. } => self.emit_function(stmt)?,
            _ => return Err(CodegenError::Unsupported(format!("{:?}", stmt))),
        })
    }

    pub fn emit_function(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        if let Statement::Fun {
            name,
            params,
            return_type,
            body,
        } = stmt
        {
            self.env.push();

            let param_tys: Vec<Ty> = params
                .iter()
                .map(|p| self.checker.resolve(&p.param_type))
                .collect();

            for (p, param_ty) in params.iter().zip(param_tys.iter()) {
                self.env.define(p.name.clone(), param_ty.clone())
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

            self.env.pop();

            if !params.is_empty() {
                Ok(format!("{ret} {name}({parameters}) {{\n{body_str}\n}}"))
            } else {
                Ok(format!("{ret} {name}(void) {{\n{body_str}\n}}"))
            }
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

            _ => return Err(Unexpected("Not an operator".to_string())),
        }))
    }

    fn infer_expr(&mut self, expr: &Expression) -> Ty {
        match expr {
            Expression::IntLiteral(_) => Ty::Int,
            Expression::FloatLiteral(_) => Ty::Float,
            Expression::BoolLiteral(_) => Ty::Bool,
            Expression::StringLiteral(_) => Ty::String,
            Expression::Identifier(name) => self.env.lookup(name).unwrap_or(Ty::Error),
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let left_ty = self.infer_expr(left);
                let right_ty = self.infer_expr(right);

                match operator {
                    Token::Add => match (&left_ty, &right_ty) {
                        (Ty::Int, Ty::Int) => Ty::Int,
                        (Ty::String, Ty::String) => Ty::String,
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => Ty::Error,
                    },
                    Token::Subtract | Token::Multiply | Token::Divide | Token::Modulus => Ty::Int,
                    Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => {
                        Ty::Bool
                    }
                    Token::Equals | Token::NotEquals => match (&left_ty, &right_ty) {
                        (Ty::Error, _) | (_, Ty::Error) => Ty::Error,
                        _ => {
                            if left_ty != right_ty {
                                return Ty::Error;
                            }
                            Ty::Bool
                        }
                    },
                    Token::LogicAnd | Token::LogicOr => Ty::Bool,
                    _ => Ty::Error,
                }
            }
            Expression::Prefix { operator, right: _ } => match operator {
                Token::Subtract | Token::Increment | Token::Decrement => Ty::Int,
                Token::LogicNot => Ty::Bool,
                _ => Ty::Error,
            },
            Expression::Index { .. } => Ty::Error,
            Expression::Call { function, args } => {
                let callee = self.infer_expr(function);
                match callee {
                    Ty::Fn { params, ret } => {
                        if args.len() != params.len() {
                            return *ret;
                        }

                        for (arg, _param) in args.iter().zip(params.iter()) {
                            let _a = self.infer_expr(arg);
                        }
                        *ret
                    }
                    Ty::Error => Ty::Error,
                    _ => Ty::Error,
                }
            }
            Expression::Field { object, name: _ } => self.infer_expr(object.as_ref()),
            Expression::MethodCall { .. } => Ty::Error,
            _ => Ty::Error,
        }
    }
}
