use crate::batch::{
    TsconfigOwnershipCache, TsconfigSourceKind, snapshot_tsconfig_compiler_options,
};
use serde_json::json;

#[test]
fn filename_defaults_and_own_overrides_survive_inheritance() {
    for (name, parent_name, parent, own, expected, allows_js) in [
        (
            "jsconfig.json",
            "base.json",
            json!({"allowJs": false, "maxNodeModuleJsDepth": 0, "skipLibCheck": false,
                "noEmit": false, "strict": true}),
            json!({"checkJs": true}),
            json!({"allowJs": true, "maxNodeModuleJsDepth": 2, "skipLibCheck": true,
                "noEmit": true, "strict": true, "checkJs": true}),
            true,
        ),
        (
            "jsconfig.json",
            "base.json",
            json!({"allowJs": true, "strict": true}),
            json!({"allowJs": false, "maxNodeModuleJsDepth": 0, "skipLibCheck": false,
                "noEmit": false}),
            json!({"allowJs": false, "maxNodeModuleJsDepth": 0, "skipLibCheck": false,
                "noEmit": false, "strict": true}),
            false,
        ),
        (
            "tsconfig.json",
            "jsconfig.json",
            json!({"strict": true}),
            json!({"checkJs": false}),
            json!({"allowJs": true, "maxNodeModuleJsDepth": 2, "skipLibCheck": true,
                "noEmit": true, "strict": true, "checkJs": false}),
            true,
        ),
        (
            "tsconfig.json",
            "base.json",
            json!({"strict": true}),
            json!({"noEmit": false}),
            json!({"strict": true, "noEmit": false}),
            false,
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("src/message.js");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const message = 42;\n").unwrap();
        std::fs::write(
            root.path().join(parent_name),
            json!({"compilerOptions": parent}).to_string(),
        )
        .unwrap();
        let config = root.path().join(name);
        std::fs::write(
            &config,
            json!({"extends": format!("./{parent_name}"), "compilerOptions": own,
                "include": ["src/**/*"]})
            .to_string(),
        )
        .unwrap();
        assert_eq!(
            serde_json::Value::Object(
                snapshot_tsconfig_compiler_options(root.path(), &config).unwrap()
            ),
            expected,
        );
        let mut ownership = TsconfigOwnershipCache::default();
        assert_eq!(ownership.project_allows_js(&config), allows_js);
        assert_eq!(
            ownership.project_owns_source(&config, &source, TsconfigSourceKind::JavaScript),
            allows_js,
        );
    }
}
