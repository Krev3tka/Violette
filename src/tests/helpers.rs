#![allow(dead_code)]

use crate::lexer::lexer::Lexer;
use crate::lexer::span::{ClearSpan, Span};
use crate::lexer::token::Token;
use crate::parser::parser::Parser;
use crate::parser::statement::{ElseIf, FunParam, IfStatement, MatchArm};
use crate::parser::types::Type;
use crate::parser::{Expression, Precedence, Statement};

#[inline]
fn dummy_span() -> Span {
    Span::default()
}

pub fn assert_stmt_tests(test_cases: Vec<(&str, Statement)>) {
    for (input, expected) in test_cases {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let mut actual = parser.parse_statement().unwrap();

        actual.clear_span();

        assert_eq!(actual, expected, "Failing case: {}", input);
    }
}

pub fn assert_expr_tests(test_cases: Vec<(&str, Expression)>) {
    for (input, expected) in test_cases {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let mut actual = parser.parse_expression(Precedence::Lowest).unwrap();

        actual.clear_span();

        assert_eq!(actual, expected, "Failing case: {}", input);
    }
}

pub fn ident(s: &str) -> Expression {
    // Если Identifier — это tuple variant: Identifier(s.to_string(), dummy_span())
    // Если struct variant:
    Expression::Identifier {
        name: s.to_string(),
        span: dummy_span(),
    }
}

pub fn int(n: isize) -> Expression {
    Expression::IntLiteral {
        val: n,
        span: dummy_span(),
    }
}

pub fn boolean(b: bool) -> Expression {
    Expression::BoolLiteral {
        val: b,
        span: dummy_span(),
    }
}

pub fn string(s: &str) -> Expression {
    Expression::StringLiteral {
        val: s.to_string(),
        span: dummy_span(),
    }
}

pub fn prefix(op: Token, r: Expression) -> Expression {
    Expression::Prefix {
        operator: op,
        right: Box::new(r),
        span: dummy_span(),
    }
}

pub fn infix(l: Expression, op: Token, r: Expression) -> Expression {
    Expression::Infix {
        left: Box::new(l),
        operator: op,
        right: Box::new(r),
        span: dummy_span(),
    }
}

pub fn postfix(l: Expression, op: Token) -> Expression {
    Expression::Postfix {
        left: Box::new(l),
        operator: op,
        span: dummy_span(),
    }
}

pub fn index(l: Expression, i: Expression) -> Expression {
    Expression::Index {
        left: Box::new(l),
        index: Box::new(i),
        span: dummy_span(),
    }
}

pub fn call(f: Expression, args: Vec<Expression>) -> Expression {
    Expression::Call {
        function: Box::new(f),
        args,
        span: dummy_span(),
    }
}

pub fn block(b: Vec<Statement>) -> Expression {
    Expression::Block {
        body: b,
        span: dummy_span(),
    }
}

pub fn match_expr(t: Expression, arms: Vec<MatchArm>) -> Expression {
    Expression::Match {
        target: Box::new(t),
        arms,
        span: dummy_span(),
    }
}

pub fn expr_stmt(expr: Expression) -> Statement {
    Statement::Expression {
        expression: expr,
        span: dummy_span(),
    }
}

pub fn let_stmt(name: &str, value: Expression) -> Statement {
    Statement::Let {
        name: name.to_string(),
        value,
        span: dummy_span(),
    }
}

pub fn const_stmt(name: &str, value: Expression) -> Statement {
    Statement::Const {
        name: name.to_string(),
        value,
        span: dummy_span(),
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
        span: dummy_span(),
    })
}

pub fn while_loop(condition: Expression, body: Vec<Statement>) -> Statement {
    Statement::While {
        condition,
        body,
        span: dummy_span(),
    }
}

pub fn for_range(var: &str, iter: Expression, body: Vec<Statement>) -> Statement {
    Statement::ForRange {
        variable: var.to_string(),
        iterable: iter,
        body,
        span: dummy_span(),
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
        span: dummy_span(),
    }
}

pub fn ret(v: Option<Expression>) -> Statement {
    Statement::Return {
        value: v,
        span: dummy_span(),
    }
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
        span: dummy_span(),
        ending_span: dummy_span(),
    }
}

pub fn struct_def(name: &str, fields: Vec<FunParam>) -> Statement {
    Statement::Struct {
        name: name.to_string(),
        fields,
        span: dummy_span(),
    }
}
