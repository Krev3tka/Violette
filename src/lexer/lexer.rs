use crate::lexer::span::{Position, Span, SpannedToken};
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

        let start = self.current_pos();

        match self.current {
            ';' => self.make_token(Token::Semicolon, start),
            ':' => self.make_token(Token::Colon, start),
            '{' => self.make_token(Token::LeftBrace, start),
            '}' => self.make_token(Token::RightBrace, start),
            '(' => self.make_token(Token::LeftParen, start),
            ')' => self.make_token(Token::RightParen, start),
            '[' => self.make_token(Token::LeftBracket, start),
            ']' => self.make_token(Token::RightBracket, start),
            ',' => self.make_token(Token::Comma, start),
            '#' => self.make_token(Token::BitOr, start),
            '^' => self.make_token(Token::BitXOR, start),
            '%' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::ModAndAssign, start)
                } else {
                    self.make_token(Token::Modulus, start)
                }
            }

            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    self.make_token(Token::LogicOr, start)
                } else {
                    self.make_token(Token::Pipe, start)
                }
            }

            '.' => {
                if self.peek_char() == '.' {
                    self.read_char();
                    self.make_token(Token::DoubleDot, start)
                } else {
                    self.make_token(Token::Dot, start)
                }
            }

            '"' => {
                self.read_char();
                let mut string_val = String::new();
                while self.current != '"' && self.current != '\0' {
                    if self.current == '\\' {
                        self.read_char();
                        match self.current {
                            'n' => string_val.push('\n'),
                            't' => string_val.push('\t'),
                            'r' => string_val.push('\r'),
                            '\\' => string_val.push('\\'),
                            '"' => string_val.push('"'),
                            '0' => string_val.push('\0'),
                            _ => string_val.push(self.current),
                        }
                    } else {
                        string_val.push(self.current);
                    }
                    self.read_char();
                }

                if self.current == '\0' {
                    self.make_token(Token::Illegal, start)
                } else {
                    self.make_token(Token::String(string_val), start)
                }
            }

            '&' => {
                if self.peek_char() == '&' {
                    self.read_char();
                    self.make_token(Token::LogicAnd, start)
                } else {
                    self.make_token(Token::BitAnd, start)
                }
            }

            '~' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    self.make_token(Token::Sprout, start)
                } else {
                    self.make_token(Token::BitNot, start)
                }
            }

            '<' => {
                if self.peek_char() == '<' {
                    self.read_char();
                    self.make_token(Token::LeftShift, start)
                } else if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::LessOrEquals, start)
                } else {
                    self.make_token(Token::Less, start)
                }
            }

            '>' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    self.make_token(Token::RightShift, start)
                } else if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::GreaterOrEquals, start)
                } else {
                    self.make_token(Token::Greater, start)
                }
            }

            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::Equals, start)
                } else if self.peek_char() == '>' {
                    self.read_char();
                    self.make_token(Token::FatArrow, start)
                } else {
                    self.make_token(Token::Assign, start)
                }
            }

            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::NotEquals, start)
                } else {
                    self.make_token(Token::LogicNot, start)
                }
            }

            '+' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::AddAndAssign, start)
                } else if self.peek_char() == '+' {
                    self.read_char();
                    self.make_token(Token::Increment, start)
                } else {
                    self.make_token(Token::Add, start)
                }
            }

            '-' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::SubAndAssign, start)
                } else if self.peek_char() == '-' {
                    self.read_char();
                    self.make_token(Token::Decrement, start)
                } else {
                    self.make_token(Token::Subtract, start)
                }
            }

            '*' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::MulAndAssign, start)
                } else if self.peek_char() == '*' {
                    self.read_char();
                    self.make_token(Token::Power, start)
                } else {
                    self.make_token(Token::Multiply, start)
                }
            }

            '/' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.make_token(Token::DivAndAssign, start)
                } else if self.peek_char() == '/' {
                    while self.current != '\n' && self.current != '\r' && self.current != '\0' {
                        self.read_char();
                    }
                    self.next_token()
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
                        SpannedToken {
                            token: Token::Illegal,
                            span: Span::new(start, self.current_pos()),
                        }
                    } else {
                        self.next_token()
                    }
                } else {
                    self.make_token(Token::Divide, start)
                }
            }

            '\0' => SpannedToken {
                token: Token::Eof,
                span: Span::new(start, start),
            },

            '\n' | '\r' => {
                while self.current == '\n' || self.current == '\r' {
                    self.read_char();
                }
                SpannedToken {
                    token: Token::Newline,
                    span: Span::new(start, self.current_pos()),
                }
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
                        SpannedToken {
                            token: Token::Illegal,
                            span: Span::new(start, self.current_pos()),
                        }
                    } else {
                        let radix = match prefix {
                            'b' => 2,
                            'o' => 8,
                            _ => 16,
                        };

                        SpannedToken {
                            token: Token::Int(
                                isize::from_str_radix(digits.as_str(), radix).unwrap_or(0),
                            ),
                            span: Span::new(start, self.current_pos()),
                        }
                    }
                } else {
                    self.read_number_literal(start)
                }
            }

            ch if ch.is_ascii_digit() => self.read_number_literal(start),

            ch if Self::is_letter(ch) => {
                let ident = self.read_identifier();
                let token = match ident.as_str() {
                    "var" => Token::Var,
                    "let" => Token::Let,
                    "const" => Token::Const,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "in" => Token::In,
                    "continue" => Token::Continue,
                    "break" => Token::Break,
                    "match" => Token::Match,
                    "fun" => Token::Fun,
                    "return" => Token::Return,
                    "bloom" => Token::Bloom,
                    "struct" => Token::Struct,
                    "interface" => Token::Interface,
                    "type" => Token::Type,
                    "open" => Token::Open,
                    "local" => Token::Local,
                    "import" => Token::Import,
                    "package" => Token::Package,
                    "int" => Token::PrimitiveType(PrimitiveType::Int),
                    "int8" => Token::PrimitiveType(PrimitiveType::Int8),
                    "int16" => Token::PrimitiveType(PrimitiveType::Int16),
                    "int32" => Token::PrimitiveType(PrimitiveType::Int32),
                    "int64" => Token::PrimitiveType(PrimitiveType::Int64),
                    "uint" => Token::PrimitiveType(PrimitiveType::Uint),
                    "uint8" => Token::PrimitiveType(PrimitiveType::Uint8),
                    "uint16" => Token::PrimitiveType(PrimitiveType::Uint16),
                    "uint32" => Token::PrimitiveType(PrimitiveType::Uint32),
                    "uint64" => Token::PrimitiveType(PrimitiveType::Uint64),
                    "float32" => Token::PrimitiveType(PrimitiveType::Float32),
                    "float64" => Token::PrimitiveType(PrimitiveType::Float64),
                    "bool" => Token::PrimitiveType(PrimitiveType::Bool),
                    "string" => Token::PrimitiveType(PrimitiveType::String),
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Identifier(ident),
                };

                SpannedToken {
                    token,
                    span: Span::new(start, self.current_pos()),
                }
            }

            _ => self.make_token(Token::Illegal, start),
        }
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

    fn current_pos(&self) -> Position {
        Position::new(self.line, self.col)
    }

    fn make_token(&mut self, token: Token, start: Position) -> SpannedToken {
        self.read_char();
        SpannedToken {
            token,
            span: Span::new(start, self.current_pos()),
        }
    }

    fn read_number_literal(&mut self, start: Position) -> SpannedToken {
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

            return SpannedToken {
                token,
                span: Span::new(start, self.current_pos()),
            };
        }

        if digits.contains('.') {
            match digits.replace('_', "").parse::<f64>() {
                Ok(v) => SpannedToken {
                    token: Token::Float64(v),
                    span: Span::new(start, self.current_pos()),
                },
                Err(_) => SpannedToken {
                    token: Token::Illegal,
                    span: Span::new(start, self.current_pos()),
                },
            }
        } else {
            match digits.replace('_', "").parse::<isize>() {
                Ok(v) => SpannedToken {
                    token: Token::Int(v),
                    span: Span::new(start, self.current_pos()),
                },
                Err(_) => SpannedToken {
                    token: Token::Illegal,
                    span: Span::new(start, self.current_pos()),
                },
            }
        }
    }
}
