use super::walk::{glob_match, is_declaration_file, parse_jsonc, slash_path, walk_glob};
use super::{
    Path, PathBuf, String, ToCompactString, TsconfigOwnershipCache, TsconfigSourceKind, Value,
};
use rayon::prelude::*;

mod augmentation;

use augmentation::is_module_augmentation;

pub(super) fn ambient_program_files(project_root: &Path, tsconfig: &Path) -> Vec<String> {
    let selection = configured_program_files(tsconfig);
    let spelling = ProgramRootSpelling::new(project_root);
    let mut files = selection
        .declarations
        .iter()
        .map(|path| spelling.file_name(path))
        .collect::<Vec<_>>();
    let mut sources = selection.source_candidates;
    if selection.needs_authoritative_source_scan {
        sources.extend(owned_typescript_sources(project_root, tsconfig));
    }
    sources.sort();
    sources.dedup();
    let augmentations = sources
        .into_par_iter()
        .filter_map(|path| {
            if !is_typescript_source(&path) {
                return None;
            }
            let content = std::fs::read_to_string(&path).ok()?;
            is_module_augmentation(&content).then(|| spelling.file_name(&path))
        })
        .collect::<Vec<_>>();
    files.extend(augmentations);
    files.sort();
    files.dedup();
    files
}

/// The root-file spelling the session program must share with the module
/// graph that the linted SFC reaches (#13077).
///
/// That graph is spelled from the session's project root, which comes from
/// the current directory as the OS reports it, and TypeScript resolves an
/// SFC's `./registry` under that spelling. A root file spelled differently is
/// a different file to Corsa, so its `declare module "./registry"` augments
/// another module identity. On Windows `std::fs::canonicalize` differs from
/// the working-directory spelling twice over: it answers with a `\\?\`
/// verbatim path (read by Corsa as the UNC root `//?/C:/`), and it expands
/// 8.3 short names and junctions that the current directory keeps. Files
/// inside the project are therefore spelled from the session root; only files
/// outside it keep their non-verbatim canonical spelling.
struct ProgramRootSpelling<'a> {
    project_root: &'a Path,
    canonical_root: PathBuf,
}

impl<'a> ProgramRootSpelling<'a> {
    fn new(project_root: &'a Path) -> Self {
        Self {
            project_root,
            canonical_root: vize_l0::path::canonicalize_non_verbatim(project_root),
        }
    }

    fn file_name(&self, path: &Path) -> String {
        let canonical = vize_l0::path::canonicalize_non_verbatim(path);
        match canonical.strip_prefix(&self.canonical_root) {
            Ok(relative) => slash_path(&self.project_root.join(relative)),
            Err(_) => slash_path(&canonical),
        }
    }
}

fn owned_typescript_sources(project_root: &Path, tsconfig: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for pattern in ["*.ts", "*.tsx", "*.mts", "*.cts"] {
        walk_glob(project_root, &["**", pattern], &mut candidates);
    }
    candidates.sort();
    candidates.dedup();
    let mut ownership = TsconfigOwnershipCache::default();
    candidates
        .into_iter()
        .filter(|path| ownership.project_owns_source(tsconfig, path, TsconfigSourceKind::Typed))
        .collect()
}

fn is_typescript_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "mts" | "cts"))
}

struct ProgramFileSelection {
    declarations: Vec<PathBuf>,
    source_candidates: Vec<PathBuf>,
    needs_authoritative_source_scan: bool,
}

fn configured_program_files(tsconfig: &Path) -> ProgramFileSelection {
    let empty = || ProgramFileSelection {
        declarations: Vec::new(),
        source_candidates: Vec::new(),
        needs_authoritative_source_scan: false,
    };
    let Some(base) = tsconfig.parent() else {
        return empty();
    };
    let Ok(content) = std::fs::read_to_string(tsconfig) else {
        return empty();
    };
    let Some(value) = parse_jsonc(&content) else {
        return empty();
    };
    let excludes = exclude_patterns(&value);
    let mut matched = Vec::new();
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        for entry in files.iter().filter_map(Value::as_str) {
            let path = resolve_config_path(base, entry);
            if path.is_file() {
                matched.push(path);
            }
        }
    }
    let mut include_needs_scan = false;
    if let Some(include) = value.get("include").and_then(Value::as_array) {
        for entry in include.iter().filter_map(Value::as_str) {
            include_needs_scan |= !entry.contains('*')
                && !entry.contains('?')
                && resolve_config_path(base, entry).is_dir();
            collect_include_entry(base, entry, &mut matched);
        }
    }
    if value.get("files").is_none() && value.get("include").is_none() {
        walk_glob(base, &["**", "*.d.ts"], &mut matched);
    }

    let mut declarations = Vec::new();
    let mut source_candidates = Vec::new();
    for path in matched {
        let relative = path.strip_prefix(base).unwrap_or(path.as_path());
        let relative = slash_path(relative);
        if is_excluded(relative.as_str(), &excludes) {
            continue;
        }
        if is_declaration_file(&path) {
            declarations.push(path);
        } else if is_typescript_source(&path) {
            source_candidates.push(path);
        }
    }
    declarations.sort();
    declarations.dedup();
    source_candidates.sort();
    source_candidates.dedup();
    ProgramFileSelection {
        declarations,
        source_candidates,
        needs_authoritative_source_scan: value.get("extends").is_some()
            || (value.get("files").is_none() && value.get("include").is_none())
            || include_needs_scan,
    }
}

#[cfg(test)]
mod tests {
    use super::{String, ambient_program_files};
    use std::{fs, path::Path};

    /// Program files are absolute and spelled from the session project root
    /// exactly as given, never canonicalized away from it (a macOS temp dir
    /// sits behind the `/var` symlink, a Windows one behind an 8.3 name) and
    /// never from a Windows `\\?\` verbatim root (#13077).
    fn relative_to(root: &Path, files: &[String]) -> Vec<String> {
        let prefix = root.to_string_lossy().replace('\\', "/");
        files
            .iter()
            .map(|path| {
                path.as_str()
                    .strip_prefix(prefix.as_str())
                    .and_then(|relative| relative.strip_prefix('/'))
                    .unwrap_or_else(|| panic!("{path} is not under {prefix}"))
                    .into()
            })
            .collect()
    }

    fn write(root: &Path, path: &str, content: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn project_program_adds_only_selected_external_module_augmentations() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "tsconfig.json",
            r#"{"include":["src/**/*.ts"],"exclude":["src/excluded.ts"]}"#,
        );
        write(
            root.path(),
            "src/env.d.ts",
            "declare const build: string;\n",
        );
        write(
            root.path(),
            "src/augment.ts",
            "import './registry';\ndeclare module './registry' { interface Registry { extra: true } }\n",
        );
        write(
            root.path(),
            "src/global.ts",
            "export {};\ndeclare global { interface Window { widget: true } }\n",
        );
        write(
            root.path(),
            "src/comment.ts",
            "export const text = 'declare module not-a-declaration';\n",
        );
        write(
            root.path(),
            "src/script.ts",
            "declare module 'ambient-but-not-an-augmentation' {}\n",
        );
        write(
            root.path(),
            "src/excluded.ts",
            "export {}; declare global { interface Window { excluded: true } }\n",
        );

        let files = ambient_program_files(root.path(), &root.path().join("tsconfig.json"));
        let relative = relative_to(root.path(), &files);
        assert_eq!(
            relative,
            vec!["src/augment.ts", "src/env.d.ts", "src/global.ts"]
        );
    }

    #[test]
    fn inherited_program_membership_uses_the_tsconfig_ownership_authority() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "base.json",
            r#"{"include":["src/**/*.ts"],"exclude":["src/excluded.ts"]}"#,
        );
        write(root.path(), "tsconfig.json", r#"{"extends":"./base.json"}"#);
        write(
            root.path(),
            "src/augment.ts",
            "export {}; declare global { interface Window { inherited: true } }\n",
        );
        write(
            root.path(),
            "src/excluded.ts",
            "export {}; declare global { interface Window { excluded: true } }\n",
        );

        let files = ambient_program_files(root.path(), &root.path().join("tsconfig.json"));
        let relative = relative_to(root.path(), &files);
        assert_eq!(relative, vec!["src/augment.ts"]);
    }

    /// The Windows split of #13077, modelled with a symlinked project root:
    /// the session reaches the project through the link, so the included
    /// augmentation must be spelled through the link too, not through the
    /// canonical target (where Corsa would see a second `registry.ts`).
    #[cfg(unix)]
    #[test]
    fn program_roots_keep_the_session_project_root_spelling() {
        let temp = tempfile::tempdir().unwrap();
        let real = temp.path().join("real");
        write(&real, "tsconfig.json", r#"{"include":["src/**/*.ts"]}"#);
        write(&real, "src/env.d.ts", "declare const build: string;\n");
        write(
            &real,
            "src/augment.ts",
            "import './registry';\ndeclare module './registry' { interface Registry { extra: true } }\n",
        );
        let linked = temp.path().join("linked");
        std::os::unix::fs::symlink(&real, &linked).unwrap();

        let files = ambient_program_files(&linked, &linked.join("tsconfig.json"));

        assert_eq!(
            relative_to(&linked, &files),
            vec!["src/augment.ts", "src/env.d.ts"]
        );
    }
}

fn exclude_patterns(value: &Value) -> Vec<String> {
    match value.get("exclude") {
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(Value::as_str)
            .map(ToCompactString::to_compact_string)
            .collect(),
        Some(_) => Vec::new(),
        None => ["node_modules", "bower_components", "jspm_packages"]
            .into_iter()
            .map(ToCompactString::to_compact_string)
            .collect(),
    }
}

fn is_excluded(relative: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let mut prefix = relative;
    loop {
        if patterns
            .iter()
            .any(|pattern| glob_match(pattern.as_str(), prefix))
        {
            return true;
        }
        let Some((parent, _)) = prefix.rsplit_once('/') else {
            return false;
        };
        prefix = parent;
    }
}

fn collect_include_entry(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    if pattern.contains('*') || pattern.contains('?') {
        collect_glob(base, pattern, out);
        return;
    }
    let path = resolve_config_path(base, pattern);
    if path.is_file() {
        out.push(path);
    } else if path.is_dir() {
        walk_glob(&path, &["**", "*.d.ts"], out);
    }
}

fn resolve_config_path(base: &Path, pattern: &str) -> PathBuf {
    let pattern = pattern.trim_start_matches("./");
    let path = Path::new(pattern);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(pattern)
    }
}

fn collect_glob(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let pattern = pattern.trim_start_matches("./");
    let absolute = Path::new(pattern).is_absolute();
    let mut root = if absolute {
        PathBuf::from("/")
    } else {
        base.to_path_buf()
    };
    let raw_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let mut index = 0usize;
    if absolute {
        while raw_parts
            .get(index)
            .is_some_and(|part| !part.contains('*') && !part.contains('?'))
        {
            let Some(part) = raw_parts.get(index) else {
                break;
            };
            root.push(part);
            index += 1;
        }
    }
    let Some(parts) = raw_parts.get(index..) else {
        return;
    };
    if parts.is_empty() {
        if root.is_file() {
            out.push(root);
        }
        return;
    }
    walk_glob(&root, parts, out);
}
