pub struct Lexer {
    input: Vec<char>,
    current: char,
    errors: String,
    position: usize,
    next_position: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // identifier
    Identifier(String),

    // flow
    Let,
    Const,
    If,
    Else,
    For,
    In,
    Continue,
    Break,
    Match,

    // functions
    Fun,
    Return,
    Bloom,

    // structs
    Struct,
    Interface,
    Enum,
    Type,
    Open,
    Local,
    Cyclic,

    //types
    IType,
    I8Type,
    I16Type,
    I32Type,
    I64Type,

    UType,
    U8Type,
    U16Type,
    U32Type,
    U64Type,

    F32Type,
    F64Type,

    BoolType,

    StringType,

    // literals
    Int(isize),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),

    Uint(usize),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),

    Float32(f32),
    Float64(f64),

    Bool(bool),

    String(String),

    // operators
    Assign,          // =
    Equals,          // ==
    NotEquals,       // !=
    Less,            // <
    Greater,         // >
    LessOrEquals,    // <=
    GreaterOrEquals, // >=
    Pipe,            // |
    Colon,           // :
    Semicolon,       // ;
    Dot,             // .
    Comma,           // ,

    Add,       // +
    Subtract,  // -
    Multiply,  // *
    Divide,    // /
    Modulus,   // %
    Power,     // **
    Increment, // ++
    Decrement, // --

    AddAndAssign, // +=
    SubAndAssign, // -=
    MulAndAssign, // *=
    DivAndAssign, // /=
    ModAndAssign, // %=

    LeftParen,  // (
    RightParen, // )
    LSB,        // [
    RSB,        // ]
    LeftBrace,  // {
    RightBrace, // }

    LogicOr,  // ||
    LogicAnd, // &&
    LogicNot, // !

    BitAnd, // &
    BitOr,  // #
    BitNot, // ~
    BitXOR, // ^

    // else things
    Import,
    Package,

    // control
    Newline,
    EOF,
    Illegal,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            current: '\0',
            errors: String::new(),
            position: 0,
            next_position: 0,
        };

        lexer.read_char();

        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespaces();

        let token = match self.current {
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '[' => Token::LSB,
            ']' => Token::RSB,
            '|' => {
                if self.peek_char() == '|' {
                    self.read_char();
                    Token::LogicOr
                } else {
                    Token::Pipe
                }
            }

            '.' => Token::Dot,
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
                    self.read_char();
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
            '~' => Token::BitNot,
            '^' => Token::BitXOR,

            '<' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::LessOrEquals
                } else {
                    Token::Less
                }
            }

            '>' => {
                if self.peek_char() == '=' {
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
                        return Token::Illegal;
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

            '\0' => Token::EOF,

            '\n' | '\r' => {
                while self.current == '\n' || self.current == '\r' {
                    self.read_char();
                }

                return Token::Newline;
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
                        return Token::Illegal;
                    } else {
                        let radix = match prefix {
                            'b' => 2,
                            'o' => 8,
                            _ => 16,
                        };

                        return Token::Int(
                            isize::from_str_radix(digits.as_str(), radix).unwrap_or(0),
                        );
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
                    "enum" => Token::Enum,
                    "type" => Token::Type,
                    "open" => Token::Open,
                    "local" => Token::Local,
                    "cyclic" => Token::Cyclic,

                    "int" => Token::IType,
                    "int8" => Token::I8Type,
                    "int16" => Token::I16Type,
                    "int32" => Token::I32Type,
                    "int64" => Token::I64Type,

                    "uint" => Token::UType,
                    "uint8" => Token::U8Type,
                    "uint16" => Token::U16Type,
                    "uint32" => Token::U32Type,
                    "uint64" => Token::U64Type,

                    "float32" => Token::F32Type,
                    "float64" => Token::F64Type,

                    "bool" => Token::BoolType,
                    "string" => Token::StringType,

                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),

                    _ => Token::Identifier(ident),
                };
            }

            _ => Token::Illegal,
        };

        self.read_char();

        token
    }

    fn read_char(&mut self) {
        if self.next_position >= self.input.len() {
            self.current = '\0';
            return;
        }

        self.current = self.input[self.next_position];

        self.position = self.next_position;
        self.next_position += 1;
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn skip_whitespaces(&mut self) {
        while self.current == ' '
            || self.current == '\t'
        {
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

    fn read_number_literal(&mut self) -> Token {
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
                    Ok(v)  => Token::Int8(v),
                    Err(_) => Token::Illegal,
                },
                ("i16", false) => match cleaned.parse::<i16>() {
                    Ok(v)  => Token::Int16(v),
                    Err(_) => Token::Illegal,
                },
                ("i32", false) => match cleaned.parse::<i32>() {
                    Ok(v)  => Token::Int32(v),
                    Err(_) => Token::Illegal,
                },
                ("i64", false) => match cleaned.parse::<i64>() {
                    Ok(v)  => Token::Int64(v),
                    Err(_) => Token::Illegal,
                },

                ("u8", false) => match cleaned.parse::<u8>() {
                    Ok(v)  => Token::Uint8(v),
                    Err(_) => Token::Illegal,
                },
                ("u16", false) => match cleaned.parse::<u16>() {
                    Ok(v)  => Token::Uint16(v),
                    Err(_) => Token::Illegal,
                },
                ("u32", false) => match cleaned.parse::<u32>() {
                    Ok(v)  => Token::Uint32(v),
                    Err(_) => Token::Illegal,
                },
                ("u64", false) => match cleaned.parse::<u64>() {
                    Ok(v)  => Token::Uint64(v),
                    Err(_) => Token::Illegal,
                },

                _ => Token::Illegal,
            };

            return token;
        }

        if digits.contains('.') {
            match digits.replace('_', "").parse::<f64>() {
                Ok(v)  => Token::Float64(v),
                Err(_) => Token::Illegal,
            }
        } else {
            match digits.replace('_', "").parse::<isize>() {
                Ok(v)  => Token::Int(v),
                Err(_) => Token::Illegal,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Token};

    #[test]
    fn let_the_speed_mend_it() {
        let input = "let x = 42.5_f64
        let y = 100_u32
        let z = 3.14_f32
        ";

        println!("test1: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                break;
            }

            assert_ne!(token, Token::Illegal);
        }

        println!("res: {}", res);

        assert_eq!(
            res,
            "Got token Let
Got token Identifier(\"x\")
Got token Assign
Got token Float64(42.5)
Got token Newline
Got token Let
Got token Identifier(\"y\")
Got token Assign
Got token Uint32(100)
Got token Newline
Got token Let
Got token Identifier(\"z\")
Got token Assign
Got token Float32(3.14)
Got token Newline
Got token EOF
"
        )
    }

    #[test]
    fn fun_fetch_user() {
        let input = "fun fetchUser(id: int) [Win(User) | Fail(string)] {} // it's fetchUser function";

        println!("test2: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                break;
            }

            assert_ne!(token, Token::Illegal);
        }

        println!("res: {}", res);

        assert_eq!(
            res,
            "Got token Fun
Got token Identifier(\"fetchUser\")
Got token LeftParen
Got token Identifier(\"id\")
Got token Colon
Got token IType
Got token RightParen
Got token LSB
Got token Identifier(\"Win\")
Got token LeftParen
Got token Identifier(\"User\")
Got token RightParen
Got token Pipe
Got token Identifier(\"Fail\")
Got token LeftParen
Got token StringType
Got token RightParen
Got token RSB
Got token LeftBrace
Got token RightBrace
Got token EOF
"
        )
    }

    #[test]
    fn the_new_order() {
        let input = "let a = 0b01101 # 0xAF & ~0o75
        if cond1 && cond2 || !cond3 {
            let res = 0x01
        } else {
            let res = 0x00
        }";

        println!("test3: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                println!("Successfully read input string");
                break;
            }

            assert_ne!(token, Token::Illegal);
        }

        println!("res: {}", res);

        assert_eq!(
            res,
            "Got token Let
Got token Identifier(\"a\")
Got token Assign
Got token Int(13)
Got token BitOr
Got token Int(175)
Got token BitAnd
Got token BitNot
Got token Int(61)
Got token Newline
Got token If
Got token Identifier(\"cond1\")
Got token LogicAnd
Got token Identifier(\"cond2\")
Got token LogicOr
Got token LogicNot
Got token Identifier(\"cond3\")
Got token LeftBrace
Got token Newline
Got token Let
Got token Identifier(\"res\")
Got token Assign
Got token Int(1)
Got token Newline
Got token RightBrace
Got token Else
Got token LeftBrace
Got token Newline
Got token Let
Got token Identifier(\"res\")
Got token Assign
Got token Int(0)
Got token Newline
Got token RightBrace
Got token EOF
"
        )
    }

    #[test]
    fn piece_by_piece() {
        let input = "let a=5+10+0xFA0_3E2&~0o10";

        println!("test4: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                println!("Successfully read input string");
                break;
            }

            assert_ne!(token, Token::Illegal);
        }

        println!("res: {}", res);

        assert_eq!(
            res,
            "Got token Let
Got token Identifier(\"a\")
Got token Assign
Got token Int(5)
Got token Add
Got token Int(10)
Got token Add
Got token Int(16384994)
Got token BitAnd
Got token BitNot
Got token Int(8)
Got token EOF
"
        )
    }

    #[test]
    fn the_art_of_shredding() {
        let input = "
        /*
        literally Violette Language test i swear
        /* additional commentary nesting /* and one more */ looks like it's done */
        */

        let hexVal = 0x1A_2B
        let s = \"String with // commentaries /* mustn't */ break\"

        let broken_expr = 5/*yeah*/+/*numbers*/10

        let i = 42
        i++
        i--
        ";

        println!("test5: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                println!("Successfully read input string");
                break;
            }

            assert_ne!(token, Token::Illegal);
        }

        println!("res: {}", res);

        assert_eq!(
            res,
            "Got token Newline
Got token Newline
Got token Let
Got token Identifier(\"hexVal\")
Got token Assign
Got token Int(6699)
Got token Newline
Got token Let
Got token Identifier(\"s\")
Got token Assign
Got token String(\"String with // commentaries /* mustn't */ break\")
Got token Newline
Got token Let
Got token Identifier(\"broken_expr\")
Got token Assign
Got token Int(5)
Got token Add
Got token Int(10)
Got token Newline
Got token Let
Got token Identifier(\"i\")
Got token Assign
Got token Int(42)
Got token Newline
Got token Identifier(\"i\")
Got token Increment
Got token Newline
Got token Identifier(\"i\")
Got token Decrement
Got token Newline
Got token EOF
"
        )
    }
}
