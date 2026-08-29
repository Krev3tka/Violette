use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::parser::Parser;
use crate::parser::{ParseError, Statement};

#[allow(dead_code)]
pub struct Program {
    pub package: String,
    pub imports: Vec<ImportItem>,
    pub declarations: Vec<Statement>,
    pub main: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportItem {
    pub module: String,
    pub symbols: Vec<String>,
    pub span: Span,
}

impl Parser {
    pub fn parse_program(&mut self, default_package: &str) -> Result<Program, ParseError> {
        let mut package = default_package.to_string();

        if matches!(self.current_token.token, Token::Package) {
            package = self.parse_package()?;
            self.next_token();
        }

        self.skip_terminators();

        let mut imports = Vec::new();

        while matches!(self.current_token.token, Token::Import) {
            let res = self.parse_imports()?;
            imports.extend(res);
            self.skip_terminators();
        }

        self.skip_terminators();

        let mut declarations = Vec::new();

        while matches!(
            self.current_token.token,
            Token::Func | Token::Struct | Token::Const | Token::Extern | Token::Extend
        ) {
            declarations.push(self.parse_statement()?);
            self.skip_terminators();
        }

        self.skip_terminators();

        let mut main = vec![];

        if !matches!(self.current_token.token, Token::Eof) {
            main = self.parse_top_level()?;
        }

        Ok(Program {
            package,
            imports,
            declarations,
            main,
        })
    }
}
