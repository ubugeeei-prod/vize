//! Preserve `.d.ts` spelling for external ESM declarations.

use std::path::Path;

pub(super) fn should_preserve_esm_declaration_spelling(path: &Path, content: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
        && declaration_is_external_esm_module(content)
        && nearest_package_is_type_module(path)
}

fn declaration_is_external_esm_module(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("import ")
            || (trimmed.starts_with("export ") && !trimmed.starts_with("export ="))
            || trimmed.starts_with("export{")
    })
}

fn nearest_package_is_type_module(path: &Path) -> bool {
    let mut current = path.parent();
    while let Some(dir) = current {
        let manifest = dir.join("package.json");
        if manifest.is_file() {
            return std::fs::read_to_string(manifest)
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|manifest| {
                    manifest
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .is_some_and(|package_type| package_type == "module");
        }
        current = dir.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::should_preserve_esm_declaration_spelling;

    #[test]
    fn preserves_external_esm_declarations_but_not_commonjs_export_assignment() {
        let root = std::env::temp_dir().join(format!(
            "vize-esm-declaration-spelling-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package.json"), r#"{"type":"module"}"#).unwrap();
        let package = root.join("node_modules/dual-pkg");
        std::fs::create_dir_all(package.join("dist")).unwrap();
        std::fs::write(package.join("package.json"), r#"{"type":"module"}"#).unwrap();

        let esm = package.join("dist/index.d.ts");
        std::fs::write(&esm, "export interface Status { id: string }\n").unwrap();
        assert!(should_preserve_esm_declaration_spelling(
            &esm,
            "export interface Status { id: string }\n",
        ));

        let augmentation = root.join("src/global.d.ts");
        std::fs::create_dir_all(augmentation.parent().unwrap()).unwrap();
        assert!(should_preserve_esm_declaration_spelling(
            &augmentation,
            "import 'dual-pkg';\ndeclare module 'dual-pkg/api/index.js' {}\n",
        ));

        let cjs = package.join("dist/cjs.d.ts");
        assert!(!should_preserve_esm_declaration_spelling(
            &cjs,
            "declare function bufferFrom(): void;\nexport = bufferFrom;\n",
        ));

        let authored = root.join("src/ambients/buffer-from.d.ts");
        std::fs::create_dir_all(authored.parent().unwrap()).unwrap();
        assert!(!should_preserve_esm_declaration_spelling(
            &authored,
            "declare module 'buffer-from' { export = bufferFrom; }\n",
        ));

        let _ = std::fs::remove_dir_all(&root);
    }
}
