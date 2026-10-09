use super::{ESCAPE_REFUSAL, capture_command, command};
use serde_json::Value;
use std::{fs, process::Command};

#[test]
#[cfg(unix)]
fn retirement_raw_log_capture_preserves_real_ansi_bytes_and_documented_cli_refusal() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/github-cli-log-escape.json"
    ))
    .unwrap();
    let raw = fixture["rawLog"].as_str().unwrap().as_bytes();
    assert_eq!(raw.len(), 175_686);
    assert_eq!(raw.iter().filter(|byte| **byte == 0x1b).count(), 4_097);
    assert_eq!(fixture["original_exact_error"], ESCAPE_REFUSAL);
    let repo = super::super::super::tests::Repo::new();
    fs::write(repo.work.join("log.bin"), raw).unwrap();
    fs::write(repo.work.join("stderr.txt"), format!("{ESCAPE_REFUSAL}\n")).unwrap();
    let script = r#"case " $* " in *' --allow-escape-sequences '*) cat log.bin;; *) cat stderr.txt >&2; exit 1;; esac"#;
    let mut refused = Command::new("sh");
    refused
        .args([
            "-c",
            script,
            "gh",
            "api",
            "repos/owner/repo/actions/jobs/113945842593/logs",
        ])
        .current_dir(&repo.work);
    let error = capture_command(refused, 113945842593).unwrap_err();
    assert!(error.contains(ESCAPE_REFUSAL));
    assert!(error.contains("stdout=0 bytes"));
    assert!(!error.contains('\u{1b}'));
    let planned = command("owner/repo", 113945842593, &repo.work);
    assert_eq!(
        planned
            .get_args()
            .map(|arg| arg.to_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "api",
            "repos/owner/repo/actions/jobs/113945842593/logs",
            "--allow-escape-sequences"
        ]
    );
    let mut accepted = Command::new("sh");
    accepted
        .args(["-c", script, "gh"])
        .args(planned.get_args())
        .current_dir(&repo.work);
    assert_eq!(capture_command(accepted, 113945842593).unwrap(), raw);
}

#[test]
#[cfg(unix)]
fn retirement_log_capture_refuses_empty_or_failed_output_without_exposing_raw_diagnostics() {
    for (script, classification) in [
        ("exit 0", "empty complete job log"),
        (
            r"printf 'partial'; printf 'https://signed.invalid/log?sig=secret ghp_test_token\n\033[31m' >&2; exit 1",
            "unrecognized GitHub CLI failure; raw diagnostics withheld",
        ),
    ] {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        let error = capture_command(command, 123).unwrap_err();
        assert!(error.contains(classification));
        assert!(error.contains("stdout="));
        assert!(error.contains("stderr="));
        for withheld in [
            "signed.invalid",
            "sig=secret",
            "ghp_test_token",
            "partial",
            "\n",
            "\r",
            "\u{1b}",
        ] {
            assert!(
                !error.contains(withheld),
                "diagnostic must withhold {withheld:?}"
            );
        }
        assert!(error.len() < 400);
    }
}
