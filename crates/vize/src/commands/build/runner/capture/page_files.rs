//! Ownership checks for page files from one observed build source.

use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_l0::String;
use vize_l0::cstr;
use vize_l0::dump::capture::{CaptureOutcome, StageCapture};

/// Clear a previous owned capture before compiling. A failed compile must
/// never leave the prior accepted feed beside its fallback module.
pub(super) fn prepare(root: &Path, relative_source: &Path) -> Result<PathBuf, String> {
    ensure_directory(root)?;
    let mut dir = root.to_path_buf();
    for component in relative_source.components() {
        let Component::Normal(name) = component else {
            return Err(cstr!(
                "--dump-dir: invalid source path {}",
                relative_source.display()
            ));
        };
        dir.push(name);
        ensure_directory(&dir)?;
    }
    remove_previous(&dir, relative_source)?;
    Ok(dir)
}

fn ensure_directory(dir: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(dir) {
        Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(cstr!(
            "--dump-dir: refusing non-directory or symlink {}",
            dir.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(dir)
                .map_err(|error| cstr!("--dump-dir: cannot create {}: {error}", dir.display()))?;
            let meta = std::fs::symlink_metadata(dir)
                .map_err(|error| cstr!("--dump-dir: cannot inspect {}: {error}", dir.display()))?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                Ok(())
            } else {
                Err(cstr!(
                    "--dump-dir: refusing non-directory or symlink {}",
                    dir.display()
                ))
            }
        }
        Err(error) => Err(cstr!(
            "--dump-dir: cannot inspect {}: {error}",
            dir.display()
        )),
    }
}

fn remove_previous(dir: &Path, relative_source: &Path) -> Result<(), String> {
    let feed_path = dir.join("stages.json");
    match std::fs::symlink_metadata(&feed_path) {
        Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(cstr!(
                "--dump-dir: refusing non-file or symlink {}",
                feed_path.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(cstr!(
                "--dump-dir: cannot inspect {}: {error}",
                feed_path.display()
            ));
        }
    }
    let bytes = std::fs::read(&feed_path)
        .map_err(|error| cstr!("--dump-dir: cannot read {}: {error}", feed_path.display()))?;
    let feed: Value = serde_json::from_slice(&bytes)
        .map_err(|error| cstr!("--dump-dir: invalid {}: {error}", feed_path.display()))?;
    if feed["schema_version"].as_u64() != Some(2)
        || feed["command"].as_str() != Some("vize-build")
        || feed["source"]["path"].as_str() != Some(relative_source.to_string_lossy().as_ref())
    {
        return Err(cstr!(
            "--dump-dir: existing {} is not this source's vize-build feed",
            feed_path.display()
        ));
    }
    let pages = feed["pages"]
        .as_array()
        .ok_or_else(|| cstr!("--dump-dir: invalid pages in {}", feed_path.display()))?;
    let mut owned = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let level = page["level"]
            .as_str()
            .ok_or_else(|| cstr!("--dump-dir: invalid page level in {}", feed_path.display()))?;
        let step = page["step"]
            .as_str()
            .ok_or_else(|| cstr!("--dump-dir: invalid page step in {}", feed_path.display()))?;
        let text = page["text"]
            .as_str()
            .ok_or_else(|| cstr!("--dump-dir: invalid page text in {}", feed_path.display()))?;
        let file = dir.join(page_filename(index, level, step)?);
        match std::fs::symlink_metadata(&file) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {
                let bytes = std::fs::read(&file).map_err(|error| {
                    cstr!("--dump-dir: cannot read {}: {error}", file.display())
                })?;
                if bytes != text.as_bytes() {
                    return Err(cstr!("--dump-dir: refusing modified {}", file.display()));
                }
                owned.push(file);
            }
            Ok(_) => {
                return Err(cstr!(
                    "--dump-dir: refusing non-file or symlink {}",
                    file.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(cstr!(
                    "--dump-dir: cannot inspect {}: {error}",
                    file.display()
                ));
            }
        }
    }
    for file in owned {
        std::fs::remove_file(&file)
            .map_err(|error| cstr!("--dump-dir: cannot remove {}: {error}", file.display()))?;
    }
    std::fs::remove_file(&feed_path)
        .map_err(|error| cstr!("--dump-dir: cannot remove {}: {error}", feed_path.display()))
}

pub(super) fn write_pages(
    dir: &Path,
    capture: &StageCapture,
    only_changed: bool,
) -> Result<(), String> {
    if !matches!(capture.outcome, CaptureOutcome::Accepted) {
        return Ok(());
    }
    let mut previous = None;
    let mut pages = Vec::new();
    for (index, page) in capture.pages.iter().enumerate() {
        let changed = previous != Some(page.text.as_str());
        previous = Some(page.text.as_str());
        if only_changed && !changed {
            continue;
        }
        let file = dir.join(page_filename(index, page.level.id(), page.step)?);
        match std::fs::symlink_metadata(&file) {
            Ok(_) => {
                return Err(cstr!(
                    "--dump-dir: refusing to overwrite {}",
                    file.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(cstr!(
                    "--dump-dir: cannot inspect {}: {error}",
                    file.display()
                ));
            }
        }
        pages.push((file, page.text.as_bytes()));
    }
    for (file, bytes) in pages {
        write_new(&file, bytes)?;
    }
    Ok(())
}

pub(super) fn write_feed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_new(path, bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            cstr!(
                "--dump-dir: refusing to overwrite or cannot create {}: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes)
        .map_err(|error| cstr!("--dump-dir: cannot write {}: {error}", path.display()))
}

fn page_filename(index: usize, level: &str, step: &str) -> Result<PathBuf, String> {
    if !matches!(level, "l0" | "l1" | "l2" | "l3" | "l4") {
        return Err(cstr!("--dump-dir: invalid capture page level"));
    }
    let safe_step = step.replace(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-', "_");
    Ok(PathBuf::from(cstr!("{index:03}-{level}.{safe_step}.dump")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::dump::capture::StageCapturePage;
    use vize_l0::level::Level;

    #[test]
    fn after_change_filters_files_but_preserves_all_executed_feed_pages() {
        let root = std::env::temp_dir().join(format!(
            "vize-build-dump-page-filter-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let source = Path::new("a.vue");
        let dir = prepare(&root, source).unwrap();
        let mut stages = StageCapture::new("dom");
        for step in ["parse", "lower"] {
            stages.pages.push(StageCapturePage {
                level: Level::L1,
                step,
                text: String::from("same text"),
            });
        }
        stages.outcome = CaptureOutcome::Accepted;
        let capture = super::super::BuildCapture {
            stages,
            authored_syntax: String::from("html"),
            compiled_syntax: String::from("html"),
            template_span: None,
        };
        super::super::write(&dir, source, capture, true).unwrap();
        let feed: Value =
            serde_json::from_slice(&std::fs::read(dir.join("stages.json")).unwrap()).unwrap();
        assert_eq!(feed["pages"].as_array().unwrap().len(), 2);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        std::fs::remove_dir_all(root).unwrap();
    }
}
