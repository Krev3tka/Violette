use crate::lexer::{Lexer, Token};
use crate::parser::Precedence::{Equals, LessGreater, Lowest, Prefix, Product, Sum};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),

    IntLiteral(isize),
    BoolLiteral(bool),
    StringLiteral(String),

    Prefix {
        operator: Token,
        right: Box<Expression>,
    },

    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Precedence {
    Lowest,      // Base syntax
    Equals,      // == !=
    LessGreater, // < > <= >=
    Sum,         // + -
    Product,     // * / %
    Prefix,      // -X, !X
    Call,        // myFunction(X)
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Equals | Token::NotEquals => Equals,
        Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => LessGreater,
        Token::Add | Token::Subtract => Sum,
        Token::Multiply | Token::Divide | Token::Modulus => Product,
        _ => Lowest,
    }
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let mut left = match &self.current_token {
            Token::Int(v) => Expression::IntLiteral(*v),
            Token::Identifier(s) => Expression::Identifier(String::from(s)),
            Token::Bool(b) => Expression::BoolLiteral(*b),
            Token::Subtract | Token::LogicNot => {
                let operator = self.current_token.clone();

                self.next_token();

                let right = self.parse_expression(Prefix);

                Expression::Prefix {
                    operator,
                    right: Box::new(right),
                }
            }
            _ => Expression::BoolLiteral(true),
        };

        while precedence < self.peek_precedence() {
            match &self.peek_token {
                Token::Add
                | Token::Subtract
                | Token::Multiply
                | Token::Divide
                | Token::Modulus
                | Token::Equals
                | Token::NotEquals
                | Token::Less
                | Token::Greater
                | Token::LessOrEquals
                | Token::GreaterOrEquals => {
                    let peek_prec = self.peek_precedence();

                    self.next_token();

                    let operator = self.current_token.clone();

                    self.next_token();

                    let right = self.parse_expression(peek_prec);

                    left = Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                }
                Token::Assign => {
                    self.next_token();
                    let operator = self.current_token.clone();
                    self.next_token();

                    let right = self.parse_expression(Equals);

                    left = Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }

        left
    }

    pub fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token())
    }

    fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token)
    }

    fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Token};
    use crate::parser::{Expression, Parser, Precedence};

    #[test]
    fn many_parsing_tests() {
        let test_cases = vec![
            ("5", Expression::IntLiteral(5)),
            (
                "-5",
                Expression::Prefix {
                    operator: Token::Subtract,
                    right: Box::new(Expression::IntLiteral(5)),
                },
            ),
            (
                "a + b",
                Expression::Infix {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: Token::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                },
            ),
            (
                "!true",
                Expression::Prefix {
                    operator: Token::LogicNot,
                    right: Box::new(Expression::BoolLiteral(true)),
                },
            ),
            (
                "20 / 5 + 2 * x",
                Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::IntLiteral(20)),
                        operator: Token::Divide,
                        right: Box::new(Expression::IntLiteral(5)),
                    }),
                    operator: Token::Add,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::IntLiteral(2)),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Identifier("x".to_string())),
                    }),
                },
            ),
        ];

        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let actual = parser.parse_expression(Precedence::Lowest);

            assert_eq!(actual, expected, "Failing case: {}", input);
        }
    }
}
