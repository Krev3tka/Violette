use crate::lexer::{Lexer, Token};

mod lexer;

fn main() {
    let input = "let x = 42;";

    println!("test1: {}\n", input);

    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        println!("Got token {:?}", token);

        if token == Token::EOF {
            println!("Successfully read input string");
            break;
        }

        if token == Token::ILLEGAL {
            println!("Illegal instruction");
            break;
        }
    }

    let input = "fun fetchUser(id: int) [Win(User) | Fail(string)] {} // это функция fetchUser";

    print!("\ntest 2: {}\n\n", input);

    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        println!("Got token {:?}", token);

        if token == Token::EOF {
            println!("Successfully read input string");
            break;
        }

        if token == Token::ILLEGAL {
            println!("Illegal instruction");
            break;
        }
    }
}
