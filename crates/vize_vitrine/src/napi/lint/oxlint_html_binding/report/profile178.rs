//! The immutable 1.78 renderer borrows the same actual diagnostic and source.
use super::super::dto::OxlintHtmlPresentation;
use super::diagnostic::{Borrowed, Original};
use miette178::{
    Diagnostic, GraphicalReportHandler, GraphicalTheme, JSONReportHandler, LabeledSpan, Severity,
    SourceCode,
};
use std::borrow::Cow;
use std::fmt;

impl SourceCode for Original<'_> {
    fn data(&self) -> &[u8] {
        self.text.as_bytes()
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}

#[derive(Debug)]
struct Adapted<'a> {
    original: &'a Borrowed<'a>,
    labels: Vec<LabeledSpan>,
}
impl<'a> Adapted<'a> {
    fn new(original: &'a Borrowed<'a>) -> Self {
        Self {
            original,
            labels: original
                .labels
                .iter()
                .map(|label| {
                    LabeledSpan::new(
                        label.label().map(Into::into),
                        label.offset() as u32,
                        label.len() as u32,
                    )
                })
                .collect(),
        }
    }
}
impl fmt::Display for Adapted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.original.message)
    }
}
impl std::error::Error for Adapted<'_> {}
impl Diagnostic for Adapted<'_> {
    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.original.code))
    }
    fn severity(&self) -> Option<Severity> {
        Some(match self.original.finding.severity {
            vize_patina::Severity::Error => Severity::Error,
            vize_patina::Severity::Warning => Severity::Warning,
        })
    }
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.original.original)
    }
    fn labels(&self) -> &[LabeledSpan] {
        &self.labels
    }
}
pub(super) fn json(diagnostic: &Borrowed<'_>) -> Result<String, String> {
    let mut output = String::new();
    JSONReportHandler::new()
        .render_report(&mut output, &Adapted::new(diagnostic))
        .map_err(|_| String::from("HTML 1.78 JSON rendering failed"))?;
    Ok(output)
}
pub(super) fn graphical(
    diagnostic: &Borrowed<'_>,
    presentation: &OxlintHtmlPresentation,
) -> Result<String, String> {
    let theme = match presentation.graphical_theme.as_str() {
        "color" => GraphicalTheme::unicode(),
        "unicode" => GraphicalTheme::unicode_nocolor(),
        _ => GraphicalTheme::none(),
    };
    let mut output = String::new();
    GraphicalReportHandler::new_themed(theme)
        .with_width(presentation.width as usize)
        .with_links(false)
        .render_report(&mut output, &Adapted::new(diagnostic))
        .map_err(|_| String::from("HTML 1.78 default rendering failed"))?;
    Ok(output)
}
