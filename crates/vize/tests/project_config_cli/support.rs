use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

pub const SETTINGS: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/project-settings-8371/vite.config.mjs.txt"
);
pub const QUOTES: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/project-settings-8371/Quotes.vue.txt"
);
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
        ("vite.config.mjs", SETTINGS),
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

pub fn assert_success(output: &Output) {
    assert!(output.status.success(), "{output:?}");
}

pub fn assert_no_dedicated_config(root: &Path) {
    assert!(fs::read_dir(root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("vize.config.")
    }));
}

#[cfg(feature = "glyph")]
pub fn formatted(root: &Path, args: &[&str]) -> vize_l0::String {
    write(root, "Quotes.vue", QUOTES);
    let mut command = vec!["fmt", "--write", "Quotes.vue"];
    command.extend_from_slice(args);
    assert_success(&run(root, &command));
    fs::read_to_string(root.join("Quotes.vue")).unwrap().into()
}
