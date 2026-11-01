//! Bare default slots have no colon spelling for Glyph to normalize.
use super::*;

#[test]
fn default_and_bare_slot_policies_preserve_the_existing_formatter_behavior() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/Slots.vue");
    let source = FIXED
        .replace("v-slot:default", "v-slot")
        .replace("v-slot:header", "#header");
    let expected = vize_glyph::format_sfc(&source, &vize_glyph::FormatOptions::default())
        .unwrap()
        .code;
    for style in [None, Some(json!({})), Some(json!({"default":"v-slot"}))] {
        let mut linter = json!({"preset":"incremental","rules":{RULE:"error"}});
        if let Some(style) = style {
            linter["ruleOptions"] = json!({RULE:style});
        }
        configure(root, linter, None);
        let config = fs::read(root.join("vite.config.mjs")).unwrap();
        fs::write(&file, &source).unwrap();
        let write = [
            "fmt",
            "--config",
            "vite.config.mjs",
            "--write",
            "src/Slots.vue",
        ];
        expect(&run(root, &write), 0);
        assert_eq!(fs::read_to_string(&file).unwrap(), expected.as_str());
        clean(root);
        expect(&run(root, &write), 0);
        assert_eq!(fs::read_to_string(&file).unwrap(), expected.as_str());
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--check",
                    "src/Slots.vue",
                ],
            ),
            0,
        );
        assert_eq!(fs::read(root.join("vite.config.mjs")).unwrap(), config);
    }
}

#[test]
fn canonical_uniform_and_each_longform_position_reach_the_cli_preflight() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/Slots.vue");
    for style in [
        json!("longform"),
        json!({"atComponent":"longform"}),
        json!({"default":"longform"}),
        json!({"named":"longform"}),
    ] {
        configure(
            root,
            json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:style}}),
            None,
        );
        fs::write(&file, FIXED).unwrap();
        let config = fs::read(root.join("vite.config.mjs")).unwrap();
        let output = run(
            root,
            &[
                "fmt",
                "--config",
                "vite.config.mjs",
                "--write",
                "src/Slots.vue",
            ],
        );
        expect(&output, 2);
        assert_eq!(output.stdout, b"");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("Invalid formatter configuration: src/Slots.vue: {REASON}\n")
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), FIXED);
        assert_eq!(fs::read(root.join("vite.config.mjs")).unwrap(), config);
    }
}
