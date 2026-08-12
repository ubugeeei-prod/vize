use super::super::*;
use super::{TestReporter, report};

#[test]
fn explicit_sets_are_isolated_and_deterministically_ordered() {
    let mut first = ReporterSet::new();
    first
        .register(TestReporter::new("vendor.zeta", b"z"))
        .unwrap();
    first
        .register(TestReporter::new("vendor.alpha", b"a"))
        .unwrap();
    let mut second = ReporterSet::new();
    second
        .register_boxed(Box::new(TestReporter::new("vendor.private", b"private")))
        .unwrap();

    assert_eq!(
        first
            .descriptors()
            .map(ReporterDescriptor::id)
            .collect::<Vec<_>>(),
        ["vendor.alpha", "vendor.zeta"]
    );
    assert!(first.get("vendor.private").is_none());
    assert!(second.get("vendor.alpha").is_none());
    assert_eq!(first.len(), 2);
    assert!(!first.is_empty());
}

#[test]
fn duplicate_ids_fail_without_replacing_the_original() {
    let mut reporters = ReporterSet::new();
    reporters
        .register(TestReporter::new("vendor.context", b"original"))
        .unwrap();
    let error = reporters
        .register(TestReporter::new("vendor.context", b"replacement"))
        .unwrap_err();

    assert_eq!(
        error,
        ReporterRegistrationError::DuplicateId("vendor.context".into())
    );
    let mut output = Vec::new();
    render_report(
        reporters.get("vendor.context").unwrap(),
        &report(),
        &mut output,
    )
    .unwrap();
    assert_eq!(output, b"original");
}

#[test]
fn invalid_descriptors_never_enter_the_set() {
    let mut reporter = TestReporter::new("INVALID", b"invalid");
    reporter.descriptor = reporter
        .descriptor
        .clone()
        .with_audiences([ReporterAudience::Automation]);
    let mut reporters = ReporterSet::new();

    assert!(matches!(
        reporters.register(reporter),
        Err(ReporterRegistrationError::InvalidContract(_))
    ));
    assert!(reporters.is_empty());
}
