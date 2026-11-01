//! Whole JSON-config and direct-CLI observations before width-conversion verdicts.
#![cfg(feature = "glyph")]
#![expect(clippy::disallowed_types, reason = "complete CLI observation packets")]

use std::{fs, path::Path, process::Command};

use serde_json::Value;
use sha2::{Digest, Sha256};

fn read_pinned(root: &Path, file: &str, expected: &str) -> Vec<u8> {
    let bytes = fs::read(root.join(file)).unwrap();
    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(actual, expected, "{file}");
    bytes
}

fn collect(case: &Value, input: &[u8], width: u32, direct: bool) -> Vec<Value> {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let config = serde_json::to_vec(&serde_json::json!({
        "formatter":{"printWidth":if direct {40} else {width}}
    }))
    .unwrap();
    fs::write(root.join("vize.config.json"), &config).unwrap();
    fs::write(root.join("Example.vue"), input).unwrap();
    fs::write(
        root.join("Unselected.vue"),
        b"<template><p>untouched</p></template>\n",
    )
    .unwrap();
    let mut packets = Vec::new();
    for (pass, mode) in ["--write", "--write", "--check"].into_iter().enumerate() {
        let before = fs::read(root.join("Example.vue")).unwrap();
        let config_before = fs::read(root.join("vize.config.json")).unwrap();
        let unselected_before = fs::read(root.join("Unselected.vue")).unwrap();
        let width_arg = width.to_string();
        let mut argv = vec!["fmt", "Example.vue", "--config", "vize.config.json", mode];
        if direct {
            argv.extend(["--print-width", width_arg.as_str()]);
        }
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .args(&argv)
            .output()
            .unwrap();
        let after = fs::read(root.join("Example.vue")).unwrap();
        let packet = serde_json::json!({
            "id":case["id"],"width":width,"direct":direct,"pass":pass,"argv":argv,
            "status":result.status.code(),"success":result.status.success(),
            "stdout":String::from_utf8(result.stdout).unwrap(),
            "stderr":String::from_utf8(result.stderr).unwrap(),
            "before":String::from_utf8(before).unwrap(),"after":String::from_utf8(after).unwrap(),
            "configBefore":String::from_utf8(config_before).unwrap(),
            "configAfter":fs::read_to_string(root.join("vize.config.json")).unwrap(),
            "unselectedBefore":String::from_utf8(unselected_before).unwrap(),
            "unselectedAfter":fs::read_to_string(root.join("Unselected.vue")).unwrap()
        });
        eprintln!("formatter-width-cli:{}", packet);
        packets.push(packet);
    }
    packets
}

#[test]
fn original_sfc_inputs_retain_every_configured_cli_packet_before_verdicts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/print-width-conversion-6098/cases.json"
    )).unwrap();
    let mut observations = Vec::new();
    for case in corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["target"] == "sfc")
    {
        let input = read_pinned(
            &root,
            case["input"].as_str().unwrap(),
            case["inputSha256"].as_str().unwrap(),
        );
        for width in corpus["widths"].as_array().unwrap() {
            observations.extend(collect(
                case,
                &input,
                u32::try_from(width.as_u64().unwrap()).unwrap(),
                false,
            ));
        }
        for width in [100, 65656] {
            observations.extend(collect(case, &input, width, true));
        }
    }
    assert_eq!(observations.len(), 84);
    for packet in &observations {
        assert_eq!(packet["status"], 0, "{packet}");
        assert_eq!(packet["success"], true, "{packet}");
        assert_eq!(packet["stdout"], "", "{packet}");
        assert_eq!(packet["configAfter"], packet["configBefore"], "{packet}");
        assert_eq!(
            packet["unselectedAfter"], packet["unselectedBefore"],
            "{packet}"
        );
        let checked = packet["pass"] == 2;
        let changed = packet["before"] != packet["after"];
        let detail = if changed {
            "Reformatted: Example.vue\n"
        } else {
            ""
        };
        let (verb, outcome) = if checked {
            ("Checked", "already formatted")
        } else if changed {
            ("Formatted", "reformatted")
        } else {
            ("Formatted", "unchanged")
        };
        assert_eq!(
            packet["stderr"],
            format!("Found 1 file(s)\n{detail}\n{verb} 1 file(s)\n  1 file(s) {outcome}\n"),
            "{packet}"
        );
        if packet["pass"] != 0 {
            assert_eq!(packet["before"], packet["after"], "{packet}");
        }
    }
    for case in corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["target"] == "sfc")
    {
        let expected = String::from_utf8(read_pinned(
            &root,
            case["defaultExpected"].as_str().unwrap(),
            case["defaultExpectedSha256"].as_str().unwrap(),
        ))
        .unwrap();
        for packet in observations
            .iter()
            .filter(|packet| packet["id"] == case["id"] && packet["width"] == 100)
        {
            assert_eq!(packet["after"], expected, "{packet}");
        }
    }
}
