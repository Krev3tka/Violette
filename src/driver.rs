use crate::codegen::codegen::Codegen;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::checker::Checker;
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

    let temp_dir = env::temp_dir().join("violette_runtime");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp runtime dir");

    let runtime_header: &str = include_str!("../vio_helpers/vio_runtime/runtime.h");
    let str_h: &str = include_str!("../vio_helpers/vio_string/vio_string.h");
    let str_c: &str = include_str!("../vio_helpers/vio_string/vio_string.c");
    let println_h: &str = include_str!("../vio_helpers/vio_io/vio_println.h");
    let println_c: &str = include_str!("../vio_helpers/vio_io/vio_println.c");
    let print_h: &str = include_str!("../vio_helpers/vio_io/vio_print.h");
    let print_c: &str = include_str!("../vio_helpers/vio_io/vio_print.c");
    let readln_h: &str = include_str!("../vio_helpers/vio_io/vio_readln.h");
    let readln_c: &str = include_str!("../vio_helpers/vio_io/vio_readln.c");

    let write_rt = |sub: &str, name: &str, content: &str| -> std::path::PathBuf {
        let dir = temp_dir.join(sub);
        fs::create_dir_all(&dir).ok();
        let path = dir.join(name);
        fs::write(&path, content).expect("Failed to write runtime file");
        path
    };

    write_rt("", "vio_runtime.h", runtime_header);
    write_rt("vio_string", "vio_string.h", str_h);
    let str_c_path = write_rt("vio_string", "vio_string.c", str_c);
    write_rt("vio_io", "vio_println.h", println_h);
    let print_c_path = write_rt("vio_io", "vio_print.c", print_c);
    write_rt("vio_io", "vio_print.h", print_h);
    let println_c_path = write_rt("vio_io", "vio_println.c", println_c);
    write_rt("vio_io", "vio_readln.h", readln_h);
    let readln_c_path = write_rt("vio_io", "vio_readln.c", readln_c);

    let c_path = env::temp_dir().join(format!(
        "{}_vio_out.c",
        Path::new(file).file_stem().unwrap().to_string_lossy()
    ));
    fs::write(&c_path, code).expect("Failed to write temporary C file");

    fs::create_dir_all("bin/").unwrap();

    let out = format!(
        "bin/{}",
        Path::new(file).file_stem().unwrap().to_str().unwrap()
    );

    let status = Command::new(&compiler)
        .arg("-std=c99")
        .arg("-O2")
        .arg(&c_path)
        .arg(&str_c_path)
        .arg(&print_c_path)
        .arg(&println_c_path)
        .arg(&readln_c_path)
        .arg("-I")
        .arg(&temp_dir)
        .arg("-o")
        .arg(out.clone())
        .status()
        .expect("Failed to execute C compiler");

    if !status.success() {
        println!("{} failed", compiler);
        return;
    }

    if command == "run" {
        let status = Command::new(format!("./{}", out))
            .status()
            .expect("Failed to execute");

        if !status.success() {
            if let Some(code) = status.code() {
                println!("Program exited with code {}", code)
            } else {
                println!("Program terminated by signal (probably stack overflow / segfault)")
            }
        }
    }
}
