//! `vize lib init`: detect the project layout and write the `lib` config section.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{ensure_project_path, project_relative_dir, write_file};
use super::output::{declared_npm_packages, json, line};
use super::{InitArgs, LibContext};

/// What `init` does to the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ConfigAction {
    /// No config file exists; `vize.config.json` is created.
    Create,
    /// `vize.config.json` gains (or with --force, replaces) its `lib` section.
    Update,
    /// A `lib` section already exists; nothing is written.
    Unchanged,
    /// The config is Pkl/TS/JS; the snippet must be added by hand.
    Manual,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitReport {
    dry_run: bool,
    source_dir: String,
    framework: &'static str,
    typescript: bool,
    allow_importing_ts_extensions: bool,
    config_path: String,
    action: ConfigAction,
    lib: serde_json::Value,
    hints: Vec<String>,
}

fn detect_source_dir(root: &Path) -> (String, &'static str) {
    let nuxt = ["nuxt.config.ts", "nuxt.config.js", "nuxt.config.mjs"]
        .iter()
        .any(|name| root.join(name).is_file());
    if nuxt && root.join("app").is_dir() {
        return ("app".into(), "nuxt");
    }
    let framework = if nuxt { "nuxt" } else { "vue" };
    if root.join("src").is_dir() || !nuxt {
        ("src".into(), framework)
    } else {
        (".".into(), framework)
    }
}

fn join_dir(base: &str, rest: &str) -> String {
    if base == "." {
        rest.into()
    } else {
        cstr!("{base}/{rest}")
    }
}

/// Insert `"lib": …` before the closing brace of a JSON object, keeping the rest verbatim.
fn insert_lib_section(source: &str, lib: &str) -> Option<String> {
    let close = source.rfind('}')?;
    let head = source.get(..close)?;
    let tail = source.get(close..)?;
    let trimmed = head.trim_end();
    let separator = if trimmed.ends_with('{') { "" } else { "," };
    Some(String::from(
        [trimmed, separator, "\n  \"lib\": ", lib, "\n", tail].concat(),
    ))
}

pub fn init(context: &LibContext, args: &InitArgs) -> LibResult<String> {
    let root = &context.root;
    // The config is another write destination: an existing symlink must not
    // bypass the project boundary when the lib section is added or replaced.
    let config_destination = context
        .config_path
        .as_deref()
        .map_or_else(|| root.join("vize.config.json"), Path::to_path_buf);
    ensure_project_path(root, &config_destination)?;
    let (source_dir, framework) = detect_source_dir(root);
    let ui_dir = match &args.ui_dir {
        Some(dir) => project_relative_dir(root, Path::new(dir.as_str()))?,
        None => join_dir(&source_dir, "components/vize"),
    };
    let composable_dir = match &args.composable_dir {
        Some(dir) => project_relative_dir(root, Path::new(dir.as_str()))?,
        None => join_dir(&source_dir, "composables/vize"),
    };
    let lib =
        serde_json::json!({ "uiDir": ui_dir.as_str(), "composableDir": composable_dir.as_str() });
    let lib_text = serde_json::to_string_pretty(&lib)
        .map_err(|error| LibError::new(cstr!("failed to serialize lib section: {error}")))?
        .replace('\n', "\n  ");

    let tsconfig = fs::read_to_string(root.join("tsconfig.json")).ok();
    let typescript = tsconfig.is_some();
    let allow_ts_extensions = tsconfig
        .as_deref()
        .is_some_and(|text| text.contains("\"allowImportingTsExtensions\": true"));
    let mut hints: Vec<String> = Vec::new();
    if typescript && !allow_ts_extensions {
        hints.push(String::from(
            "pulled sources import siblings as `./x.ts`: set compilerOptions.allowImportingTsExtensions (with noEmit or a bundler) in tsconfig.json",
        ));
    }
    if !declared_npm_packages(root).iter().any(|name| name == "vue") {
        hints.push(String::from(
            "add `vue` (^3.5.0) to package.json; pulled items import it",
        ));
    }

    let (config_path, action, new_text): (PathBuf, ConfigAction, Option<String>) =
        match &context.config_path {
            None => (
                root.join("vize.config.json"),
                ConfigAction::Create,
                Some(String::from(
                    ["{\n  \"lib\": ", lib_text.as_str(), "\n}\n"].concat(),
                )),
            ),
            Some(path)
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json") =>
            {
                let source =
                    fs::read_to_string(path).map_err(|error| LibError::io("read", path, &error))?;
                let value: serde_json::Value = serde_json::from_str(&source)
                    .map_err(|error| LibError::new(cstr!("invalid {}: {error}", path.display())))?;
                match value.get("lib") {
                    Some(_) if !args.force => (path.clone(), ConfigAction::Unchanged, None),
                    Some(_) => {
                        let mut value = value;
                        if let Some(object) = value.as_object_mut() {
                            object.insert("lib".into(), lib.clone());
                        }
                        let text = serde_json::to_string_pretty(&value).map_err(|error| {
                            LibError::new(cstr!("failed to serialize config: {error}"))
                        })?;
                        (path.clone(), ConfigAction::Update, Some(cstr!("{text}\n")))
                    }
                    None => {
                        let text = insert_lib_section(&source, &lib_text).ok_or_else(|| {
                            LibError::new(cstr!("{} is not a JSON object", path.display()))
                        })?;
                        (path.clone(), ConfigAction::Update, Some(text))
                    }
                }
            }
            Some(path) => (path.clone(), ConfigAction::Manual, None),
        };
    if action == ConfigAction::Manual {
        let snippet = if config_path
            .extension()
            .is_some_and(|extension| extension == "pkl")
        {
            cstr!("lib {{\n  uiDir = \"{ui_dir}\"\n  composableDir = \"{composable_dir}\"\n}}")
        } else {
            cstr!("lib: {{ uiDir: \"{ui_dir}\", composableDir: \"{composable_dir}\" }},")
        };
        hints.push(cstr!("add to {}:\n{snippet}", config_path.display()));
    }
    if !args.dry_run
        && let Some(text) = &new_text
    {
        write_file(&config_path, text.as_bytes())?;
    }
    let config_display: String = cstr!(
        "{}",
        config_path
            .strip_prefix(root)
            .unwrap_or(&config_path)
            .display()
    );
    let report = InitReport {
        dry_run: args.dry_run,
        source_dir,
        framework,
        typescript,
        allow_importing_ts_extensions: allow_ts_extensions,
        config_path: config_display,
        action,
        lib,
        hints,
    };
    if context.json {
        return json(&report);
    }
    let mut out = String::default();
    line(&mut out, format_args!("framework   {}", report.framework));
    line(&mut out, format_args!("source dir  {}", report.source_dir));
    line(
        &mut out,
        format_args!("typescript  {}", if typescript { "yes" } else { "no" }),
    );
    line(&mut out, format_args!("ui dir      {ui_dir}"));
    line(&mut out, format_args!("composables {composable_dir}"));
    let verb = match (report.action, args.dry_run) {
        (ConfigAction::Create, true) => "would create",
        (ConfigAction::Create, false) => "created",
        (ConfigAction::Update, true) => "would update",
        (ConfigAction::Update, false) => "updated",
        (ConfigAction::Unchanged, _) => "kept existing lib section in (pass --force to replace)",
        (ConfigAction::Manual, _) => "cannot edit; add the lib section to",
    };
    line(&mut out, format_args!("{verb} {}", report.config_path));
    for hint in &report.hints {
        line(&mut out, format_args!("hint: {hint}"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::insert_lib_section;

    #[test]
    fn inserts_into_empty_and_populated_objects() {
        assert_eq!(
            insert_lib_section("{}\n", "{}").unwrap(),
            "{\n  \"lib\": {}\n}\n"
        );
        assert_eq!(
            insert_lib_section("{\n  \"formatter\": { \"semi\": false }\n}\n", "{}").unwrap(),
            "{\n  \"formatter\": { \"semi\": false },\n  \"lib\": {}\n}\n"
        );
        assert!(insert_lib_section("[]", "{}").is_none());
    }
}
