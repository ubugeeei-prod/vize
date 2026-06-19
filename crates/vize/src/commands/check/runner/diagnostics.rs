//! Diagnostic reporting, JSON serialization, and profile artifacts for the
//! `check` runner.

use std::{fs, path::Path, path::PathBuf};

use vize_carton::{FxHashSet, String as CompactString, cstr, profile, profiler::global_profiler};

use crate::commands::check::path_cache::CanonicalPathCache;
use crate::commands::check::reporting::JsonOutput;

pub(super) fn emit_json_output(json_output: JsonOutput) {
    match serde_json::to_string_pretty(&json_output) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("Failed to serialize check output: {error}");
            std::process::exit(1);
        }
    }
}

/// Whether a registered file's diagnostics should be reported. Only listed
/// source files are reported; ambient and transitively-registered files exist
/// only to resolve cross-file types. Project-level diagnostics (anchored to a
/// tsconfig or the project root, not a source file) describe the whole check and
/// are always reported.
pub(super) fn is_reported(
    reported: &FxHashSet<PathBuf>,
    path: &Path,
    canonical_paths: &mut CanonicalPathCache,
) -> bool {
    if !is_source_path(path) {
        return true;
    }

    reported.contains(&canonical_paths.canonicalize(path))
}

fn is_source_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx"
            )
        })
}

pub(super) fn is_suppressed_false_positive(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2320)
        && diagnostic
            .message
            .contains("Interface 'ImportMeta' cannot simultaneously extend types")
        && diagnostic.message.contains("NitroStaticBuildFlags")
        && diagnostic.message.contains("NitroImportMeta")
}

#[allow(clippy::disallowed_types)]
pub(super) fn render_diagnostics(
    diagnostics: &[vize_canon::BatchDiagnostic],
) -> std::collections::BTreeMap<std::string::String, Vec<std::string::String>> {
    let mut grouped = std::collections::BTreeMap::<
        std::string::String,
        Vec<(u32, u32, std::string::String)>,
    >::new();

    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            1 => "error",
            2 => "warning",
            3 => "info",
            _ => "hint",
        };
        let code = diagnostic
            .code
            .map(|code| cstr!(" [TS{}]", code))
            .unwrap_or_default();
        let rendered = cstr!(
            "{}:{}:{}{} {}",
            severity,
            diagnostic.line + 1,
            diagnostic.column + 1,
            code,
            diagnostic.message
        )
        .into();
        grouped
            .entry(diagnostic.file.to_string_lossy().into_owned())
            .or_default()
            .push((diagnostic.line, diagnostic.column, rendered));
    }

    grouped
        .into_iter()
        .map(|(file, mut diagnostics)| {
            diagnostics.sort_by(|left, right| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
                    .then_with(|| left.2.cmp(&right.2))
            });
            let rendered = diagnostics
                .into_iter()
                .map(|(_, _, rendered)| rendered)
                .collect();
            (file, rendered)
        })
        .collect()
}

/// Whether a `--save-virtual-ts-for` target names the shared ambient helpers
/// file (`__vize_helpers.d.ts`) rather than a per-`.vue` virtual module.
///
/// Matched purely on the file name so the flag accepts the bare
/// `__vize_helpers.d.ts`, a relative path, or an absolute path interchangeably.
fn is_shared_helpers_target(requested_path: &Path) -> bool {
    requested_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
}

/// Save the generated virtual TypeScript for a single `--save-virtual-ts-for`
/// target.
///
/// A target that names the shared helpers file (`__vize_helpers.d.ts`) writes
/// the program-wide helpers preamble ([`SHARED_PREAMBLE_DTS`]) verbatim at the
/// requested location. Every other target resolves to a generated per-`.vue`
/// virtual module and is written next to its source as `<file>.virtual.ts`,
/// exactly as before.
///
/// [`SHARED_PREAMBLE_DTS`]: vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
pub(super) fn save_virtual_ts_for_path<'a>(
    requested_path: &Path,
    cwd: &Path,
    candidates: impl IntoIterator<Item = (&'a Path, &'a str)>,
) -> Result<PathBuf, CompactString> {
    if is_shared_helpers_target(requested_path) {
        return save_shared_helpers_virtual_ts(requested_path, cwd);
    }

    let requested_path = normalize_requested_virtual_ts_path(cwd, requested_path);
    let Some((original_path, content)) = candidates
        .into_iter()
        .find(|(candidate_path, _)| paths_refer_to_same_file(candidate_path, &requested_path))
    else {
        return Err(cstr!(
            "Virtual TS for {} was not generated",
            requested_path.display()
        ));
    };

    let target = virtual_ts_save_path(original_path)?;
    write_virtual_ts(&target, content)
}

/// Save several `--save-virtual-ts-for` targets in one run, writing each one in
/// turn and reporting every saved path. A failure on any target aborts the run.
pub(super) fn save_virtual_ts_targets<'a, C>(
    requested_paths: &[PathBuf],
    cwd: &Path,
    candidates: impl Fn() -> C,
    quiet: bool,
) where
    C: IntoIterator<Item = (&'a Path, &'a str)>,
{
    for requested_path in requested_paths {
        match save_virtual_ts_for_path(requested_path, cwd, candidates()) {
            Ok(target) => {
                if !quiet {
                    eprintln!("Saved Virtual TS to {}", target.display());
                }
            }
            Err(error) => {
                eprintln!("\x1b[31mError:\x1b[0m {}", error);
                std::process::exit(1);
            }
        }
    }
}

/// Write the shared ambient helpers preamble to the requested location.
fn save_shared_helpers_virtual_ts(
    requested_path: &Path,
    cwd: &Path,
) -> Result<PathBuf, CompactString> {
    let target = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        cwd.join(requested_path)
    };
    write_virtual_ts(&target, vize_canon::virtual_ts::SHARED_PREAMBLE_DTS)
}

fn write_virtual_ts(target: &Path, content: &str) -> Result<PathBuf, CompactString> {
    let bytes = content.len();
    match profile!(
        "cli.check.save_virtual_ts.write",
        fs::write(target, content)
    ) {
        Ok(()) => {
            global_profiler().record_fs_write(bytes);
            Ok(target.to_path_buf())
        }
        Err(error) => {
            global_profiler().record_fs_write_failure(bytes);
            Err(cstr!("Failed to write {}: {}", target.display(), error))
        }
    }
}

fn virtual_ts_save_path(original_path: &Path) -> Result<PathBuf, CompactString> {
    let file_name = original_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            cstr!(
                "Cannot derive Virtual TS output path for {}",
                original_path.display()
            )
        })?;
    let mut target = original_path.to_path_buf();
    target.set_file_name(cstr!("{file_name}.virtual.ts").as_str());
    Ok(target)
}

fn normalize_requested_virtual_ts_path(cwd: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    vize_carton::path::canonicalize_non_verbatim(&absolute)
}

fn paths_refer_to_same_file(candidate_path: &Path, requested_path: &Path) -> bool {
    let candidate_path = vize_carton::path::canonicalize_non_verbatim(candidate_path);
    candidate_path == requested_path
}

pub(super) fn write_profile_virtual_ts(files: &[&vize_canon::VirtualFile]) {
    let profile_dir = PathBuf::from("node_modules/.vize/check-profile");
    if profile_dir.exists() {
        match profile!(
            "cli.check.profile_artifact.remove_dir_all",
            fs::remove_dir_all(&profile_dir)
        ) {
            Ok(()) => global_profiler().record_fs_remove_dir_all(),
            Err(error) => {
                global_profiler().record_fs_remove_dir_all_failure();
                eprintln!(
                    "Failed to clean profile directory {}: {}",
                    profile_dir.display(),
                    error
                );
                return;
            }
        }
    }

    match profile!(
        "cli.check.profile_artifact.create_dir_all",
        fs::create_dir_all(&profile_dir)
    ) {
        Ok(()) => global_profiler().record_fs_create_dir_all(),
        Err(error) => {
            global_profiler().record_fs_create_dir_all_failure();
            eprintln!("Failed to create profile directory: {}", error);
            return;
        }
    }

    for file in files {
        let file_name = file
            .original_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| cstr!("{name}.ts"))
            .unwrap_or_else(|| "unknown.ts".into());
        let target = profile_dir.join(file_name.as_str());
        let bytes = file.content.len();
        match profile!(
            "cli.check.profile_artifact.write",
            fs::write(&target, &file.content)
        ) {
            Ok(()) => global_profiler().record_fs_write(bytes),
            Err(error) => {
                global_profiler().record_fs_write_failure(bytes);
                eprintln!("Failed to write {}: {}", target.display(), error);
            }
        }
    }

    eprintln!(
        "\x1b[33mProfile:\x1b[0m Virtual TS files written to {}",
        profile_dir.display()
    );
}

#[cfg(test)]
mod tests {
    use super::{is_shared_helpers_target, save_virtual_ts_for_path, virtual_ts_save_path};

    #[test]
    fn virtual_ts_save_path_appends_virtual_ts_after_full_file_name() {
        let path = std::path::Path::new("/workspace/src/Hoge.vue");

        assert_eq!(
            virtual_ts_save_path(path).unwrap(),
            std::path::PathBuf::from("/workspace/src/Hoge.vue.virtual.ts")
        );
    }

    #[test]
    fn save_virtual_ts_for_path_writes_matching_canonical_file() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("src").join("Hoge.vue");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "<template />").unwrap();
        let canonical_source = source.canonicalize().unwrap();

        let saved = save_virtual_ts_for_path(
            std::path::Path::new("src/Hoge.vue"),
            temp.path(),
            [(canonical_source.as_path(), "const value = 1;\n")],
        )
        .unwrap();

        assert_eq!(
            saved,
            canonical_source.with_file_name("Hoge.vue.virtual.ts")
        );
        assert_eq!(
            std::fs::read_to_string(saved).unwrap(),
            "const value = 1;\n"
        );
    }

    #[test]
    fn is_shared_helpers_target_matches_helpers_file_name_in_any_form() {
        let bare = std::path::Path::new(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME);
        assert!(is_shared_helpers_target(bare));
        assert!(is_shared_helpers_target(std::path::Path::new(
            "some/nested/dir/__vize_helpers.d.ts"
        )));
        assert!(is_shared_helpers_target(std::path::Path::new(
            "/abs/__vize_helpers.d.ts"
        )));
        assert!(!is_shared_helpers_target(std::path::Path::new(
            "src/App.vue"
        )));
        assert!(!is_shared_helpers_target(std::path::Path::new(
            "__vize_helpers.ts"
        )));
    }

    #[test]
    fn save_virtual_ts_for_path_writes_shared_helpers_preamble() {
        let temp = tempfile::tempdir().unwrap();

        // No `.vue` candidate matches the helpers target; it is served from the
        // shared preamble constant instead of the per-file virtual modules.
        let saved = save_virtual_ts_for_path(
            std::path::Path::new(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME),
            temp.path(),
            std::iter::empty(),
        )
        .unwrap();

        assert_eq!(
            saved,
            temp.path()
                .join(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
        );
        assert_eq!(
            std::fs::read_to_string(saved).unwrap(),
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
        );
    }

    #[test]
    fn save_virtual_ts_for_path_writes_shared_helpers_to_absolute_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("out").join("__vize_helpers.d.ts");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();

        // An unrelated cwd must not redirect an absolute helpers target.
        let saved = save_virtual_ts_for_path(
            &target,
            std::path::Path::new("/nonexistent"),
            std::iter::empty(),
        )
        .unwrap();

        assert_eq!(saved, target);
        assert_eq!(
            std::fs::read_to_string(saved).unwrap(),
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
        );
    }
}
