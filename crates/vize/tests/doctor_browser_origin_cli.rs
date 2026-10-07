//! Public Doctor projection consumes the same frozen original sources as lint.
use std::{fs, path::Path, process::Command};
use vize_doctor::DoctorReport;
#[path = "support/browser_origin_current_reference.rs"]
mod current_reference;

#[test]
fn browser_diagnostics_use_authored_script_and_template_bytes() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-7907");
    let source: serde_json::Value =
        serde_json::from_slice(&fs::read(corpus.join("source.json")).unwrap()).unwrap();
    let workspace = tempfile::tempdir().unwrap();
    let cases = source["cases"].as_array().unwrap();
    let current = current_reference::validate(&corpus, &source).unwrap();
    assert_eq!(cases.len(), 11);
    for case in cases {
        let filename = case["path"].as_str().unwrap();
        let input = fs::read(corpus.join(case["file"].as_str().unwrap())).unwrap();
        fs::write(workspace.path().join(filename), &input).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(workspace.path())
            .args([
                "doctor",
                filename,
                "--format",
                "json",
                "--rule",
                "VIZE_DOCTOR_CF_BROWSER_API_SSR",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report: DoctorReport = serde_json::from_slice(&output.stdout).unwrap();
        let actual: Vec<_> = report
            .findings()
            .iter()
            .map(|finding| {
                assert_eq!(finding.code, "VIZE_DOCTOR_CF_BROWSER_API_SSR");
                assert_eq!(finding.primary.path, filename);
                assert_eq!(
                    finding.message,
                    "Browser API used in potentially SSR context"
                );
                assert!(finding.related.is_empty());
                (finding.primary.start, finding.primary.end)
            })
            .collect();
        let locations = if case["path"] == current["path"] {
            &current["currentDoctorLocations"]
        } else {
            &case["doctorLocations"]
        };
        let expected: Vec<_> = locations
            .as_array()
            .unwrap()
            .iter()
            .map(|location| {
                (
                    location["start"].as_u64().unwrap() as u32,
                    location["end"].as_u64().unwrap() as u32,
                )
            })
            .collect();
        assert_eq!(actual, expected, "{filename}: {output:?}");
        if case["path"] == current["path"] {
            assert_ne!(locations, &case["doctorLocations"]);
        }
        assert_eq!(fs::read(workspace.path().join(filename)).unwrap(), input);
        fs::remove_file(workspace.path().join(filename)).unwrap();
    }
}
