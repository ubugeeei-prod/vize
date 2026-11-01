//! Complete configured CLI packets for the authored #6098 ordinal boundary.
#![cfg(feature = "glyph")]

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-attribute-group-order-6098/App.vue.txt"
);
const ORDERED: &str = "<template>\n  <div z-early=\"e\" a-late=\"l\"></div>\n</template>\n";
const UNCHANGED: &str = "Found 1 file(s)\n\nFormatted 1 file(s)\n  1 file(s) unchanged\n";
const CHANGED: &str =
    "Found 1 file(s)\nReformatted: Order.vue\n\nFormatted 1 file(s)\n  1 file(s) reformatted\n";
const CHECKED: &str = "Found 1 file(s)\n\nChecked 1 file(s)\n  1 file(s) already formatted\n";

fn groups(count: usize, matched: bool) -> Vec<Vec<&'static str>> {
    let mut groups = vec![Vec::new(); count];
    if let Some(first) = groups.first_mut() {
        first.push("z-early");
    }
    if matched && count > 1 {
        groups.last_mut().unwrap().push("a-late");
    }
    groups
}

fn capture(root: &Path, mode: &str) -> Value {
    let argv = ["fmt", "Order.vue", "--config", "vite.config.mjs", mode];
    let before = fs::read(root.join("Order.vue")).unwrap();
    let config_before = fs::read(root.join("vite.config.mjs")).unwrap();
    let unselected_before = fs::read(root.join("Unselected.vue")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(argv)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    json!({
        "argv":argv,"status":output.status.code(),"statusDebug":vize_l0::cstr!("{:?}",output.status),"success":output.status.success(),
        "stdout":output.stdout,"stderr":output.stderr,
        "before":before,"after":fs::read(root.join("Order.vue")).unwrap(),
        "configBefore":config_before,"configAfter":fs::read(root.join("vite.config.mjs")).unwrap(),
        "unselectedBefore":unselected_before,
        "unselectedAfter":fs::read(root.join("Unselected.vue")).unwrap()
    })
}

#[test]
fn canonical_config_keeps_wide_group_order_and_complete_cli_fixed_points() {
    let mut cases = Vec::new();
    for (count, matched) in [
        (1, false),
        (2, true),
        (255, false),
        (256, false),
        (257, true),
        (512, true),
    ] {
        for order in ["alphabetical", "as-written"] {
            cases.push((
                json!({"count":count,"matched":matched,"order":order}),
                SOURCE,
                ORDERED,
                json!({"printWidth":200,"attributeGroups":groups(count,matched),"attributeSortOrder":order}),
            ));
        }
    }
    let spread = "<template>\n  <div a-late=\"l\" z-early=\"e\" v-bind=\"attrs\" a-late=\"r\" z-early=\"s\"></div>\n</template>\n";
    let spread_expected = "<template>\n  <div z-early=\"e\" a-late=\"l\" v-bind=\"attrs\" z-early=\"s\" a-late=\"r\"></div>\n</template>\n";
    let event =
        "<template>\n  <div a-late=\"l\" @click=\"run\" z-early=\"e\"></div>\n</template>\n";
    let duplicate =
        "<template>\n  <div a-late=\"l\" z-early=\"e\" a-late=\"r\"></div>\n</template>\n";
    for (id, source, expected, options) in [
        (
            "spread",
            spread,
            spread_expected,
            json!({"attributeGroups":groups(257,true)}),
        ),
        (
            "event",
            event,
            event,
            json!({"attributeGroups":groups(257,true)}),
        ),
        (
            "duplicates",
            duplicate,
            duplicate,
            json!({"attributeGroups":groups(257,true)}),
        ),
        (
            "disabled",
            SOURCE,
            SOURCE,
            json!({"attributeGroups":groups(257,true),"sortAttributes":false}),
        ),
        ("builtin-default", SOURCE, SOURCE, json!({})),
        (
            "first-match",
            SOURCE,
            SOURCE,
            json!({"attributeGroups":[["*"],["z-early"]]}),
        ),
    ] {
        let mut options = options;
        options["printWidth"] = json!(200);
        cases.push((json!({"id":id}), source, expected, options));
    }
    let mut observations = Vec::new();
    for (id, source, expected, options) in cases {
        let project = tempfile::tempdir().unwrap();
        fs::write(project.path().join("Order.vue"), source).unwrap();
        fs::write(project.path().join("Unselected.vue"), SOURCE).unwrap();
        crate::support::vite_config::write(
            project.path(),
            serde_json::to_vec(&json!({"formatter":options})).unwrap(),
        );
        let attempts: Vec<_> = ["--write", "--write", "--check"]
            .map(|mode| capture(project.path(), mode))
            .into();
        observations.push(json!({"id":id,"source":source,"expected":expected,"options":options,"attempts":attempts}));
    }
    // JUnit retains success streams too. All cases/processes precede verdicts.
    eprintln!("attribute-group-cli-observations:{}", json!(observations));
    assert_eq!(observations.len(), 18);
    for row in observations {
        let expected = row["expected"].as_str().unwrap();
        let source = row["source"].as_str().unwrap();
        for (index, attempt) in row["attempts"].as_array().unwrap().iter().enumerate() {
            assert_eq!(attempt["status"], 0, "{row:#}");
            assert_eq!(attempt["success"], true, "{row:#}");
            assert_eq!(attempt["stdout"], json!([]), "{row:#}");
            let stderr = if index == 2 {
                CHECKED
            } else if index == 0 && source != expected {
                CHANGED
            } else {
                UNCHANGED
            };
            assert_eq!(attempt["stderr"], json!(stderr.as_bytes()), "{row:#}");
            assert_eq!(attempt["after"], json!(expected.as_bytes()), "{row:#}");
            assert_eq!(attempt["configAfter"], attempt["configBefore"], "{row:#}");
            assert_eq!(
                attempt["unselectedAfter"], attempt["unselectedBefore"],
                "{row:#}"
            );
            if index > 0 {
                assert_eq!(attempt["before"], attempt["after"], "{row:#}");
            }
        }
    }
}
