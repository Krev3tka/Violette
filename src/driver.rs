use crate::codegen::codegen::Codegen;
use crate::diagnostics::diagnostics::Diagnostics;
use crate::lexer::lexer::Lexer;
use crate::parser::Statement;
use crate::parser::parser::Parser;
use crate::parser::program::{ImportItem, Program};
use crate::typechecker::checker::Checker;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

const STD_PRELUDE: &str = include_str!("../std/prelude.vio");
const STD_MATH: &str = include_str!("../std/math.vio");
const STD_INT: &str = include_str!("../std/int.vio");
const STD_STRING: &str = include_str!("../std/string.vio");

fn parse_module(source: &str, pkg_name: &str) -> Program {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser
        .parse_program(pkg_name)
        .expect("Failed to parse stdlib")
}

fn filter_declarations(declarations: Vec<Statement>, symbols: &[String]) -> Vec<Statement> {
    if symbols.is_empty() {
        return declarations;
    }

    declarations
        .into_iter()
        .filter(|stmt| match stmt {
            Statement::ExternFunc { .. } => true,
            Statement::Func { name, .. }
            | Statement::Const { name, .. }
            | Statement::Struct { name, .. } => symbols.contains(name),
            _ => false,
        })
        .collect()
}

fn resolve_import(import_item: &ImportItem, file_path: &Path) -> Vec<Statement> {
    let (module_name, symbols) =
        if import_item.symbols.is_empty() && import_item.module.contains('.') {
            let parts: Vec<&str> = import_item.module.rsplitn(2, '.').collect();
            (parts[1], vec![parts[0].to_string()])
        } else {
            (import_item.module.as_str(), import_item.symbols.clone())
        };

    let module_ast = match module_name {
        "math" => parse_module(STD_MATH, "math"),
        "int" => parse_module(STD_INT, "int"),
        "string" => parse_module(STD_STRING, "string"),
        _ => {
            let local_path = file_path
                .parent()
                .unwrap_or(Path::new(""))
                .join(format!("{}.vio", module_name));

            if local_path.exists() {
                let local_src =
                    fs::read_to_string(&local_path).expect("Failed to read local module");
                parse_module(&local_src, module_name)
            } else {
                println!("Failed to load module {}", module_name);
                return vec![];
            }
        }
    };

    filter_declarations(module_ast.declarations, &symbols)
}

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
    let file_path = Path::new(file);

    let default_package = file_path
        .parent()
        .and_then(|f| f.file_name())
        .and_then(|n| n.to_str())
        .filter(|&name| !name.is_empty() && name != ".")
        .unwrap_or("main")
        .to_string();

    let Some(compiler) = find_cc() else {
        return println!("Didn't find any C compilers");
    };

    let input = fs::read_to_string(file).expect("Failed to read file");

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer);

    let mut ast = match parser.parse_program(&default_package) {
        Ok(prg) => prg,
        Err(e) => return println!("Parse error: {}", e),
    };

    let prelude_ast = parse_module(STD_PRELUDE, "std");

    let mut all_declarations = Vec::new();
    let mut imported_symbols = std::collections::HashSet::new();

    let mut add_imported_decls = |decls: Vec<Statement>| {
        for stmt in decls {
            match &stmt {
                Statement::Func { name, .. }
                | Statement::ExternFunc { name, .. }
                | Statement::Const { name, .. }
                | Statement::Struct { name, .. } => {
                    if imported_symbols.insert(name.clone()) {
                        all_declarations.push(stmt);
                    }
                }
                _ => all_declarations.push(stmt),
            }
        }
    };

    for import_item in &prelude_ast.imports {
        add_imported_decls(resolve_import(import_item, file_path))
    }

    for import_item in &ast.imports {
        add_imported_decls(resolve_import(import_item, file_path));
    }

    all_declarations.extend(ast.declarations);
    ast.declarations = all_declarations;

    let mut checker = Checker::default();

    checker.check_program(&ast);

    if !checker.errors.is_empty() {
        for err in checker.errors {
            println!("{}", err.message(file))
        }
        std::process::exit(1);
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
    let scanln_h: &str = include_str!("../vio_helpers/vio_io/vio_scanln.h");
    let scanln_c: &str = include_str!("../vio_helpers/vio_io/vio_scanln.c");
    let int_h: &str = include_str!("../vio_helpers/vio_casting/vio_int.h");
    let int_c: &str = include_str!("../vio_helpers/vio_casting/vio_int.c");

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
    write_rt("vio_io", "vio_scanln.h", scanln_h);
    let scanln_c_path = write_rt("vio_io", "vio_scanln.c", scanln_c);
    write_rt("vio_casting", "vio_int.h", int_h);
    let int_c_path = write_rt("vio_casting", "vio_int.c", int_c);

    let c_path = env::temp_dir().join(format!(
        "{}_vio_out.c",
        Path::new(file).file_stem().unwrap().to_string_lossy()
    ));
    fs::write(&c_path, code).expect("Failed to write temporary C file");

    let bin_dir = env::temp_dir().join("violette_bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");

    let out = bin_dir.join(
        Path::new(file)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    let status = Command::new(&compiler)
        .arg("-std=gnu99")
        .arg("-O0")
        .arg("-w")
        .arg(&c_path)
        .arg(&str_c_path)
        .arg(&print_c_path)
        .arg(&println_c_path)
        .arg(&scanln_c_path)
        .arg(&int_c_path)
        .arg("-I")
        .arg(&temp_dir)
        .arg("-o")
        .arg(out.clone())
        .arg("-lm")
        .status()
        .expect("Failed to execute C compiler");

    if !status.success() {
        println!("{} failed", compiler);
        return;
    }

    if command == "run" {
        let status = Command::new(&out).status().expect("Failed to execute");

        if !status.success() {
            if let Some(code) = status.code() {
                println!("Program exited with code {}", code)
            } else {
                println!("Program terminated by signal (probably stack overflow / segfault)")
            }
        }
    }
}
