use serde_json::{self, Value, json};
use std::{fs, path::Path};

use super::lsp_smoke::{LspSession, assert_json_eq, file_url};

#[path = "./e2e.rs"]
mod editor_e2e;
#[path = "../../../editors/zed/src/initialization_options.rs"]
mod initialization_options;

pub fn run(repo_root: &Path) -> Result<(), String> {
    let fixture = repo_root.join("tests/_fixtures/differential/lsp/zed-workspace-profile");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(fixture.join("case.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut cases = manifest["cases"]
        .as_array()
        .ok_or_else(|| "Zed profile fixture cases are missing".to_string())?
        .clone();
    // Retain case.json as the frozen authored baseline. Only its declared
    // no-config default has a policy successor; dedicated packets stay exact.
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(fixture.join("current-default-policy.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let index = cases
        .iter()
        .position(|case| case["id"] == policy["historical"]["caseId"])
        .ok_or_else(|| "historical no-config baseline is missing".to_string())?;
    cases[index] = policy["currentDefault"].clone();
    cases.push(policy["explicitFormattingFalse"].clone());
    let root = std::env::temp_dir().join(format!(
        "vize-zed-profile-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    let result = (|| {
        for case in &cases {
            let id = case["id"]
                .as_str()
                .ok_or_else(|| "Zed profile case id is missing".to_string())?;
            let workspace = root.join(id);
            editor_e2e::prepare_real_vue_workspace(&workspace, false)?;
            // This helper-created runtime config is not part of either report.
            fs::remove_file(workspace.join("vize.config.json"))
                .map_err(|error| error.to_string())?;
            let runtime = editor_e2e::resolve_corsa_path(repo_root)?;
            let runtime_link = workspace.join("node_modules/.bin/tsgo");
            fs::create_dir_all(runtime_link.parent().unwrap())
                .map_err(|error| error.to_string())?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(runtime, runtime_link).map_err(|error| error.to_string())?;
            #[cfg(windows)]
            fs::copy(runtime, workspace.join("node_modules/.bin/tsgo.exe"))
                .map_err(|error| error.to_string())?;
            let config = match case["config"].as_str() {
                Some(filename) => {
                    let config = fs::read_to_string(fixture.join(filename))
                        .map_err(|error| error.to_string())?;
                    fs::write(workspace.join("vize.config.ts"), &config)
                        .map_err(|error| error.to_string())?;
                    Some(config)
                }
                None => None,
            };
            let source = fs::read_to_string(fixture.join("Profile.vue.txt"))
                .map_err(|error| error.to_string())?;
            let document = workspace.join("src/Profile.vue");
            fs::write(&document, &source).map_err(|error| error.to_string())?;
            let options = initialization_options::initialization_options(
                case.get("explicit")
                    .filter(|value| !value.is_null())
                    .cloned(),
                |filename| fs::read_to_string(workspace.join(filename)).is_ok(),
            );
            assert_json_eq(&options, case["initializationOptions"].clone(), id)?;
            let uri = file_url(&document)?;
            let mut session = LspSession::spawn(repo_root)?;
            let result = (|| {
                let initialization = session.initialize(&workspace, options.clone())?;
                for (provider, expected, presence) in [
                    ("hoverProvider", "hoverProvider", "hoverProviderPresent"),
                    (
                        "documentFormattingProvider",
                        "formattingProvider",
                        "formattingProviderPresent",
                    ),
                ] {
                    let observed = initialization["capabilities"].get(provider);
                    assert_json_eq(
                        &json!(observed.is_some()),
                        case[presence].clone(),
                        &format!("{id} {provider} wire presence"),
                    )?;
                    assert_json_eq(
                        observed.unwrap_or(&Value::Null),
                        case[expected].clone(),
                        &format!("{id} {provider}"),
                    )?;
                }
                session.notify(
                    "textDocument/didOpen",
                    json!({ "textDocument": {
                        "uri": uri, "languageId": "vue", "version": 1, "text": source
                    } }),
                )?;
                let hover = session.request(
                    "textDocument/hover",
                    json!({ "textDocument": { "uri": uri },
                        "position": { "line": 1, "character": 8 } }),
                )?;
                assert_json_eq(&hover, case["hover"].clone(), &format!("{id} whole hover"))?;
                let formatting = if case.get("formattedSource").is_some()
                    || case.get("formattingResult").is_some()
                {
                    let edits = session.request(
                        "textDocument/formatting",
                        json!({"textDocument":{"uri":uri},
                            "options":{"insertSpaces":true,"tabSize":2}}),
                    )?;
                    if let Some(expected) = case.get("formattedSource") {
                        let formatted = apply_formatting_edits(&source, &edits)?;
                        assert_json_eq(
                            &json!(formatted),
                            expected.clone(),
                            &format!("{id} whole formatted buffer"),
                        )?;
                    } else {
                        assert_json_eq(
                            &edits,
                            case["formattingResult"].clone(),
                            &format!("{id} disabled formatting result"),
                        )?;
                    }
                    Some(edits)
                } else {
                    None
                };
                let mut receipt = json!({ "zedWorkspaceProfile": id, "source": source, "config": config,
                    "initializationOptions": options, "initialization": initialization,
                    "hover": hover });
                if let Some(formatting) = formatting {
                    receipt["formatting"] = formatting;
                }
                println!("{receipt}");
                Ok(())
            })();
            let shutdown = session.shutdown();
            result.and(shutdown)?;
        }
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&root).map_err(|error| error.to_string());
    result.and(cleanup)?;
    println!(
        "zed workspace-profile real-server scenarios passed: {}",
        cases.len()
    );
    Ok(())
}

// Apply every protocol edit to the original buffer; no first-edit or prefix oracle.
fn apply_formatting_edits(source: &str, value: &Value) -> Result<String, String> {
    fn offset(source: &str, position: &Value) -> Result<usize, String> {
        let line = position["line"].as_u64().ok_or("missing edit line")? as usize;
        let character = position["character"]
            .as_u64()
            .ok_or("missing edit character")? as usize;
        let mut prefix = 0;
        for (index, text) in source.split('\n').enumerate() {
            if index == line {
                let mut units = 0;
                for (byte, ch) in text.char_indices() {
                    if units == character {
                        return Ok(prefix + byte);
                    }
                    units += ch.len_utf16();
                }
                if units == character {
                    return Ok(prefix + text.len());
                }
                return Err("formatting position splits or exceeds a UTF-16 line".into());
            }
            prefix += text.len() + 1;
        }
        Err("formatting position exceeds source".into())
    }
    let mut edits = value
        .as_array()
        .ok_or("formatting returned no edit vector")?
        .iter()
        .map(|edit| {
            Ok((
                offset(source, &edit["range"]["start"])?,
                offset(source, &edit["range"]["end"])?,
                edit["newText"]
                    .as_str()
                    .ok_or("missing formatting replacement")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
    let mut output = source.to_string();
    let mut previous_start = source.len();
    for (start, end, replacement) in edits {
        if start > end || end > previous_start {
            return Err("overlapping formatting edits".into());
        }
        output.replace_range(start..end, replacement);
        previous_start = start;
    }
    Ok(output)
}
