use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::checker::{Checker, TypeError};

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
        "package main

        let x = true + 5",

        "package double

        fun double(x: int) int {
            return x * 2
        }

        print(double(6))",

        "package t

        fun main() {

        }",

        "package config

        import (
            os,
            strings,
            time,
            math
        )

        fun main() {
            let content = os.ReadFile(\"log.txt\")
        }

        let x = 3 + 2"
    ];

    let mut errors = vec![];

    for input in inputs {
        errors.push(check(input));
    }

    println!("errors: {:?}", errors);
}
