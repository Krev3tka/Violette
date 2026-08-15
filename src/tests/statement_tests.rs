#[cfg(test)]
pub mod statements_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::PrimitiveType::{Int, String};
    use crate::lexer::token::Token::{Add, Assign, Subtract};
    use crate::lexer::token::{PrimitiveType, Token};
    use crate::parser::Expression::{Identifier, Infix, IntLiteral};
    use crate::parser::parser::Parser;
    use crate::parser::statement::{ElseIf, FunParam, IfStatement, StructParam};
    use crate::parser::types::Type::{Primitive, Union};
    use crate::parser::types::{Type, TypePath};
    use crate::parser::{Expression, Statement};
    use crate::tests::helpers::{assert_stmt_tests, ident, infix, int, let_stmt};

    #[test]
    fn basic_statements() {
        let test_cases = vec![
            (
                "let x = 5",
                Statement::Let {
                    name: "x".to_string(),
                    value: IntLiteral(5),
                },
            ),
            (
                "const THREE_HOURS_IN_SECONDS = 3 * 24 * 60 ** 2",
                Statement::Const {
                    name: "THREE_HOURS_IN_SECONDS".to_string(),
                    value: Infix {
                        left: Box::new(Infix {
                            left: Box::new(IntLiteral(3)),
                            operator: Token::Multiply,
                            right: Box::new(IntLiteral(24)),
                        }),
                        operator: Token::Multiply,
                        right: Box::new(Infix {
                            left: Box::new(IntLiteral(60)),
                            operator: Token::Power,
                            right: Box::new(IntLiteral(2)),
                        }),
                    },
                },
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
                    then_block: vec![let_stmt("b", infix(ident("a"), Token::Subtract, int(3)))],
                    else_if: vec![
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(3)),
                            block: vec![let_stmt("b", infix(ident("a"), Token::Add, int(4)))],
                        },
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(5)),
                            block: vec![let_stmt("b", infix(ident("a"), Token::Subtract, int(2)))],
                        },
                    ],
                    else_block: vec![let_stmt("b", infix(ident("a"), Token::Add, int(5)))],
                }),
            ),
        ];

        assert_stmt_tests(test_cases)
    }

    #[test]
    fn for_whose_advantage() {
        let test_cases = vec![
            (
                "for i = someVar; i < 10; i++ {
    let a = 5 * i
    let b = i * 3
}",
                Statement::ForCounter {
                    init: Box::new(Statement::Let {
                        name: "i".to_string(),
                        value: Identifier("someVar".to_string()),
                    }),
                    condition: Expression::Infix {
                        left: Box::new(Identifier("i".to_string())),
                        operator: Token::Less,
                        right: Box::new(IntLiteral(10)),
                    },
                    post: Expression::Postfix {
                        left: Box::new(Identifier("i".to_string())),
                        operator: Token::Increment,
                    },
                    body: vec![
                        Statement::Let {
                            name: "a".to_string(),
                            value: Expression::Infix {
                                left: Box::new(IntLiteral(5)),
                                operator: Token::Multiply,
                                right: Box::new(Identifier("i".to_string())),
                            },
                        },
                        Statement::Let {
                            name: "b".to_string(),
                            value: Expression::Infix {
                                left: Box::new(Identifier("i".to_string())),
                                operator: Token::Multiply,
                                right: Box::new(IntLiteral(3)),
                            },
                        },
                    ],
                },
            ),
            (
                "for x in thru(1, 10) {
    let a = 5 * x
    let b = x * 3
}",
                Statement::ForRange {
                    variable: "x".to_string(),
                    iterable: Expression::Call {
                        function: Box::new(Identifier("thru".to_string())),
                        args: vec![IntLiteral(1), IntLiteral(10)],
                    },
                    body: vec![
                        Statement::Let {
                            name: "a".to_string(),
                            value: Expression::Infix {
                                left: Box::new(IntLiteral(5)),
                                operator: Token::Multiply,
                                right: Box::new(Identifier("x".to_string())),
                            },
                        },
                        Statement::Let {
                            name: "b".to_string(),
                            value: Expression::Infix {
                                left: Box::new(Identifier("x".to_string())),
                                operator: Token::Multiply,
                                right: Box::new(IntLiteral(3)),
                            },
                        },
                    ],
                },
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
                        Statement::Expression {
                            expression: Expression::Postfix {
                                left: Box::new(Identifier("left".to_string())),
                                operator: Token::Increment,
                            },
                        },
                        Statement::Expression {
                            expression: Expression::Postfix {
                                left: Box::new(Identifier("right".to_string())),
                                operator: Token::Decrement,
                            },
                        },
                    ],
                },
            ),
        ];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn fun_fetch_user_ii() {
        let input = "fun fetch_user(db: Sql.databases.psql, count: int) [Win(User) | Fail(NotFound) | Fail(NotConnected)] {
    return count + 5
}";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let actual = parser.parse_function();

        assert_eq!(
            actual.unwrap(),
            Statement::Fun {
                name: "fetch_user".to_string(),
                params: vec![
                    FunParam {
                        name: "db".to_string(),
                        param_type: Type::Named(TypePath {
                            segments: vec![
                                "Sql".to_string(),
                                "databases".to_string(),
                                "psql".to_string()
                            ]
                        })
                    },
                    FunParam {
                        name: "count".to_string(),
                        param_type: Type::Primitive(PrimitiveType::Int)
                    }
                ],

                return_type: Some(Union(vec![
                    Type::Generic {
                        name: "Win".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["User".to_string()]
                        }))
                    },
                    Type::Generic {
                        name: "Fail".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["NotFound".to_string()]
                        }))
                    },
                    Type::Generic {
                        name: "Fail".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["NotConnected".to_string()]
                        }))
                    }
                ])),

                body: vec![Statement::Return {
                    value: Some(Expression::Infix {
                        left: Box::new(Identifier("count".to_string())),
                        operator: Token::Add,
                        right: Box::new(IntLiteral(5))
                    })
                }]
            }
        )
    }

    #[test]
    fn binary_search() {
        let test_cases = vec![
            (
                "let ans = arr[mid + 1]",
                Statement::Let {
                    name: "ans".to_string(),
                    value: Expression::Index {
                        left: Box::new(Identifier("arr".to_string())),
                        index: Box::new(Expression::Infix {
                            left: Box::new(Identifier("mid".to_string())),
                            operator: Token::Add,
                            right: Box::new(IntLiteral(1)),
                        }),
                    },
                },
            ),
            (
                "fun BinarySearch(arr: std.vector, target: int) [Win(int) | Fail(NotFound)] {
    let left = 0
    let right = len(arr)

    for left < right {
        let mid = left + (right - left) / 2

        if arr[mid] < target {
            left = mid + 1
        } else if arr[mid] > target {
            right = mid - 1
        } else {
            return Win(mid)
        }
    }

    return Fail(NotFound)
}",
                Statement::Fun {
                    name: "BinarySearch".to_string(),
                    params: vec![
                        FunParam {
                            name: "arr".to_string(),
                            param_type: Type::Named(TypePath {
                                segments: vec!["std".to_string(), "vector".to_string()],
                            }),
                        },
                        FunParam {
                            name: "target".to_string(),
                            param_type: Type::Primitive(Int),
                        },
                    ],
                    return_type: Some(Union(vec![
                        Type::Generic {
                            name: "Win".to_string(),
                            param: Box::new(Primitive(Int)),
                        },
                        Type::Generic {
                            name: "Fail".to_string(),
                            param: Box::new(Type::Named(TypePath {
                                segments: vec!["NotFound".to_string()],
                            })),
                        },
                    ])),
                    body: vec![
                        Statement::Let {
                            name: "left".to_string(),
                            value: IntLiteral(0),
                        },
                        Statement::Let {
                            name: "right".to_string(),
                            value: Expression::Call {
                                function: Box::new(Identifier("len".to_string())),
                                args: vec![Identifier("arr".to_string())],
                            },
                        },
                        Statement::ForCondition {
                            condition: Expression::Infix {
                                left: Box::new(Identifier("left".to_string())),
                                operator: Token::Less,
                                right: Box::new(Identifier("right".to_string())),
                            },
                            body: vec![
                                Statement::Let {
                                    name: "mid".to_string(),
                                    value: Expression::Infix {
                                        left: Box::new(Identifier("left".to_string())),
                                        operator: Token::Add,
                                        right: Box::new(Expression::Infix {
                                            left: Box::new(Expression::Infix {
                                                left: Box::new(Identifier("right".to_string())),
                                                operator: Token::Subtract,
                                                right: Box::new(Identifier("left".to_string())),
                                            }),
                                            operator: Token::Divide,
                                            right: Box::new(IntLiteral(2)),
                                        }),
                                    },
                                },
                                Statement::If(IfStatement {
                                    condition: Infix {
                                        left: Box::new(Expression::Index {
                                            left: Box::new(Identifier("arr".to_string())),
                                            index: Box::new(Identifier("mid".to_string())),
                                        }),
                                        operator: Token::Less,
                                        right: Box::new(Identifier("target".to_string())),
                                    },
                                    then_block: vec![Statement::Expression {
                                        expression: Infix {
                                            left: Box::new(Identifier("left".to_string())),
                                            operator: Token::Assign,
                                            right: Box::new(Infix {
                                                left: Box::new(Identifier("mid".to_string())),
                                                operator: Add,
                                                right: Box::new(IntLiteral(1)),
                                            }),
                                        },
                                    }],
                                    else_if: vec![ElseIf {
                                        condition: Infix {
                                            left: Box::new(Expression::Index {
                                                left: Box::new(Identifier("arr".to_string())),
                                                index: Box::new(Identifier("mid".to_string())),
                                            }),
                                            operator: Token::Greater,
                                            right: Box::new(Identifier("target".to_string())),
                                        },
                                        block: vec![Statement::Expression {
                                            expression: Infix {
                                                left: Box::new(Identifier("right".to_string())),
                                                operator: Assign,
                                                right: Box::new(Infix {
                                                    left: Box::new(Identifier("mid".to_string())),
                                                    operator: Subtract,
                                                    right: Box::new(IntLiteral(1)),
                                                }),
                                            },
                                        }],
                                    }],
                                    else_block: vec![Statement::Return {
                                        value: Some(Expression::Call {
                                            function: Box::new(Identifier("Win".to_string())),
                                            args: vec![Identifier("mid".to_string())],
                                        }),
                                    }],
                                }),
                            ],
                        },
                        Statement::Return {
                            value: Some(Expression::Call {
                                function: Box::new(Identifier("Fail".to_string())),
                                args: vec![Identifier("NotFound".to_string())],
                            }),
                        },
                    ],
                },
            ),
        ];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn sprouting_stem() {
        let test_cases = vec![(
            "let result = url ~> fetch ~> parse ~> validate",
            Statement::Let {
                name: "result".to_string(),
                value: Expression::Call {
                    function: Box::new(Identifier("validate".to_string())),
                    args: vec![Expression::Call {
                        function: Box::new(Identifier("parse".to_string())),
                        args: vec![Expression::Call {
                            function: Box::new(Identifier("fetch".to_string())),
                            args: vec![Expression::Identifier("url".to_string())],
                        }],
                    }],
                },
            },
        )];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn structuring_answer() {
        let test_cases = vec![(
            "struct Person {
    name: string,
    age: int,
    weight: int
}",
            Statement::Struct {
                name: "Person".to_string(),
                fields: vec![
                    StructParam {
                        name: "name".to_string(),
                        param_type: Primitive(String),
                    },
                    StructParam {
                        name: "age".to_string(),
                        param_type: Primitive(Int),
                    },
                    StructParam {
                        name: "weight".to_string(),
                        param_type: Primitive(Int),
                    },
                ],
            },
        )];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn function_body_keeps_all_statements() {
        let input = "fun outer() {\
            if c {\
                y()\
            }\
            z()\
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let stmt = parser.parse_statement().expect("outer must be parsed");

        match stmt {
            Statement::Fun { body, .. } => assert_eq!(body.len(), 2),
            other => panic!("expected Fun, got {:?}", other),
        }
    }
}
