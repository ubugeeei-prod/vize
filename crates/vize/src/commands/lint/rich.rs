//! `vize lint --format rich`: lint results through the Davinci diagnostic
//! renderer (P4-14a), in the locale `--locale` selects.
//!
//! This is the CLI edge the renderer's contract names: the one place a
//! `vize_carton` translator becomes a [`Catalog`], and the interim mapping
//! from Patina's `LintDiagnostic` onto the unified `vize_davinci::Diagnostic`.
//! The mapping is deliberately mechanical — a label becomes a secondary
//! part, a fix becomes a titled run of suggestions, help becomes a footer —
//! and P4-6c's canonical Patina conversion replaces it, taking the
//! file-absolute span fix for FP-1 with it.

use std::borrow::Cow;
use std::io::IsTerminal;

use vize_davinci::diagnostic::{Advisory, Diagnostic, DiagnosticPart, Exemption, PartKind, Stage};
use vize_davinci::render::{Catalog, EnglishCatalog, Phrase, Renderer, SourceFile};
use vize_fresco::{
    ColorSupport, TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
use vize_patina::{HelpRenderTarget, LintDiagnostic, LintResult, OutputFormat, render_help};
use vize_s0::i18n::{Locale, Translator, translator};
use vize_s0::{FxHashMap, Span, String, cstr};

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
            renderer.render_into(&mut out, &file, Some(lint.rule_name), &unify(lint));
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

/// Patina's error-severity findings reach the unified channel without
/// witnesses until P4-6c's conversion; they are exempt by inventory
/// (`davinci-road/plan/witness-exemptions.tsv`), never silently.
static PATINA_LINT: Exemption = Exemption::new("vize", "patina-lint");

/// A Patina diagnostic on the unified channel. Fix parts come before the
/// general help so the help stays a footer rather than titling the fix.
pub(crate) fn unify(lint: &LintDiagnostic) -> Diagnostic {
    let span = Span::new(lint.start, lint.end);
    let message = lint.message.as_str();
    let mut diagnostic = match lint.severity {
        vize_patina::Severity::Error => {
            Diagnostic::legacy_error(&PATINA_LINT, Stage::Semantic, span, message)
        }
        vize_patina::Severity::Warning => {
            Diagnostic::new(Advisory::Warning, Stage::Semantic, span, message)
        }
    };
    for label in &lint.labels {
        let label_span = Span::new(label.start, label.end);
        let part = DiagnosticPart::new(PartKind::Secondary, label_span, label.message.as_str());
        diagnostic = diagnostic.with_part(part);
    }
    if let Some(fix) = &lint.fix
        && !fix.edits.is_empty()
    {
        if !fix.message.trim().is_empty() {
            let title = DiagnosticPart::new(PartKind::Help, span, fix.message.as_str());
            diagnostic = diagnostic.with_part(title);
        }
        for edit in &fix.edits {
            let edit_span = Span::new(edit.start, edit.end);
            let part = DiagnosticPart::new(PartKind::Suggestion, edit_span, edit.new_text.as_str());
            diagnostic = diagnostic.with_part(part);
        }
    }
    if let Some(help) = &lint.help {
        let help = render_help(help, HelpRenderTarget::PlainText);
        if !help.trim().is_empty() {
            let part = DiagnosticPart::new(PartKind::Help, span, help.as_str());
            diagnostic = diagnostic.with_part(part);
        }
    }
    diagnostic
}
