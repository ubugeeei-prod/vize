use super::{discover_in_walk, platform_suffix, write_file};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn nested_vize_optional_platform_compiler_is_the_fallback() {
    let suffix = platform_suffix();
    if suffix.is_empty() {
        return;
    }
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    fs::write(project.join("package.json"), "{}").unwrap();
    let vize = project.join("node_modules").join("vize");
    let package_name = format!("@typescript/typescript-{suffix}");
    fs::create_dir_all(&vize).unwrap();
    fs::write(
        vize.join("package.json"),
        format!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    )
    .unwrap();
    let platform = vize.join("node_modules").join(&package_name);
    fs::create_dir_all(platform.join("lib")).unwrap();
    fs::write(
        platform.join("package.json"),
        format!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    )
    .unwrap();
    let executable = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    write_file(&platform.join("lib").join(executable));

    let found = discover_in_walk(&[project.to_path_buf()]).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        platform.join("lib").join(executable).canonicalize().unwrap()
    );
    let _ = Path::new(&package_name);
}
