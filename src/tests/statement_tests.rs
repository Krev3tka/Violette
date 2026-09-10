#[cfg(test)]
pub mod statements_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::span::Span;
    use crate::lexer::token::PrimitiveType::{Int, String};
    use crate::lexer::token::{PrimitiveType, Token};
    use crate::parser::Statement;
    use crate::parser::parser::Parser;
    use crate::parser::statement::{ElseIf, FuncParam, StructParam};
    use crate::parser::types::Type::{Primitive, Union};
    use crate::parser::types::{Type, TypePath};
    use crate::tests::helpers::{
        assert_stmt_tests, call, const_stmt, expr_stmt, for_counter, for_range, func, ident,
        if_stmt, index, infix, int, let_stmt, postfix, ret, struct_def, while_loop,
    };
    use crate::typechecker::error::BindingKind;

    #[test]
    fn basic_statements() {
        let test_cases = vec![
            ("let x = 5", let_stmt("x", int(5))),
            (
                "const THREE_HOURS_IN_SECONDS = 3 * 24 * 60 ** 2",
                const_stmt(
                    "THREE_HOURS_IN_SECONDS",
                    infix(
                        infix(int(3), Token::Multiply, int(24)),
                        Token::Multiply,
                        infix(int(60), Token::Power, int(2)),
                    ),
                ),
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
                if_stmt(
                    infix(ident("a"), Token::Greater, int(7)),
                    vec![let_stmt("b", infix(ident("a"), Token::Subtract, int(3)))],
                    vec![
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(3)),
                            block: vec![let_stmt("b", infix(ident("a"), Token::Add, int(4)))],
                            span: Span::default(),
                        },
                        ElseIf {
                            condition: infix(ident("a"), Token::Less, int(5)),
                            block: vec![let_stmt("b", infix(ident("a"), Token::Subtract, int(2)))],
                            span: Span::default(),
                        },
                    ],
                    vec![let_stmt("b", infix(ident("a"), Token::Add, int(5)))],
                ),
            ),
        ];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn for_whose_advantage() {
        let test_cases = vec![
            (
                "for i = someVar; i < 10; i++ {
    let a = 5 * i
    let b = i * 3
}",
                for_counter(
                    let_stmt("i", ident("someVar")),
                    infix(ident("i"), Token::Less, int(10)),
                    postfix(ident("i"), Token::Increment),
                    vec![
                        let_stmt("a", infix(int(5), Token::Multiply, ident("i"))),
                        let_stmt("b", infix(ident("i"), Token::Multiply, int(3))),
                    ],
                ),
            ),
            (
                "for x in thru(1, 10) {
    let a = 5 * x
    let b = x * 3
}",
                for_range(
                    "x",
                    call(ident("thru"), vec![int(1), int(10)]),
                    vec![
                        let_stmt("a", infix(int(5), Token::Multiply, ident("x"))),
                        let_stmt("b", infix(ident("x"), Token::Multiply, int(3))),
                    ],
                ),
            ),
            (
                "while left < right {
    left++
    right--
}",
                while_loop(
                    infix(ident("left"), Token::Less, ident("right")),
                    vec![
                        expr_stmt(postfix(ident("left"), Token::Increment)),
                        expr_stmt(postfix(ident("right"), Token::Decrement)),
                    ],
                ),
            ),
        ];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn func_fetch_user_ii() {
        let input = "func fetch_user(db: Sql.databases.psql, count: int) [Win(User) | Fail(NotFound) | Fail(NotConnected)] {
    return count + 5
}";

        assert_stmt_tests(vec![(
            input,
            func(
                "fetch_user",
                vec![
                    FuncParam {
                        name: "db".to_string(),
                        param_type: Type::Named(TypePath {
                            segments: vec![
                                "Sql".to_string(),
                                "databases".to_string(),
                                "psql".to_string(),
                            ],
                        }),
                        span: Span::default(),
                        kind: BindingKind::Param,
                        is_ref: false,
                    },
                    FuncParam {
                        name: "count".to_string(),
                        param_type: Type::Primitive(PrimitiveType::Int),
                        span: Span::default(),
                        kind: BindingKind::Param,
                        is_ref: false,
                    },
                ],
                Some(Union(vec![
                    Type::Generic {
                        name: "Win".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["User".to_string()],
                        })),
                    },
                    Type::Generic {
                        name: "Fail".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["NotFound".to_string()],
                        })),
                    },
                    Type::Generic {
                        name: "Fail".to_string(),
                        param: Box::new(Type::Named(TypePath {
                            segments: vec!["NotConnected".to_string()],
                        })),
                    },
                ])),
                vec![ret(Some(infix(ident("count"), Token::Add, int(5))))],
            ),
        )]);
    }

    #[test]
    fn binary_search() {
        let test_cases = vec![
            (
                "let ans = arr[mid + 1]",
                let_stmt(
                    "ans",
                    index(ident("arr"), infix(ident("mid"), Token::Add, int(1))),
                ),
            ),
            (
                "func BinarySearch(arr: std.vector, target: int) [Win(int) | Fail(NotFound)] {
    let left = 0
    let right = len(arr)

    while left < right {
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
                func(
                    "BinarySearch",
                    vec![
                        FuncParam {
                            name: "arr".to_string(),
                            param_type: Type::Named(TypePath {
                                segments: vec!["std".to_string(), "vector".to_string()],
                            }),
                            span: Span::default(),
                            kind: BindingKind::Param,
                            is_ref: false,
                        },
                        FuncParam {
                            name: "target".to_string(),
                            param_type: Type::Primitive(Int),
                            span: Span::default(),
                            kind: BindingKind::Param,
                            is_ref: false,
                        },
                    ],
                    Some(Union(vec![
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
                    vec![
                        let_stmt("left", int(0)),
                        let_stmt("right", call(ident("len"), vec![ident("arr")])),
                        while_loop(
                            infix(ident("left"), Token::Less, ident("right")),
                            vec![
                                let_stmt(
                                    "mid",
                                    infix(
                                        ident("left"),
                                        Token::Add,
                                        infix(
                                            infix(ident("right"), Token::Subtract, ident("left")),
                                            Token::Divide,
                                            int(2),
                                        ),
                                    ),
                                ),
                                if_stmt(
                                    infix(
                                        index(ident("arr"), ident("mid")),
                                        Token::Less,
                                        ident("target"),
                                    ),
                                    vec![expr_stmt(infix(
                                        ident("left"),
                                        Token::Assign,
                                        infix(ident("mid"), Token::Add, int(1)),
                                    ))],
                                    vec![ElseIf {
                                        condition: infix(
                                            index(ident("arr"), ident("mid")),
                                            Token::Greater,
                                            ident("target"),
                                        ),
                                        block: vec![expr_stmt(infix(
                                            ident("right"),
                                            Token::Assign,
                                            infix(ident("mid"), Token::Subtract, int(1)),
                                        ))],
                                        span: Span::default(),
                                    }],
                                    vec![ret(Some(call(ident("Win"), vec![ident("mid")])))],
                                ),
                            ],
                        ),
                        ret(Some(call(ident("Fail"), vec![ident("NotFound")]))),
                    ],
                ),
            ),
        ];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn sprouting_stem() {
        let test_cases = vec![(
            "let result = url ~> fetch ~> parse ~> validate",
            let_stmt(
                "result",
                call(
                    ident("validate"),
                    vec![call(
                        ident("parse"),
                        vec![call(ident("fetch"), vec![ident("url")])],
                    )],
                ),
            ),
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
            struct_def(
                "Person",
                vec![
                    StructParam {
                        name: "name".to_string(),
                        param_type: Primitive(String),
                        span: Span::default(),
                        kind: BindingKind::Let,
                    },
                    StructParam {
                        name: "age".to_string(),
                        param_type: Primitive(Int),
                        span: Span::default(),
                        kind: BindingKind::Let,
                    },
                    StructParam {
                        name: "weight".to_string(),
                        param_type: Primitive(Int),
                        span: Span::default(),
                        kind: BindingKind::Let,
                    },
                ],
            ),
        )];

        assert_stmt_tests(test_cases);
    }

    #[test]
    fn funcction_body_keeps_all_statements() {
        let input = "func outer() {
            if c {
                y()
            }
            z()
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let stmt = parser.parse_statement().expect("outer must be parsed");

        match stmt {
            Statement::Func { body, .. } => assert_eq!(body.len(), 2),
            other => panic!("expected Fun, got {:?}", other),
        }
    }
}
