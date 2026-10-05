//! Retain complete original runtime fixture observations before cleanup.

use std::path::Path;

pub(super) fn retain<T: serde::Serialize>(name: &str, root: &Path, snapshot: &Option<T>) {
    if let Some(capture) = std::env::var_os("VIZE_DEFAULT_PROP_CAPTURE") {
        if let Err(error) = write_capture(Path::new(&capture), name, root, snapshot) {
            panic!("original runtime fixture capture failed: {error}");
        }
    }
    assert!(
        snapshot.is_some() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
        "required native original runtime fixture {name} did not return a complete diagnostic snapshot"
    );
}

fn write_capture<T: serde::Serialize>(
    capture: &Path,
    name: &str,
    root: &Path,
    snapshot: &Option<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = capture.join("imported-runtime-controls").join(name);
    let inputs = target.join("inputs");
    std::fs::create_dir_all(&inputs)?;
    std::fs::write(
        target.join("diagnostics.json"),
        serde_json::to_vec_pretty(snapshot)?,
    )?;
    std::fs::copy(root.join("tsconfig.json"), inputs.join("tsconfig.json"))?;
    for directory in ["src", "packages"] {
        let source = root.join(directory);
        if source.is_dir() {
            copy_inputs(&source, &inputs.join(directory))?;
        }
    }
    for package in ["vue", "vite"] {
        let source = root.join("node_modules").join(package);
        if source.is_dir() {
            copy_declarations(&source, &inputs.join("node_modules").join(package))?;
        }
    }
    Ok(())
}

fn copy_inputs(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let output = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_inputs(&entry.path(), &output)?;
        } else if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), output)?;
        }
    }
    Ok(())
}

fn copy_declarations(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let output = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_declarations(&entry.path(), &output)?;
        } else if entry.file_type()?.is_file() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == "package.json"
                || name.ends_with(".d.ts")
                || name.ends_with(".d.mts")
                || name.ends_with(".d.cts")
            {
                std::fs::copy(entry.path(), output)?;
            }
        }
    }
    Ok(())
}
