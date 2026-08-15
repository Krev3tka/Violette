#[derive(Debug)]
#[allow(dead_code)]
pub enum CodegenError {
    Unsupported(String),
    Unexpected(String),
}
