#![allow(clippy::module_inception)]

use std::env;
use crate::codegen::codegen::Codegen;
use crate::driver::compile;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::checker::{Checker, TypeError};

mod codegen;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;
mod typechecker;
mod driver;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Error: Missing required arguments. Expected at least 2 arguments, but received {}\n\nUsage: violette <build|run> <file.vio>", args.len());
        return;
    }

    let command = &args[1];
    let file = &args[2];

    compile(command, file);
}
