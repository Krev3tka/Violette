#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Identifier name (e.g., `foo`, `calculateSum`).
    Identifier(String),

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

    /// Keyword `fun`.
    Fun,

    /// Keyword `return`.
    Return,

    /// Keyword `bloom`.
    Bloom,

    /// Keyword `struct`.
    Struct,

    /// Keyword `interface`.
    Interface,

    /// Keyword `type`.
    Type,

    /// Keyword `open`.
    Open,

    /// Keyword `local`.
    Local,

    /// Primitive type kind.
    PrimitiveType(PrimitiveType),

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
    /// Used for right-exclusive ranges and type annotations.
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

    /// Right-inclusive range operator `..`.
    DoubleDot,

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

    /// Keyword `import`.
    Import,

    /// Keyword `package`.
    Package,

    /// Newline separator.
    Newline,

    /// End of file marker.
    Eof,

    /// Unrecognized or illegal token.
    Illegal,
}

/// Primitive scalar and built-in types supported by Violette.
#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    /// Pointer-sized signed integer.
    Int,
    /// 8-bit signed integer.
    Int8,
    /// 16-bit signed integer.
    Int16,
    /// 32-bit signed integer.
    Int32,
    /// 64-bit signed integer.
    Int64,

    /// Pointer-sized unsigned integer.
    Uint,
    /// 8-bit unsigned integer.
    Uint8,
    /// 16-bit unsigned integer.
    Uint16,
    /// 32-bit unsigned integer.
    Uint32,
    /// 64-bit unsigned integer.
    Uint64,

    /// 32-bit floating-point number.
    Float32,
    /// 64-bit floating-point number.
    Float64,

    /// Boolean true/false value.
    Bool,

    /// Built-in string type.
    String,
}
