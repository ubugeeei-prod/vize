#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::Value;
use std::{
    collections::{BTreeSet, HashSet},
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/editors/archive.rs"]
mod editor_archive;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let default_vsix = root.join("editors/vscode/dist/vize.vsix");
    let vsix = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(default_vsix);
    let vsix = absolute_from_cwd(&vsix)?;
    editor_archive::assert_size(&vsix, "VSIX", 20_000, 5_000_000)?;
    let mut entries = editor_archive::list_zip(&vsix)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "VSIX")?;
    assert_safe_vsix_entries(&entries)?;

    let required = [
        "[Content_Types].xml",
        "extension.vsixmanifest",
        "extension/LICENSE.txt",
        "extension/changelog.md",
        "extension/dist/extension.cjs",
        "extension/icons/vue.svg",
        "extension/language-configuration.json",
        "extension/node_modules/@vizejs/typescript-vue-plugin/component-contracts.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/import-resolution.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/index.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/module-resolution.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/package.json",
        "extension/node_modules/@vizejs/typescript-vue-plugin/virtual-modules.cjs",
        "extension/package.json",
        "extension/readme.md",
        "extension/syntaxes/art-vue.tmLanguage.json",
        "extension/syntaxes/vue-script.tmLanguage.json",
        "extension/syntaxes/vue.tmLanguage.json",
    ];
    editor_archive::require_entries(
        &vsix,
        &entries,
        &required,
        editor_archive::read_zip_text,
        "VSIX",
    )?;
    require_binary_entries(&entries, &["extension/icons/logo.png"])?;
    assert_allowed_entries(&entries)?;
    assert_forbidden_entries(&entries)?;

    let package_json = editor_archive::read_zip_json(&vsix, "extension/package.json")?;
    let workspace_version = editor_archive::workspace_version(&root)?;
    assert_json_string(&package_json, &["name"], "vize")?;
    assert_json_string(&package_json, &["displayName"], "Vize")?;
    assert_json_string(&package_json, &["publisher"], "ubugeeei")?;
    assert_json_string(&package_json, &["version"], &workspace_version)?;
    assert_json_string(&package_json, &["main"], "./dist/extension.cjs")?;
    assert_json_string(&package_json, &["engines", "vscode"], "^1.75.0")?;
    assert_json_string(
        &package_json,
        &["dependencies", "vscode-languageclient"],
        "9.0.1",
    )?;

    assert_unique_json_strings(&package_json, &["activationEvents"])?;
    let activation_events = json_string_array(&package_json, &["activationEvents"])?;
    assert_no_hidden_host_test_commands(&activation_events, "activationEvents")?;
    let commands = package_json
        .pointer("/contributes/commands")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing contributes.commands".to_string())?;
    let mut command_names = Vec::new();
    for command in commands {
        let name = command
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| "command contribution missing command".to_string())?;
        command_names.push(name.to_string());
        assert_json_string(command, &["category"], "Vize")?;
        if command
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            return Err(format!("{name} must have a title"));
        }
        if !activation_events.contains(&format!("onCommand:{name}")) {
            return Err(format!("activationEvents missing onCommand:{name}"));
        }
    }
    assert_unique_values(&command_names, "contributes.commands")?;
    assert_no_hidden_host_test_commands(&command_names, "contributes.commands")?;
    for event in ["onLanguage:vue", "onLanguage:art-vue"] {
        if !activation_events.contains(&event.to_string()) {
            return Err(format!("activationEvents missing {event}"));
        }
    }

    assert_typescript_vue_plugin(&vsix, &package_json)?;
    assert_language_contributions(&package_json)?;
    assert_configuration_defaults(&package_json)?;

    let extension_bundle = editor_archive::read_zip_text(&vsix, "extension/dist/extension.cjs")?;
    for needle in ["exports.activate=", "exports.deactivate="] {
        editor_archive::expect_contains(
            &extension_bundle,
            needle,
            &format!("extension.cjs missing {needle}"),
        )?;
    }
    for forbidden in ["sourceMappingURL=", "vscode-languageclient/node"] {
        if extension_bundle.contains(forbidden) {
            return Err(format!("extension.cjs must not contain {forbidden}"));
        }
    }
    for dependency in find_static_requires(&extension_bundle) {
        if dependency != "vscode" && !is_node_builtin(&dependency) {
            return Err(format!(
                "extension.cjs has an unpackaged runtime require: {dependency}"
            ));
        }
    }

    let vsix_manifest = editor_archive::read_zip_text(&vsix, "extension.vsixmanifest")?;
    for needle in [
        r#"Id="vize""#,
        &format!(r#"Version="{workspace_version}""#),
        r#"Publisher="ubugeeei""#,
        "Microsoft.VisualStudio.Code",
    ] {
        editor_archive::expect_contains(
            &vsix_manifest,
            needle,
            &format!("VSIX manifest missing {needle}"),
        )?;
    }

    println!(
        "VSIX smoke passed: {} ({} files)",
        common::relative_path(&root, &vsix),
        entries.len()
    );
    Ok(())
}

fn require_binary_entries(entries: &[String], required: &[&str]) -> Result<(), String> {
    let set = entries.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for name in required {
        if !set.contains(name) {
            return Err(format!("VSIX is missing required file: {name}"));
        }
    }
    Ok(())
}

fn absolute_from_cwd(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|error| format!("cannot read current dir: {error}"))
}

fn assert_safe_vsix_entries(entries: &[String]) -> Result<(), String> {
    for name in entries {
        if name.contains('\\') {
            return Err(format!("VSIX entry must use POSIX separators: {name}"));
        }
        if name.contains('\0') {
            return Err(format!("VSIX entry contains a NUL byte: {name}"));
        }
        if name.starts_with('/') {
            return Err(format!("VSIX entry must be relative: {name}"));
        }
        if name.split('/').any(|part| part == "..") {
            return Err(format!("VSIX entry must not traverse: {name}"));
        }
        if !(name.starts_with("extension/")
            || name == "extension.vsixmanifest"
            || name == "[Content_Types].xml")
        {
            return Err(format!(
                "VSIX entry has an unexpected top-level path: {name}"
            ));
        }
    }
    Ok(())
}

fn assert_allowed_entries(entries: &[String]) -> Result<(), String> {
    let exact = [
        "extension/LICENSE.txt",
        "extension/changelog.md",
        "extension/dist/extension.cjs",
        "extension/icons/logo.png",
        "extension/icons/vue.svg",
        "extension/language-configuration.json",
        "extension/node_modules/@vizejs/typescript-vue-plugin/component-contracts.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/import-resolution.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/index.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/module-resolution.cjs",
        "extension/node_modules/@vizejs/typescript-vue-plugin/package.json",
        "extension/node_modules/@vizejs/typescript-vue-plugin/virtual-modules.cjs",
        "extension/package.json",
        "extension/readme.md",
        "extension/syntaxes/art-vue.tmLanguage.json",
        "extension/syntaxes/vue-script.tmLanguage.json",
        "extension/syntaxes/vue.tmLanguage.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    for name in entries
        .iter()
        .filter(|entry| entry.starts_with("extension/"))
    {
        if !exact.contains(name.as_str()) {
            return Err(format!("VSIX ships an unexpected extension file: {name}"));
        }
    }
    Ok(())
}

fn assert_forbidden_entries(entries: &[String]) -> Result<(), String> {
    for name in entries {
        let forbidden = name.starts_with("extension/.github/")
            || name.starts_with("extension/.vscode-test/")
            || name.starts_with("extension/.vscode/")
            || (name.starts_with("extension/dist/") && name.ends_with(".map"))
            || (name.starts_with("extension/node_modules/")
                && !matches!(
                    name.as_str(),
                    "extension/node_modules/@vizejs/typescript-vue-plugin/component-contracts.cjs"
                        | "extension/node_modules/@vizejs/typescript-vue-plugin/import-resolution.cjs"
                        | "extension/node_modules/@vizejs/typescript-vue-plugin/index.cjs"
                        | "extension/node_modules/@vizejs/typescript-vue-plugin/module-resolution.cjs"
                        | "extension/node_modules/@vizejs/typescript-vue-plugin/package.json"
                        | "extension/node_modules/@vizejs/typescript-vue-plugin/virtual-modules.cjs"
                ))
            || name == "extension/package-lock.json"
            || name == "extension/pnpm-lock.yaml"
            || name == "extension/pnpm-workspace.yaml"
            || name.starts_with("extension/src/")
            || name.starts_with("extension/test/")
            || name.starts_with("extension/tests/")
            || name.starts_with("extension/test-fixtures/")
            || name == "extension/tsconfig.json"
            || name == "extension/vite.config.ts"
            || name.ends_with(".vsix");
        if forbidden {
            return Err(format!("VSIX must not ship {name}"));
        }
    }
    Ok(())
}

fn assert_typescript_vue_plugin(vsix: &Path, package_json: &Value) -> Result<(), String> {
    let plugins = package_json
        .pointer("/contributes/typescriptServerPlugins")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing contributes.typescriptServerPlugins".to_string())?;
    if plugins.len() != 1 {
        return Err("typescriptServerPlugins must contain one entry".to_string());
    }
    let plugin = &plugins[0];
    assert_json_string(plugin, &["name"], "@vizejs/typescript-vue-plugin")?;
    if plugin
        .get("enableForWorkspaceTypeScriptVersions")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("typescript plugin must enable workspace TypeScript versions".to_string());
    }
    let plugin_package = editor_archive::read_zip_json(
        vsix,
        "extension/node_modules/@vizejs/typescript-vue-plugin/package.json",
    )?;
    assert_json_string(&plugin_package, &["name"], "@vizejs/typescript-vue-plugin")?;
    assert_json_string(&plugin_package, &["main"], "index.cjs")?;
    let index = editor_archive::read_zip_text(
        vsix,
        "extension/node_modules/@vizejs/typescript-vue-plugin/index.cjs",
    )?;
    let virtual_modules = editor_archive::read_zip_text(
        vsix,
        "extension/node_modules/@vizejs/typescript-vue-plugin/virtual-modules.cjs",
    )?;
    editor_archive::expect_contains(
        &index,
        "function init({ typescript: ts })",
        "typescript-vue-plugin index.cjs is missing init",
    )?;
    editor_archive::expect_contains(
        &virtual_modules,
        "function installVueVirtualModules(ts, info)",
        "typescript-vue-plugin virtual modules are missing install hook",
    )?;
    Ok(())
}

fn assert_language_contributions(package_json: &Value) -> Result<(), String> {
    let languages = package_json
        .pointer("/contributes/languages")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing contributes.languages".to_string())?;
    assert_language(languages, "vue", ".vue")?;
    assert_language(languages, "art-vue", ".art.vue")?;
    let grammars = package_json
        .pointer("/contributes/grammars")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing contributes.grammars".to_string())?;
    assert_grammar(
        grammars,
        "vue",
        "source.vue",
        "./syntaxes/vue.tmLanguage.json",
    )?;
    assert_grammar(
        grammars,
        "art-vue",
        "source.art-vue",
        "./syntaxes/art-vue.tmLanguage.json",
    )?;
    Ok(())
}

fn assert_language(languages: &[Value], id: &str, extension: &str) -> Result<(), String> {
    let language = languages
        .iter()
        .find(|language| language.get("id").and_then(Value::as_str) == Some(id))
        .ok_or_else(|| format!("missing language contribution: {id}"))?;
    let aliases = json_string_array(language, &["aliases"])?;
    if !aliases.contains(&(if id == "vue" { "Vue" } else { "Art Vue" }).to_string()) {
        return Err(format!("language {id} missing alias"));
    }
    let extensions = json_string_array(language, &["extensions"])?;
    if !extensions.contains(&extension.to_string()) {
        return Err(format!("language {id} missing extension {extension}"));
    }
    assert_json_string(
        language,
        &["configuration"],
        "./language-configuration.json",
    )?;
    assert_json_string(language, &["icon", "light"], "./icons/vue.svg")?;
    assert_json_string(language, &["icon", "dark"], "./icons/vue.svg")?;
    Ok(())
}

fn assert_grammar(
    grammars: &[Value],
    language: &str,
    scope: &str,
    path: &str,
) -> Result<(), String> {
    let grammar = grammars
        .iter()
        .find(|grammar| grammar.get("language").and_then(Value::as_str) == Some(language))
        .ok_or_else(|| format!("missing grammar contribution: {language}"))?;
    assert_json_string(grammar, &["scopeName"], scope)?;
    assert_json_string(grammar, &["path"], path)?;
    let embedded = grammar
        .get("embeddedLanguages")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("grammar {language} missing embeddedLanguages"))?;
    for (scope, language) in [
        ("source.css", "css"),
        ("source.css.less", "less"),
        ("source.css.scss", "scss"),
        ("source.js", "javascript"),
        ("source.json", "json"),
        ("source.ts", "typescript"),
        ("text.html.basic", "html"),
    ] {
        if embedded.get(scope).and_then(Value::as_str) != Some(language) {
            return Err(format!(
                "grammar {language} missing embedded language {scope}"
            ));
        }
    }
    Ok(())
}

fn assert_configuration_defaults(package_json: &Value) -> Result<(), String> {
    let properties = package_json
        .pointer("/contributes/configuration/properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing contributes.configuration.properties".to_string())?;
    if properties
        .get("vize.enable")
        .and_then(|value| value.get("default"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("vize.enable has an unexpected default".to_string());
    }
    assert_json_string(
        properties
            .get("vize.serverPath")
            .ok_or("missing vize.serverPath")?,
        &["default"],
        "",
    )?;
    assert_json_string(
        properties
            .get("vize.trace.server")
            .ok_or("missing vize.trace.server")?,
        &["default"],
        "off",
    )?;
    for (key, property) in properties {
        if key == "vize.serverPath"
            || key == "vize.trace.server"
            || (key.starts_with("vize.autoInsert.") && key != "vize.autoInsert.enable")
        {
            continue;
        }
        let expected = !matches!(
            key.as_str(),
            "vize.diagnostics.enable"
                | "vize.formatting.enable"
                | "vize.optionsApi.enable"
                | "vize.legacyVue2.enable"
                | "vize.autoInsert.enable"
        );
        if property.get("default").and_then(Value::as_bool) != Some(expected) {
            return Err(format!("{key} has an unexpected default"));
        }
    }
    Ok(())
}

fn assert_json_string(value: &Value, path: &[&str], expected: &str) -> Result<(), String> {
    editor_archive::expect_json_string(value, path, expected)
}

fn json_string_array(value: &Value, path: &[&str]) -> Result<Vec<String>, String> {
    let node = path
        .iter()
        .try_fold(value, |node, key| node.get(*key))
        .ok_or_else(|| format!("missing JSON array {}", path.join(".")))?;
    let array = node
        .as_array()
        .ok_or_else(|| format!("{} must be an array", path.join(".")))?;
    array
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| format!("{} must contain only strings", path.join(".")))
        })
        .collect()
}

fn assert_unique_json_strings(value: &Value, path: &[&str]) -> Result<(), String> {
    assert_unique_values(&json_string_array(value, path)?, &path.join("."))
}

fn assert_unique_values(values: &[String], label: &str) -> Result<(), String> {
    let unique = values.iter().collect::<HashSet<_>>();
    if unique.len() != values.len() {
        return Err(format!("{label} must not contain duplicates"));
    }
    Ok(())
}

fn assert_no_hidden_host_test_commands(values: &[String], label: &str) -> Result<(), String> {
    for value in values {
        let command = value.strip_prefix("onCommand:").unwrap_or(value);
        if command.starts_with("vize.test.") {
            return Err(format!(
                "{label} must not expose hidden host smoke command: {value}"
            ));
        }
    }
    Ok(())
}

fn find_static_requires(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search = source;
    while let Some(at) = search.find("require(") {
        search = &search[at + "require(".len()..];
        let trimmed = search.trim_start();
        let Some(quote) = trimmed
            .chars()
            .next()
            .filter(|ch| matches!(ch, '"' | '\'' | '`'))
        else {
            continue;
        };
        let body = &trimmed[quote.len_utf8()..];
        if let Some(end) = body.find(quote) {
            out.push(body[..end].to_string());
            search = &body[end + quote.len_utf8()..];
        } else {
            break;
        }
    }
    out
}

fn is_node_builtin(specifier: &str) -> bool {
    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    matches!(
        specifier,
        "assert"
            | "buffer"
            | "child_process"
            | "crypto"
            | "events"
            | "fs"
            | "http"
            | "https"
            | "module"
            | "net"
            | "os"
            | "path"
            | "process"
            | "stream"
            | "string_decoder"
            | "timers"
            | "tty"
            | "url"
            | "util"
            | "v8"
            | "vm"
            | "zlib"
    )
}
