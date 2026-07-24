use crate::lexer::token::Token;
use crate::parser::parser::Parser;
use crate::parser::{ParseError, Statement};

pub struct Program {
    pub package: String,
    pub imports: Vec<String>,
    pub declarations: Vec<Statement>,
    pub main: Vec<Statement>
}

impl Parser {
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let package = self.parse_package()?;

        self.next_token();
        self.skip_newlines();

        let imports = match self.current_token.token.clone() {
            Token::Import => {
                let res = self.parse_imports()?;
                self.next_token();

                res
            },
            _ => vec![]
        };

        self.skip_newlines();

        let mut declarations = Vec::new();

        while matches!(self.current_token.token, Token::Fun | Token::Struct | Token::Const) {
            declarations.push(
                self.parse_statement()?
            );
            self.skip_newlines();
        }

        self.skip_newlines();

        let mut main = vec![];

        if !matches!(self.current_token.token, Token::EOF) {
            main = self.parse_block()?;
        }

        Ok(Program {
            package,
            imports,
            declarations,
            main
        })
    }
}