use crate::codegen::codegen::Codegen;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::checker::Checker;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

pub fn find_cc() -> Option<String> {
    if let Ok(cc) = env::var("CC") {
        return Some(cc);
    }

    for cand in ["cc", "clang", "gcc"] {
        if Command::new(cand).arg("--version").output().is_ok() {
            return Some(cand.to_string());
        }
    }

    None
}

pub fn compile(command: &str, file: &str) {
    let Some(compiler) = find_cc() else {
        return println!("Didn't find any C compilers");
    };

    let input = fs::read_to_string(file).expect("Failed to read file");

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer);

    let ast = match parser.parse_program() {
        Ok(prg) => prg,
        Err(e) => return println!("Parse error: {}", e),
    };

    let mut checker = Checker::default();

    checker.check_program(&ast);

    if !checker.errors.is_empty() {
        return println!("Type errors: {:?}", checker.errors);
    }

    let mut codegen = Codegen::new();

    let code = match codegen.emit_program(ast) {
        Ok(out) => out,
        Err(e) => return println!("Codegen error: {:?}", e),
    };

    let c_path = env::temp_dir().join(format!(
        "{}_vio_out.c",
        Path::new(file).file_stem().unwrap().to_string_lossy()
    ));

    let mut c_file = File::create(&c_path)
        .unwrap_or_else(|_| panic!("Failed to create temporary C file, file_name: {c_path:?}"));

    c_file
        .write_all(code.as_bytes())
        .expect("Failed to write to temporary C file");

    fs::create_dir_all("bin/").unwrap();

    let out = format!(
        "bin/{}",
        Path::new(file).file_stem().unwrap().to_str().unwrap()
    );

    let status = Command::new(&compiler)
        .args([
            "-std=c99",
            "-O3",
            c_path.to_str().unwrap(),
            "-o",
            out.as_str(),
        ])
        .status()
        .map_err(|e| e.to_string())
        .expect("Failed to compile");

    if !status.success() {
        println!("{} failed", compiler);
        return;
    }

    if command == "run" {
        let status = Command::new(format!("./{}", out))
            .status()
            .expect("Failed to execute");

        if !status.success() {
            println!("{} failed", compiler);
        }
    }
}
