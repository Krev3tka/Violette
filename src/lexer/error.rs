#[derive(Clone, Debug, PartialEq)]
pub enum LexError {
    /// Used for unclosed quotes in string literals.
    ///
    /// ### Example:
    /// ```Violette
    /// func main() {
    ///     // LexError::UnclosedString
    ///     let s = "Hello,
    /// }
    /// ```
    UnclosedString,

    /// Used for unclosed block commentaries.
    ///
    /// ### Example:
    /// ```Violette
    /// func main() {
    ///     // LexError::UnclosedComment
    ///     /* imagine as if there is kinda smart text
    /// }
    /// ```
    UnclosedComment,

    /// Used for empty radix digit
    ///
    /// ### Example:
    /// ```Violette
    /// func main() {
    ///     // LexError::EmptyRadixDigits
    ///     let a = 0x
    /// }
    /// ```
    EmptyRadixDigits { radix: &'static str },

    /// Used for invalid number type suffix
    ///
    /// ### Example:
    /// ```Violette
    /// func main() {
    ///     // LexError::InvalidNumberSuffix
    ///     let num = 123_xyz
    /// }
    /// ```
    InvalidNumberSuffix(String),

    /// Used for integer overflow cases
    ///
    /// ### Example:
    /// ```Violette
    /// func main() {
    ///     // LexError::NumberOverflow
    ///     let num = 200_i8
    /// }
    /// ```
    NumberOverflow(String),

    /// Used for tracing unexpected characters (general case)
    UnexpectedChar(char),
}
