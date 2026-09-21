//! One `vize explain` page: a code's catalogued description, its metadata and
//! its help, worded in one locale.
//!
//! Everything on a page comes from the producers' metadata and the catalogue;
//! nothing is written per code here. English descriptions are the rules' own
//! `RuleMeta` text and the compiler's `ErrorCode::message()`, so an English
//! page says exactly what `vize lint` and the compiler say.

use vize_davinci::render::{Catalog, Phrase};
use vize_patina::{HelpRenderTarget, Severity, render_help, rule_docs_path};
use vize_relief::CompilerError;
use vize_s0::i18n::Locale;
use vize_s0::{String, cstr};

use super::catalog::LocaleCatalog;
use super::subjects::{Rule, Subject};

/// ANSI styling for a page, or none.
#[derive(Clone, Copy)]
struct Paint(bool);

impl Paint {
    fn wrap(self, out: &mut String, escape: &str, text: &str) {
        if self.0 {
            out.push_str(escape);
            out.push_str(text);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(text);
        }
    }

    fn code(self, out: &mut String, text: &str) {
        self.wrap(out, "\x1b[1;96m", text);
    }

    fn strong(self, out: &mut String, text: &str) {
        self.wrap(out, "\x1b[1m", text);
    }

    fn label(self, out: &mut String, text: &str) {
        self.wrap(out, "\x1b[1;94m", text);
    }
}

/// The page for `subject` in `catalog`'s locale.
pub(crate) fn page(subject: &Subject, catalog: &LocaleCatalog, color: bool) -> String {
    let paint = Paint(color);
    let mut out = String::default();
    match subject {
        Subject::Compiler(code) => {
            let kind = catalog.format("explain.kind.compiler", &[]);
            heading(&mut out, paint, code.code(), &kind);
            let message = code.localized_message(catalog.locale());
            line(&mut out, paint, &message);
            out.push('\n');
            let severity = if CompilerError::new(*code, None).is_recoverable() {
                Phrase::Warning
            } else {
                Phrase::Error
            };
            field(
                &mut out,
                paint,
                catalog,
                "explain.stage",
                code.stage().as_str(),
            );
            field(
                &mut out,
                paint,
                catalog,
                "explain.severity",
                catalog.phrase(severity),
            );
            if *code != vize_relief::ErrorCode::ExtendPoint {
                let help = code.localized_help(catalog.locale());
                help_section(&mut out, paint, catalog, &help, color);
            }
        }
        Subject::Rule(rule) => rule_page(&mut out, paint, catalog, rule, color),
    }
    out
}

fn rule_page(out: &mut String, paint: Paint, catalog: &LocaleCatalog, rule: &Rule, color: bool) {
    let kind = catalog.format("explain.kind.rule", &[]);
    heading(out, paint, rule.name, &kind);
    let description = match catalog.locale() {
        Locale::En => rule.description,
        _ => catalog
            .text(&cstr!("{}.description", rule.name))
            .unwrap_or(rule.description),
    };
    line(out, paint, description);
    out.push('\n');
    field(out, paint, catalog, "explain.category", rule.category);
    let severity = match rule.default_severity {
        Severity::Error => Phrase::Error,
        Severity::Warning => Phrase::Warning,
    };
    let severity = catalog.phrase(severity);
    field(out, paint, catalog, "explain.default_severity", severity);
    let autofix = catalog.format(
        if rule.fixable {
            "explain.autofix.yes"
        } else {
            "explain.autofix.no"
        },
        &[],
    );
    field(out, paint, catalog, "explain.autofix", &autofix);
    field(
        out,
        paint,
        catalog,
        "explain.docs",
        rule_docs_path(rule.name),
    );
    if let Some(help) = catalog.text(&cstr!("{}.help", rule.name)) {
        help_section(out, paint, catalog, help, color);
    }
}

fn heading(out: &mut String, paint: Paint, code: &str, kind: &str) {
    paint.code(out, code);
    out.push_str(" · ");
    out.push_str(kind);
    out.push('\n');
}

fn line(out: &mut String, paint: Paint, text: &str) {
    paint.strong(out, text.trim_end());
    out.push('\n');
}

fn field(out: &mut String, paint: Paint, catalog: &LocaleCatalog, key: &str, value: &str) {
    paint.label(out, &catalog.format(key, &[]));
    out.push_str(": ");
    out.push_str(value);
    out.push('\n');
}

/// `help:` and the help text, markdown rendered for the terminal, indented.
fn help_section(out: &mut String, paint: Paint, catalog: &LocaleCatalog, help: &str, color: bool) {
    out.push('\n');
    paint.wrap(out, "\x1b[1;96m", catalog.phrase(Phrase::Help));
    out.push_str(":\n");
    let target = if color {
        HelpRenderTarget::Ansi
    } else {
        HelpRenderTarget::PlainText
    };
    for text in render_help(help, target).lines() {
        if text.trim().is_empty() {
            out.push('\n');
        } else {
            out.push_str("  ");
            out.push_str(text.trim_end());
            out.push('\n');
        }
    }
}
