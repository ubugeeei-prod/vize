//! The real Content Mapper host must preserve generic constructor slots (#10416).

use std::path::PathBuf;
use vize_l0::cstr;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

fn assert_invalid(output: &std::process::Output, extra_columns: usize) {
    assert_eq!(output.status.code(), Some(2), "{}", output_text(output));
    assert_eq!(output.stderr, b"", "{}", output_text(output));
    let stdout = std::str::from_utf8(&output.stdout)
        .unwrap()
        .replace('\\', "/");
    let first_column = 57 + extra_columns;
    let second_column = 70 + extra_columns;
    assert_eq!(
        stdout,
        cstr!(
            "src/Invalid.vue(6,{first_column}): error TS2339: Property 'nope' does not exist on type '{{ id: number; name: string; }}'.\nsrc/Invalid.vue(7,{second_column}): error TS2339: Property 'nope' does not exist on type '{{ id: number; name: string; }}'.\n"
        )
    );
}

fn assert_valid(output: &std::process::Output) {
    assert_eq!(output.status.code(), Some(0), "{}", output_text(output));
    assert_eq!(output.stdout, b"", "{}", output_text(output));
    assert_eq!(output.stderr, b"", "{}", output_text(output));
}

#[test]
fn generic_constructor_slots_match_authored_member_diagnostics() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper constructor slots: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/generic-constructor-slots-10416");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    for (newline, prefix) in [("\n", ""), ("\r\n", ""), ("\n", "🧭'\""), ("\r\n", "🧭'\"")] {
        let project = tempfile::Builder::new()
            .prefix("content-mapper-generic-constructor-slots-")
            .tempdir_in(&cases_root)
            .unwrap();
        std::fs::create_dir_all(project.path().join("src")).unwrap();
        for name in ["GenericList.ts", "Plain.vue", "Invalid.vue"] {
            let mut source =
                std::fs::read_to_string(fixture.join("src").join(cstr!("{name}.txt").as_str()))
                    .unwrap();
            if name.ends_with(".vue") {
                source = source.replace(">{{", &cstr!(">{prefix}{{{{"));
            }
            std::fs::write(
                project.path().join("src").join(name),
                source.replace('\n', newline),
            )
            .unwrap();
        }
        let mut config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap())
                .unwrap();
        config["contentMappers"] =
            serde_json::json!([{ "package": "vize", "extensions": [".vue"] }]);
        std::fs::write(
            project.path().join("tsconfig.json"),
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());

        let path = project.path().join("src/Invalid.vue");
        let invalid = std::fs::read_to_string(&path).unwrap();
        assert_eq!(invalid.matches(".nope").count(), 2);
        let repaired = invalid.replace(".nope", ".name");
        std::fs::write(&path, &repaired).unwrap();
        assert_valid(&check_project(&tsgo, project.path(), "tsconfig.json"));
        std::fs::write(&path, &invalid).unwrap();
        assert_invalid(
            &check_project(&tsgo, project.path(), "tsconfig.json"),
            prefix.encode_utf16().count(),
        );
        std::fs::write(&path, &repaired).unwrap();
        assert_valid(&check_project(&tsgo, project.path(), "tsconfig.json"));
        std::fs::write(&path, &invalid).unwrap();
        assert_invalid(
            &check_project(&tsgo, project.path(), "tsconfig.json"),
            prefix.encode_utf16().count(),
        );
    }
}
