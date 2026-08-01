#[derive(Debug)]
pub enum CodegenError {
    Unsupported(String),
    Unexpected(String),
}
