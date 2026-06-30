#[cfg(test)]
pub mod statement_tests {
    use crate::lexer::{Lexer, Token};
    use crate::parser::{ElseIf, Expression, IfStatement, Statement};
    use crate::parser::Expression::{Identifier, IntLiteral};
    use crate::parser::parser::Parser;

    fn ident(s: &str) -> Expression { Expression::Identifier(s.to_string()) }
    fn int(n: isize) -> Expression { Expression::IntLiteral(n) }
    fn infix(l: Expression, op: Token, r: Expression) -> Expression {
        Expression::Infix { left: Box::new(l), operator: op, right: Box::new(r) }
    }
    fn let_stmt(name: &str, val: Expression) -> Statement {
        Statement::Let { name: name.to_string(), value: val }
    }

    #[test]
    fn let_the_speed_mend_it() {
        let test_cases = vec![
            (
                "let x = 5",
                Statement::Let {
                    name: "x".to_string(),
                    value: Expression::IntLiteral(5)
                }
            ),
            (
                "const THREE_HOURS_IN_SECONDS = 3 * 24 * 60 ** 2",
                Statement::Const {
                    name: "THREE_HOURS_IN_SECONDS".to_string(),
                    value: Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::IntLiteral(3)),
                            operator: Token::Multiply,
                            right: Box::new(Expression::IntLiteral(24)),
                        }),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::IntLiteral(60)),
                            operator: Token::Power,
                            right: Box::new(Expression::IntLiteral(2)),
                        })
                    }
                }
            ),

            (
                "\
if a > 7 {
    let b = a - 3
} else if a < 3 {
    let b = a + 4
} else if a < 5 {
    let b = a - 2
} else {
    let b = a + 5
}",
                Statement::If(IfStatement {
                    condition: infix(ident("a"), Token::Greater, int(7)),
                    then_block: vec![
                        let_stmt("b", infix(ident("a"), Token::Subtract, int(3))),
                    ],
                    else_if: vec![
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(3)),
                            block: vec![
                                let_stmt("b", infix(ident("a"), Token::Add, int(4))),
                            ],
                        },
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(5)),
                            block: vec![
                                let_stmt("b", infix(ident("a"), Token::Subtract, int(2))),
                            ],
                        },
                    ],
                    else_block: Some(vec![
                        let_stmt("b", infix(ident("a"), Token::Add, int(5))),
                    ]),
                })
            )
        ];

        test_cases.into_iter().for_each(|(input, expected)| {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_statement();

            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input);
        });
    }

    #[test]
    fn for_whose_advantage() {
        let test_cases = vec![
            (
                "for i = 0; i < 10; i++ {
    let a = 5 * i
    let b = i * 3
}",

                Statement::ForCounter {
                    init: Box::new(Statement::Let {
                        name: "i".to_string(),
                        value: Expression::IntLiteral(0)
                    }),
                    condition: Expression::Infix {
                        left: Box::new(Identifier("i".to_string())),
                        operator: Token::Less,
                        right: Box::new(IntLiteral(10))
                    },
                    post: Expression::Postfix {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        operator: Token::Increment
                    },
                    body: vec![
                        Statement::Let {
                            name: "a".to_string(),
                            value: Expression::Infix {
                                left: Box::new(IntLiteral(5)),
                                operator: Token::Multiply,
                                right: Box::new(Identifier("i".to_string()))
                            }
                        },
                        Statement::Let {
                            name: "b".to_string(),
                            value: Expression::Infix {
                                left: Box::new(Identifier("i".to_string())),
                                operator: Token::Multiply,
                                right: Box::new(IntLiteral(3))
                            }
                        },
                    ]
                }
            ),
            (
                "for x in thru(1, 10) {
    let a = 5 * x
    let b = x * 3
}",
                Statement::ForRange {
                    variable: "x".to_string(),
                    iterable: Expression::Call {
                        function: Box::new(Expression::Identifier("thru".to_string())),
                        args: vec![
                            Expression::IntLiteral(1),
                            Expression::IntLiteral(10),
                        ]
                    },
                    body: vec![
                        Statement::Let {
                            name: "a".to_string(),
                            value: Expression::Infix {
                                left: Box::new(IntLiteral(5)),
                                operator: Token::Multiply,
                                right: Box::new(Identifier("x".to_string()))
                            }
                        },
                        Statement::Let {
                            name: "b".to_string(),
                            value: Expression::Infix {
                                left: Box::new(Identifier("x".to_string())),
                                operator: Token::Multiply,
                                right: Box::new(IntLiteral(3))
                            }
                        },
                    ]
                }
            ),
            (
                "for left < right {
    left++
    right--
}",
                Statement::ForCondition {
                    condition: Expression::Infix {
                        left: Box::new(Identifier("left".to_string())),
                        operator: Token::Less,
                        right: Box::new(Identifier("right".to_string())),
                    },
                    body: vec![
                        Statement::ExpressionStatement {
                            expression: Expression::Postfix {
                                left: Box::new(Identifier("left".to_string())),
                                operator: Token::Increment
                            }
                        },
                        Statement::ExpressionStatement {
                            expression: Expression::Postfix {
                                left: Box::new(Identifier("right".to_string())),
                                operator: Token::Decrement
                            }
                        }
                    ]
                }
            )
        ];

        test_cases.into_iter().for_each(|(input, expected)| {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_statement();

            assert_eq!(actual.unwrap(), expected, "Failing case: {}", input)
        })
    }
}