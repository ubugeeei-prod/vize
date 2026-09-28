//! Page files owned by a single `vize dump --dump-dir` invocation.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_l0::dump::capture::{CaptureOutcome, StageCapture};

use super::{fail, required_field, write_json};

pub(super) fn write_pages(
    dir: &Path,
    capture: &StageCapture,
    feed: &Value,
    only_changed: bool,
    source: &Path,
) {
    std::fs::create_dir_all(dir)
        .unwrap_or_else(|error| fail(source, &format!("cannot create {}: {error}", dir.display())));
    remove_previous_pages(dir, source);
    let mut previous = None;
    for (index, page) in capture
        .pages
        .iter()
        .filter(|_| matches!(capture.outcome, CaptureOutcome::Accepted))
        .enumerate()
    {
        let changed = previous != Some(page.text.as_str());
        previous = Some(page.text.as_str());
        if only_changed && !changed {
            continue;
        }
        let file = dir.join(page_filename(index, page.level.id(), page.step, source));
        match std::fs::symlink_metadata(&file) {
            Ok(_) => fail(source, &format!("refusing to overwrite {}", file.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => fail(
                source,
                &format!("cannot inspect {}: {error}", file.display()),
            ),
        }
        std::fs::write(&file, page.text.as_bytes()).unwrap_or_else(|error| {
            fail(source, &format!("cannot write {}: {error}", file.display()))
        });
    }
    write_json(&dir.join("product-stage-feed.json"), feed, source);
}

fn remove_previous_pages(dir: &Path, source: &Path) {
    let feed_path = dir.join("product-stage-feed.json");
    let old_feed = match std::fs::read_to_string(&feed_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => fail(
            source,
            &format!("cannot read {}: {error}", feed_path.display()),
        ),
    };
    let value: Value = serde_json::from_str(&old_feed)
        .unwrap_or_else(|error| fail(source, &format!("invalid {}: {error}", feed_path.display())));
    if value.get("schema_version").and_then(Value::as_u64) != Some(2)
        || value.get("command").and_then(Value::as_str) != Some("vize-dump")
    {
        fail(
            source,
            "existing product-stage-feed.json is not a vize dump feed",
        );
    }
    let pages = required_field(&value, "pages", source)
        .as_array()
        .unwrap_or_else(|| fail(source, "existing product-stage-feed.json has invalid pages"));
    let mut owned = Vec::<PathBuf>::new();
    for (index, page) in pages.iter().enumerate() {
        let level = required_field(page, "level", source)
            .as_str()
            .unwrap_or_else(|| fail(source, "existing page has invalid level"));
        let step = required_field(page, "step", source)
            .as_str()
            .unwrap_or_else(|| fail(source, "existing page has invalid step"));
        let text = required_field(page, "text", source)
            .as_str()
            .unwrap_or_else(|| fail(source, "existing page has invalid text"));
        let file = dir.join(page_filename(index, level, step, source));
        match std::fs::read(&file) {
            Ok(bytes) if bytes == text.as_bytes() => owned.push(file),
            Ok(_) => fail(
                source,
                &format!("refusing to remove modified {}", file.display()),
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => fail(source, &format!("cannot read {}: {error}", file.display())),
        }
    }
    for file in owned {
        std::fs::remove_file(&file).unwrap_or_else(|error| {
            fail(
                source,
                &format!("cannot remove {}: {error}", file.display()),
            )
        });
    }
    std::fs::remove_file(&feed_path).unwrap_or_else(|error| {
        fail(
            source,
            &format!("cannot remove {}: {error}", feed_path.display()),
        )
    });
}

fn page_filename(index: usize, level: &str, step: &str, source: &Path) -> String {
    if !matches!(level, "l0" | "l1" | "l2" | "l3" | "l4") {
        fail(source, "capture page has invalid level");
    }
    let safe_step = step.replace(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-', "_");
    format!("{index:03}-{level}.{safe_step}.dump")
}
