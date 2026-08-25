use std::path::{Path, PathBuf};
use std::process::Command;

use super::{copy_fixture, install_packages, output_text};

pub fn emit_vue_declaration_library(
    tsgo: &Path,
    library_path: &Path,
    app_source: &str,
    index_source: &str,
) {
    install_packages(library_path);
    std::fs::create_dir_all(library_path.join("src")).unwrap();
    std::fs::write(library_path.join("src/App.vue"), app_source).unwrap();
    std::fs::write(library_path.join("src/index.ts"), index_source).unwrap();
    std::fs::write(
        library_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "declaration": true,
    "declarationMap": true,
    "emitDeclarationOnly": true,
    "rootDir": "src",
    "outDir": "dist"
  },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let emit = Command::new(tsgo)
        .current_dir(library_path)
        .args([
            "--runExternalCode",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ])
        .output()
        .unwrap();
    assert!(emit.status.success(), "{}", output_text(&emit));
}

pub fn install_packed_vue_consumer(
    consumer_path: &Path,
    library_path: &Path,
    consumer_source: &str,
) -> PathBuf {
    install_packages(consumer_path);
    std::fs::create_dir_all(consumer_path.join("src")).unwrap();
    std::fs::write(consumer_path.join("src/index.ts"), consumer_source).unwrap();
    std::fs::write(
        consumer_path.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let package_root = consumer_path.join("node_modules/@scope/emitted-vue");
    std::fs::create_dir_all(&package_root).unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/emitted-vue",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./index.js"
    }
  }
}"#,
    )
    .unwrap();
    copy_fixture(&library_path.join("dist"), &package_root.join("dist"));
    copy_fixture(&library_path.join("src"), &package_root.join("src"));
    package_root
}
