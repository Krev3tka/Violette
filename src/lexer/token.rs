#[allow(dead_code)]
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
    Cherry,

    // structs
    Struct,
    Interface,
    Type,
    Open,
    Local,
    Cyclic,

    // types
    PrimitiveType(PrimitiveType),

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
    Sprout,          // ~>
    FatArrow,        // =>

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

    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }

    LogicOr,  // ||
    LogicAnd, // &&
    LogicNot, // !

    BitAnd, // &
    BitOr,  // #
    BitNot, // ~
    BitXOR, // ^

    LeftShift,  // <<
    RightShift, // >>

    // else things
    Import,
    Package,

    // control
    Newline,
    Eof,
    Illegal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    Int,
    Int8,
    Int16,
    Int32,
    Int64,

    Uint,
    Uint8,
    Uint16,
    Uint32,
    Uint64,

    Float32,
    Float64,

    Bool,

    String,
}
