//! Corsa 7.0.2 keeps unchanged files' recorded diagnostics when a file that
//! affects the global scope joins the program or changes; TypeScript 6.0.3
//! rechecks them. The build info records those files and their versions, so a
//! run can tell when the diagnostics it reused may be stale.

use std::path::Path;

use serde::Deserialize;
use serde::de::IgnoredAny;
use vize_carton::String;

/// Build info bytes read before a run, compared with the bytes it leaves.
pub(super) struct RecordedBuildInfo(Vec<u8>);

impl RecordedBuildInfo {
    pub(super) fn read(path: &Path) -> Option<Self> {
        std::fs::read(path).ok().map(Self)
    }

    /// Whether the run recorded a different set or version of global-scope
    /// files. Corsa leaves the file untouched when nothing changed.
    pub(super) fn global_scope_changed(&self, path: &Path) -> bool {
        let Ok(current) = std::fs::read(path) else {
            return false;
        };
        if current == self.0 {
            return false;
        }
        match (global_scope_files(&self.0), global_scope_files(&current)) {
            (Some(recorded), Some(current)) => recorded != current,
            // Unreadable recorded state cannot vouch for the reused diagnostics.
            (None, Some(_)) => true,
            (_, None) => false,
        }
    }
}

#[derive(Deserialize)]
struct BuildInfo {
    #[serde(rename = "fileNames", default)]
    file_names: Vec<String>,
    #[serde(rename = "fileInfos", default)]
    file_infos: Vec<FileInfo>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FileInfo {
    Detailed {
        version: String,
        #[serde(rename = "affectsGlobalScope", default)]
        affects_global_scope: bool,
    },
    // A bare version string: the file does not affect the global scope.
    Version(IgnoredAny),
}

/// Sorted `(file, version)` pairs of the files that affect the global scope.
fn global_scope_files(bytes: &[u8]) -> Option<Vec<(String, String)>> {
    let info: BuildInfo = serde_json::from_slice(bytes).ok()?;
    if info.file_names.len() != info.file_infos.len() {
        return None;
    }
    let mut files = info
        .file_names
        .into_iter()
        .zip(info.file_infos)
        .filter_map(|(name, info)| match info {
            FileInfo::Detailed {
                version,
                affects_global_scope: true,
            } => Some((name, version)),
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort_unstable();
    Some(files)
}

#[cfg(test)]
mod tests {
    use super::{RecordedBuildInfo, global_scope_files};
    use vize_carton::String;

    fn build_info(globals: &str, module_version: &str) -> String {
        let mut text = String::from(
            r#"{"version":"7.0.2","fileNames":["lib.es5.d.ts","./src/globals.ts","./src/a.ts"],"fileInfos":[{"version":"l1","affectsGlobalScope":true},"#,
        );
        text.push_str(globals);
        text.push(',');
        text.push_str(module_version);
        text.push_str(r#"],"semanticDiagnosticsPerFile":[3]}"#);
        text
    }

    #[test]
    fn global_scope_files_keep_only_flagged_entries() {
        let text = build_info(
            r#"{"version":"g1","signature":"s","affectsGlobalScope":true}"#,
            r#""a1""#,
        );
        assert_eq!(
            global_scope_files(text.as_bytes()),
            Some(vec![
                ("./src/globals.ts".into(), "g1".into()),
                ("lib.es5.d.ts".into(), "l1".into()),
            ])
        );
        let misaligned = r#"{"fileNames":["a.ts"],"fileInfos":[]}"#;
        assert_eq!(global_scope_files(misaligned.as_bytes()), None);
    }

    #[test]
    fn only_global_scope_changes_invalidate_reused_diagnostics() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("out.tsbuildinfo");
        let initial = build_info(r#""g0""#, r#""a1""#);
        std::fs::write(&path, &initial).unwrap();
        let recorded = RecordedBuildInfo::read(&path).unwrap();
        assert!(!recorded.global_scope_changed(&path));
        // An ordinary module edit keeps Corsa's own invalidation.
        std::fs::write(&path, build_info(r#""g0""#, r#""a2""#)).unwrap();
        assert!(!recorded.global_scope_changed(&path));
        // A newly imported module that adds `declare global`.
        let joined = build_info(r#"{"version":"g0","affectsGlobalScope":true}"#, r#""a2""#);
        std::fs::write(&path, joined).unwrap();
        assert!(recorded.global_scope_changed(&path));
        // An edited global declaration.
        let recorded = RecordedBuildInfo::read(&path).unwrap();
        let edited = build_info(r#"{"version":"g1","affectsGlobalScope":true}"#, r#""a2""#);
        std::fs::write(&path, edited).unwrap();
        assert!(recorded.global_scope_changed(&path));
        // A run that wrote no build info has nothing to compare.
        std::fs::remove_file(&path).unwrap();
        assert!(!recorded.global_scope_changed(&path));
    }
}
