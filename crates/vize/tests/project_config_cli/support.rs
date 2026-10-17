#![expect(
    clippy::disallowed_types,
    reason = "adapted fixture source and process evidence use std strings"
)]

use std::{
    fs,
    path::Path,
    process::{Command, Output},
    sync::LazyLock,
};

// Adapt the historical source only when copying it into a live project.
pub fn canonical_settings(source: &str) -> String {
    source
        .replace("vize:", "vue:")
        .replace("lsp:", "languageServer:")
}

pub static SETTINGS: LazyLock<String> = LazyLock::new(|| {
    canonical_settings(include_str!(
        "../../../../tests/_fixtures/differential/config/project-settings-8371/vite.config.mjs.txt"
    ))
});
pub const QUOTES: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/project-settings-8371/Quotes.vue.txt"
);
#[cfg(feature = "glyph")]
pub const FORMATTED_SINGLE: &str = "<script setup>\nconst message = 'hello';\n</script>\n\n<template>\n  <p>{{ message }}</p>\n</template>\n";
#[cfg(feature = "glyph")]
pub const FORMATTED_DOUBLE: &str = "<script setup>\nconst message = \"hello\";\n</script>\n\n<template>\n  <p>{{ message }}</p>\n</template>\n";
#[cfg(feature = "glyph")]
pub const PRESERVED_SINGLE: &str = "<script setup>\nconst message = 'hello';\n</script>\n\n<template><p>{{ message }}</p></template>\n";
#[cfg(feature = "glyph")]
pub const PRESERVED_DOUBLE: &str = "<script setup>\nconst message = \"hello\";\n</script>\n\n<template><p>{{ message }}</p></template>\n";
pub const CONSUMER: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/project-settings-8371/Consumer.vue.txt"
);
pub const INVALID_TS: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/project-settings-8371/invalid.ts.txt"
);

pub fn write(root: &Path, name: &str, source: &str) {
    let file = root.join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, source).unwrap();
}

pub fn project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    for (name, source) in [
        (
            "package.json",
            r#"{"name":"project-settings-fixture","private":true,"type":"module"}"#,
        ),
        ("vite.config.mjs", SETTINGS.as_str()),
        (
            "tsconfig.json",
            include_str!(
                "../../../../tests/_fixtures/differential/config/project-settings-8371/tsconfig.json.txt"
            ),
        ),
        (
            "App.vue",
            include_str!(
                "../../../../tests/_fixtures/differential/config/project-settings-8371/App.vue.txt"
            ),
        ),
        ("Quotes.vue", QUOTES),
        (
            "Image.vue",
            include_str!(
                "../../../../tests/_fixtures/differential/config/project-settings-8371/Image.vue.txt"
            ),
        ),
        (
            "src/Child.vue",
            include_str!(
                "../../../../tests/_fixtures/differential/config/project-settings-8371/Child.vue.txt"
            ),
        ),
        ("src/Consumer.vue", CONSUMER),
        (
            "src/contracts.ts",
            include_str!(
                "../../../../tests/_fixtures/differential/config/project-settings-8371/contracts.ts.txt"
            ),
        ),
    ] {
        write(root, name, source);
    }
    project
}

pub fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args(args)
        .output()
        .unwrap()
}

pub const LEGACY_CONFIG_ERROR: &str = "vize.config.* is no longer supported; move Vue settings to vue: in vite.config.* and TypeScript settings to tsconfig.json";

pub fn assert_legacy_config_rejected(output: &Output) {
    // Invalid configuration is a usage failure, distinct from lint diagnostics.
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        vize_l0::cstr!(
            "\x1b[31mError:\x1b[0m failed to parse vize.config.json: {LEGACY_CONFIG_ERROR}\n"
        )
        .as_str(),
        "{output:?}",
    );
    assert!(output.stdout.is_empty(), "{output:?}");
}

pub fn assert_success(output: &Output) {
    assert!(output.status.success(), "{output:?}");
}

pub fn assert_no_dedicated_config(root: &Path) {
    const DEDICATED_FILES: [&str; 5] = [
        "vize.config.pkl",
        "vize.config.ts",
        "vize.config.js",
        "vize.config.mjs",
        "vize.config.json",
    ];
    let inventory = DEDICATED_FILES.map(|name| (name, root.join(name).exists()));
    assert_eq!(inventory, DEDICATED_FILES.map(|name| (name, false)));
}

pub fn normalized_report(root: &Path, mut report: serde_json::Value) -> serde_json::Value {
    fn replace_root(value: &mut serde_json::Value, root: &str) {
        match value {
            serde_json::Value::String(text) => *text = text.replace(root, "<project>"),
            serde_json::Value::Array(values) => {
                for value in values {
                    replace_root(value, root);
                }
            }
            serde_json::Value::Object(values) => {
                for value in values.values_mut() {
                    replace_root(value, root);
                }
            }
            _ => {}
        }
    }
    replace_root(&mut report, root.canonicalize().unwrap().to_str().unwrap());
    replace_root(&mut report, root.to_str().unwrap());
    report
}

#[cfg(feature = "glyph")]
pub fn formatted(root: &Path, args: &[&str]) -> vize_l0::String {
    write(root, "Quotes.vue", QUOTES);
    let mut command = vec!["fmt", "--write", "Quotes.vue"];
    command.extend_from_slice(args);
    assert_success(&run(root, &command));
    fs::read_to_string(root.join("Quotes.vue")).unwrap().into()
}
