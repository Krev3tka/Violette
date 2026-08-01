use crate::codegen::codegen::Codegen;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::checker::Checker;

mod codegen;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;
mod typechecker;

fn check(input: &str) -> String {
    let lexer = Lexer::new(input);

    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program().unwrap();

    let mut checker = Checker::default();

    checker.check_program(&ast);

    let codegen = Codegen::new();

    codegen.out
}

fn main() {
    let inputs = vec![
        "package main

        let x = true != 5",
        "package double

        fun double(x: int) [int] {
            return x * 2
        }

        print(double(6))",
        "package t

        fun apply(f: fun (int) [int], x: int) [int | string | bool] {
            return f(x)
        }

        fun main() {
            print(apply(print, 5))
        }",
        "package config

        import (
            os,
            strings,
            time,
            math
        )

        fun now() [time.Time] {
            return time.Now
        }

        fun main() {
            let content = os.ReadFile(\"log.txt\")
            let res = os.ReadFile(\"x\") == 5

            let time = now(5)
        }

        let x = 3 + 2",
        "package Point

        struct Point {
            x: int,
            y: int
        };

        fun getX(p: Point) [int] {
            return p.x + 1;
        }

        fun getXInCondition(p: Point) [int] {
            if p.x > 5 {
                return p.x
            }
        }

        fun getNotX(p: Point) [int] {
            return p.i
        }

        fun getXNotFromPoint(p: Point) [int] {
            return o.x
        }

        fun getXFromPrimitive(p: int) [int] {
            return p.x
        }",
    ];

    for input in inputs {
        println!("{}", check(input));
    }
}
