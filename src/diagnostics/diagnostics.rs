use crate::diagnostics::labeling::Label;
use crate::lexer::span::Span;

pub trait Diagnostics {
    fn message(&self, path: &str) -> String;
    fn report(
        &self,
        path: String,
        main_span: Span,
        message: String,
        labels: &[Label],
        help: Option<&str>,
    ) -> String;
}
