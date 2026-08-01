use crate::codegen::error::CodegenError;
use crate::codegen::error::CodegenError::Unexpected;
use crate::lexer::token::Token;
use crate::parser::statement::FunParam;
use crate::parser::{Expression, Statement};
use crate::typechecker::checker::Checker;
use crate::typechecker::env::Env;
use crate::typechecker::types::Ty;

pub struct Codegen {
    pub out: String,
    indent: usize,
    temp_counter: u32,
    checker: Checker,
    env: Env,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            out: String::from(
                "#include <stdint.h>\n#include <stdbool.h>\n#include <stdio.h>\n#include <math.h>\n\n",
            ),
            indent: 0,
            temp_counter: 0,
            checker: Checker::default(),
            env: Env::default(),
        }
    }

    pub fn emit(&mut self, s: &str) {
        self.out
            .push_str(format!("{}{}\n", " ".repeat(self.indent * 4), s).as_str());
    }

    pub fn fresh_temp(&mut self) -> String {
        let res = format!("_tmp{}", self.temp_counter);
        self.temp_counter += 1;

        res
    }

    pub fn c_type(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Int => "int64_t".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::String => "char*".to_string(),
            Ty::Struct(s) => format!("struct {}", s),
            Ty::Unit => "void".to_string(),
            Ty::Fn { params, ret } => format!(
                "{} (*{})({})",
                self.c_type(ret.as_ref()),
                self.fresh_temp(),
                params
                    .iter()
                    .map(|x| { self.c_type(x) })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            _ => "unknown".to_string(),
        }
    }

    pub fn emit_expression(&mut self, expr: &Expression) -> Result<String, CodegenError> {
        let res = match expr.clone() {
            Expression::IntLiteral(i) => i.to_string(),
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
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "this expression: {:?}",
                    expr
                )));
            }
        };

        self.emit(res.as_str());

        Ok(res)
    }

    pub fn emit_statement(&mut self, stmt: &Statement) -> Result<String, CodegenError> {
        let res = match stmt {
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
            Statement::ExpressionStatement { expression } => {
                format!("{};", self.emit_expression(expression)?)
            }
            Statement::Fun {
                ..
            } => {
                self.emit_function(stmt)?;

                String::new()
            }
            _ => return Err(CodegenError::Unsupported(format!("{:?}", stmt))),
        };

        self.emit(res.as_str());

        Ok(res)
    }

    pub fn emit_function(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
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

            let mut ret = String::from("void");

            if let Some(ty) = return_type {
                let ty = self.checker.resolve(ty);
                ret = self.c_type(&ty)
            }

            let res = format!(
                "{} {}({}) {{",
                ret,
                name,
                params
                    .iter()
                    .map(|FunParam { name, param_type }| {
                        let ty = self.checker.resolve(param_type);
                        format!("{} {}", self.c_type(&ty), name.clone())
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            self.emit(res.as_str());

            self.indent += 1;

            for s in body {
                let stmt_str = self.emit_statement(s)?;
                self.emit(stmt_str.as_str());
            }

            self.indent -= 1;

            self.emit("}");

            self.env.pop();
        }

        Ok(())
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
