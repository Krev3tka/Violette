#[cfg(test)]
pub mod hir_tests {
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;
    use crate::parser::Precedence::Lowest;
    use crate::typechecker::checker::Checker;

    #[test]
    fn check_expr_i() {
        let test = "3 * 2 / (5 + 9) % 3";

        let lexer = Lexer::new(test);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression(Lowest).unwrap();

        let mut checker = Checker::default();

        println!("{:?}", checker.check_expr(&expr, None))
    }

    #[test]
    fn check_expr_ii() {
        let test = "~(0b101 | 0x2A3) & 0o637 & ~912";

        let lexer = Lexer::new(test);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression(Lowest).unwrap();

        let mut checker = Checker::default();

        println!("{:?}", checker.check_expr(&expr, None))
    }

    #[test]
    fn ranging() {
        let test = "0..=15";

        let lexer = Lexer::new(test);
        let mut parser = Parser::new(lexer);

        let expr = parser.parse_expression(Lowest).unwrap();

        let mut checker = Checker::default();

        println!("{:?}", checker.check_expr(&expr, None));
    }
}