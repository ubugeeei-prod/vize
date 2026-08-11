use super::super::*;

#[test]
fn descriptor_normalizes_and_round_trips_stably() {
    let descriptor = ReporterDescriptor::new(
        "vendor.context",
        "Vendor-neutral Context",
        "application/vnd.vendor.context+json",
        ReporterTransport::Document,
    )
    .with_format_version(3)
    .with_file_extension("json")
    .with_audiences([
        ReporterAudience::Ai,
        ReporterAudience::Automation,
        ReporterAudience::Ai,
    ])
    .with_capabilities([
        ReporterCapability::Evidence,
        ReporterCapability::Findings,
        ReporterCapability::Evidence,
    ]);

    descriptor.validate().unwrap();
    assert_eq!(
        descriptor.audiences(),
        [ReporterAudience::Automation, ReporterAudience::Ai]
    );
    assert_eq!(
        descriptor.capabilities(),
        [ReporterCapability::Findings, ReporterCapability::Evidence]
    );
    let json = serde_json::to_string(&descriptor).unwrap();
    assert_eq!(
        serde_json::from_str::<ReporterDescriptor>(&json).unwrap(),
        descriptor
    );
}

#[test]
fn descriptor_rejects_every_ambiguous_wire_boundary() {
    for (descriptor, field) in [
        (
            descriptor("Vendor.Context", "Context", "application/json"),
            "id",
        ),
        (
            descriptor("vendor.context", "\n", "application/json"),
            "displayName",
        ),
        (
            descriptor(
                "vendor.context",
                "Context",
                "Application/JSON; charset=utf-8",
            ),
            "mediaType",
        ),
        (
            descriptor("vendor.context", "Context", "application/json").with_format_version(0),
            "formatVersion",
        ),
        (
            descriptor("vendor.context", "Context", "application/json")
                .with_file_extension(".json"),
            "fileExtension",
        ),
        (
            ReporterDescriptor::new(
                "vendor.context",
                "Context",
                "application/json",
                ReporterTransport::Document,
            )
            .with_capabilities([ReporterCapability::Findings]),
            "audiences",
        ),
        (
            ReporterDescriptor::new(
                "vendor.context",
                "Context",
                "application/json",
                ReporterTransport::Document,
            )
            .with_audiences([ReporterAudience::Ai]),
            "capabilities",
        ),
    ] {
        let error = descriptor.validate().unwrap_err();
        assert_eq!(error.field(), field);
        let value = serde_json::to_value(&descriptor).unwrap();
        assert!(serde_json::from_value::<ReporterDescriptor>(value).is_err());
    }
}

#[test]
fn descriptor_rejects_unknown_fields_and_contract_versions() {
    let descriptor = descriptor("vendor.context", "Context", "application/json");
    let mut unknown = serde_json::to_value(&descriptor).unwrap();
    unknown["providerSecret"] = "must-not-pass".into();
    assert!(serde_json::from_value::<ReporterDescriptor>(unknown).is_err());

    let mut future = serde_json::to_value(descriptor).unwrap();
    future["contractVersion"] = (DOCTOR_REPORTER_CONTRACT_VERSION + 1).into();
    assert!(serde_json::from_value::<ReporterDescriptor>(future).is_err());
}

fn descriptor(id: &str, display_name: &str, media_type: &str) -> ReporterDescriptor {
    ReporterDescriptor::new(id, display_name, media_type, ReporterTransport::Document)
        .with_audiences([ReporterAudience::Ai])
        .with_capabilities([ReporterCapability::Findings])
}
