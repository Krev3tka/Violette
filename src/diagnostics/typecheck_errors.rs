use crate::diagnostics::diagnostics::Diagnostics;
use crate::diagnostics::labeling::{Label, LabelStyle};
use crate::lexer::span::Span;
use crate::typechecker::checker::TypeError;
use crate::typechecker::error::{BindingKind, DefinitionKind, LoopControlKind};
use crate::typechecker::types::Ty;
use colored::Colorize;

impl Diagnostics for TypeError {
    fn message(&self, path: &str) -> String {
        match self {
            TypeError::AssignmentToImmutable {
                name,
                kind,
                decl_span,
                assign_span,
            } => {
                let (kind_str, help_message) = match kind {
                    BindingKind::Let => (
                        "let-bound",
                        "variable was declared with `let`, consider using `var` to make it mutable"
                            .to_string(),
                    ),
                    BindingKind::Const => (
                        "const",
                        "constants are immutable and cannot be reassigned".to_string(),
                    ),
                    BindingKind::Param => (
                        "parameter",
                        format!(
                            "parameters are immutable by default, add `var` before the name: `var {}: ...`",
                            name
                        ),
                    ),
                    _ => unreachable!(),
                };

                let labels = [
                    Label::secondary(
                        *decl_span,
                        format!("first variable defined as {} here", kind_str),
                        path.to_string(),
                    ),
                    Label::primary(
                        *assign_span,
                        format!("couldn't assign to this {} variable", kind_str),
                        path.to_string(),
                    ),
                ];

                self.report(
                    path.to_string(),
                    *assign_span,
                    format!("couldn't assign again to {} variable `{}`", kind_str, name),
                    &labels,
                    Some(help_message.as_str()),
                )
            }
            TypeError::DuplicateDefinition {
                name,
                first_span,
                second_span,
                def_kind,
            } => {
                let kind_str = match def_kind {
                    DefinitionKind::Var => "variable",
                    DefinitionKind::Fun => "function",
                    DefinitionKind::Struct => "struct",
                };
                let labels = [
                    Label::secondary(
                        *first_span,
                        format!("first definition of {kind_str} `{name}` is here"),
                        path.to_string(),
                    ),
                    Label::primary(
                        *second_span,
                        format!("second definition of {kind_str} `{name}` is here"),
                        path.to_string(),
                    ),
                ];

                self.report(
                    path.to_string(),
                    *second_span,
                    format!("couldn't re-define {kind_str} `{name}`"),
                    &labels,
                    Some(format!("try to change {kind_str} name pls").as_str()),
                )
            }
            TypeError::OutsideLoop { kind, span } => {
                let kind_str = match kind {
                    LoopControlKind::Break => "break",
                    LoopControlKind::Continue => "continue",
                };

                let labels = [Label::primary(
                    *span,
                    format!("found `{}` word outside of a loop", kind_str),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!("found `{}` outside of a loop", kind_str),
                    &labels,
                    Some(
                        format!("try to remove `{}` or move it into a loop pls", kind_str).as_str(),
                    ),
                )
            }
            TypeError::MissingReturn {
                name,
                func_span: fun_span,
                close_brace_span,
            } => {
                let labels = [
                    Label::secondary(
                        *fun_span,
                        format!("there is a definition of `{name}` function"),
                        path.to_string(),
                    ),
                    Label::primary(
                        *close_brace_span,
                        format!("missing return at the `{name}` function"),
                        path.to_string(),
                    ),
                ];

                self.report(
                    path.to_string(),
                    *fun_span,
                    format!("there's no `return` in every path in `{name}` function"),
                    &labels,
                    Some(
                        "try to add `return` keyword with an expression at the end of function pls",
                    ),
                )
            }
            TypeError::Mismatch {
                expected,
                found,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!(
                        "there's a type mismatch, expected `{}`, got `{}`",
                        expected, found
                    ),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    "found mismatched types".to_string(),
                    &labels,
                    None,
                )
            }
            TypeError::NoSuchMethod { ty, method, span } => {
                let labels = [Label::primary(
                    *span,
                    format!("method not found on type `{}`", ty),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!("no method named `{}` found for type `{}`", method, ty),
                    &labels,
                    match ty {
                        Ty::Int | Ty::Float | Ty::Bool | Ty::String => {
                            Some("primitive types cannot have custom methods in Violette")
                        }
                        _ => None,
                    },
                )
            }
            TypeError::MethodFoundAsGlobal {
                ty,
                method,
                span,
                help,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!(
                        "method not found on type `{}`",
                        match ty {
                            Ty::Struct(s) => s.clone(),
                            _ => format!("{}", ty),
                        }
                    ),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!(
                        "no method named `{}` found for type `{}`",
                        method,
                        match ty {
                            Ty::Struct(s) => s.clone(),
                            _ => format!("{}", ty),
                        }
                    ),
                    &labels,
                    Some(help),
                )
            }

            TypeError::UnknownName { name, span } => {
                let labels = [Label::primary(
                    *span,
                    format!("nothing is named like `{}`", name),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!("found something undefined: `{}`", name),
                    &labels,
                    // somehow to implement searching similar names
                    None,
                )
            }
            TypeError::NotCallable { name, ty, span } => {
                let labels = [Label::primary(
                    *span,
                    format!("couldn't call `{}` as a function", name),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!("`{}` isn't callable cause it has `{}` type", name, ty),
                    &labels,
                    None,
                )
            }
            TypeError::ArityMismatch {
                name,
                expected,
                found,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!("expected {} arguments, but found {}", expected, found),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!(
                        "function `{}` takes {} arguments but only {} were found",
                        name, expected, found
                    ),
                    &labels,
                    None,
                )
            }
            TypeError::UnknownField {
                struct_name,
                field,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!("field `{}` isn't found in struct `{}`", field, struct_name),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!(
                        "struct `{}` possibly has no field named `{}`",
                        struct_name, field
                    ),
                    &labels,
                    None,
                )
            }
            TypeError::NoFields { ty, span } => {
                let labels = [Label::primary(
                    *span,
                    format!("type `{}` can have no fields", ty),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!("cannot access field on non-struct type `{}`", ty),
                    &labels,
                    Some("field access `.` is only available on struct instances"),
                )
            }
            TypeError::InvalidBinaryOperator {
                operator,
                left,
                right,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!(
                        "couldn't apply binary operator to `{}` and `{}`",
                        left, right
                    ),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!(
                        "operator `{:?}` cannot be applied to types `{}` and `{}`",
                        operator, left, right
                    ),
                    &labels,
                    None,
                )
            }
            TypeError::InvalidUnaryOperator {
                operator,
                operand,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    format!("couldn't apply unary operator to type `{}`", operand),
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    format!(
                        "operator `{:?}` cannot be applied to type `{}`",
                        operator, operand
                    ),
                    &labels,
                    None,
                )
            }
            TypeError::Unsupported { desc, span } => {
                let labels = [Label::primary(*span, desc.clone(), path.to_string())];

                self.report(path.to_string(), *span, desc.clone(), &labels, None)
            }
            TypeError::ConflictingEntryPoint {
                first_decl_span,
                second_decl_span,
            } => {
                let (earlier_span, later_span, is_main_first) =
                    if first_decl_span.start.line < second_decl_span.start.line {
                        (first_decl_span, second_decl_span, false)
                    } else {
                        (second_decl_span, first_decl_span, true)
                    };

                let first_msg = if is_main_first {
                    "explicit `main` function is defined here"
                } else {
                    "top-level code starts here"
                };

                let second_msg = if is_main_first {
                    "conflicting top-level code cannot work together with `func main()`"
                } else {
                    "conflicting `func main()` cannot work together with top-level code"
                };

                let labels = [
                    Label::secondary(*earlier_span, first_msg.to_string(), path.to_string()),
                    Label::primary(*later_span, second_msg.to_string(), path.to_string()),
                ];

                self.report(
                    path.to_string(),
                    *second_decl_span,
                    "program cannot have both top-level statements and a `main` function"
                        .to_string(),
                    &labels,
                    Some("choose either scripting style or explicit `func main() { ... }`\n\t or just check your code for extra-entry points"),
                )
            }
            TypeError::NotFullMatch { target_name: _, missed_patterns, span } => {
                let labels = [
                    Label::primary(*span, if missed_patterns.contains(",") {
                        format!("patterns `{}` are not covered",
                                missed_patterns
                                    .split(", ")
                                    .collect::<Vec<_>>()
                                    .join("`, `"))
                    } else {
                        format!("pattern `{}` is not covered", missed_patterns)
                    }, path.to_string())
                ];

                self.report(
                    path.to_string(),
                    *span,
                    "not all cases in `match` expression are covered".to_string(),
                    &labels,
                    Some(format!(
                        "try to add `{} => {{ ... }}` or `_ => {{ ... }} pls`",
                         missed_patterns
                             .split(", ")
                             .collect::<Vec<_>>()[0])
                        .as_str()
                    )
                )


            }
        }
    }

    fn report(
        &self,
        path: String,
        main_span: Span,
        message: String,
        labels: &[Label],
        help: Option<&str>,
    ) -> String {
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
                " {empty:>width$}  ╭─{line}\n",
                empty = "",
                width = width,
                line = "─".repeat(line_len)
            )
            .as_str(),
        );

        for label in labels {
            let file_content = std::fs::read_to_string(&label.file_path).unwrap_or_default();
            let file_lines: Vec<&str> = file_content.lines().collect();

            match label.style {
                LabelStyle::Primary => {
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width)
                                .as_str(),
                        );
                    }
                    res.push_str(
                        format!(
                            " {line:>width$}  \u{2502}  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = file_lines
                                .get(label.span.start.line.saturating_sub(1))
                                .copied()
                                .unwrap_or("<source unavailable>")
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
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width)
                                .as_str(),
                        );
                    }
                }
                LabelStyle::Secondary => {
                    if !message.contains("outside of a loop") {
                        res.push_str(
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width)
                                .as_str(),
                        );
                    }
                    res.push_str(
                        format!(
                            " {line:>width$}  \u{2502}  {code_line}\n",
                            line = label.span.start.line,
                            width = width,
                            code_line = file_lines
                                .get(label.span.start.line.saturating_sub(1))
                                .copied()
                                .unwrap_or("<source unavailable>")
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
                            format!(" {empty:>width$}  \u{2502}\n", empty = "", width = width)
                                .as_str(),
                        );
                    }
                }
            }
        }

        res.push_str(
            format!(
                " {empty:>width$}  ╰─{line}\n",
                empty = "",
                width = width,
                line = "─".repeat(line_len)
            )
            .as_str(),
        );

        if let Some(message) = help {
            res.push_str(
                format!(
                    "{help_text} {message}\n",
                    help_text = "help:".green().bold(),
                    message = message.green().bold()
                )
                .as_str(),
            )
        }

        res
    }
}
