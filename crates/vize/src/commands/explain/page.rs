//! One `vize explain` page: a code's catalogued description, its contract,
//! its fixture examples and its help, worded in one locale.
//!
//! Everything on a page comes from the producers' metadata and the catalogue.
//! English descriptions are the rules' own `RuleMeta` text and the compiler's
//! `ErrorCode::message()`, so an English page says exactly what `vize lint`
//! and the compiler say.

use vize_davinci::diagnostic::Severity as Claim;
use vize_davinci::render::{Catalog, Phrase};
use vize_patina::{HelpRenderTarget, render_help, rule_docs_path};
use vize_relief::CompilerError;
use vize_s0::i18n::Locale;
use vize_s0::{String, cstr};

use super::catalog::LocaleCatalog;
use super::examples;
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
            line(&mut out, paint, &code.localized_message(catalog.locale()));
            out.push('\n');
            field(
                &mut out,
                paint,
                catalog,
                "explain.stage",
                code.stage().as_str(),
            );
            let severity = if CompilerError::new(*code, None).is_recoverable() {
                Phrase::Warning
            } else {
                Phrase::Error
            };
            field(
                &mut out,
                paint,
                catalog,
                "explain.severity",
                catalog.phrase(severity),
            );
            if *code != vize_relief::ErrorCode::ExtendPoint {
                help_section(
                    &mut out,
                    paint,
                    catalog,
                    &code.localized_help(catalog.locale()),
                    color,
                );
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
    if let Some(tier) = rule.tier {
        field(out, paint, catalog, "explain.tier", tier);
    }
    if let Some(domain) = rule.domain {
        field(out, paint, catalog, "explain.domain", domain);
    }
    field(
        out,
        paint,
        catalog,
        "explain.severity",
        catalog.phrase(severity_phrase(rule.severity)),
    );
    let category = lowercase(rule.category);
    field(out, paint, catalog, "explain.category", category.as_str());
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
    if let Some(fixtures) = examples::get(rule.name) {
        if let Some(invalid) = &fixtures.invalid {
            example(out, paint, catalog, "explain.example.invalid", invalid);
        }
        if let Some(valid) = &fixtures.valid {
            example(out, paint, catalog, "explain.example.valid", valid);
        }
    }
    if let Some(help) = catalog.text(&cstr!("{}.help", rule.name)) {
        help_section(out, paint, catalog, help, color);
    }
}

fn severity_phrase(severity: Claim) -> Phrase {
    match severity {
        Claim::Error => Phrase::Error,
        Claim::Warning => Phrase::Warning,
        Claim::Info => Phrase::Info,
        Claim::Hint => Phrase::Hint,
    }
}

fn lowercase(text: &str) -> String {
    let mut out = String::new("");
    out.extend(text.chars().map(|ch| ch.to_ascii_lowercase()));
    out
}

fn heading(out: &mut String, paint: Paint, code: &str, kind: &str) {
    paint.code(out, code);
    out.push_str(" · ");
    out.push_str(kind);
    out.push('\n');
}

fn line(out: &mut String, paint: Paint, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        out.push('\n');
        return;
    }
    for row in text.lines() {
        paint.strong(out, row.trim_end());
        out.push('\n');
    }
}

fn field(out: &mut String, paint: Paint, catalog: &LocaleCatalog, key: &str, value: &str) {
    paint.label(out, &catalog.format(key, &[]));
    out.push_str(": ");
    out.push_str(value);
    out.push('\n');
}

fn example(out: &mut String, paint: Paint, catalog: &LocaleCatalog, key: &str, body: &str) {
    out.push('\n');
    paint.label(out, &catalog.format(key, &[]));
    out.push_str(":\n");
    for row in body.lines() {
        out.push_str("  ");
        out.push_str(row.trim_end());
        out.push('\n');
    }
}

/// `help:` and the help text, markdown rendered for the terminal, indented.
fn help_section(out: &mut String, paint: Paint, catalog: &LocaleCatalog, help: &str, color: bool) {
    if help.trim().is_empty() {
        return;
    }
    out.push('\n');
    paint.wrap(out, "\x1b[1;96m", catalog.phrase(Phrase::Help));
    out.push_str(":\n");
    // `render_help`'s ANSI path walks bytes as characters, which corrupts
    // Japanese and Chinese. Colour the label above; style the body only when
    // it is ASCII.
    let target = if color && help.is_ascii() {
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
