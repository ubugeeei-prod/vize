use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-build-profile-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

#[test]
fn build_stats_profile_reports_source_plate_facts() {
    let project_root = temp_project_dir("stats-profile-source-facts");
    let source = r#"<template><div>Hello</div></template>
"#;
    let self_component_source = r#"<template><App /></template>
"#;
    write_project_file(&project_root, "src/App.vue", self_component_source);
    write_project_file(&project_root, "src/Foo.vue", source);
    write_project_file(&project_root, "src/Bar.vue", source);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "--format",
            "stats",
            "--profile",
            "--threads",
            "1",
            "src/App.vue",
            "src/Bar.vue",
            "src/Foo.vue",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "Stats compile cache",
        "cache.stats_compile.hits",
        "cache.stats_compile.misses",
        "cache.stats_compile.stores",
        "cache.stats_compile.bypasses",
        "cache.stats_compile.bypass.self_component",
        "Source facts",
        "source.plate.sfc.requests",
        "source.block.template.bytes",
        "source.cache.hit.files",
        "source.cache.bypass.self_component.files",
        "Product lanes",
        "lane.atelier.dom.requests",
        "Vue dialects",
        "dialect.vue3.files",
        "Template syntax",
        "template_syntax.standard.files",
        "lane atelier.dom, plate source.sfc",
    ] {
        assert!(stderr.contains(expected), "missing {expected:?}\n{stderr}");
    }

    let _ = fs::remove_dir_all(project_root);
}
