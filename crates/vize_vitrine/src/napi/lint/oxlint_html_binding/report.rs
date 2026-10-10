//! Borrow actual diagnostics/original bytes for the only final conversion.
use super::super::file_collection::oxlint_html_profile::HostProfile;
use super::super::oxlint_html::Operation;
use super::dto::OxlintHtmlOptions;
use std::fmt::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_l0::cstr;
use vize_patina::Severity;

mod diagnostic;
mod message;
mod profile178;
mod text;

pub(super) struct Rendered {
    pub output: String,
    pub json_diagnostics: String,
    pub errors: usize,
    pub warnings: usize,
}

pub(super) fn validate(options: &OxlintHtmlOptions) -> Result<(), String> {
    if !matches!(
        options.format.as_str(),
        "default" | "json" | "unix" | "stylish"
    ) {
        return Err("HTML presentation requires an explicit qualified output format".into());
    }
    if !matches!(
        options.presentation.graphical_theme.as_str(),
        "plain" | "unicode" | "color"
    ) || options.presentation.links
        || options.presentation.width != 400
        || options.presentation.cwd != options.cwd
    {
        return Err("HTML presentation requires the original piped-child context".into());
    }
    Ok(())
}

pub(super) fn render(
    operation: &Operation,
    options: &OxlintHtmlOptions,
) -> Result<Rendered, String> {
    catch_unwind(AssertUnwindSafe(|| render_inner(operation, options))).map_err(|_| {
        String::from(
            "HTML renderer hit an internal defect; original host packet remains authoritative",
        )
    })?
}

fn render_inner(operation: &Operation, options: &OxlintHtmlOptions) -> Result<Rendered, String> {
    use oxlint_html_renderer186::{GraphicalReportHandler, GraphicalTheme, JSONReportHandler};
    let theme = match options.presentation.graphical_theme.as_str() {
        "color" => GraphicalTheme::unicode(),
        "unicode" => GraphicalTheme::unicode_nocolor(),
        _ => GraphicalTheme::none(),
    };
    let handler = GraphicalReportHandler::new_themed(theme)
        .with_width(options.presentation.width as usize)
        .with_links(false);
    let mut output = String::new();
    let mut fragments = Vec::new();
    let mut errors = 0;
    let mut warnings = 0;
    for (original, file) in operation.selection.originals.iter().zip(&operation.files) {
        let source = std::str::from_utf8(&original.bytes)
            .map_err(|_| String::from("retained HTML is not UTF8"))?;
        for finding in &file.result.diagnostics {
            if finding.severity == Severity::Error {
                errors += 1;
            } else {
                warnings += 1;
            }
            let diagnostic = diagnostic::Borrowed::new(
                finding,
                source,
                &original.cwd_relative,
                &operation.projection.settings.help_level,
            )?;
            let mut json = String::new();
            if operation.selection.host == HostProfile::Oxlint186 {
                JSONReportHandler::new()
                    .render_report(&mut json, &diagnostic)
                    .map_err(|_| String::from("HTML JSON rendering failed"))?;
            } else {
                json = profile178::json(&diagnostic)?;
            }
            fragments.push(json);
            if options.format == "default" {
                if operation.selection.host == HostProfile::Oxlint186 {
                    handler
                        .render_report(&mut output, &diagnostic)
                        .map_err(|_| String::from("HTML default rendering failed"))?;
                } else {
                    output.push_str(&profile178::graphical(&diagnostic, &options.presentation)?);
                }
            }
        }
        if matches!(options.format.as_str(), "unix" | "stylish") {
            output.push_str(&text::file(file, original, operation, options)?);
        }
    }
    let json_diagnostics = cstr!("[{}]", fragments.join(",")).as_str().into();
    if !operation.files.is_empty() {
        match options.format.as_str() {
            "default" => {
                if errors + warnings > 0 {
                    output.push('\n');
                }
                writeln!(
                    output,
                    "Found {warnings} warning{} and {errors} error{}.",
                    if warnings == 1 { "" } else { "s" },
                    if errors == 1 { "" } else { "s" }
                )
                .map_err(|_| String::from("HTML summary rendering failed"))?;
                writeln!(
                    output,
                    "Vize HTML: {} file{} in {:.3}ms.",
                    operation.executed_file_count,
                    if operation.executed_file_count == 1 {
                        ""
                    } else {
                        "s"
                    },
                    operation.elapsed.as_secs_f64() * 1000.0
                )
                .map_err(|_| String::from("HTML timing rendering failed"))?;
            }
            "unix" if errors + warnings > 0 => {
                writeln!(
                    output,
                    "\n{} problem{}",
                    errors + warnings,
                    if errors + warnings == 1 { "" } else { "s" }
                )
                .map_err(|_| String::from("HTML Unix summary rendering failed"))?;
            }
            "stylish" if errors + warnings > 0 => text::footer(
                &mut output,
                errors,
                warnings,
                options.presentation.stylish_no_color,
            )?,
            _ => {}
        }
    }
    Ok(Rendered {
        output,
        json_diagnostics,
        errors,
        warnings,
    })
}
