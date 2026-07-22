use crate::lexer::lexer::Lexer;
use crate::parser::Statement;
use crate::parser::parser::Parser;
use crate::typechecker::checker::{Checker, FnSig, StructSig, TypeError};
use std::collections::HashMap;

mod lexer;
mod parser;
mod tests;
mod typechecker;

fn check(input: &str) -> Vec<TypeError> {
    let lexer = Lexer::new(input);

    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program().unwrap();

    let mut checker = Checker::default();

    checker.check_program(&ast);

    checker.errors
}

fn main() {
    let inputs = vec![
        "fun f(x: int) [int] {
            let y = x + 1
            return y
        }",
        "fun f(x: int) int | bool { return true }",
        "fun f(x: int) [string | bool | int] { return x + 1}",
        "fun f(x: int) string { return \"hello\" + \"world\"}",
        "fun f() [bool] { return 1 }",
        "fun f() bool { return 1 <= 2 }",
        "fun g() int { return true + 1 }",

        "fun f() int {
            if 5 > 4 {
                return 5
            } else if 3 < 2 {
                return 32
            } else {
                return 4
            }
        }",
        "fun caller() int { return f(1, 2) }
        fun f(x: int) int { return x }"
    ];

    let mut errors = vec![];

    for input in inputs {
        errors.push(check(input));
    }

    println!("errors: {:?}", errors);
}
