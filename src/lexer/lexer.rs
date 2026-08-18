use crate::lexer::span::{Span, SpannedToken};
use crate::lexer::token::{PrimitiveType, Token};

pub struct Lexer {
    input: Vec<char>,
    current: char,
    position: usize,
    next_position: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            current: '\0',
            position: 0,
            next_position: 0,
            line: 1,
            col: 0,
        };

        lexer.read_char();

        lexer
    }

    pub fn next_token(&mut self) -> SpannedToken {
        self.skip_whitespaces();

        let start = Span::new(self.line, self.col);

        let token = match self.current {
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    Token::LogicOr
                } else {
                    Token::Pipe
                }
            }

            '.' => {
                if self.peek_char() == '.' {
                    self.read_char();
                    Token::DoubleDot
                } else {
                    Token::Dot
                }
            }
            ',' => Token::Comma,

            '"' => {
                self.read_char();
                let mut string_val = String::new();
                while self.current != '"' && self.current != '\0' {
                    string_val.push(self.current);
                    self.read_char();
                }

                if self.current == '\0' {
                    Token::Illegal
                } else {
                    Token::String(string_val)
                }
            }

            '&' => {
                if self.peek_char() == '&' {
                    self.read_char();
                    Token::LogicAnd
                } else {
                    Token::BitAnd
                }
            }

            '#' => Token::BitOr,
            '~' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    Token::Sprout
                } else {
                    Token::BitNot
                }
            }
            '^' => Token::BitXOR,

            '<' => {
                if self.peek_char() == '<' {
                    self.read_char();
                    Token::LeftShift
                } else if self.peek_char() == '=' {
                    self.read_char();
                    Token::LessOrEquals
                } else {
                    Token::Less
                }
            }

            '>' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    Token::RightShift
                } else if self.peek_char() == '=' {
                    self.read_char();
                    Token::GreaterOrEquals
                } else {
                    Token::Greater
                }
            }

            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::Equals
                } else if self.peek_char() == '>' {
                    self.read_char();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }

            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEquals
                } else {
                    Token::LogicNot
                }
            }

            '+' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::AddAndAssign
                } else if self.peek_char() == '+' {
                    self.read_char();
                    Token::Increment
                } else {
                    Token::Add
                }
            }

            '-' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::SubAndAssign
                } else if self.peek_char() == '-' {
                    self.read_char();
                    Token::Decrement
                } else {
                    Token::Subtract
                }
            }

            '*' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::MulAndAssign
                } else if self.peek_char() == '*' {
                    self.read_char();
                    Token::Power
                } else {
                    Token::Multiply
                }
            }

            '/' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::DivAndAssign
                } else if self.peek_char() == '/' {
                    while self.current != '\n' && self.current != '\r' && self.current != '\0' {
                        self.read_char()
                    }

                    return self.next_token();
                } else if self.peek_char() == '*' {
                    self.read_char();
                    self.read_char();

                    let mut depth = 1;

                    while depth > 0 && self.current != '\0' {
                        if self.current == '/' && self.peek_char() == '*' {
                            depth += 1;
                            self.read_char();
                            self.read_char();
                        } else if self.current == '*' && self.peek_char() == '/' {
                            depth -= 1;
                            self.read_char();
                            self.read_char();
                        } else {
                            self.read_char();
                        }
                    }

                    if depth > 0 {
                        return SpannedToken {
                            token: Token::Illegal,
                            span: start,
                        };
                    }

                    return self.next_token();
                } else {
                    Token::Divide
                }
            }

            '%' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::ModAndAssign
                } else {
                    Token::Modulus
                }
            }

            '\0' => Token::Eof,

            '\n' | '\r' => {
                while self.current == '\n' || self.current == '\r' {
                    self.read_char();
                }

                return SpannedToken {
                    token: Token::Newline,
                    span: start,
                };
            }

            '0' => {
                let next = self.peek_char();

                if next == 'b' || next == 'o' || next == 'x' || next == 'X' {
                    let prefix = next;
                    self.read_char();
                    self.read_char();

                    let mut digits = String::new();

                    while match prefix {
                        'b' => self.current == '0' || self.current == '1' || self.current == '_',
                        'o' => self.current >= '0' && self.current <= '7' || self.current == '_',
                        _ => self.current.is_ascii_hexdigit() || self.current == '_',
                    } {
                        if self.current != '_' {
                            digits.push(self.current);
                        }
                        self.read_char();
                    }

                    if digits.is_empty() {
                        return SpannedToken {
                            token: Token::Illegal,
                            span: start,
                        };
                    } else {
                        let radix = match prefix {
                            'b' => 2,
                            'o' => 8,
                            _ => 16,
                        };

                        return SpannedToken {
                            token: Token::Int(
                                isize::from_str_radix(digits.as_str(), radix).unwrap_or(0),
                            ),
                            span: start,
                        };
                    }
                } else {
                    return self.read_number_literal();
                }
            }

            ch if ch.is_ascii_digit() => {
                return self.read_number_literal();
            }

            ch if Self::is_letter(ch) => {
                let ident = self.read_identifier();

                return match ident.as_str() {
                    "var" => SpannedToken {
                        token: Token::Var,
                        span: start,
                    },
                    "let" => SpannedToken {
                        token: Token::Let,
                        span: start,
                    },
                    "const" => SpannedToken {
                        token: Token::Const,
                        span: start,
                    },
                    "if" => SpannedToken {
                        token: Token::If,
                        span: start,
                    },
                    "else" => SpannedToken {
                        token: Token::Else,
                        span: start,
                    },
                    "for" => SpannedToken {
                        token: Token::For,
                        span: start,
                    },
                    "in" => SpannedToken {
                        token: Token::In,
                        span: start,
                    },
                    "continue" => SpannedToken {
                        token: Token::Continue,
                        span: start,
                    },
                    "break" => SpannedToken {
                        token: Token::Break,
                        span: start,
                    },
                    "match" => SpannedToken {
                        token: Token::Match,
                        span: start,
                    },

                    "fun" => SpannedToken {
                        token: Token::Fun,
                        span: start,
                    },
                    "return" => SpannedToken {
                        token: Token::Return,
                        span: start,
                    },
                    "bloom" => SpannedToken {
                        token: Token::Bloom,
                        span: start,
                    },

                    "struct" => SpannedToken {
                        token: Token::Struct,
                        span: start,
                    },
                    "interface" => SpannedToken {
                        token: Token::Interface,
                        span: start,
                    },
                    "type" => SpannedToken {
                        token: Token::Type,
                        span: start,
                    },
                    "open" => SpannedToken {
                        token: Token::Open,
                        span: start,
                    },
                    "local" => SpannedToken {
                        token: Token::Local,
                        span: start,
                    },

                    "import" => SpannedToken {
                        token: Token::Import,
                        span: start,
                    },

                    "package" => SpannedToken {
                        token: Token::Package,
                        span: start,
                    },

                    "int" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Int),
                        span: start,
                    },
                    "int8" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Int8),
                        span: start,
                    },
                    "int16" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Int16),
                        span: start,
                    },
                    "int32" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Int32),
                        span: start,
                    },
                    "int64" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Int64),
                        span: start,
                    },

                    "uint" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Uint),
                        span: start,
                    },
                    "uint8" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Uint8),
                        span: start,
                    },
                    "uint16" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Uint16),
                        span: start,
                    },
                    "uint32" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Uint32),
                        span: start,
                    },
                    "uint64" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Uint64),
                        span: start,
                    },

                    "float32" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Float32),
                        span: start,
                    },
                    "float64" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Float64),
                        span: start,
                    },

                    "bool" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::Bool),
                        span: start,
                    },
                    "string" => SpannedToken {
                        token: Token::PrimitiveType(PrimitiveType::String),
                        span: start,
                    },
                    "true" => SpannedToken {
                        token: Token::Bool(true),
                        span: start,
                    },
                    "false" => SpannedToken {
                        token: Token::Bool(false),
                        span: start,
                    },

                    _ => SpannedToken {
                        token: Token::Identifier(ident),
                        span: start,
                    },
                };
            }

            _ => Token::Illegal,
        };

        self.read_char();

        SpannedToken { token, span: start }
    }

    fn read_char(&mut self) {
        if self.current == '\n' {
            self.line += 1;
            self.col = 0;
        }
        if self.next_position >= self.input.len() {
            self.current = '\0';
            return;
        }

        self.current = self.input[self.next_position];

        self.position = self.next_position;
        self.next_position += 1;
        self.col += 1;
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn skip_whitespaces(&mut self) {
        while self.current == ' ' || self.current == '\t' {
            self.read_char()
        }
    }

    fn peek_char(&self) -> char {
        if self.next_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.next_position]
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut id = String::new();

        while Self::is_letter(self.current) || self.current.is_ascii_digit() {
            id.push(self.current);
            self.read_char();
        }

        id
    }

    fn read_suffix(&mut self) -> String {
        let mut suffix = String::new();

        while self.current.is_alphanumeric() {
            suffix.push(self.current);
            self.read_char();
        }

        suffix
    }

    fn read_number_literal(&mut self) -> SpannedToken {
        let start = Span::new(self.line, self.col);
        let mut digits = String::new();

        while self.current.is_ascii_digit() || self.current == '_' {
            if self.current == '_' && self.peek_char().is_ascii_alphabetic() {
                break;
            }
            if self.current != '_' {
                digits.push(self.current);
            }
            self.read_char();
        }

        if self.current == '.' && self.peek_char().is_ascii_digit() {
            digits.push(self.current);
            self.read_char();
            while self.current.is_ascii_digit() {
                digits.push(self.current);
                self.read_char();
            }
        }

        if self.current == '_' {
            self.read_char();
            let suffix = self.read_suffix();

            let cleaned = digits.replace('_', "");

            let token = match (suffix.as_str(), digits.contains('.')) {
                ("f32", true) => Token::Float32(cleaned.parse().unwrap_or(0.0)),
                ("f64", true) => Token::Float64(cleaned.parse().unwrap_or(0.0)),

                ("i8", false) => match cleaned.parse::<i8>() {
                    Ok(v) => Token::Int8(v),
                    Err(_) => Token::Illegal,
                },
                ("i16", false) => match cleaned.parse::<i16>() {
                    Ok(v) => Token::Int16(v),
                    Err(_) => Token::Illegal,
                },
                ("i32", false) => match cleaned.parse::<i32>() {
                    Ok(v) => Token::Int32(v),
                    Err(_) => Token::Illegal,
                },
                ("i64", false) => match cleaned.parse::<i64>() {
                    Ok(v) => Token::Int64(v),
                    Err(_) => Token::Illegal,
                },

                ("u8", false) => match cleaned.parse::<u8>() {
                    Ok(v) => Token::Uint8(v),
                    Err(_) => Token::Illegal,
                },
                ("u16", false) => match cleaned.parse::<u16>() {
                    Ok(v) => Token::Uint16(v),
                    Err(_) => Token::Illegal,
                },
                ("u32", false) => match cleaned.parse::<u32>() {
                    Ok(v) => Token::Uint32(v),
                    Err(_) => Token::Illegal,
                },
                ("u64", false) => match cleaned.parse::<u64>() {
                    Ok(v) => Token::Uint64(v),
                    Err(_) => Token::Illegal,
                },

                _ => Token::Illegal,
            };

            return SpannedToken { token, span: start };
        }

        if digits.contains('.') {
            match digits.replace('_', "").parse::<f64>() {
                Ok(v) => SpannedToken {
                    token: Token::Float64(v),
                    span: start,
                },
                Err(_) => SpannedToken {
                    token: Token::Illegal,
                    span: start,
                },
            }
        } else {
            match digits.replace('_', "").parse::<isize>() {
                Ok(v) => SpannedToken {
                    token: Token::Int(v),
                    span: start,
                },
                Err(_) => SpannedToken {
                    token: Token::Illegal,
                    span: start,
                },
            }
        }
    }
}
