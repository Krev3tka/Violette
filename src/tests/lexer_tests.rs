#[cfg(test)]
mod tests {
    use crate::lexer::lexer::Lexer;
    use crate::lexer::token::Token;

    #[test]
    fn let_the_speed_mend_it() {
        let input = "let x = 42.5_f64
        let y = 100_u32
        let z = 3.14_f32
        ";

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
        let input =
            "fun fetchUser(id: int) [Win(User) | Fail(string)] {} // it's fetchUser function";

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

        assert_eq!(
            res,
            "Got token Fun
Got token Identifier(\"fetchUser\")
Got token LeftParen
Got token Identifier(\"id\")
Got token Colon
Got token PrimitiveType(Int)
Got token RightParen
Got token LSB
Got token Identifier(\"Win\")
Got token LeftParen
Got token Identifier(\"User\")
Got token RightParen
Got token Pipe
Got token Identifier(\"Fail\")
Got token LeftParen
Got token PrimitiveType(String)
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
