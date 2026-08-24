use crate::lexer::span::Span;

#[derive(Clone)]
pub struct Label {
    pub(crate) span: Span,
    pub(crate) message: String,
    pub(crate) style: LabelStyle,
}

#[derive(Clone, Copy)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Primary,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            style: LabelStyle::Secondary,
        }
    }
}
