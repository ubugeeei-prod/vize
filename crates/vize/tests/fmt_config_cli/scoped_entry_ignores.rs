use super::*;

#[test]
fn fmt_check_keeps_files_excluded_only_from_scoped_linter_settings() {
    const CONFIG: &str = include_str!(
        "../../../../tests/_fixtures/config-scoped-entry-ignores-11146/vue-options.json"
    );
    const NORMAL: &str = include_str!(
        "../../../../tests/_fixtures/config-scoped-entry-ignores-11146/button.vue.txt"
    );
    const STORY: &str = include_str!(
        "../../../../tests/_fixtures/config-scoped-entry-ignores-11146/button.story.vue.txt"
    );
    let project = tempfile::tempdir().unwrap();
    vite_config::write(project.path(), CONFIG);
    write_project_file(project.path(), "src/button.vue", NORMAL);
    write_project_file(project.path(), "src/button.story.vue", STORY);
    for (pattern, expected) in [
        (
            "src/**/*.vue",
            "Found 2 file(s)\nWould reformat: src/button.story.vue\n\nChecked 2 file(s)\n  1 file(s) would be reformatted\n  1 file(s) already formatted\n",
        ),
        (
            "src/button.story.vue",
            "Found 1 file(s)\nWould reformat: src/button.story.vue\n\nChecked 1 file(s)\n  1 file(s) would be reformatted\n",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("RAYON_NUM_THREADS", "1")
            .args(["fmt", "--check", pattern])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert_eq!(output.stdout, Vec::<u8>::new(), "{output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().replace('\\', "/"),
            expected
        );
        assert_eq!(
            fs::read_to_string(project.path().join("src/button.vue")).unwrap(),
            NORMAL
        );
        assert_eq!(
            fs::read_to_string(project.path().join("src/button.story.vue")).unwrap(),
            STORY
        );
        assert_eq!(
            fs::read_to_string(project.path().join("vite.config.mjs")).unwrap(),
            vite_config::source(CONFIG)
        );
    }
}
