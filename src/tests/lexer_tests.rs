#[cfg(test)]
#[allow(clippy::approx_constant)]
mod lexing_tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::PrimitiveType::Int;
    use crate::lexer::token::{PrimitiveType, Token};
    use pretty_assertions::assert_eq;

    fn lex(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        let mut res = Vec::new();

        loop {
            let spanned_token = lexer.next_token();
            res.push(spanned_token.token.clone());

            if spanned_token.token == Token::Eof {
                break;
            }

            assert_ne!(spanned_token.token, Token::Illegal);
        }

        res
    }

    pub fn assert_tokens(input: &str, expected: Vec<Token>) {
        let actual = lex(input);

        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_eq!(a, e, "Token #{i} mismatch : got {a:?}, expected {e:?}");
        }

        assert_eq!(
            actual.len(),
            expected.len(),
            "token count mismatch: got {}, expected {}",
            actual.len(),
            expected.len()
        );
    }

    #[test]
    fn let_it_be() {
        let input = "let x = 42.5_f64
        let y = 100_u32
        let z = 3.14_f32
        ";

        assert_tokens(
            input,
            vec![
                Token::Let,
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Float64(42.5),
                Token::Newline,
                Token::Let,
                Token::Identifier("y".to_string()),
                Token::Assign,
                Token::Uint32(100),
                Token::Newline,
                Token::Let,
                Token::Identifier("z".to_string()),
                Token::Assign,
                Token::Float32(3.14),
                Token::Newline,
                Token::Eof,
            ],
        )
    }

    #[test]
    fn fun_fetch_user() {
        let input =
            "fun fetchUser(id: int) [Win(User) | Fail(string)] {} // it's fetchUser function";

        assert_tokens(
            input,
            vec![
                Token::Fun,
                Token::Identifier("fetchUser".to_string()),
                Token::LeftParen,
                Token::Identifier("id".to_string()),
                Token::Colon,
                Token::PrimitiveType(Int),
                Token::RightParen,
                Token::LeftBracket,
                Token::Identifier("Win".to_string()),
                Token::LeftParen,
                Token::Identifier("User".to_string()),
                Token::RightParen,
                Token::Pipe,
                Token::Identifier("Fail".to_string()),
                Token::LeftParen,
                Token::PrimitiveType(PrimitiveType::String),
                Token::RightParen,
                Token::RightBracket,
                Token::LeftBrace,
                Token::RightBrace,
                Token::Eof,
            ],
        )
    }

    #[test]
    fn bit_operations() {
        let input = "let a = 0b01101 # 0xAF & ~0o75
        if cond1 && cond2 || !cond3 {
            let res = 0x01
        } else {
            let res = 0x00
        }";

        assert_tokens(
            input,
            vec![
                Token::Let,
                Token::Identifier("a".to_string()),
                Token::Assign,
                Token::Int(13),
                Token::BitOr,
                Token::Int(175),
                Token::BitAnd,
                Token::BitNot,
                Token::Int(61),
                Token::Newline,
                Token::If,
                Token::Identifier("cond1".to_string()),
                Token::LogicAnd,
                Token::Identifier("cond2".to_string()),
                Token::LogicOr,
                Token::LogicNot,
                Token::Identifier("cond3".to_string()),
                Token::LeftBrace,
                Token::Newline,
                Token::Let,
                Token::Identifier("res".to_string()),
                Token::Assign,
                Token::Int(1),
                Token::Newline,
                Token::RightBrace,
                Token::Else,
                Token::LeftBrace,
                Token::Newline,
                Token::Let,
                Token::Identifier("res".to_string()),
                Token::Assign,
                Token::Int(0),
                Token::Newline,
                Token::RightBrace,
                Token::Eof,
            ],
        )
    }

    #[test]
    fn mixer() {
        let input = "let a=5+10+0xFA0_3E2&~0o10";

        assert_tokens(
            input,
            vec![
                Token::Let,
                Token::Identifier("a".to_string()),
                Token::Assign,
                Token::Int(5),
                Token::Add,
                Token::Int(10),
                Token::Add,
                Token::Int(16384994),
                Token::BitAnd,
                Token::BitNot,
                Token::Int(8),
                Token::Eof,
            ],
        )
    }

    #[test]
    fn string_literal_reading() {
        let input = "
        f(\"x\")";

        assert_eq!(
            lex(input),
            vec![
                Token::Newline,
                Token::Identifier("f".to_string()),
                Token::LeftParen,
                Token::String("x".to_string()),
                Token::RightParen,
                Token::Eof
            ]
        )
    }

    #[test]
    fn final_test() {
        let input = "
        /*
        literally Violette Language test i swear
        /* comments nesting /* and one more */ looks like it's done */
        */

        let hexVal = 0x1A_2B
        let s = \"String with // commentaries /* mustn't */ break\"

        let broken_expr = 5/*yeah*/+/*numbers*/10

        let i = 42
        i++
        i--
        ";

        assert_tokens(
            input,
            vec![
                Token::Newline,
                Token::Newline,
                Token::Let,
                Token::Identifier("hexVal".to_string()),
                Token::Assign,
                Token::Int(6699),
                Token::Newline,
                Token::Let,
                Token::Identifier("s".to_string()),
                Token::Assign,
                Token::String("String with // commentaries /* mustn't */ break".to_string()),
                Token::Newline,
                Token::Let,
                Token::Identifier("broken_expr".to_string()),
                Token::Assign,
                Token::Int(5),
                Token::Add,
                Token::Int(10),
                Token::Newline,
                Token::Let,
                Token::Identifier("i".to_string()),
                Token::Assign,
                Token::Int(42),
                Token::Newline,
                Token::Identifier("i".to_string()),
                Token::Increment,
                Token::Newline,
                Token::Identifier("i".to_string()),
                Token::Decrement,
                Token::Newline,
                Token::Eof,
            ],
        )
    }
}
