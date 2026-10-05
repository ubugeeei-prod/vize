//! Hidden configured manifests retain module and missing reference roots.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]
use super::collect_hidden_ambient_declaration_files;
use crate::commands::check::tsconfig_inputs::TsconfigInputCache;

#[test]
fn hidden_path_manifests_keep_module_and_missing_reference_roots() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join(".nuxt/types/builder-env.d.ts"),
        "import \"../../vendor/client/index\";\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/a.ts"),
        "export const text: string = 'source';\n",
    )
    .unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"include":[".nuxt/env.d.ts","src/**/*"]}"#,
    )
    .unwrap();
    for reference in ["types/builder-env.d.ts", "types/missing.d.ts"] {
        std::fs::write(
            root.join(".nuxt/env.d.ts"),
            vize_l0::cstr!("/// <reference path=\"{reference}\" />\nexport {{}};\n").as_bytes(),
        )
        .unwrap();
        let files = collect_hidden_ambient_declaration_files(
            &root,
            Some(&root.join("tsconfig.json")),
            &mut TsconfigInputCache::default(),
        );
        assert_eq!(files, vec![root.join(".nuxt/env.d.ts")]);
    }
}
