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
    let cases = manifest["cases"]
        .as_array()
        .ok_or_else(|| "Zed profile fixture cases are missing".to_string())?;
    let root = std::env::temp_dir().join(format!(
        "vize-zed-profile-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    let result = (|| {
        for case in cases {
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
                for (provider, expected) in [
                    ("hoverProvider", "hoverProvider"),
                    ("documentFormattingProvider", "formattingProvider"),
                ] {
                    assert_json_eq(
                        initialization["capabilities"]
                            .get(provider)
                            .unwrap_or(&Value::Null),
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
                println!(
                    "{}",
                    json!({ "zedWorkspaceProfile": id, "source": source, "config": config,
                        "initializationOptions": options, "initialization": initialization,
                        "hover": hover })
                );
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
