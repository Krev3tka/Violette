use crate::lexer::span::Span;

#[derive(Clone)]
pub struct Label {
    pub file_path: String,
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

#[derive(Clone, Copy)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>, file_path: String) -> Self {
        Self {
            file_path,
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>, file_path: String) -> Self {
        Self {
            file_path,
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}
