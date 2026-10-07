#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::support::{Fixture, position};
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn project<const N: usize>(files: &[(&str, String); N]) -> (Fixture, [String; N]) {
    let (name, source) = files.first().expect("nonempty authored project");
    let rest: Vec<_> = files
        .iter()
        .skip(1)
        .map(|(name, source)| (*name, source.as_str()))
        .collect();
    let fixture = Fixture::new_with_cross_file_component_project(source, name, &rest);
    let uris = std::array::from_fn(|i| fixture.write_file(files[i].0, &files[i].1));
    (fixture, uris)
}

pub fn expected(files: &[(&str, String)], references: Value, rename: Value) -> Value {
    json!({
        "references": references,
        "rename": rename,
        "files": files.iter().map(|(name, text)| json!({
            "file": name, "text": text, "disk": text, "diagnostics": [],
        })).collect::<Vec<_>>(),
        "independentRepair": files.iter().map(|(name, text)| json!({
            "file": name, "text": text, "disk": text, "version": 3, "diagnostics": [],
        })).collect::<Vec<_>>(),
    })
}

pub fn capture<const N: usize>(
    fixture: &Fixture,
    files: &[(&str, String); N],
    context: &str,
    expected: &Value,
    actual: &Value,
) {
    let root = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let profile = std::env::var_os("NEXTEST_PROFILE")?;
            Some(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()?
                    .parent()?
                    .join("target/nextest")
                    .join(profile),
            )
        });
    let Some(root) = root else {
        return;
    };
    let root = root.join("original-rename-report-transactions");
    std::fs::create_dir_all(&root).unwrap();
    let name = fixture.project_root().file_name().unwrap();
    let packet = json!({
        "context":context,"sourceSha":std::env::var("SOURCE_SHA").ok(),
        "cliBinary":env!("CARGO_BIN_EXE_vize"),
        "requireTsgo":std::env::var("VIZE_TEST_REQUIRE_TSGO").ok(),
        "disableTsgo":std::env::var("VIZE_TEST_DISABLE_TSGO").ok(),
        "initialFiles":files.iter().map(|(name,text)|json!({"file":name,"text":text})).collect::<Vec<_>>(),
        "tsconfig":fixture.read_file("tsconfig.json"),
        "vizeConfig":fixture.read_file("vize.config.json"),
        "expected":expected,"actual":actual,
    });
    std::fs::write(
        root.join(name).with_extension("json"),
        serde_json::to_vec_pretty(&packet).unwrap(),
    )
    .unwrap();
}

pub fn observe<const N: usize>(
    fixture: &mut Fixture,
    files: &[(&str, String); N],
    uris: &[String; N],
    query: (&str, &str, &str, &str),
    goldens: &[(&str, String); N],
    context: &str,
) -> Value {
    for ((_, source), uri) in files.iter().zip(uris) {
        assert_eq!(fixture.open_file(uri, source), json!([]), "{context}");
    }
    let (uri, source, needle, new_name) = query;
    let references = fixture.request_file_with(
        "textDocument/references",
        uri,
        source,
        needle,
        json!({"context":{"includeDeclaration":true}}),
    );
    let rename = fixture.request_file_with(
        "textDocument/rename",
        uri,
        source,
        needle,
        json!({"newName":new_name}),
    );
    let changes = rename["changes"].as_object();
    let repaired: Vec<_> = files
        .iter()
        .zip(uris)
        .map(|((_, source), uri)| apply(source, changes.and_then(|c| c.get(uri))))
        .collect();
    for ((name, source), text) in files.iter().zip(&repaired) {
        fixture.write_file(name, text.as_ref().unwrap_or(source));
    }
    let mut results = Vec::new();
    for (((name, source), uri), applied) in files.iter().zip(uris).zip(&repaired) {
        let text = applied.as_ref().unwrap_or(source);
        let diagnostics = fixture.change_file(uri, text, 2);
        let mut result = json!({
            "file": name, "text": text, "disk": fixture.read_file(name),
            "diagnostics": diagnostics,
        });
        if let Err(error) = applied {
            result["applicationError"] = json!(error);
        }
        results.push(result);
    }
    // Preserve every actual version-2 observation before independently
    // installing the authored repair. This never substitutes expected edits
    // for the actual response or weakens an application/diagnostic mismatch.
    for (name, text) in goldens {
        fixture.write_file(name, text);
    }
    let mut independent_repair = Vec::new();
    for ((name, text), uri) in goldens.iter().zip(uris) {
        let diagnostics = fixture.change_file(uri, text, 3);
        independent_repair.push(json!({
            "file":name,"text":text,"disk":fixture.read_file(name),
            "version":3,"diagnostics":diagnostics,
        }));
    }
    fixture.shutdown();
    json!({
        "references":references,"rename":rename,"files":results,
        "independentRepair":independent_repair,
    })
}

pub fn location(uri: &str, source: &str, needle: &str, length: usize) -> Value {
    json!({"uri":uri,"range":range(source, needle, length)})
}

pub fn edit(source: &str, needle: &str, length: usize, replacement: &str) -> Value {
    json!({"range":range(source, needle, length),"newText":replacement})
}

fn range(source: &str, needle: &str, length: usize) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + length as u64);
    json!({"start":start,"end":end})
}

fn apply(source: &str, entries: Option<&Value>) -> Result<String, String> {
    let Some(entries) = entries else {
        return Ok(source.to_owned());
    };
    let mut edits = entries
        .as_array()
        .ok_or_else(|| format!("invalid edit array: {entries}"))?
        .iter()
        .map(|edit| {
            let start = offset(source, &edit["range"]["start"])?;
            let end = offset(source, &edit["range"]["end"])?;
            let replacement = edit["newText"]
                .as_str()
                .ok_or_else(|| format!("invalid edit text: {edit}"))?;
            if start > end {
                return Err(format!("reversed edit: {edit}"));
            }
            Ok((start, end, replacement))
        })
        .collect::<Result<Vec<_>, String>>()?;
    edits.sort_by_key(|&(start, end, _)| (start, end));
    for pair in edits.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(format!("overlapping actual edits: {entries}"));
        }
    }
    let mut result = source.to_owned();
    for &(start, end, replacement) in edits.iter().rev() {
        result.replace_range(start..end, replacement);
    }
    Ok(result)
}

fn offset(source: &str, position: &Value) -> Result<usize, String> {
    let invalid = || format!("invalid native edit position: {position}");
    let line = position["line"].as_u64().ok_or_else(invalid)? as usize;
    let character = position["character"].as_u64().ok_or_else(invalid)? as usize;
    let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let selected = source.split('\n').nth(line).ok_or_else(invalid)?;
    let selected = selected.strip_suffix('\r').unwrap_or(selected);
    let mut utf16 = 0;
    for (at, ch) in selected.char_indices() {
        if utf16 == character {
            return Ok(start + at);
        }
        utf16 += ch.len_utf16();
    }
    if utf16 == character {
        Ok(start + selected.len())
    } else {
        Err(invalid())
    }
}
