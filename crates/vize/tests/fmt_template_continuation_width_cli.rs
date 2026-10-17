//! Whole configured check/write/fixed-point custody for inline template syntax.
#![cfg(feature = "glyph")]
#![expect(
    clippy::disallowed_macros,
    reason = "historical JavaScript config carrier"
)]
#[path = "support/vite_config.rs"]
mod vite_config;

use std::{error::Error, fs, path::Path, process::Command};

fn replay(
    source: &[u8],
    expected: &[u8],
    name: &str,
    config: &[u8],
    explicit: bool,
) -> Result<(), Box<dyn Error>> {
    let project = tempfile::tempdir()?;
    let root = project.path();
    fs::write(root.join("Example.vue"), source)?;
    let native = std::str::from_utf8(config)?;
    let config = if let Some(native) = native.strip_prefix("export default ") {
        let native = native.trim().trim_end_matches(';');
        format!("export default {{ vue: {native} }};\n")
    } else {
        vite_config::source(native)
    }
    .into_bytes();
    fs::write(root.join(name), &config)?;
    for (pass, mode) in ["--check", "--write", "--write", "--write", "--check"]
        .iter()
        .enumerate()
    {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command.current_dir(root).arg("fmt");
        if explicit {
            command.args(["--config", name]);
        }
        let output = command.args([*mode, "Example.vue"]).output()?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(pass == 0 && source != expected)),
            "{output:?}"
        );
        assert_eq!(output.stdout, b"");
        assert_eq!(
            fs::read(root.join("Example.vue"))?,
            if pass == 0 { source } else { expected }
        );
        assert_eq!(fs::read(root.join(name))?, config);
    }
    Ok(())
}

#[test]
fn all_whole_vectors_and_original_json_ts_configs_keep_complete_bytes() -> Result<(), Box<dyn Error>>
{
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/corpus.json"
    ))?;
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876",
    );
    let cases = corpus["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), 11);
    for case in cases {
        let source = fs::read(directory.join(case["input"].as_str().ok_or("missing input")?))?;
        let expected = fs::read(directory.join(case["output"].as_str().ok_or("missing output")?))?;
        let config = serde_json::to_vec(&serde_json::json!({ "formatter": case["options"] }))?;
        replay(&source, &expected, "vite.config.mjs", &config, false)?;
    }
    let source = fs::read(directory.join("original-example.input"))?;
    let expected = fs::read(directory.join("original-example.expected"))?;
    for (name, fixture, explicit) in [
        ("vite.config.mjs", "vize.config.ts.txt", false),
        ("vite.config.ts", "vize.config.ts.txt", true),
        ("vite.config.mts", "vize.config.json.txt", true),
    ] {
        replay(
            &source,
            &expected,
            name,
            &fs::read(directory.join(fixture))?,
            explicit,
        )?;
    }
    Ok(())
}
