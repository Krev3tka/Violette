use colored::Colorize;
use crate::diagnostics::labeling::{Label, LabelStyle};
use crate::lexer::span::Span;
use crate::typechecker::error::{BindingKind, TypeError};

pub trait Diagnostics {
    fn message(&self, path: &str) -> String;
    fn report(&self, path: String, main_span: Span, message: String, labels: &[Label]) -> String;
}

impl Diagnostics for TypeError {
    fn message(&self, path: &str) -> String {
        match self {
            TypeError::AssignmentToImmutable {
                name,
                kind,
                decl_span,
                assign_span
            } => {
                let kind_str = match kind {
                    BindingKind::Let => "immutable",
                    BindingKind::Const => "const",
                    _ => unreachable!()
                };

                let labels = [
                    Label::secondary(*decl_span, format!("first variable defined as {} here", kind_str)),
                    Label::primary(*assign_span, format!("couldn't assign to this {} variable", kind_str))
                ];

                self.report(
                    path.to_string(),
                    *assign_span,
                    format!("couldn't assign again to {} variable `{}`", kind_str, name),
                    &labels
                )
            }
            TypeError::DuplicateDefinition {
                name,
                first_span,
                second_span
            } => {

                let labels = [
                    Label::secondary(*first_span, format!("first definition of `{name}` is here")),
                    Label::primary(*second_span, format!("second definition of `{name}` is here"))
                ];

                self.report(
                    path.to_string(),
                    *second_span,
                    format!("couldn't re-define `{name}`"),
                    &labels
                )
            }
            _ => format!("{:?}", self)
        }
    }

    fn report(&self,
              path: String,
              main_span: Span,
              message: String,
              labels: &[Label]
    ) -> String {
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut res = "error: ".red().bold().to_string();

        res.push_str(message.as_str());
        res.push_str(
            format!(
                "\n  > {}:{}:{}\n",
                path,
                main_span.start.line,
                main_span.start.col
            )
                .as_str()
        );

        let underline = |span: Span| {
            let len = span.end.col.saturating_sub(span.start.col).max(1);
            format!(
                "{}{}",
                " ".repeat(span.start.col),
                "^".repeat(len)
            )
        };

        let max_line = match labels.iter().max_by_key(|l| {
            l.span.start.line
        }) {
            Some(v) => v,
            None => unreachable!()
        };

        let width = max_line.span.start.line
            .to_string()
            .len();

        for label in labels {

            match label.style {
                LabelStyle::Primary => {
                    res.push_str(format!(" {empty:>width$}  |\n", empty = "", width = width).as_str());
                    res.push_str(
                        format!(
                            " {line:>width$}  |  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = lines[label.span.start.line.saturating_sub(1)]
                        )
                            .as_str()
                    );
                    let indent = format!("{:>width$}", "", width = width);
                    res.push_str(
                        format!(
                            " {indent}  | {decl_mark} {message}\n",
                            decl_mark = underline(label.span).red().bold(),
                            message = label.message.red().bold()
                        )
                            .as_str()
                    );
                    res.push_str(format!(" {empty:>width$}  |\n", empty = "", width = width).as_str());
                },
                LabelStyle::Secondary => {
                    res.push_str(format!(" {empty:>width$}  |\n", empty = "", width = width).as_str());
                    res.push_str(
                        format!(
                            " {line:>width$}  |  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = lines[label.span.start.line.saturating_sub(1)]
                        )
                            .as_str()
                    );
                    let indent = format!("{:>width$}", "", width = width);
                    res.push_str(
                        format!(
                            " {indent}  | {decl_mark} {message}\n",
                            decl_mark = underline(label.span).replace("^", "_").blue().bold(),
                            message = label.message.blue().bold()
                        )
                            .as_str()
                    );
                    res.push_str(format!(" {empty:>width$}  |\n", empty = "", width = width).as_str());
                }
            }
    }

        res
    }
}