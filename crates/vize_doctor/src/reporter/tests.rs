//! Shared reporter fixtures and responsibility-scoped tests.

mod contract;
mod execution;
mod registry;

use std::io::Write;

use super::*;
use crate::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport, FindingAssessment,
    FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};

pub(super) struct TestReporter {
    descriptor: ReporterDescriptor,
    payload: &'static [u8],
}

impl TestReporter {
    pub(super) fn new(id: &str, payload: &'static [u8]) -> Self {
        Self {
            descriptor: ReporterDescriptor::new(
                id,
                "Test Reporter",
                "application/vnd.test+json",
                ReporterTransport::Document,
            )
            .with_audiences([ReporterAudience::Automation])
            .with_capabilities([ReporterCapability::Findings]),
            payload,
        }
    }
}

impl DoctorReporter for TestReporter {
    fn descriptor(&self) -> &ReporterDescriptor {
        &self.descriptor
    }

    fn write_report(
        &self,
        _report: &DoctorReport,
        output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError> {
        output.write_all(self.payload)?;
        Ok(())
    }
}

pub(super) fn report() -> DoctorReport {
    DoctorReport::new(
        "workspace",
        [DoctorFinding::new(
            "VIZE_DOCTOR_TEST",
            DoctorCategory::Correctness,
            FindingAssessment::new(
                FindingSeverity::Warning,
                FindingConfidence::High,
                FindingImpact::Medium,
                HealthPenalty::new(10, "Test penalty"),
            ),
            SourceLocation::new("src/App.vue", 4, 9),
            "Test finding",
            "Test message",
            AnalysisProvenance::new("test-analysis", RuleCost::Low),
        )],
    )
}
