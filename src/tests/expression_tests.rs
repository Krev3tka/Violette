#[cfg(test)]
mod expression_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Token;
    use crate::parser::parser::Parser;
    use crate::parser::{Expression, Precedence, Statement};

    #[test]
    fn high_five_ghost() {
        let test_cases = vec![
            ("5", Expression::IntLiteral(5)),
            (
                "-5",
                Expression::Prefix {
                    operator: Token::Subtract,
                    right: Box::new(Expression::IntLiteral(5)),
                },
            ),
            (
                "!true",
                Expression::Prefix {
                    operator: Token::LogicNot,
                    right: Box::new(Expression::BoolLiteral(true)),
                },
            ),
            (
                "(((5) ** (3)) ** (2))",
                Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::IntLiteral(5)),
                        operator: Token::Power,
                        right: Box::new(Expression::IntLiteral(3)),
                    }),
                    operator: Token::Power,
                    right: Box::new(Expression::IntLiteral(2)),
                },
            ),
            (
                "a=b=5+5**5**2*-x==!true!=false",
                Expression::Infix {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: Token::Assign,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Identifier("b".to_string())),
                        operator: Token::Assign,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Infix {
                                left: Box::new(Expression::Infix {
                                    left: Box::new(Expression::IntLiteral(5)),
                                    operator: Token::Add,
                                    right: Box::new(Expression::Infix {
                                        left: Box::new(Expression::Infix {
                                            left: Box::new(Expression::IntLiteral(5)),
                                            operator: Token::Power,
                                            right: Box::new(Expression::Infix {
                                                left: Box::new(Expression::IntLiteral(5)),
                                                operator: Token::Power,
                                                right: Box::new(Expression::IntLiteral(2)),
                                            }),
                                        }),
                                        operator: Token::Multiply,
                                        right: Box::new(Expression::Prefix {
                                            operator: Token::Subtract,
                                            right: Box::new(Expression::Identifier(
                                                "x".to_string(),
                                            )),
                                        }),
                                    }),
                                }),
                                operator: Token::Equals,
                                right: Box::new(Expression::Prefix {
                                    operator: Token::LogicNot,
                                    right: Box::new(Expression::BoolLiteral(true)),
                                }),
                            }),
                            operator: Token::NotEquals,
                            right: Box::new(Expression::BoolLiteral(false)),
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
                        left: Box::new(Expression::IntLiteral(1)),
                        operator: Token::LeftShift,
                        right: Box::new(Expression::IntLiteral(8)),
                    },
                },
            ),
            (
                "let res = base + 2 >> offset - 1",
                Statement::Let {
                    name: "res".to_string(),
                    value: Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::Identifier("base".to_string())),
                            operator: Token::Add,
                            right: Box::new(Expression::IntLiteral(2)),
                        }),
                        operator: Token::RightShift,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Identifier("offset".to_string())),
                            operator: Token::Subtract,
                            right: Box::new(Expression::IntLiteral(1)),
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
}
