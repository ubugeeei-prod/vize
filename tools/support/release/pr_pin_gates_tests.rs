//! Whole primary API rows from #8315: protected and push checks share G.
use super::delivery::gates;
use crate::pr_checks;
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/_fixtures/tooling/release/protected-check-suites/actual-8315-merge-group-and-push.json"
    )).unwrap()
}

fn selection(packet: &Value) -> Result<Vec<Value>, String> {
    gates::protected_checks(
        packet["runs"].as_array().unwrap(),
        packet["checks"].as_array().unwrap(),
        packet["repository"].as_str().unwrap(),
        packet["G"].as_str().unwrap(),
        packet["integration"].as_u64().unwrap(),
        packet["parent"].as_str().unwrap(),
    )
}

fn required(packet: &Value, checks: &[Value]) -> Result<bool, String> {
    let selected: Vec<_> = checks
        .iter()
        .map(|check| {
            json!({
                "name": check["name"], "link": check["details_url"],
                "bucket": if check["status"] == "completed" && check["conclusion"] == "success" {
                    "pass"
                } else { "fail" },
            })
        })
        .collect();
    pr_checks::required_checks(
        &packet["rules"],
        checks,
        &selected,
        packet["G"].as_str().unwrap(),
    )
}

#[test]
fn actual_same_sha_push_does_not_replace_qualified_queue_checks() {
    let packet = fixture();
    let all = packet["checks"].as_array().unwrap();
    assert_eq!(all.len(), 8);
    assert!(
        required(&packet, all)
            .unwrap_err()
            .contains("Ambiguous required PR check")
    );
    let qualified = selection(&packet).unwrap();
    assert_eq!(qualified.len(), 4);
    assert!(
        qualified
            .iter()
            .all(|c| c["check_suite"]["id"] == 102371818744_u64)
    );
    assert!(required(&packet, &qualified).unwrap());
}

#[test]
fn selected_suite_retains_missing_failed_and_ambiguous_required_laws() {
    let packet = fixture();
    let checks = selection(&packet).unwrap();
    let missing = checks[1..].to_vec();
    assert!(!required(&packet, &missing).unwrap());
    let mut failed = checks.clone();
    failed[0]["conclusion"] = json!("failure");
    assert!(!required(&packet, &failed).unwrap());
    let mut duplicate = checks.clone();
    duplicate.push(checks[0].clone());
    assert!(required(&packet, &duplicate).is_err());
    let mut foreign_app = checks;
    foreign_app[0]["app"]["id"] = json!(1);
    assert!(!required(&packet, &foreign_app).unwrap());
}

#[test]
fn suite_and_exact_head_cannot_borrow_success_from_push() {
    for change in ["foreign-suite", "missing-suite", "wrong-head"] {
        let mut packet = fixture();
        for c in packet["checks"].as_array_mut().unwrap() {
            if c["check_suite"]["id"] != 102371818744_u64 {
                continue;
            }
            match change {
                "foreign-suite" => c["check_suite"]["id"] = json!(102381502901_u64),
                "missing-suite" => {
                    c.as_object_mut().unwrap().remove("check_suite");
                }
                _ => c["head_sha"] = json!("a".repeat(40)),
            }
        }
        let checks = selection(&packet).unwrap();
        assert!(checks.is_empty(), "{change}");
        assert!(!required(&packet, &checks).unwrap());
    }
}

#[test]
fn latest_exact_queue_run_must_qualify_and_authenticate_its_suite() {
    for change in [
        "missing-suite",
        "duplicate-suite",
        "failed",
        "pending",
        "foreign-branch",
    ] {
        let mut packet = fixture();
        let runs = packet["runs"].as_array_mut().unwrap();
        let index = runs
            .iter()
            .position(|r| r["id"] == 37786681534_u64)
            .unwrap();
        match change {
            "missing-suite" => {
                runs[index]
                    .as_object_mut()
                    .unwrap()
                    .remove("check_suite_id");
            }
            "duplicate-suite" => {
                runs[index]["check_suite_id"] =
                    runs.iter().find(|r| r["id"] == 37786679711_u64).unwrap()["check_suite_id"]
                        .clone()
            }
            "foreign-branch" => runs[index]["head_branch"] = json!("main"),
            _ => {
                let mut newer = runs[index].clone();
                newer["id"] = json!(37786681535_u64);
                newer["conclusion"] = json!("failure");
                if change == "pending" {
                    newer["status"] = json!("in_progress");
                }
                runs.push(newer);
            }
        }
        assert!(selection(&packet).is_err(), "{change}");
    }
}
