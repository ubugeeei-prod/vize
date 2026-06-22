use super::{collect_default_check_files, relative_paths, unique_case_dir};

#[test]
fn default_collection_skips_generated_codegen_declaration_modules() {
    let case_dir = unique_case_dir("tsconfig-generated-codegen-dts");
    let _ = std::fs::remove_dir_all(&case_dir);
    std::fs::create_dir_all(case_dir.join("src")).unwrap();
    std::fs::create_dir_all(case_dir.join("types/codegen")).unwrap();
    std::fs::write(case_dir.join("src/env.d.ts"), "declare const X: string;").unwrap();
    std::fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    std::fs::write(
        case_dir.join("types/codegen/schema.d.ts"),
        "export enum AimQuestionDisplayKind { Text = 'TEXT' }\n",
    )
    .unwrap();
    std::fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*.vue", "src/**/*.d.ts", "types/codegen/schema.d.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/App.vue", "src/env.d.ts"]
    );

    let _ = std::fs::remove_dir_all(&case_dir);
}
