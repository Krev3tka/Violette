use crate::diagnostics::diagnostics::Diagnostics;
use crate::diagnostics::labeling::Label;
use crate::lexer::span::Span;
use crate::typechecker::checker::TypeError;
use crate::typechecker::error::{BindingKind, DefinitionKind, LoopControlKind};
use crate::typechecker::types::Ty;
use ariadne::{Color, Config, LabelAttach, Report, ReportKind, Source};

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
                        "let",
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
                        format!("variable defined as {} here", kind_str),
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
                    DefinitionKind::Var => "variable", // unused, variables are redefinable
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
                annotation_span,
            } => {
                let optional_second_label_which_may_appear_sometimes_due_to_possible_type_annotation_in_a_couple_of_cases_and_also_storing_error_hint_message =
                    annotation_span.as_ref().map(|ann_span| {
                        Label::secondary(
                            *Box::new(ann_span),
                            format!("expected due to this type annotation: {}", expected),
                            path.to_string(),
                        )
                    });

                let mut labels = vec![Label::primary(
                    *span,
                    format!(
                        "there's a type mismatch, expected `{}`, got `{}`",
                        expected, found
                    ),
                    path.to_string(),
                )];

                if let Some(label) = optional_second_label_which_may_appear_sometimes_due_to_possible_type_annotation_in_a_couple_of_cases_and_also_storing_error_hint_message {
                    labels.push(label);
                }

                // labels.reverse();

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
                    None,
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
                        match ty.as_ref() {
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
                        match ty.as_ref() {
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
            TypeError::NotFullMatch {
                target_name: _,
                missed_patterns,
                span,
            } => {
                let labels = [Label::primary(
                    *span,
                    if missed_patterns.contains(",") {
                        format!(
                            "patterns `{}` are not covered",
                            missed_patterns.split(", ").collect::<Vec<_>>().join("`, `")
                        )
                    } else {
                        format!("pattern `{}` is not covered", missed_patterns)
                    },
                    path.to_string(),
                )];

                self.report(
                    path.to_string(),
                    *span,
                    "not all cases in `match` expression are covered".to_string(),
                    &labels,
                    Some(
                        format!(
                            "try to add `{} => {{ ... }}` or `_ => {{ ... }} pls`",
                            missed_patterns.split(", ").collect::<Vec<_>>()[0]
                        )
                        .as_str(),
                    ),
                )
            }
            TypeError::IfNoElse {
                if_span,
                closing_span,
            } => {
                let labels = &[
                    Label::secondary(*if_span, "if-expression starts here", path.to_string()),
                    Label::primary(*closing_span, "there is no `else` here", path.to_string()),
                ];

                self.report(
                    path.to_string(),
                    *closing_span,
                    "this `if` block needs an `else` branch".to_string(),
                    labels,
                    Some("every `if` expression must have an `else` for producing value in every path no matter which branch is happening")
                )
            }
            _ => todo!(),
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
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let main_range = main_span.to_range(&src);

        let config = Config::default()
            .with_label_attach(LabelAttach::Middle)
            // .with_cross_gap(true)
            .with_compact(false);

        let mut report =
            Report::build(ReportKind::Custom("Error", Color::Red), (&path, main_range))
                .with_config(config)
                .with_message(message);

        for lbl in labels {
            let color = if lbl.style.is_primary() {
                Color::Red
            } else {
                Color::Cyan
            };

            let range = lbl.span.to_range(&src);
            let order = -(range.start as i32);

            report = report.with_label(
                ariadne::Label::new((&path, range))
                    .with_message(&lbl.message)
                    .with_color(color)
                    .with_order(order),
            )
        }

        if let Some(help_msg) = help {
            report = report.with_help(help_msg);
        }

        let mut buf = Vec::new();

        report
            .finish()
            .write((&path, Source::from(&src)), &mut buf)
            .unwrap();

        String::from_utf8(buf).unwrap_or_default()
    }
}
