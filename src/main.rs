#![allow(clippy::module_inception)]

use crate::driver::compile;
use std::env;

mod codegen;
mod lexer;
mod parser;

mod driver;
#[cfg(test)]
mod tests;
mod typechecker;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(
            "Error: Missing required arguments. Expected at least 2 arguments, but received {}\n\nUsage: violette <build|run> <file.vio>",
            args.len()
        );
        return;
    }

    let command = &args[1];
    let file = &args[2];

    compile(command, file);
}
