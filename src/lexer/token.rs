use crate::lexer::error::LexError;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Lowercase identifier name (e.g., `foo`, `calculateSum`).
    LowerIdent(String),

    /// Keyword `let`.
    Let,

    /// Keyword `var`.
    Var,

    /// Keyword `const`.
    Const,

    /// Keyword `if`.
    If,

    /// Keyword `else`.
    Else,

    /// Keyword `while`.
    While,

    /// Keyword `for`.
    For,

    /// Keyword `in`.
    In,

    /// Keyword `continue`.
    Continue,

    /// Keyword `break`.
    Break,

    /// Keyword `match`.
    Match,

    /// Keyword `func`.
    Func,

    /// Keyword `extern`
    Extern,

    /// Keyword `return`.
    Return,

    /// Keyword `struct`.
    Struct,

    /// Keyword `extend`.
    Extend,

    /// Keyword `interface`.
    Interface,

    /// Keyword `variant`.
    Variant,

    /// Keyword `type`.
    Type,

    /// Keyword `open`.
    Open,

    /// Keyword `local`.
    Local,

    /// Uppercase identifier.
    UpperIdent(String),

    /// Pointer-sized signed integer literal.
    Int(isize),

    /// 8-bit signed integer literal.
    Int8(i8),

    /// 16-bit signed integer literal.
    Int16(i16),

    /// 32-bit signed integer literal.
    Int32(i32),

    /// 64-bit signed integer literal.
    Int64(i64),

    /// Pointer-sized unsigned integer literal.
    Uint(usize),

    /// 8-bit unsigned integer literal.
    Uint8(u8),

    /// 16-bit unsigned integer literal.
    Uint16(u16),

    /// 32-bit unsigned integer literal.
    Uint32(u32),

    /// 64-bit unsigned integer literal.
    Uint64(u64),

    /// 32-bit float literal.
    Float32(f32),

    /// 64-bit float literal.
    Float64(f64),

    /// Bool literal.
    Bool(bool),

    /// String literal.
    String(String),

    /// Char literal.
    Char(char),

    /// Assignment operator `=`.
    Assign,

    /// Equality operator `==`.
    Equals,

    /// Non-equality operator `!=`.
    NotEquals,

    /// Less than operator `<`.
    Less,

    /// Greater than operator `>`.
    Greater,

    /// Less than or equal to operator `<=`.
    LessOrEquals,

    /// Greater than or equal to operator `>=`.
    GreaterOrEquals,

    /// Pipe operator `|`.
    ///
    /// Used for union type enumeration and postfix error propagation.
    Pipe,

    /// Colon operator `:`.
    ///
    /// Used for type annotations.
    Colon,

    /// Semicolon `;`.
    Semicolon,

    /// Field access or navigation dot `.`.
    Dot,

    /// Comma separator `,`.
    Comma,

    /// Pipeline / sprout operator `~>`.
    Sprout,

    /// Fat arrow operator `=>`.
    FatArrow,

    /// Right-exclusive range operator `..`.
    DoubleDot,

    /// Right-inclusive range operator `..=`.
    DoubleDotAssign,

    /// Addition operator `+`.
    Add,

    /// Subtraction operator `-`.
    Subtract,

    /// Multiplication operator `*`.
    Multiply,

    /// Division operator `/`.
    Divide,

    /// Modulus operator `%`.
    Modulus,

    /// Exponentiation operator `**`.
    Power,

    /// Increment operator `++`.
    Increment,

    /// Decrement operator `--`.
    Decrement,

    /// Addition assignment operator `+=`.
    AddAndAssign,

    /// Subtraction assignment operator `-=`.
    SubAndAssign,

    /// Multiplication assignment operator `*=`.
    MulAndAssign,

    /// Division assignment operator `/=`.
    DivAndAssign,

    /// Modulus assignment operator `%=`.
    ModAndAssign,

    /// Opening parenthesis `(`.
    LeftParen,

    /// Closing parenthesis `)`.
    RightParen,

    /// Opening square bracket `[`.
    LeftBracket,

    /// Closing square bracket `]`.
    RightBracket,

    /// Opening curly brace `{`.
    LeftBrace,

    /// Closing curly brace `}`.
    RightBrace,

    /// Logical OR operator `||`.
    LogicOr,

    /// Logical AND operator `&&`.
    LogicAnd,

    /// Logical NOT operator `!`.
    LogicNot,

    /// Bitwise AND operator `&`.
    BitAnd,

    /// Bitwise OR operator `#`.
    BitOr,

    /// Bitwise NOT operator `~`.
    BitNot,

    /// Bitwise XOR operator `^`.
    BitXOR,

    /// Bitwise left shift operator `<<`.
    LeftShift,

    /// Bitwise right shift operator `>>`.
    RightShift,

    /// Backslash operator `\`.
    BackSlash,

    /// Arrow operator `->`.
    Arrow,

    /// Keyword `import`.
    Import,

    /// Keyword `package`.
    Package,

    /// Newline separator.
    Newline,

    /// End of file marker.
    Eof,

    /// Unrecognized or illegal token.
    Illegal(LexError),
}
