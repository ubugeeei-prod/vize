use std::io::{self, Write};

use super::super::*;
use super::{TestReporter, report};
use crate::DoctorReport;

#[test]
fn execution_receipt_counts_streamed_output_without_buffering() {
    let reporter = TestReporter::new("vendor.context", b"context");
    let mut output = Vec::new();

    let receipt = render_report(&reporter, &report(), &mut output).unwrap();

    assert_eq!(output, b"context");
    assert_eq!(receipt.reporter_id(), "vendor.context");
    assert_eq!(receipt.reporter_format_version(), 1);
    assert_eq!(receipt.report_format_version(), 1);
    assert_eq!(receipt.findings_emitted(), 1);
    assert_eq!(receipt.bytes_written(), 7);
}

#[test]
fn destination_failures_preserve_partial_output_telemetry() {
    struct LimitedWriter {
        remaining: usize,
    }

    impl Write for LimitedWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "budget exhausted"));
            }
            let written = buffer.len().min(self.remaining);
            self.remaining -= written;
            Ok(written)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let reporter = TestReporter::new("vendor.context", b"context");
    let mut output = LimitedWriter { remaining: 3 };
    let error = render_report(&reporter, &report(), &mut output).unwrap_err();

    assert_eq!(error.reporter_id(), Some("vendor.context"));
    assert_eq!(error.bytes_written(), 3);
    let ReporterFailure::Rendering { error, .. } = error else {
        panic!("expected rendering failure")
    };
    assert_eq!(error.kind(), ReporterErrorKind::Write);
}

#[test]
fn built_in_json_preserves_the_stable_report_and_pretty_default() {
    let report = report();
    let reporter = JsonReporter::new();
    let mut pretty = Vec::new();
    let receipt = render_report(&reporter, &report, &mut pretty).unwrap();

    assert!(reporter.pretty());
    assert_eq!(receipt.bytes_written(), pretty.len() as u64);
    assert_eq!(
        serde_json::from_slice::<DoctorReport>(&pretty).unwrap(),
        report
    );
    assert!(pretty.ends_with(b"\n"));
    assert!(pretty.windows(2).any(|window| window == b"  "));

    let mut compact = Vec::new();
    render_report(
        &JsonReporter::new().with_pretty(false),
        &report,
        &mut compact,
    )
    .unwrap();
    assert!(compact.len() < pretty.len());
    assert_eq!(
        serde_json::from_slice::<DoctorReport>(&compact).unwrap(),
        report
    );
}
