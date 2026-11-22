use crate::lexer::span::Span;

#[allow(dead_code)]
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

impl LabelStyle {
    pub fn is_primary(&self) -> bool {
        matches!(self, LabelStyle::Primary)
    }
}
