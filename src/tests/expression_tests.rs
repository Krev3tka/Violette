#[cfg(test)]
mod expressions_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Token;
    use crate::lexer::token::Token::{
        Add, Assign, Equals, Greater, LeftShift, LessOrEquals, LogicNot, Multiply, NotEquals,
        Power, RightShift, Subtract,
    };
    use crate::parser::Expression::{Call, Identifier, IntLiteral};
    use crate::parser::parser::Parser;
    use crate::parser::statement::MatchArm;
    use crate::parser::{Expression, Precedence};
    use crate::tests::helpers::{
        block, boolean, call, expr_stmt, ident, infix, int, let_stmt, match_expr, prefix, string,
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
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_expression(Precedence::Lowest);
            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
        }
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
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_statement();
            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
        }
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
                                    },
                                    MatchArm {
                                        pattern: infix(ident("num"), LessOrEquals, int(5)),
                                        body: boolean(false),
                                    },
                                ],
                            ))]),
                        },
                        MatchArm {
                            pattern: call(ident("Fail"), vec![ident("r")]),
                            body: call(ident("print"), vec![ident("r")]),
                        },
                    ],
                )),
            ),
        ];
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_statement();
            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
        }
    }
    #[test]
    fn logic() {
        let test_cases = vec![
            (
                "a || b && c",
                Expression::Infix {
                    left: Box::new(Identifier("a".to_string())),
                    operator: Token::LogicOr,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Identifier("b".to_string())),
                        operator: Token::LogicAnd,
                        right: Box::new(Identifier("c".to_string())),
                    }),
                },
            ),
            (
                "1 # 2 ^ 3 & 4",
                Expression::Infix {
                    left: Box::new(IntLiteral(1)),
                    operator: Token::BitOr,
                    right: Box::new(Expression::Infix {
                        left: Box::new(IntLiteral(2)),
                        operator: Token::BitXOR,
                        right: Box::new(Expression::Infix {
                            left: Box::new(IntLiteral(3)),
                            operator: Token::BitAnd,
                            right: Box::new(IntLiteral(4)),
                        }),
                    }),
                },
            ),
            (
                "~x",
                Expression::Prefix {
                    operator: Token::BitNot,
                    right: Box::new(Identifier("x".to_string())),
                },
            ),
        ];
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_expression(Precedence::Lowest);
            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
        }
    }
    #[test]
    fn piper() {
        let input = "match res {Win(v) => fetch(v)|, Fail(e) => e}";
        let expected = Expression::Match {
            target: Box::new(Identifier("res".to_string())),
            arms: vec![
                MatchArm {
                    pattern: Call {
                        function: Box::new(Identifier("Win".to_string())),
                        args: vec![Identifier("v".to_string())],
                    },
                    body: Expression::Postfix {
                        left: Box::new(Call {
                            function: Box::new(Identifier("fetch".to_string())),
                            args: vec![Identifier("v".to_string())],
                        }),
                        operator: Token::Pipe,
                    },
                },
                MatchArm {
                    pattern: Call {
                        function: Box::new(Identifier("Fail".to_string())),
                        args: vec![Identifier("e".to_string())],
                    },
                    body: Identifier("e".to_string()),
                },
            ],
        };
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let actual = parser.parse_expression(Precedence::Lowest);
        assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
    }
}
