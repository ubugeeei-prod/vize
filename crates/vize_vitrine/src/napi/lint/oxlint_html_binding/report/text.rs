//! Original-file Unix and Stylish contracts; no path/source is reread.
use super::super::super::file_collection::oxlint_html_profile::Original;
use super::super::super::oxlint_html::{FileResult, Operation};
use super::super::dto::OxlintHtmlOptions;
use super::diagnostic::Borrowed;
use oxlint_html_renderer186::source_impls::SpanScanner;
use std::fmt::Write;
use vize_l0::cstr;
use vize_patina::Severity;

pub(super) fn file(
    file: &FileResult,
    original: &Original,
    operation: &Operation,
    options: &OxlintHtmlOptions,
) -> Result<String, String> {
    if file.result.diagnostics.is_empty() {
        return Ok(String::new());
    }
    let source = std::str::from_utf8(&original.bytes)
        .map_err(|_| String::from("retained HTML is not UTF8"))?;
    let mut scanner = SpanScanner::new(original.bytes.as_slice(), 0, 0);
    let mut entries = file
        .result
        .diagnostics
        .iter()
        .map(|finding| {
            let diagnostic = Borrowed::new(
                finding,
                source,
                &original.cwd_relative,
                &operation.projection.settings.help_level,
            )?;
            let location = scanner
                .read_span(oxc_span::Span::new(finding.start, finding.end))
                .ok_or_else(|| String::from("HTML span has no retained original location"))?;
            Ok((location.line() + 1, location.column() + 1, diagnostic))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut output = String::new();
    if options.format == "unix" {
        for (line, column, diagnostic) in entries {
            writeln!(
                output,
                "{}:{line}:{column}: {} [{}/{}]",
                diagnostic.original.name,
                diagnostic.message,
                if diagnostic.finding.severity == Severity::Error {
                    "Error"
                } else {
                    "Warning"
                },
                diagnostic.code
            )
            .map_err(|_| String::from("HTML Unix rendering failed"))?;
        }
        return Ok(output);
    }
    entries.sort_by_key(|entry| entry.0);
    let width = entries
        .iter()
        .map(|(line, column, _)| cstr!("{line}:{column}").len())
        .max()
        .unwrap_or(0);
    let heading = if options.presentation.stylish_relative.unwrap_or(false) {
        &original.cwd_relative
    } else {
        &original.path
    };
    let name = heading
        .to_str()
        .ok_or_else(|| String::from("HTML filename is not UTF8"))?;
    let no_color = options.presentation.stylish_no_color;
    let (underline, reset) = if no_color {
        ("", "")
    } else {
        ("\u{1b}[4m", "\u{1b}[0m")
    };
    writeln!(output, "\n{underline}{name}{reset}")
        .map_err(|_| String::from("HTML Stylish heading failed"))?;
    for (line, column, diagnostic) in entries {
        let position = cstr!("{line}:{column}");
        let position = position.as_str();
        let severity = if diagnostic.finding.severity == Severity::Error {
            if no_color {
                "error"
            } else {
                "\u{1b}[31merror\u{1b}[0m"
            }
        } else if no_color {
            "warning"
        } else {
            "\u{1b}[33mwarning\u{1b}[0m"
        };
        let (dim, end) = if no_color {
            ("", "")
        } else {
            ("\u{1b}[2m", "\u{1b}[0m")
        };
        writeln!(
            output,
            "  {dim}{position:width$}{end}  {severity}  {}  {dim}{}{end}",
            diagnostic.message, diagnostic.code
        )
        .map_err(|_| String::from("HTML Stylish diagnostic failed"))?;
    }
    Ok(output)
}

pub(super) fn footer(
    output: &mut String,
    errors: usize,
    warnings: usize,
    no_color: bool,
) -> Result<(), String> {
    let total = errors + warnings;
    let color = if no_color {
        ""
    } else if errors > 0 {
        "\u{1b}[31m"
    } else {
        "\u{1b}[33m"
    };
    let reset = if no_color { "" } else { "\u{1b}[0m" };
    writeln!(
        output,
        "\n{color}✖ {total} problem{} ({errors} error{}, {warnings} warning{}){reset}",
        if total == 1 { "" } else { "s" },
        if errors == 1 { "" } else { "s" },
        if warnings == 1 { "" } else { "s" }
    )
    .map_err(|_| String::from("HTML Stylish summary failed"))
}
