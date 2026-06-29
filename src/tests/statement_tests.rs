#[cfg(test)]
pub mod statement_tests {
    use crate::lexer::{Lexer, Token};
    use crate::parser::{ElseIf, Expression, IfStatement, Statement};
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
    fn many_parsing_stmt_tests() {
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
}