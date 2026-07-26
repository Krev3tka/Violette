use crate::lexer::token::{PrimitiveType, Token};
use crate::parser::Expression::{BoolLiteral, Identifier, IntLiteral, StringLiteral};
use crate::parser::statement::{ElseIf, FunParam, IfStatement, MatchArm};
use crate::parser::types::{Type, TypePath};
use crate::parser::{Expression, Statement};

pub fn ident(s: &str) -> Expression {
    Identifier(s.to_string())
}
pub fn int(n: isize) -> Expression {
    IntLiteral(n)
}
pub fn boolean(b: bool) -> Expression {
    BoolLiteral(b)
}
pub fn string(s: &str) -> Expression {
    StringLiteral(s.to_string())
}

pub fn prefix(op: Token, r: Expression) -> Expression {
    Expression::Prefix {
        operator: op,
        right: Box::new(r),
    }
}

pub fn infix(l: Expression, op: Token, r: Expression) -> Expression {
    Expression::Infix {
        left: Box::new(l),
        operator: op,
        right: Box::new(r),
    }
}

pub fn postfix(l: Expression, op: Token) -> Expression {
    Expression::Postfix {
        left: Box::new(l),
        operator: op,
    }
}

pub fn index(l: Expression, i: Expression) -> Expression {
    Expression::Index {
        left: Box::new(l),
        index: Box::new(i),
    }
}

pub fn call(f: Expression, args: Vec<Expression>) -> Expression {
    Expression::Call {
        function: Box::new(f),
        args,
    }
}

pub fn block(b: Vec<Statement>) -> Expression {
    Expression::Block { body: b }
}

pub fn match_expr(t: Expression, arms: Vec<MatchArm>) -> Expression {
    Expression::Match {
        target: Box::new(t),
        arms,
    }
}

pub fn field(o: Expression, name: &str) -> Expression {
    Expression::Field {
        object: Box::new(o),
        name: name.to_string(),
    }
}

pub fn method(o: Expression, name: &str, args: Vec<Expression>) -> Expression {
    Expression::MethodCall {
        object: Box::new(o),
        name: name.to_string(),
        args,
    }
}

pub fn lambda(params: Vec<FunParam>, ret: Option<Type>, body: Vec<Statement>) -> Expression {
    Expression::Lambda {
        params,
        return_type: ret,
        body,
    }
}

pub fn expr_stmt(expr: Expression) -> Statement {
    Statement::ExpressionStatement { expression: expr }
}

pub fn let_stmt(name: &str, val: Expression) -> Statement {
    Statement::Let {
        name: name.to_string(),
        value: val,
    }
}

pub fn const_stmt(name: &str, val: Expression) -> Statement {
    Statement::Const {
        name: name.to_string(),
        value: val,
    }
}

pub fn if_stmt(
    cond: Expression,
    then: Vec<Statement>,
    elif: Vec<ElseIf>,
    else_block: Vec<Statement>,
) -> Statement {
    Statement::If(IfStatement {
        condition: cond,
        then_block: then,
        else_if: elif,
        else_block,
    })
}

pub fn for_cond(condition: Expression, body: Vec<Statement>) -> Statement {
    Statement::ForCondition { condition, body }
}

pub fn for_range(var: &str, iter: Expression, body: Vec<Statement>) -> Statement {
    Statement::ForRange {
        variable: var.to_string(),
        iterable: iter,
        body,
    }
}

pub fn for_counter(
    init: Statement,
    cond: Expression,
    post: Expression,
    body: Vec<Statement>,
) -> Statement {
    Statement::ForCounter {
        init: Box::new(init),
        condition: cond,
        post,
        body,
    }
}

pub fn ret(v: Option<Expression>) -> Statement {
    Statement::Return { value: v }
}

pub fn fun(
    name: &str,
    params: Vec<FunParam>,
    ret: Option<Type>,
    body: Vec<Statement>,
) -> Statement {
    Statement::Fun {
        name: name.to_string(),
        params,
        return_type: ret,
        body,
    }
}

pub fn struct_def(name: &str, fields: Vec<FunParam>) -> Statement {
    Statement::Struct {
        name: name.to_string(),
        fields,
    }
}

pub fn ret_void() -> Statement {
    Statement::Return { value: None }
}

pub fn if_only(cond: Expression, block: Vec<Statement>) -> Statement {
    Statement::If(IfStatement {
        condition: cond,
        then_block: block,
        else_if: vec![],
        else_block: vec![],
    })
}

pub fn call_named(name: &str, args: Vec<Expression>) -> Expression {
    call(ident(name), args)
}

pub fn add(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Add, r)
}

pub fn sub(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Subtract, r)
}

pub fn mul(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Multiply, r)
}

pub fn div(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Divide, r)
}

pub fn lt(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Less, r)
}

pub fn gt(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Greater, r)
}

pub fn le(l: Expression, r: Expression) -> Expression {
    infix(l, Token::LessOrEquals, r)
}

pub fn ge(l: Expression, r: Expression) -> Expression {
    infix(l, Token::GreaterOrEquals, r)
}

pub fn eq(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Equals, r)
}

pub fn ne(l: Expression, r: Expression) -> Expression {
    infix(l, Token::NotEquals, r)
}

pub fn assign(l: Expression, r: Expression) -> Expression {
    infix(l, Token::Assign, r)
}

pub fn else_if(cond: Expression, b: Vec<Statement>) -> ElseIf {
    ElseIf {
        condition: cond,
        block: b,
    }
}
pub fn param(name: &str, t: Type) -> FunParam {
    FunParam {
        name: name.to_string(),
        param_type: t,
    }
}

pub fn arm(pattern: Expression, body: Expression) -> MatchArm {
    MatchArm { pattern, body }
}

pub fn t_int() -> Type {
    Type::Primitive(PrimitiveType::Int)
}

pub fn t_string() -> Type {
    Type::Primitive(PrimitiveType::String)
}

pub fn t_bool() -> Type {
    Type::Primitive(PrimitiveType::Bool)
}
pub fn t_prim(p: PrimitiveType) -> Type {
    Type::Primitive(p)
}

/// Одиночное имя — `t_named("User")`.
pub fn t_named(name: &str) -> Type {
    Type::Named(TypePath {
        segments: vec![name.to_string()],
    })
}

/// Путь через точки — `t_path(&["std", "vector"])`.
pub fn t_path(segments: &[&str]) -> Type {
    Type::Named(TypePath {
        segments: segments.iter().map(|s| s.to_string()).collect(),
    })
}

pub fn t_fn(params: Vec<Type>, ret: Type) -> Type {
    Type::Fn {
        params,
        ret: Some(Box::new(ret)),
    }
}

pub fn t_generic(name: &str, p: Type) -> Type {
    Type::Generic {
        name: name.to_string(),
        param: Box::new(p),
    }
}

pub fn t_union(variants: Vec<Type>) -> Type {
    Type::Union(variants)
}