#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // identifier
    IDENTIFIER(String),

    // flow
    LET,
    CONST,
    IF,
    ELSE,
    FOR,
    CONTINUE,
    BREAK,
    MATCH,
    COMMENTARY,     // //

    // functions
    FUN,
    RETURN,

    // structs
    STRUCT,
    INTERFACE,
    ENUM,
    TYPE,
    OPEN,
    LOCAL,

    //types
    ITYPE,
    I8TYPE,
    I16TYPE,
    I32TYPE,
    I64TYPE,

    UTYPE,
    U8TYPE,
    U16TYPE,
    U32TYPE,
    U64TYPE,

    F32TYPE,
    F64TYPE,

    BOOL_TYPE,

    STRING_TYPE,

    // literals
    INT(isize),
    INT8(i8),
    INT16(i16),
    INT32(i32),
    INT64(i64),

    UINT(usize),
    UINT8(u8),
    UINT16(u16),
    UINT32(u32),
    UINT64(u64),

    FLOAT32(f32),
    FLOAT64(f64),

    BOOL(bool),

    STRING(String),

    // operators
    ASSIGN,         // =
    EQUALS,         // ==
    NOTEQUALS,      // !=
    LESS,           // <
    GREATER,        // >
    LOE,            // <=
    GOE,            // >=
    PIPE,           // |
    COLON,          // :
    SEMICOLON,      // ;
    DOT,            // .
    COMMA,          // ,

    ADD,            // +
    SUBTRACT,       // -
    MULTIPLY,       // *
    DIVIDE,         // /
    MODULUS,        // %
    INCREMENT,      // ++
    DECREMENT,      // --

    ADD_AND_ASSIGN,  // +=
    SUB_AND_ASSIGN,  // -=
    MUL_AND_ASSIGN,  // *=
    DIV_AND_ASSIGN,  // /=
    MOD_AND_ASSIGN,  // %=

    LEFT_PAREN,       // (
    RIGHT_PAREN,      // )
    LSB,              // [
    RSB,              // ]
    LBRACE,           // {
    RBRACE,           // }

    LOGIC_OR,         // ||
    LOGIC_AND,        // &&
    LOGIC_NOT,        // !

    // else things
    IMPORT,
    PACKAGE,

    // control
    EOF,
    ILLEGAL,
}

pub struct Lexer {
    input: Vec<char>,
    current: char,
    errors: String,
    position: usize,
    next_position: usize,
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
            ';' => Token::SEMICOLON,
            ':' => Token::COLON,
            '{' => Token::LBRACE,
            '}' => Token::RBRACE,
            '(' => Token::LEFT_PAREN,
            ')' => Token::RIGHT_PAREN,
            '[' => Token::LSB,
            ']' => Token::RSB,
            '|' => Token::PIPE,
            '.' => Token::DOT,
            ',' => Token::COMMA,

            '<' => {
                if self.peek_char() == '=' {
                    Token::LOE
                } else {
                    Token::LESS
                }
            },

            '>' => if self.peek_char() == '=' {
                Token::GOE
            } else {
                Token::GREATER
            }

            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::EQUALS
                } else {
                    Token::ASSIGN
                }
            }

            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NOTEQUALS
                } else {
                    Token::LOGIC_NOT
                }
            }

            '+' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::ADD_AND_ASSIGN
                } else if self.peek_char() == '+' {
                    Token::INCREMENT
                } else {
                    Token::ADD
                }
            }

            '-' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::SUB_AND_ASSIGN
                } else if self.peek_char() == '-' {
                    Token::DECREMENT
                } else {
                    Token::SUBTRACT
                }
            }

            '*' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::MUL_AND_ASSIGN
                } else {
                    Token::MULTIPLY
                }
            }

            '/' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::DIV_AND_ASSIGN
                } else if self.peek_char() == '/' {
                    while self.current != '\n' && self.current != '\r' && self.current != '\0' {
                        self.read_char()
                    }
                    return Token::COMMENTARY;
                } else {
                    Token::DIVIDE
                }
            }

            '\0' => Token::EOF,

            ch if ch.is_ascii_digit() => {
                return self.read_number_literal();
            }

            ch if Self::is_letter(ch) => {
                let ident = self.read_identifier();

                return match ident.as_str() {
                    "let" => Token::LET,
                    "const" => Token::CONST,
                    "if" => Token::IF,
                    "else" => Token::ELSE,
                    "for" => Token::FOR,
                    "continue" => Token::CONTINUE,
                    "break" => Token::BREAK,
                    "match" => Token::MATCH,

                    "fun" => Token::FUN,
                    "return" => Token::RETURN,

                    "struct" => Token::STRUCT,
                    "interface" => Token::INTERFACE,
                    "enum" => Token::ENUM,
                    "type" => Token::TYPE,
                    "open" => Token::OPEN,
                    "local" => Token::LOCAL,

                    "int" => Token::ITYPE,
                    "int8" => Token::I8TYPE,
                    "int16" => Token::I16TYPE,
                    "int32" => Token::I32TYPE,
                    "int64" => Token::I64TYPE,

                    "uint" => Token::UTYPE,
                    "uint8" => Token::U8TYPE,
                    "uint16" => Token::U16TYPE,
                    "uint32" => Token::U32TYPE,
                    "uint64" => Token::U64TYPE,

                    "float32" => Token::F32TYPE,
                    "float64" => Token::F64TYPE,

                    "bool" => Token::BOOL_TYPE,
                    "string" => Token::STRING_TYPE,

                    _ => Token::IDENTIFIER(ident),
                }
            }

            _ => Token::ILLEGAL,
        };

        self.read_char();

        token
    }

    fn read_char(&mut self) {
        if self.next_position >= self.input.len() {
            self.current = '\0';
            return
        }

        self.current = self.input[self.next_position];

        self.position = self.next_position;
        self.next_position += 1;
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn skip_whitespaces(&mut self) {
        while self.current == ' ' ||
            self.current == '\t' ||
            self.current == '\n' ||
            self.current == '\r'
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

        while self.current.is_ascii_digit() {
            digits.push(self.current);
            self.read_char()
        }

        if self.current == '.' && self.peek_char().is_ascii_digit() {
            digits.push(self.current);
            self.read_char();
            while self.current.is_ascii_digit() {
                digits.push(self.current);
                self.read_char()
            }

            if self.current == '_' {
                self.read_char();

                let suffix = self.read_suffix();

                let token = match suffix.as_str() {
                    "f32" => Token::FLOAT32(digits.parse().unwrap()),
                    "f64" => Token::FLOAT64(digits.parse().unwrap()),

                    _ => Token::ILLEGAL
                };

                return token;
            }

            Token::FLOAT32(digits.parse().unwrap())
        } else {
            if self.current == '_' {
                self.read_char();

                let suffix = self.read_suffix();

                let token = match suffix.as_str() {
                    "i8" => Token::INT8(digits.parse().unwrap()),
                    "i16" => Token::INT16(digits.parse().unwrap()),
                    "i32" => Token::INT32(digits.parse().unwrap()),
                    "i64" => Token::INT64(digits.parse().unwrap()),

                    "u8" => Token::UINT8(digits.parse().unwrap()),
                    "u16" => Token::UINT16(digits.parse().unwrap()),
                    "u32" => Token::UINT32(digits.parse().unwrap()),
                    "u64" => Token::UINT64(digits.parse().unwrap()),

                    _ => Token::ILLEGAL
                };

                return token
            }

            Token::INT(digits.parse().unwrap())
        }


    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Token};

    #[test]
    fn let_x_test() {
        let input = "let x = (42.3_f64).to_string();";

        println!("test1: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                println!("Successfully read input string");
                break;
            }

            assert_ne!(token, Token::ILLEGAL);
        }

        println!("res: {}", res);

        assert_eq!(res, "Got token LET
Got token IDENTIFIER(\"x\")
Got token ASSIGN
Got token LEFT_PAREN
Got token FLOAT64(42.3)
Got token RIGHT_PAREN
Got token DOT
Got token IDENTIFIER(\"to_string\")
Got token LEFT_PAREN
Got token RIGHT_PAREN
Got token SEMICOLON
Got token EOF
")
    }

    #[test]
    fn fun_fetch_user() {
        let input = "fun fetchUser(id: int) [Win(User) | Fail(string)] {} // это функция fetchUser";

        println!("test2: {}\n", input);

        let mut lexer = Lexer::new(input);

        let mut res: String = String::new();

        loop {
            let token = lexer.next_token();
            res.push_str(format!("Got token {:?}\n", token).as_str());

            if token == Token::EOF {
                println!("Successfully read input string");
                break;
            }

            assert_ne!(token, Token::ILLEGAL);
        }

        println!("res: {}", res);

        assert_eq!(res, "Got token FUN
Got token IDENTIFIER(\"fetchUser\")
Got token LEFT_PAREN
Got token IDENTIFIER(\"id\")
Got token COLON
Got token ITYPE
Got token RIGHT_PAREN
Got token LSB
Got token IDENTIFIER(\"Win\")
Got token LEFT_PAREN
Got token IDENTIFIER(\"User\")
Got token RIGHT_PAREN
Got token PIPE
Got token IDENTIFIER(\"Fail\")
Got token LEFT_PAREN
Got token STRING_TYPE
Got token RIGHT_PAREN
Got token RSB
Got token LBRACE
Got token RBRACE
Got token COMMENTARY
Got token EOF
")
    }
}