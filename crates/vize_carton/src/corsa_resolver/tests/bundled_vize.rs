use super::{discover_in_walk, platform_suffix, write_file};
use std::fs;
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

    pnpm_sibling_link_resolves(&package_name, executable);
}

#[cfg(unix)]
fn pnpm_sibling_link_resolves(package_name: &str, executable: &str) {
    let temp = TempDir::new().unwrap();
    let store = temp.path().join("store").join("node_modules");
    let vize = store.join("vize");
    fs::create_dir_all(&vize).unwrap();
    fs::write(
        vize.join("package.json"),
        format!(r#"{{"name":"vize","optionalDependencies":{{"{package_name}":"7.0.2"}}}}"#),
    )
    .unwrap();
    let platform = store.join(package_name);
    fs::create_dir_all(platform.join("lib")).unwrap();
    fs::write(
        platform.join("package.json"),
        format!(r#"{{"name":"{package_name}","version":"7.0.2"}}"#),
    )
    .unwrap();
    write_file(&platform.join("lib").join(executable));

    let project = temp.path().join("project");
    fs::create_dir_all(project.join("node_modules")).unwrap();
    fs::write(project.join("package.json"), "{}").unwrap();
    std::os::unix::fs::symlink(&vize, project.join("node_modules").join("vize")).unwrap();

    let found = discover_in_walk(&[project]).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        platform.join("lib").join(executable).canonicalize().unwrap()
    );
}

#[cfg(not(unix))]
fn pnpm_sibling_link_resolves(_package_name: &str, _executable: &str) {}
