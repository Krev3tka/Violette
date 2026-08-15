#![allow(clippy::module_inception)]

use std::env;
use crate::driver::compile;

mod codegen;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;
mod typechecker;
mod driver;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Error: Missing required arguments. Expected at least 2 arguments, but received {}\n\nUsage: violette <build|run> <file.vio>", args.len());
        return;
    }

    let command = &args[1];
    let file = &args[2];

    compile(command, file);
}
