use colored::Colorize;
use crate::diagnostics::diagnostics::Diagnostics;
use crate::diagnostics::labeling::{Label, LabelStyle};
use crate::lexer::span::Span;
use crate::typechecker::checker::TypeError;
use crate::typechecker::error::{BindingKind, DefinitionKind, LoopControlKind};
use crate::typechecker::types::Ty;

impl Diagnostics for TypeError {
    fn message(&self, path: &str) -> String {
        match self {
            TypeError::AssignmentToImmutable {
                name,
                kind,
                decl_span,
                assign_span,
            } => {
                let kind_str = match kind {
                    BindingKind::Let => "immutable",
                    BindingKind::Const => "const",
                    _ => unreachable!(),
                };

                let labels = [
                    Label::secondary(
                        *decl_span,
                        format!("first variable defined as {} here", kind_str),
                    ),
                    Label::primary(
                        *assign_span,
                        format!("couldn't assign to this {} variable", kind_str),
                    ),
                ];

                self.report(
                    path.to_string(),
                    *assign_span,
                    format!("couldn't assign again to {} variable `{}`", kind_str, name),
                    &labels,
                    Some(format!("try to use `var` instead of `{}`", match kind {
                        BindingKind::Let => "let",
                        BindingKind::Const => "const",
                        _ => unreachable!(),
                    }).as_str())
                )
            }
            TypeError::DuplicateDefinition {
                name,
                first_span,
                second_span,
                def_kind
            } => {
                let kind_str = match def_kind {
                    DefinitionKind::Var => "variable",
                    DefinitionKind::Fun => "function",
                    DefinitionKind::Struct => "struct",
                };
                let labels = [
                    Label::secondary(*first_span, format!("first definition of {kind_str} `{name}` is here")),
                    Label::primary(
                        *second_span,
                        format!("second definition of {kind_str} `{name}` is here"),
                    ),
                ];

                self.report(
                    path.to_string(),
                    *second_span,
                    format!("couldn't re-define {kind_str} `{name}`"),
                    &labels,
                    Some(format!("try to change {kind_str} name pls").as_str())
                )
            }
            TypeError::OutsideLoop {
                kind,
                span
            } => {
                let kind_str = match kind {
                    LoopControlKind::Break => "break",
                    LoopControlKind::Continue => "continue"
                };

                let labels = [
                    Label::primary(*span, format!("couldn't `{}` word outside of a loop", kind_str))
                ];

                self.report(
                    path.to_string(),
                    *span,
                    format!("found `{}` outside of a loop", kind_str),
                    &labels,
                    Some(format!("try to remove `{}` or move it into a loop pls", kind_str).as_str())
                )
            }
            TypeError::MissingReturn {
                name,
                fun_span,
                close_brace_span
            } => {
                let labels = [
                    Label::secondary(*fun_span, format!("there is a definition of `{name}` function")),
                    Label::primary(*close_brace_span, format!("missing return at the `{name}` function"))
                ];

                self.report(
                    path.to_string(),
                    *fun_span,
                    format!("there's no `return` in every path in `{name}` function"),
                    &labels,
                    Some("try to add `return` keyword with an expression at the end of function pls")
                )
            }
            TypeError::Mismatch {
                expected,
                found,
                span
            } => {
                let labels = [
                    Label::primary(*span, format!("there's a type mismatch, expected `{:?}`, got `{:?}`", expected, found),)
                ];

                self.report(
                    path.to_string(),
                    *span,
                    "found mismatched types".to_string(),
                    &labels,
                    None
                )
            },
            TypeError::NoSuchMethod {
                ty,
                method,
                span
            } => {
                let labels = [
                    Label::primary(*span, format!("method not found on type `{:?}`", ty))
                ];

                self.report(
                    path.to_string(),
                    *span,
                    format!("no method named `{}` found for type `{:?}`", method, ty),
                    &labels,
                    match ty {
                        Ty::Int
                        | Ty::Float
                        | Ty::Bool
                        | Ty::String => Some("primitive types cannot have custom methods in Violette"),
                        _ => None
                    }
                )
            }
            _ => format!("{:?}", self),
        }
    }

    fn report(&self, path: String, main_span: Span, message: String, labels: &[Label], help: Option<&str>) -> String {
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut res = "error: ".magenta().bold().to_string();

        res.push_str(message.as_str());
        res.push_str(
            format!(
                "\n  > {}:{}:{}\n",
                path, main_span.start.line, main_span.start.col
            )
                .as_str(),
        );

        let underline = |span: Span| {
            let len = span.end.col.saturating_sub(span.start.col).max(1);
            format!("{}{}", " ".repeat(span.start.col), "~".repeat(len))
        };

        let max_line = match labels.iter().max_by_key(|l| l.span.start.line) {
            Some(v) => v,
            None => unreachable!(),
        };

        let width = max_line.span.start.line.to_string().len();

        let is_compact = labels.len() == 1;

        let line_len = if is_compact { 50 } else { 60 };

        res.push_str(
            format!(
                " {empty:>width$}  ╭─{line}\n", empty = "", width = width, line = "─".repeat(line_len)
            ).as_str()

        );

        for label in labels {
            match label.style {
                LabelStyle::Primary => {
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width).as_str(),
                        );
                    }
                    res.push_str(
                        format!(
                            " {line:>width$}  \u{2502}  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = lines[label.span.start.line.saturating_sub(1)]
                        )
                            .as_str(),
                    );
                    let indent = format!("{:>width$}", "", width = width);
                    res.push_str(
                        format!(
                            " {indent}  \u{2502} {decl_mark} {message}\n",
                            decl_mark = underline(label.span).magenta().bold(),
                            message = label.message.magenta().bold()
                        )
                            .as_str(),
                    );
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width).as_str(),
                        );
                    }
                }
                LabelStyle::Secondary => {
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width).as_str(),
                        );
                    }
                    res.push_str(
                        format!(
                            " {line:>width$}  \u{2502}  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = lines[label.span.start.line.saturating_sub(1)]
                        )
                            .as_str(),
                    );
                    let indent = format!("{:>width$}", "", width = width);
                    res.push_str(
                        format!(
                            " {indent}  \u{2502} {decl_mark} {message}\n",
                            decl_mark = underline(label.span).replace("~", "_").yellow().bold(),
                            message = label.message.yellow().bold()
                        )
                            .as_str(),
                    );
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width).as_str(),
                        );
                    }
                }
            }
        }

        res.push_str(
            format!(
                " {empty:>width$}  ╰─{line}\n", empty = "", width = width, line = "─".repeat(line_len)
            ).as_str()
        );

        if let Some(message) = help { res.push_str(
            format!(
                "{help_text} {message}\n",
                help_text = "help:".green().bold(),
                message = message.green().bold()
            )
                .as_str()
        ) }

        res
    }
}