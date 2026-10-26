#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! #10772: excluding every `.vue` file from Oxfmt through `fmt.ignorePatterns`
//! hands Vue files to `vize fmt` instead of removing them from it.

use std::{fs, path::Path, process::Command};

const APP: &str = r#"<script setup lang="ts">
const label = "hello";
</script>

<template>
  <p class="greeting">{{ label }}</p>
</template>
"#;

fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn fmt_check(root: &Path, files: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["fmt", "--check"])
        .args(files)
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn oxfmt_side_vue_exclusion_keeps_vue_files_with_vize_fmt() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(root, "package.json", r#"{ "type": "module" }"#);
    write(
        root,
        "vite.config.mjs",
        r#"export default {
  fmt: { semi: false, ignorePatterns: ["**/*.vue", "src/generated/**"] },
  vue: { formatter: { semi: false } },
};
"#,
    );
    write(root, "src/App.vue", APP);
    write(root, "src/generated/Generated.vue", APP);

    // The `;` is reported instead of "No ... files found".
    let (status, stderr) = fmt_check(root, &["src/App.vue"]);
    assert_eq!(status, Some(1), "{stderr}");
    assert!(stderr.contains("Would reformat: src/App.vue"), "{stderr}");
    assert!(!stderr.contains("No .vue"), "{stderr}");

    // Other shared exclusions still apply, and naming the file says why.
    let (status, stderr) = fmt_check(root, &["src/App.vue", "src/generated/Generated.vue"]);
    assert_eq!(status, Some(1), "{stderr}");
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(
        stderr.contains(
            "Skipping src/generated/Generated.vue: excluded by the configured ignore patterns"
        ),
        "{stderr}"
    );

    // Default discovery reports absolute paths.
    let (status, stderr) = fmt_check(root, &[]);
    assert_eq!(status, Some(1), "{stderr}");
    assert!(stderr.contains("/src/App.vue\n"), "{stderr}");
    assert!(!stderr.contains("Generated"), "{stderr}");
}
