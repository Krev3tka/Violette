#[cfg(test)]
mod expressions_tests {
    use crate::lexer::span::Span;
    use crate::lexer::token::Token;
    use crate::lexer::token::Token::{
        Add, Assign, Equals, Greater, LeftShift, LessOrEquals, LogicNot, Multiply, NotEquals,
        Power, RightShift, Subtract,
    };
    use crate::parser::Expression;
    use crate::parser::Expression::{Call, Identifier, IntLiteral};
    use crate::parser::statement::MatchArm;
    use crate::tests::helpers::{
        assert_expr_tests, assert_stmt_tests, block, boolean, call, expr_stmt, ident, infix, int,
        let_stmt, match_expr, prefix, string,
    };

    #[test]
    fn arithmetic_operations() {
        let test_cases = vec![
            ("5", int(5)),
            ("-5", prefix(Subtract, int(5))),
            ("!true", prefix(LogicNot, boolean(true))),
            (
                "(((5) ** (3)) ** (2))",
                infix(infix(int(5), Power, int(3)), Power, int(2)),
            ),
            (
                "a=b=5+5**5**2*-x==!true!=false",
                infix(
                    ident("a"),
                    Assign,
                    infix(
                        ident("b"),
                        Assign,
                        infix(
                            infix(
                                infix(
                                    int(5),
                                    Add,
                                    infix(
                                        infix(int(5), Power, infix(int(5), Power, int(2))),
                                        Multiply,
                                        prefix(Subtract, ident("x")),
                                    ),
                                ),
                                Equals,
                                prefix(LogicNot, boolean(true)),
                            ),
                            NotEquals,
                            boolean(false),
                        ),
                    ),
                ),
            ),
        ];
        assert_expr_tests(test_cases)
    }
    #[test]
    fn redshift_blueshift() {
        let test_cases = vec![
            (
                "let mask = 1 << 8",
                let_stmt("mask", infix(int(1), LeftShift, int(8))),
            ),
            (
                "let res = base + 2 >> offset - 1",
                let_stmt(
                    "res",
                    infix(
                        infix(ident("base"), Add, int(2)),
                        RightShift,
                        infix(ident("offset"), Subtract, int(1)),
                    ),
                ),
            ),
        ];
        assert_stmt_tests(test_cases)
    }
    #[test]
    fn matching_souls() {
        let test_cases = vec![
            (
                "let user = match res {
                    Win(u) => u,
                    Fail(r) => {
                        println(\"Error: \" + r)
                        NewUser()
                    }
                }",
                let_stmt(
                    "user",
                    match_expr(
                        ident("res"),
                        vec![
                            MatchArm {
                                pattern: call(ident("Win"), vec![ident("u")]),
                                body: ident("u"),
                                span: Span::default(),
                            },
                            MatchArm {
                                pattern: call(ident("Fail"), vec![ident("r")]),
                                body: block(vec![
                                    expr_stmt(call(
                                        ident("println"),
                                        vec![infix(string("Error: "), Add, ident("r"))],
                                    )),
                                    expr_stmt(call(ident("NewUser"), Vec::new())),
                                ]),
                                span: Span::default(),
                            },
                        ],
                    ),
                ),
            ),
            (
                "match res {
    Win(num) => {
        match num {
            num > 5 => true,
            num <= 5 => false,
        }
    }
    Fail(r) => print(r)
}",
                expr_stmt(match_expr(
                    ident("res"),
                    vec![
                        MatchArm {
                            pattern: call(ident("Win"), vec![ident("num")]),
                            body: block(vec![expr_stmt(match_expr(
                                ident("num"),
                                vec![
                                    MatchArm {
                                        pattern: infix(ident("num"), Greater, int(5)),
                                        body: boolean(true),
                                        span: Span::default(),
                                    },
                                    MatchArm {
                                        pattern: infix(ident("num"), LessOrEquals, int(5)),
                                        body: boolean(false),
                                        span: Span::default(),
                                    },
                                ],
                            ))]),
                            span: Span::default(),
                        },
                        MatchArm {
                            pattern: call(ident("Fail"), vec![ident("r")]),
                            body: call(ident("print"), vec![ident("r")]),
                            span: Span::default(),
                        },
                    ],
                )),
            ),
        ];
        assert_stmt_tests(test_cases)
    }
    #[test]
    fn logic() {
        let test_cases = vec![
            (
                "a || b && c",
                Expression::Infix {
                    left: Box::new(Identifier {
                        name: "a".to_string(),
                        span: Span::default(),
                    }),
                    operator: Token::LogicOr,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Identifier {
                            name: "b".to_string(),
                            span: Span::default(),
                        }),
                        operator: Token::LogicAnd,
                        right: Box::new(Identifier {
                            name: "c".to_string(),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
            ),
            (
                "1 # 2 ^ 3 & 4",
                Expression::Infix {
                    left: Box::new(IntLiteral {
                        val: 1,
                        span: Span::default(),
                    }),
                    operator: Token::BitOr,
                    right: Box::new(Expression::Infix {
                        left: Box::new(IntLiteral {
                            val: 2,
                            span: Span::default(),
                        }),
                        operator: Token::BitXOR,
                        right: Box::new(Expression::Infix {
                            left: Box::new(IntLiteral {
                                val: 3,
                                span: Span::default(),
                            }),
                            operator: Token::BitAnd,
                            right: Box::new(IntLiteral {
                                val: 4,
                                span: Span::default(),
                            }),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
            ),
            (
                "~x",
                Expression::Prefix {
                    operator: Token::BitNot,
                    right: Box::new(ident("x")),
                    span: Span::default(),
                },
            ),
        ];
        assert_expr_tests(test_cases)
    }
    #[test]
    fn piper() {
        let input = "match res {Win(v) => fetch(v)|, Fail(e) => e}";
        let expected = Expression::Match {
            target: Box::new(ident("res")),
            arms: vec![
                MatchArm {
                    pattern: Call {
                        function: Box::new(ident("Win")),
                        args: vec![ident("v")],
                        span: Span::default(),
                    },
                    body: Expression::Postfix {
                        left: Box::new(Call {
                            function: Box::new(ident("fetch")),
                            args: vec![ident("v")],
                            span: Span::default(),
                        }),
                        operator: Token::Pipe,
                        span: Span::default(),
                    },
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Call {
                        function: Box::new(ident("Fail")),
                        args: vec![ident("e")],
                        span: Span::default(),
                    },
                    body: ident("e"),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        assert_expr_tests(vec![(input, expected)]);
    }
}
