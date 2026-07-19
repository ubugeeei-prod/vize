//! Deterministic terminal and automation output.

use std::io::{self, Write};
use vize_doctor::{DoctorFinding, DoctorReport, FindingSeverity};

use super::{DoctorError, DoctorFormat};

pub(super) fn write_report(report: &DoctorReport, format: DoctorFormat) -> Result<(), DoctorError> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match format {
        DoctorFormat::Text => write_text(&mut output, report).map_err(DoctorError::Write),
        DoctorFormat::Json => {
            let serialized =
                serde_json::to_string_pretty(report).map_err(DoctorError::Serialize)?;
            writeln!(output, "{serialized}").map_err(DoctorError::Write)
        }
    }
}

fn write_text(output: &mut impl Write, report: &DoctorReport) -> io::Result<()> {
    let summary = report.summary();
    writeln!(output, "Vize Doctor")?;
    writeln!(output, "Workspace: {}", report.workspace())?;
    writeln!(output, "Health: {}/100", summary.overall_score)?;
    writeln!(
        output,
        "Findings: {} error(s), {} warning(s), {} notice(s)",
        summary.counts.errors, summary.counts.warnings, summary.counts.notices
    )?;

    for finding in report.findings() {
        write_finding(output, finding)?;
    }
    if report.findings().is_empty() {
        writeln!(
            output,
            "\nNo health findings in the analyzed application graph."
        )?;
    }
    Ok(())
}

fn write_finding(output: &mut impl Write, finding: &DoctorFinding) -> io::Result<()> {
    let severity = match finding.assessment.severity {
        FindingSeverity::Error => "error",
        FindingSeverity::Warning => "warning",
        FindingSeverity::Notice => "notice",
    };
    writeln!(
        output,
        "\n[{severity}] {} — {}",
        finding.code, finding.title
    )?;
    writeln!(
        output,
        "  {}:{}..{}",
        finding.primary.path, finding.primary.start, finding.primary.end
    )?;
    writeln!(output, "  {}", finding.message)?;
    if let Some(scenario) = &finding.failure_scenario {
        writeln!(output, "  Failure scenario: {scenario}")?;
    }
    Ok(())
}
