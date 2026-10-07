//! Public Doctor findings over every original and independently authored context.
use std::{fs, path::Path, process::Command};
use vize_doctor::DoctorReport;

#[test]
fn browser_ssr_contexts_preserve_complete_selected_doctor_findings_and_source_bytes() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-7908");
    let source: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("source.json")).unwrap()).unwrap();
    let cases = source["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 38);
    let workspace = tempfile::tempdir().unwrap();
    for case in cases {
        let filename = case["path"].as_str().unwrap();
        let input = fs::read(directory.join(case["file"].as_str().unwrap())).unwrap();
        let target = workspace.path().join(filename);
        fs::write(&target, &input).unwrap();
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
        assert!(output.status.success(), "{filename}: {output:?}");
        assert!(output.stderr.is_empty(), "{filename}: {output:?}");
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
        let expected: Vec<_> = case["doctorLocations"]
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
        assert_eq!(fs::read(&target).unwrap(), input);
        fs::remove_file(target).unwrap();
    }
}
