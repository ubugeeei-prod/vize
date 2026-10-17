//! `vize lib init`: detect the project layout and write the `lib` config section.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use vize_l0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{ensure_project_path, project_relative_dir, write_file};
use super::output::{declared_npm_packages, json, line};
use super::{InitArgs, LibContext};

/// What `init` does to the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ConfigAction {
    /// No config file exists; `vite.config.mjs` is created with `vue.lib`.
    Create,
    /// The shared config already declares library settings; nothing is written.
    Unchanged,
    /// The existing Vite configuration must be edited by hand.
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

pub fn init(context: &LibContext, args: &InitArgs) -> LibResult<String> {
    let root = &context.root;
    // Preserve Vite files the loader could not evaluate, for example when Node
    // is unavailable. Never create or edit a removed dedicated config format.
    let discovered_config = [
        "vite.config.ts",
        "vite.config.mts",
        "vite.config.cts",
        "vite.config.js",
        "vite.config.mjs",
        "vite.config.cjs",
    ]
    .into_iter()
    .map(|name| root.join(name))
    .find(|path| fs::symlink_metadata(path).is_ok());
    // The config is another write destination: an existing symlink must not
    // bypass the project boundary when the lib section is added or replaced.
    let config_destination = discovered_config
        .as_deref()
        .map_or_else(|| root.join("vite.config.mjs"), Path::to_path_buf);
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
        match &discovered_config {
            None => (
                root.join("vite.config.mjs"),
                ConfigAction::Create,
                Some(cstr!(
                    "export default {};\n",
                    serde_json::to_string_pretty(&serde_json::json!({ "vue": { "lib": lib } }))
                        .map_err(|error| LibError::new(cstr!(
                            "failed to serialize Vite config: {error}"
                        )))?
                )),
            ),
            Some(path) if context.config != Default::default() && !args.force => {
                (path.clone(), ConfigAction::Unchanged, None)
            }
            Some(path) => (path.clone(), ConfigAction::Manual, None),
        };
    if action == ConfigAction::Manual {
        let snippet = cstr!(
            "vue: {{ lib: {{ uiDir: \"{ui_dir}\", composableDir: \"{composable_dir}\" }} }},"
        );
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
        (ConfigAction::Unchanged, _) => "kept existing lib section in (pass --force to replace)",
        (ConfigAction::Manual, _) => "cannot edit; add the lib section to",
    };
    line(&mut out, format_args!("{verb} {}", report.config_path));
    for hint in &report.hints {
        line(&mut out, format_args!("hint: {hint}"));
    }
    Ok(out)
}
