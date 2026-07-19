#[cfg(test)]
mod expression_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Token;
    use crate::parser::Expression::{BoolLiteral, Call, Identifier, IntLiteral};
    use crate::parser::parser::Parser;
    use crate::parser::statement::MatchArm;
    use crate::parser::{Expression, Precedence, Statement};
    #[test]
    fn arithmetic_operations() {
        let test_cases = vec![
            ("5", IntLiteral(5)),
            (
                "-5",
                Expression::Prefix {
                    operator: Token::Subtract,
                    right: Box::new(IntLiteral(5)),
                },
            ),
            (
                "!true",
                Expression::Prefix {
                    operator: Token::LogicNot,
                    right: Box::new(BoolLiteral(true)),
                },
            ),
            (
                "(((5) ** (3)) ** (2))",
                Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(IntLiteral(5)),
                        operator: Token::Power,
                        right: Box::new(IntLiteral(3)),
                    }),
                    operator: Token::Power,
                    right: Box::new(IntLiteral(2)),
                },
            ),
            (
                "a=b=5+5**5**2*-x==!true!=false",
                Expression::Infix {
                    left: Box::new(Identifier("a".to_string())),
                    operator: Token::Assign,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Identifier("b".to_string())),
                        operator: Token::Assign,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Infix {
                                left: Box::new(Expression::Infix {
                                    left: Box::new(IntLiteral(5)),
                                    operator: Token::Add,
                                    right: Box::new(Expression::Infix {
                                        left: Box::new(Expression::Infix {
                                            left: Box::new(IntLiteral(5)),
                                            operator: Token::Power,
                                            right: Box::new(Expression::Infix {
                                                left: Box::new(IntLiteral(5)),
                                                operator: Token::Power,
                                                right: Box::new(IntLiteral(2)),
                                            }),
                                        }),
                                        operator: Token::Multiply,
                                        right: Box::new(Expression::Prefix {
                                            operator: Token::Subtract,
                                            right: Box::new(Identifier("x".to_string())),
                                        }),
                                    }),
                                }),
                                operator: Token::Equals,
                                right: Box::new(Expression::Prefix {
                                    operator: Token::LogicNot,
                                    right: Box::new(BoolLiteral(true)),
                                }),
                            }),
                            operator: Token::NotEquals,
                            right: Box::new(BoolLiteral(false)),
                        }),
                    }),
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
    fn redshift_blueshift() {
        let test_cases = vec![
            (
                "let mask = 1 << 8",
                Statement::Let {
                    name: "mask".to_string(),
                    value: Expression::Infix {
                        left: Box::new(IntLiteral(1)),
                        operator: Token::LeftShift,
                        right: Box::new(IntLiteral(8)),
                    },
                },
            ),
            (
                "let res = base + 2 >> offset - 1",
                Statement::Let {
                    name: "res".to_string(),
                    value: Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Identifier("base".to_string())),
                            operator: Token::Add,
                            right: Box::new(IntLiteral(2)),
                        }),
                        operator: Token::RightShift,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Identifier("offset".to_string())),
                            operator: Token::Subtract,
                            right: Box::new(IntLiteral(1)),
                        }),
                    },
                },
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
                Statement::Let {
                    name: "user".to_string(),
                    value: Expression::Match {
                        target: Box::new(Identifier("res".to_string())),
                        arms: vec![
                            MatchArm {
                                pattern: Call {
                                    function: Box::new(Identifier("Win".to_string())),
                                    args: vec![Identifier("u".to_string())],
                                },
                                body: Identifier("u".to_string()),
                            },
                            MatchArm {
                                pattern: Call {
                                    function: Box::new(Identifier("Fail".to_string())),
                                    args: vec![Identifier("r".to_string())],
                                },
                                body: Expression::Block {
                                    body: vec![
                                        Statement::ExpressionStatement {
                                            expression: Call {
                                                function: Box::new(Identifier(
                                                    "println".to_string(),
                                                )),
                                                args: vec![Expression::Infix {
                                                    left: Box::new(Expression::StringLiteral(
                                                        "Error: ".to_string(),
                                                    )),
                                                    operator: Token::Add,
                                                    right: Box::new(Identifier("r".to_string())),
                                                }],
                                            },
                                        },
                                        Statement::ExpressionStatement {
                                            expression: Call {
                                                function: Box::new(Identifier(
                                                    "NewUser".to_string(),
                                                )),
                                                args: Vec::new(),
                                            },
                                        },
                                    ],
                                },
                            },
                        ],
                    },
                },
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
                Statement::ExpressionStatement {
                    expression: Expression::Match {
                        target: Box::new(Identifier("res".to_string())),
                        arms: vec![
                            MatchArm {
                                pattern: Call {
                                    function: Box::new(Identifier("Win".to_string())),
                                    args: vec![Identifier("num".to_string())],
                                },
                                body: Expression::Block {
                                    body: vec![Statement::ExpressionStatement {
                                        expression: Expression::Match {
                                            target: Box::new(Identifier("num".to_string())),
                                            arms: vec![
                                                MatchArm {
                                                    pattern: Expression::Infix {
                                                        left: Box::new(Identifier(
                                                            "num".to_string(),
                                                        )),
                                                        operator: Token::Greater,
                                                        right: Box::new(IntLiteral(5)),
                                                    },
                                                    body: BoolLiteral(true),
                                                },
                                                MatchArm {
                                                    pattern: Expression::Infix {
                                                        left: Box::new(Identifier(
                                                            "num".to_string(),
                                                        )),
                                                        operator: Token::LessOrEquals,
                                                        right: Box::new(IntLiteral(5)),
                                                    },
                                                    body: BoolLiteral(false),
                                                },
                                            ],
                                        },
                                    }],
                                },
                            },
                            MatchArm {
                                pattern: Call {
                                    function: Box::new(Identifier("Fail".to_string())),
                                    args: vec![Identifier("r".to_string())],
                                },
                                body: Call {
                                    function: Box::new(Identifier("print".to_string())),
                                    args: vec![Identifier("r".to_string())],
                                },
                            },
                        ],
                    },
                },
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
