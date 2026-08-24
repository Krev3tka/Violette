#[cfg(test)]
mod checking_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::span::{Position, Span};
    use crate::lexer::token::Token;
    use crate::parser::parser::Parser;
    use crate::typechecker::checker::Checker;
    use crate::typechecker::checker::TypeError::NotCallable;
    use crate::typechecker::checker::TypeError::{ArityMismatch, InvalidBinaryOperator, Mismatch};
    use crate::typechecker::types::Ty;

    #[test]
    fn main_tail() {
        let inputs = vec![
            "\
package main
let x = true != 5",
            "\
package main
let x = 5 != 5",
            "\
package main
let x = true == false",
            "\
package main
let x = true != 5
let y = x + 1",
        ];

        let mut res = vec![];

        for input in inputs {
            let lexer = Lexer::new(input);

            let mut parser = Parser::new(lexer);

            let ast = parser.parse_program().unwrap();

            let mut checker = Checker::default();

            checker.check_program(&ast);

            res.push(checker.errors)
        }

        assert_eq!(
            res,
            vec![
                vec![InvalidBinaryOperator {
                    operator: Token::NotEquals,
                    left: Box::new(Ty::Bool),
                    right: Ty::Int,
                    span: Span::new(Position { line: 2, col: 9 }, Position { line: 2, col: 17 })
                }],
                vec![],
                vec![],
                vec![InvalidBinaryOperator {
                    operator: Token::NotEquals,
                    left: Box::new(Ty::Bool),
                    right: Ty::Int,
                    span: Span::new(Position { line: 2, col: 9 }, Position { line: 2, col: 18 })
                }],
            ]
        );
    }

    #[test]
    fn built_in_unions() {
        let inputs = vec![
            "\
package double

fun double(x: int) [int] {
    return x * 2
}

double(\"hi\")
double(6, 7)",
            "\
package double

fun double(x: int) [string] {
    return x * 2
}",
            "\
package double

print(true)
print(\"x\")
print()
5(3)",
        ];

        let mut res = vec![];

        for input in inputs {
            let lexer = Lexer::new(input);

            let mut parser = Parser::new(lexer);

            let ast = parser.parse_program().unwrap();

            let mut checker = Checker::default();

            checker.check_program(&ast);

            res.push(checker.errors)
        }

        assert_eq!(
            res,
            vec![
                vec![
                    Mismatch {
                        expected: Ty::Int,
                        found: Ty::String,
                        span: Span::new(
                            Position { line: 7, col: 8 },
                            Position { line: 7, col: 12 }
                        )
                    },
                    ArityMismatch {
                        name: "double".to_string(),
                        expected: 1,
                        found: 2,
                        span: Span::new(Position { line: 8, col: 1 }, Position { line: 8, col: 7 })
                    }
                ],
                vec![Mismatch {
                    expected: Ty::String,
                    found: Ty::Int,
                    span: Span::new(Position { line: 4, col: 5 }, Position { line: 4, col: 11 })
                }],
                vec![
                    ArityMismatch {
                        name: "print".to_string(),
                        expected: 1,
                        found: 0,
                        span: Span::new(Position { line: 5, col: 1 }, Position { line: 5, col: 6 })
                    },
                    NotCallable {
                        ty: Ty::Int,
                        span: Span::new(Position { line: 6, col: 1 }, Position { line: 6, col: 2 })
                    }
                ]
            ]
        )
    }
}
