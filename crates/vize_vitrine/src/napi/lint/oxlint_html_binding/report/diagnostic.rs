//! Borrow retained original text and actual Patina findings during presentation.
use std::borrow::Cow;
use std::fmt;
use std::path::Path;

use oxlint_html_renderer186::{Diagnostic, LabeledSpan, Severity, SourceCode};
use vize_l0::cstr;
use vize_patina::LintDiagnostic;

#[derive(Debug)]
pub(super) struct Original<'a> {
    pub text: &'a str,
    pub name: &'a str,
}

impl SourceCode for Original<'_> {
    fn data(&self) -> &[u8] {
        self.text.as_bytes()
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}

#[derive(Debug)]
pub(super) struct Borrowed<'a> {
    pub finding: &'a LintDiagnostic,
    pub original: Original<'a>,
    pub message: String,
    pub code: String,
    pub labels: Vec<LabeledSpan>,
}

impl<'a> Borrowed<'a> {
    pub fn new(
        finding: &'a LintDiagnostic,
        source: &'a str,
        filename: &'a Path,
        help_level: &str,
    ) -> Result<Self, String> {
        let name = filename
            .to_str()
            .ok_or_else(|| String::from("HTML renderer requires an original UTF8 path"))?;
        let valid = |start: u32, end: u32| source.get(start as usize..end as usize).is_some();
        if !valid(finding.start, finding.end)
            || finding
                .labels
                .iter()
                .any(|label| !valid(label.start, label.end))
        {
            return Err("HTML finding has a span outside the retained original UTF8 text".into());
        }
        let mut labels = vec![LabeledSpan::new(
            None,
            finding.start as usize,
            (finding.end - finding.start) as usize,
        )];
        labels.extend(finding.labels.iter().map(|label| {
            LabeledSpan::new(
                Some(label.message.as_str().into()),
                label.start as usize,
                (label.end - label.start) as usize,
            )
        }));
        Ok(Self {
            finding,
            original: Original { text: source, name },
            message: super::message::format(finding, help_level)?,
            code: cstr!("vize({})", finding.rule_name).as_str().into(),
            labels,
        })
    }
}

impl fmt::Display for Borrowed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for Borrowed<'_> {}
impl Diagnostic for Borrowed<'_> {
    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.code))
    }
    fn severity(&self) -> Option<Severity> {
        Some(match self.finding.severity {
            vize_patina::Severity::Error => Severity::Error,
            vize_patina::Severity::Warning => Severity::Warning,
        })
    }
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.original)
    }
    fn labels(&self) -> &[LabeledSpan] {
        &self.labels
    }
}
