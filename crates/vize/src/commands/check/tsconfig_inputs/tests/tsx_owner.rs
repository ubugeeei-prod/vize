use super::unique_case_dir;
use std::fs;

#[test]
fn tsconfig_for_files_can_use_referenced_tsx_owner_when_jsx_is_enabled() {
    let case_dir = unique_case_dir("tsconfig-reference-tsx-owner");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("packages/ui/src")).unwrap();
    let widget = case_dir.join("packages/ui/src/Widget.tsx");
    fs::write(&widget, "export const Widget = () => <button />").unwrap();
    let root = case_dir.join("tsconfig.json");
    fs::write(
        &root,
        r#"{
  "files": [],
  "references": [{ "path": "./packages/ui/tsconfig.json" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/ui/tsconfig.json"),
        r#"{
  "compilerOptions": {
    "jsx": "preserve",
    "jsxImportSource": "vue"
  },
  "include": ["src/**/*.tsx"]
}"#,
    )
    .unwrap();

    let owner = super::super::resolve_tsconfig_for_files(
        Some(&root),
        &[widget],
        true,
        &mut super::TsconfigInputCache::default(),
    );

    assert_eq!(owner, Some(case_dir.join("packages/ui/tsconfig.json")));

    let _ = fs::remove_dir_all(&case_dir);
}
