//! Full configured CLI streams, source preservation and three writes for #7876.
#![cfg(feature = "glyph")]
use std::{error::Error, fs, path::Path, process::Command};

#[test]
fn original_and_control_cli_commands_match_complete_streams_and_bytes() -> Result<(), Box<dyn Error>>
{
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/corpus.json"
    ))?;
    let cases = corpus["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), 16);
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/expression-print-width-7876",
    );
    let mut commands = 0;
    for case in cases {
        let id = case["id"].as_str().ok_or("missing ID")?;
        let input = fs::read(directory.join(case["input"].as_str().ok_or("missing input")?))?;
        let expected = fs::read(directory.join(case["output"].as_str().ok_or("missing output")?))?;
        let project = tempfile::tempdir()?;
        let file = project.path().join("Example.vue");
        fs::write(&file, &input)?;
        fs::write(
            project.path().join("vize.config.json"),
            serde_json::to_vec(&serde_json::json!({"formatter":case["options"]}))?,
        )?;
        for (pass, mode) in ["--check", "--write", "--write", "--write", "--check"]
            .into_iter()
            .enumerate()
        {
            let changed = pass < 2 && input != expected;
            let checked = mode == "--check";
            let before = fs::read(&file)?;
            let result = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args(["fmt", mode, "Example.vue"])
                .output()?;
            let detail = if changed {
                format!(
                    "{}: Example.vue\n",
                    if checked {
                        "Would reformat"
                    } else {
                        "Reformatted"
                    }
                )
            } else {
                std::string::String::new()
            };
            let verb = if checked { "Checked" } else { "Formatted" };
            let outcome = match (checked, changed) {
                (true, true) => "would be reformatted",
                (true, false) => "already formatted",
                (false, true) => "reformatted",
                (false, false) => "unchanged",
            };
            let stderr =
                format!("Found 1 file(s)\n{detail}\n{verb} 1 file(s)\n  1 file(s) {outcome}\n");
            assert_eq!(
                result.status.code(),
                Some(i32::from(checked && changed)),
                "{id} pass{pass}"
            );
            assert_eq!(result.stdout, b"", "{id} pass{pass}: stdout");
            assert_eq!(result.stderr, stderr.as_bytes(), "{id} pass{pass}: stderr");
            let after = fs::read(&file)?;
            if checked {
                assert_eq!(after, before, "{id} pass{pass}: check must preserve bytes");
            }
            assert_eq!(
                after.as_slice(),
                if pass == 0 {
                    input.as_slice()
                } else {
                    expected.as_slice()
                },
                "{id} pass{pass}: file bytes"
            );
            commands += 1;
        }
    }
    assert_eq!(commands, 80);
    Ok(())
}
