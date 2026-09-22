//! `vize lint --format rich`: lint results through the Davinci diagnostic
//! renderer (P4-14a), in the locale `--locale` selects.
//!
//! This is the CLI edge the renderer's contract names: the one place a
//! `vize_carton` translator becomes a [`Catalog`]. Patina diagnostics join the
//! unified channel through P4-6c's canonical conversion
//! ([`vize_patina::output::unified::to_unified`]: contract-clamped severity,
//! counted exemptions, ranges checked against the authored file); this module
//! only arranges the result for the terminal — the fix's own message titles
//! its suggestions and Markdown help becomes a plain-text footer.

use std::borrow::Cow;
use std::io::IsTerminal;

use vize_davinci::diagnostic::{Diagnostic, DiagnosticPart, PartKind};
use vize_davinci::render::{Catalog, EnglishCatalog, Phrase, Renderer, SourceFile};
use vize_fresco::{
    ColorSupport, TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
use vize_patina::output::unified::{UnifiedError, to_unified};
use vize_patina::{HelpRenderTarget, LintDiagnostic, LintResult, OutputFormat, render_help};
use vize_s0::i18n::{Locale, Translator, translator};
use vize_s0::{FxHashMap, SourceRoot, String, cstr};

/// The `--format` value selecting this renderer.
const RICH: &str = "rich";

/// Parse `--format`, exiting with usage on an unknown name. `rich` reports
/// through [`OutputFormat::Ansi`] for everything but the rendering itself: it
/// is a whole-report format that renders details even under `--quiet`.
pub(super) fn parse_format(format: &str) -> (OutputFormat, bool) {
    if format == RICH {
        return (OutputFormat::Ansi, true);
    }
    let parsed = OutputFormat::parse(format).unwrap_or_else(|| {
        eprintln!(
            "Unknown lint output format '{format}'. Expected one of: text, rich, ansi, plain, json, stylish, markdown, html, agent"
        );
        std::process::exit(2);
    });
    (parsed, false)
}

/// Parse `--locale`, exiting with usage on an unknown locale.
pub(super) fn parse_locale(locale: &str) -> Locale {
    Locale::parse(locale).unwrap_or_else(|| {
        eprintln!("Unknown locale '{locale}'. Expected one of: en, ja, zh");
        std::process::exit(2);
    })
}

/// Render `results` richly when `rich` is set, through Patina's formatter
/// otherwise.
pub(super) fn format_results(
    rich: bool,
    locale: Locale,
    results: &[LintResult],
    sources: &[(String, String)],
    format: OutputFormat,
) -> String {
    if !rich {
        return vize_patina::format_results(results, sources, format);
    }
    let color = color_enabled();
    render(results, sources, locale, color)
}

fn color_enabled() -> bool {
    let probe = TerminalCapabilityProbe::from_process(80, 24, std::io::stdout().is_terminal());
    let capabilities = TerminalCapabilities::resolve(&probe, TerminalProfileOptions::default());
    capabilities.color().value() != ColorSupport::Monochrome
}

/// The shipped translator in one locale, as renderer vocabulary. A phrase the
/// locale lacks falls back to the built-in English, never to its key; the
/// TS-53 catalog check keeps that fallback unreachable.
pub(crate) struct LocaleCatalog {
    translator: &'static Translator,
    locale: Locale,
}

impl LocaleCatalog {
    pub(crate) fn new(locale: Locale) -> Self {
        Self {
            translator: translator(),
            locale,
        }
    }

    fn format(&self, key: &str, vars: &[(&str, &str)]) -> String {
        self.translator
            .format(self.locale, key, vars)
            .as_str()
            .into()
    }

    fn count(&self, noun: &str, count: usize) -> String {
        let form = if count == 1 { "one" } else { "other" };
        let count = cstr!("{count}");
        self.format(&cstr!("render.summary.{noun}.{form}"), &[("count", &count)])
    }
}

impl Catalog for LocaleCatalog {
    fn phrase(&self, phrase: Phrase) -> &str {
        if self.translator.has_key(self.locale, phrase.key())
            && let Cow::Borrowed(text) = self.translator.get(self.locale, phrase.key())
        {
            return text;
        }
        EnglishCatalog.phrase(phrase)
    }
}

/// Render every diagnostic of every result, then the summary line.
pub(crate) fn render(
    results: &[LintResult],
    sources: &[(String, String)],
    locale: Locale,
    color: bool,
) -> String {
    let catalog = LocaleCatalog::new(locale);
    let renderer = Renderer::new(&catalog).with_color(color);
    let texts: FxHashMap<&str, &str> = sources
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    let mut out = String::default();
    let (mut errors, mut warnings) = (0, 0);
    for result in results {
        errors += result.error_count;
        warnings += result.warning_count;
        let text = texts.get(result.filename.as_str()).copied().unwrap_or("");
        let file = SourceFile::new(result.filename.as_str(), text);
        for lint in &result.diagnostics {
            match unify(lint, text) {
                Ok(diagnostic) => {
                    renderer.render_into(&mut out, &file, Some(lint.rule_name), &diagnostic);
                }
                Err(_) => refused(&mut out, &catalog, lint),
            }
            out.push('\n');
        }
    }
    let files = catalog.count("files", results.len());
    let summary = if errors == 0 && warnings == 0 {
        catalog.format("render.summary.clean", &[("files", &files)])
    } else {
        let errors = catalog.count("errors", errors);
        let warnings = catalog.count("warnings", warnings);
        let vars = [
            ("errors", &*errors),
            ("warnings", &*warnings),
            ("files", &*files),
        ];
        catalog.format("render.summary", &vars)
    };
    if color {
        out.push_str("\x1b[1m");
        out.push_str(&summary);
        out.push_str("\x1b[0m\n");
    } else {
        out.push_str(&summary);
        out.push('\n');
    }
    out
}

/// A diagnostic the unified channel refuses (an unknown rule, or a range
/// outside the file — a producer defect) still reaches the reader, as a bare
/// headline rather than an excerpt drawn on the wrong line.
fn refused(out: &mut String, catalog: &LocaleCatalog, lint: &LintDiagnostic) {
    let word = catalog.phrase(match lint.severity {
        vize_patina::Severity::Error => Phrase::Error,
        vize_patina::Severity::Warning => Phrase::Warning,
    });
    out.push_str(&cstr!("{word}[{}]: {}\n", lint.rule_name, lint.message));
}

/// `lint` on the unified channel, arranged for the terminal: the fix's
/// message titles its suggestions (a help part directly before a suggestion
/// run), and Markdown help follows as a plain-text footer.
pub(crate) fn unify(lint: &LintDiagnostic, source: &str) -> Result<Diagnostic, UnifiedError> {
    let root = SourceRoot::new(source).map_err(|_| UnifiedError::SpanOutsideSource {
        rule: lint.rule_name,
        span: vize_s0::Span::new(lint.start, lint.end),
    })?;
    let mut diagnostic = to_unified(lint, root)?;
    let parts = core::mem::take(&mut diagnostic.parts);
    let (fixes, rest): (Vec<_>, Vec<_>) = parts
        .into_iter()
        .partition(|part| part.kind == PartKind::Suggestion);
    let mut footers = Vec::new();
    for part in rest {
        if part.kind == PartKind::Help {
            let help = render_help(&part.message, HelpRenderTarget::PlainText);
            if !help.trim().is_empty() {
                footers.push(DiagnosticPart::new(
                    PartKind::Help,
                    part.span,
                    help.as_str(),
                ));
            }
        } else {
            diagnostic.parts.push(part);
        }
    }
    if let Some(fix) = lint.fix.as_ref().filter(|_| !fixes.is_empty())
        && !fix.message.trim().is_empty()
    {
        let title = DiagnosticPart::new(PartKind::Help, diagnostic.span, fix.message.as_str());
        diagnostic.parts.push(title);
    }
    diagnostic.parts.extend(fixes);
    diagnostic.parts.extend(footers);
    Ok(diagnostic)
}

#[cfg(test)]
mod tests {
    use super::{LocaleCatalog, refused, unify};
    use vize_patina::LintDiagnostic;
    use vize_patina::output::unified::UnifiedError;
    use vize_s0::i18n::Locale;
    use vize_s0::{Span, String};

    #[test]
    fn a_range_outside_the_file_is_refused_yet_still_reported() {
        let lint = LintDiagnostic::warn("vue/no-v-html", "v-html", 3, 40);
        let outside = UnifiedError::SpanOutsideSource {
            rule: "vue/no-v-html",
            span: Span::new(3, 40),
        };
        assert_eq!(unify(&lint, "<p/>").err(), Some(outside));
        let mut out = String::default();
        refused(&mut out, &LocaleCatalog::new(Locale::Ja), &lint);
        assert_eq!(out, "警告[vue/no-v-html]: v-html\n");
    }
}
