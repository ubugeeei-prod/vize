//! Host-side dump of the stages from the exact SFC compile that built a file.

use std::path::Path;

use vize_curator::inspector::{ProductCaptureSource, product_capture_value};
use vize_l0::String;
use vize_l0::cstr;
use vize_l0::dump::capture::CaptureOutcome;
use vize_l0::hash::hash_str;

use super::compile::BuildCapture;

pub(super) fn write(
    root: &Path,
    relative_source: &Path,
    mut capture: BuildCapture,
    after_change: bool,
) -> Result<(), String> {
    let dir = root.join(relative_source);
    std::fs::create_dir_all(&dir)
        .map_err(|error| cstr!("--dump-dir: cannot create {}: {error}", dir.display()))?;
    // An earlier run may have selected a native backend or emitted more
    // stages. Clear only files with this command's generated page shape so a
    // later fallback never inherits pages from that earlier run.
    for entry in std::fs::read_dir(&dir)
        .map_err(|error| cstr!("--dump-dir: cannot read {}: {error}", dir.display()))?
    {
        let entry =
            entry.map_err(|error| cstr!("--dump-dir: cannot read {}: {error}", dir.display()))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".dump")
            && name.split_once('-').is_some_and(|(index, _)| {
                !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit())
            })
        {
            std::fs::remove_file(entry.path()).map_err(|error| {
                cstr!(
                    "--dump-dir: cannot remove {}: {error}",
                    entry.path().display()
                )
            })?;
        }
    }

    // Only a selected native backend has pages. Filtering the sidecar also
    // makes the versioned feed describe exactly the files written below.
    if !matches!(capture.stages.outcome, CaptureOutcome::Accepted) {
        capture.stages.pages.clear();
    } else if after_change {
        let mut previous = None;
        capture.stages.pages.retain(|page| {
            let current = hash_str(&page.text);
            let changed = previous != Some(current);
            previous = Some(current);
            changed
        });
    }

    for (index, page) in capture.stages.pages.iter().enumerate() {
        let path = dir.join(cstr!("{index:03}-{}.{}.dump", page.level.id(), page.step));
        std::fs::write(&path, page.text.as_bytes())
            .map_err(|error| cstr!("--dump-dir: cannot write {}: {error}", path.display()))?;
    }

    let source_path = relative_source.to_string_lossy();
    let feed = product_capture_value(
        "vize-build",
        ProductCaptureSource {
            path: Some(&source_path),
            container: "vue-sfc",
            authored_syntax: &capture.authored_syntax,
            compiled_syntax: &capture.compiled_syntax,
            template_span: capture.template_span,
        },
        &capture.stages,
    );
    let path = dir.join("stages.json");
    let bytes = serde_json::to_vec_pretty(&feed)
        .map_err(|error| cstr!("--dump-dir: cannot serialize {}: {error}", path.display()))?;
    std::fs::write(&path, bytes)
        .map_err(|error| cstr!("--dump-dir: cannot write {}: {error}", path.display()))
}
