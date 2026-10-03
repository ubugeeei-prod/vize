//! Independent compiler and editor providers keep distinct diagnostic contracts.

use super::{require, require_equal, support};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_l0::{Allocator, cstr};
use vize_l1::embed::Lang;
use vize_l4::targets::ts::project_program;

pub(super) fn capture(
    root: &Path,
    families: &[(&str, &Path, &[serde_json::Value])],
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut compiler_inputs = Vec::new();
    let mut editor_inputs = Vec::new();
    for (family, directory, cases) in families {
        for case in *cases {
            let arena = Allocator::default();
            let source = case
                .get("source")
                .and_then(serde_json::Value::as_str)
                .ok_or("oracle source")?;
            let id = case
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or("oracle case identity")?;
            let lang = match case.get("kind").and_then(serde_json::Value::as_str) {
                Some("js") => Lang::Js,
                Some("ts") => Lang::Ts,
                _ => return Err("oracle language".into()),
            };
            let file = support::file(&arena, source, lang).ok_or("oracle original File")?;
            require(file.is_complete(), "oracle actual File completes")?;
            let projection = project_program(&file).map_err(|_| "oracle actual projection")?;
            let extension = projection.source_kind().extension();
            let physical = directory.join(cstr!("reference-{id}.{extension}").as_str());
            require(
                std::fs::read(&physical)? == source.as_bytes(),
                "oracle physical original bytes",
            )?;
            compiler_inputs.push(serde_json::json!({
                "case":case,"extension":extension,"projected":projection.document().as_str()
            }));
            editor_inputs.push(serde_json::json!({
                "family":family,"case":case,"extension":extension,"physicalPath":physical,
                "projected":projection.document().as_str()
            }));
        }
    }
    // This unchanged helper still asserts every frozen pre-emit expectation.
    // It is not the expected total editor vector and grants no editor parity.
    let compiler = run(
        root,
        include_str!("../../../../tests/tooling/support/native-program-typescript.ts"),
        &serde_json::Value::Array(compiler_inputs),
    )?;
    require(
        compiler.get("version") == Some(&serde_json::json!("6.0.3")),
        "compiler oracle version",
    )?;
    require(
        compiler
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|cases| cases.len() == editor_inputs.len()),
        "every original compiler golden asserted",
    )?;
    let editor = run(
        root,
        include_str!("../../../../tests/tooling/support/native-program-editor-typescript.ts"),
        &serde_json::Value::Array(editor_inputs.clone()),
    )?;
    require(
        editor.get("version") == Some(&serde_json::json!("6.0.3")),
        "independent editor oracle version",
    )?;
    let captures = editor
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or("complete editor captures")?;
    require(
        captures.len() == editor_inputs.len(),
        "every original editor view generated",
    )?;
    for (input, output) in editor_inputs.iter().zip(captures) {
        for key in ["family", "extension"] {
            require_equal(
                output.get(key).ok_or("editor identity")?,
                input.get(key).ok_or("oracle input identity")?,
                "editor capture identity",
            )?;
        }
        require_equal(
            output.get("id").ok_or("editor case identity")?,
            input.pointer("/case/id").ok_or("oracle case identity")?,
            "editor case identity",
        )?;
        require_equal(
            output
                .pointer("/opened/source")
                .ok_or("opened exact source")?,
            input.pointer("/case/source").ok_or("oracle source")?,
            "opened snapshot retains original BOM and bytes",
        )?;
        require_equal(
            output
                .pointer("/openedProjection/source")
                .ok_or("actual emitted editor snapshot")?,
            input
                .get("projected")
                .ok_or("actual genuine emitted bytes")?,
            "native editor oracle uses the genuine L4 document exactly",
        )?;
    }
    // Logs retain the complete independent raw fields and both coordinate views.
    eprintln!(
        "independent original compiler oracle: {}",
        serde_json::to_string(&compiler)?
    );
    eprintln!(
        "independent complete editor oracle: {}",
        serde_json::to_string(&editor)?
    );
    Ok(editor)
}

pub(super) fn case<'a>(
    oracle: &'a serde_json::Value,
    family: &str,
    id: &str,
) -> Result<&'a serde_json::Value, Box<dyn std::error::Error>> {
    let cases = oracle
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or("editor cases")?;
    let mut found = cases.iter().filter(|case| {
        case.get("family").and_then(serde_json::Value::as_str) == Some(family)
            && case.get("id").and_then(serde_json::Value::as_str) == Some(id)
    });
    let result = found.next().ok_or("matching independent editor capture")?;
    require(found.next().is_none(), "unique independent editor capture")?;
    Ok(result)
}

fn run(
    root: &Path,
    helper: &str,
    input: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(input)?;
    let mut child = Command::new("node")
        .args(["--input-type=module", "-e", helper])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let write = child
        .stdin
        .take()
        .ok_or("oracle stdin")?
        .write_all(&payload);
    if let Err(error) = write {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.into());
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        eprintln!(
            "independent oracle failed: {}",
            std::str::from_utf8(&output.stderr)?
        );
        return Err("independent provider oracle failed".into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
