use crate::parser::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },

    Const {
        name: String,
        value: Expression,
    },

    If(IfStatement),

    ForCondition {
        condition: Expression,
        body: Vec<Statement>
    },

    ForRange {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>
    },

    ForCounter {
        init: Box<Statement>,
        condition: Expression,
        post: Expression,
        body: Vec<Statement>,
    }


}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_if: Vec<ElseIf>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElseIf {
    pub condition: Expression,
    pub block: Vec<Statement>,
}