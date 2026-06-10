//! Diagnostic reporting, JSON serialization, and profile artifacts for the
//! `check` runner.

use std::{fs, path::Path, path::PathBuf};

use vize_carton::{FxHashSet, String as CompactString, cstr, profile, profiler::global_profiler};

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

/// Whether a registered file's diagnostics should be reported. For an explicit
/// subset (`reported` is `Some`), only the requested files are reported; ambient
/// and transitively-registered files exist only to resolve cross-file types.
pub(super) fn is_reported(reported: &Option<FxHashSet<PathBuf>>, path: &Path) -> bool {
    match reported {
        None => true,
        Some(set) => {
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            set.contains(&canonical)
        }
    }
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

pub(super) fn save_virtual_ts_for_path<'a>(
    requested_path: &Path,
    cwd: &Path,
    candidates: impl IntoIterator<Item = (&'a Path, &'a str)>,
) -> Result<PathBuf, CompactString> {
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
    let bytes = content.len();
    match profile!(
        "cli.check.save_virtual_ts.write",
        fs::write(&target, content)
    ) {
        Ok(()) => {
            global_profiler().record_fs_write(bytes);
            Ok(target)
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
    absolute.canonicalize().unwrap_or(absolute)
}

fn paths_refer_to_same_file(candidate_path: &Path, requested_path: &Path) -> bool {
    let candidate_path = candidate_path
        .canonicalize()
        .unwrap_or_else(|_| candidate_path.to_path_buf());
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
    use super::{save_virtual_ts_for_path, virtual_ts_save_path};

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
}
