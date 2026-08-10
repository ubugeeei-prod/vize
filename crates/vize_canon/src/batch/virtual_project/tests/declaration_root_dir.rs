//! Declaration emit must honor a configured `rootDir` (#3355).
//!
//! The emitted layout is decided by the `rootDir` written into the generated
//! declaration tsconfig. Inferring it from the common source directory agrees
//! with an explicit `rootDir` only while every file sits under that directory;
//! add one file outside it and the common directory collapses toward the
//! project root, so every declaration keeps its source directory prefix
//! (`dist/lib/index.d.ts` instead of `dist/index.d.ts`).
//!
//! Each case therefore registers a file *outside* the configured `rootDir`, so
//! the inferred value and the configured one differ and the assertion can tell
//! them apart.

use std::fs;
use std::path::Path;

use super::{VirtualProject, unique_case_dir};
use crate::batch::project_virtual_root;

/// Writes a project whose sources span `lib/` and the project root, so the
/// common source directory is the root while `rootDir` names `lib`.
fn write_project(
    case_dir: &Path,
    tsconfig: &str,
    extra_files: &[(&str, &str)],
) -> std::path::PathBuf {
    let _ = fs::remove_dir_all(case_dir);
    let lib_dir = case_dir.join("lib");
    fs::create_dir_all(&lib_dir).unwrap();
    fs::write(case_dir.join("tsconfig.json"), tsconfig).unwrap();
    for (relative_path, contents) in extra_files {
        let path = case_dir.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    let vue_path = lib_dir.join("RootDir.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">\ndefineProps<{ label: string }>()\n</script>\n",
    )
    .unwrap();
    fs::write(lib_dir.join("index.ts"), "export const version = '1.0.0'\n").unwrap();
    fs::write(case_dir.join("extra.ts"), "export const buildFlag = true\n").unwrap();

    vue_path
}

fn declaration_root_dir(case_dir: &Path, compiler_options: &str) -> String {
    declaration_root_dir_from_tsconfig(
        case_dir,
        &format!("{{\n  \"compilerOptions\": {{\n{compiler_options}\n  }}\n}}"),
        &[],
    )
}

fn declaration_root_dir_from_tsconfig(
    case_dir: &Path,
    tsconfig: &str,
    extra_files: &[(&str, &str)],
) -> String {
    let vue_path = write_project(case_dir, tsconfig, extra_files);
    let mut project = VirtualProject::new(case_dir).unwrap();
    project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
    project.register_path(&vue_path).unwrap();
    project
        .register_path(&case_dir.join("lib/index.ts"))
        .unwrap();
    project.register_path(&case_dir.join("extra.ts")).unwrap();
    project.materialize().unwrap();
    project
        .write_declaration_tsconfig(&case_dir.join("dist"), false)
        .unwrap();

    let config_path = project_virtual_root(case_dir).join("tsconfig.declaration.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    value["compilerOptions"]["rootDir"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn declaration_tsconfig_honors_a_configured_root_dir() {
    let case_dir = unique_case_dir("declaration-root-dir-configured");
    let root_dir = declaration_root_dir(&case_dir, "    \"rootDir\": \"./lib\"");

    let virtual_root = fs::canonicalize(project_virtual_root(&case_dir)).unwrap();
    assert_eq!(
        Path::new(&root_dir),
        virtual_root.join("lib"),
        "a configured rootDir must reach the declaration tsconfig rebased onto \
         the virtual mirror; inferring the common source directory here would \
         yield the mirror root and prefix every declaration with `lib/`"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn declaration_tsconfig_resolves_an_inherited_root_dir_from_its_declaring_config() {
    // A relative `rootDir` resolves against the tsconfig that declares it, so a
    // base config in `configs/` naming `../lib` means `<project>/lib`. Resolving
    // it against the extending config instead would aim above the project root,
    // where no mirror counterpart exists, and silently fall back to inference.
    let case_dir = unique_case_dir("declaration-root-dir-extends");
    let root_dir = declaration_root_dir_from_tsconfig(
        &case_dir,
        "{\n  \"extends\": \"./configs/base.json\"\n}",
        &[(
            "configs/base.json",
            "{\n  \"compilerOptions\": {\n    \"rootDir\": \"../lib\"\n  }\n}",
        )],
    );

    let virtual_root = fs::canonicalize(project_virtual_root(&case_dir)).unwrap();
    assert_eq!(
        Path::new(&root_dir),
        virtual_root.join("lib"),
        "an inherited rootDir must resolve against the base config directory"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn declaration_tsconfig_infers_the_root_dir_when_none_is_configured() {
    let case_dir = unique_case_dir("declaration-root-dir-inferred");
    let root_dir = declaration_root_dir(&case_dir, "    \"strict\": true");

    let virtual_root = fs::canonicalize(project_virtual_root(&case_dir)).unwrap();
    assert_eq!(
        Path::new(&root_dir),
        virtual_root,
        "with no rootDir configured the inferred common source directory must \
         still be used, which for sources spanning lib/ and the project root is \
         the mirror root"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn declaration_tsconfig_ignores_a_root_dir_outside_the_project() {
    // No mirror counterpart exists for a directory above the project root, so
    // the layout falls back to inference rather than emitting a path that does
    // not exist inside the virtual project.
    let case_dir = unique_case_dir("declaration-root-dir-outside");
    let root_dir = declaration_root_dir(&case_dir, "    \"rootDir\": \"../outside\"");

    let virtual_root = fs::canonicalize(project_virtual_root(&case_dir)).unwrap();
    assert_eq!(Path::new(&root_dir), virtual_root);

    let _ = fs::remove_dir_all(&case_dir);
}
