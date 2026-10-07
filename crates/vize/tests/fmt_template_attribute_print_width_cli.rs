//! Configured public CLI observes complete directive quote-layout bytes (#7876).
#![cfg(feature = "glyph")]

use std::{error::Error, fs, path::Path, process::Command};

fn fmt(root: &Path, args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .arg("fmt")
        .args(args)
        .output()
}

#[test]
fn configured_check_write_and_second_check_match_complete_reference_bytes()
-> Result<(), Box<dyn Error>> {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/corpus.json"
    ))?;
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/formatter-regressions/directive-print-width-7876",
    );
    let cases = corpus["cases"].as_array().ok_or("missing cases")?;
    let mut executed = 0;
    for case in cases.iter().filter(|case| case["api"] == "sfc") {
        let id = case["id"].as_str().ok_or("missing ID")?;
        let input = fs::read(directory.join(case["input"].as_str().ok_or("missing input")?))?;
        let expected = fs::read(directory.join(case["output"].as_str().ok_or("missing output")?))?;
        let project = tempfile::tempdir()?;
        let root = project.path();
        let file = root.join("Example.vue");
        fs::write(&file, &input)?;
        fs::write(
            root.join("vize.config.json"),
            serde_json::to_vec(&serde_json::json!({ "formatter": case["options"] }))?,
        )?;
        let checked = fmt(root, &["--check", "Example.vue"])?;
        assert_eq!(
            checked.status.code(),
            Some(i32::from(input != expected)),
            "{id} check"
        );
        assert_eq!(fs::read(&file)?, input, "{id} check must preserve source");
        let written = fmt(root, &["--write", "Example.vue"])?;
        assert_eq!(written.status.code(), Some(0), "{id} write");
        assert_eq!(fs::read(&file)?, expected, "{id} complete written bytes");
        assert_eq!(
            fmt(root, &["--check", "Example.vue"])?.status.code(),
            Some(0),
            "{id} second check"
        );
        assert_eq!(
            fs::read(&file)?,
            expected,
            "{id} second check preserves bytes"
        );
        executed += 1;
    }
    assert_eq!(executed, 15);
    Ok(())
}
