//! Shard edges for tsconfig `paths` aliases (#10360).
//!
//! A shared prefix such as `@/*` is not an import edge. Importers couple only
//! when they load the same module: a partitioned file when the target is in
//! the program, otherwise one key per substituted module. A `"*"` pattern
//! trims to an empty prefix and must not match every specifier through
//! `starts_with("")`; a miss stays uncoupled. Targets under `node_modules`
//! stay dependency cost.

use std::path::{Component, Path, PathBuf};

use super::{VirtualProject, resolve_virtual_import};
use vize_carton::{FxHashMap, String as CompactString, ToCompactString, cstr};

const EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts", ".vue", ".js", ".jsx", ".mjs",
    ".cjs",
];

const INDEX_FILES: &[&str] = &[
    "index.ts",
    "index.tsx",
    "index.mts",
    "index.cts",
    "index.d.ts",
    "index.vue",
    "index.js",
    "index.jsx",
    "index.mjs",
    "index.cjs",
];

pub(super) enum AliasEdge {
    /// Partitioned file this specifier resolves to.
    File(usize),
    /// Importers of one missing first-party module. Not the paths prefix.
    Module(CompactString),
}

pub(super) struct AliasMap {
    patterns: Vec<AliasPattern>,
}

struct AliasPattern {
    key: CompactString,
    exact: bool,
    prefix: CompactString,
    suffix: CompactString,
    targets: Vec<AliasTarget>,
}

struct AliasTarget {
    before: CompactString,
    after: CompactString,
    wildcard: bool,
    node_modules: bool,
}

impl AliasMap {
    pub(super) fn from_project(project: &VirtualProject) -> Self {
        let mut patterns: Vec<AliasPattern> = Vec::new();
        for (pattern, target) in project.dependency_alias_map() {
            let pattern = pattern.as_str();
            let target = target.as_str();
            let position = patterns
                .iter()
                .position(|candidate| candidate.key == pattern);
            if let Some(position) = position
                && let Some(existing) = patterns.get_mut(position)
            {
                existing.targets.push(AliasTarget::parse(target));
                continue;
            }
            let (exact, prefix, suffix) = split_pattern(pattern);
            patterns.push(AliasPattern {
                key: pattern.to_compact_string(),
                exact,
                prefix,
                suffix,
                targets: vec![AliasTarget::parse(target)],
            });
        }
        Self { patterns }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub(super) fn edge(
        &self,
        specifier: &str,
        project: &VirtualProject,
        index_by_virtual: &FxHashMap<&Path, usize>,
        index_by_original: &FxHashMap<&Path, usize>,
    ) -> Option<AliasEdge> {
        let (pattern, capture) = self.select(specifier)?;
        if pattern.dependency_only() {
            return None;
        }
        let mut module_key = None;
        for target in &pattern.targets {
            if target.node_modules {
                continue;
            }
            let Some(substituted) = target.substitute(capture) else {
                continue;
            };
            if path_has_node_modules(substituted.as_str()) {
                continue;
            }
            let absolute = absolute_target(project, substituted.as_str());
            if let Some(index) =
                resolve_partition_file(project, &absolute, index_by_virtual, index_by_original)
            {
                return Some(AliasEdge::File(index));
            }
            if module_key.is_none() {
                module_key = Some(cstr!("{}", absolute.display()));
            }
        }
        // An empty `"*"` prefix matches every specifier. Couple a real file
        // above, but do not invent a shared key for `vue` and the rest.
        if pattern.empty_wildcard() {
            return None;
        }
        module_key.map(AliasEdge::Module)
    }

    fn select<'a>(&'a self, specifier: &'a str) -> Option<(&'a AliasPattern, &'a str)> {
        for pattern in &self.patterns {
            if pattern.exact
                && let Some(capture) = pattern.capture(specifier)
            {
                return Some((pattern, capture));
            }
        }
        let mut best: Option<(&AliasPattern, &str, usize)> = None;
        for pattern in &self.patterns {
            if pattern.exact {
                continue;
            }
            let Some(capture) = pattern.capture(specifier) else {
                continue;
            };
            if best.is_none_or(|(_, _, length)| pattern.prefix.len() > length) {
                best = Some((pattern, capture, pattern.prefix.len()));
            }
        }
        best.map(|(pattern, capture, _)| (pattern, capture))
    }
}

impl AliasPattern {
    fn dependency_only(&self) -> bool {
        !self.targets.is_empty() && self.targets.iter().all(|target| target.node_modules)
    }

    fn empty_wildcard(&self) -> bool {
        !self.exact && self.prefix.is_empty()
    }

    fn capture<'a>(&self, specifier: &'a str) -> Option<&'a str> {
        if self.suffix.contains('*') {
            return None;
        }
        if self.exact {
            return (specifier == self.prefix.as_str()).then_some("");
        }
        specifier
            .strip_prefix(self.prefix.as_str())
            .and_then(|rest| rest.strip_suffix(self.suffix.as_str()))
    }
}

impl AliasTarget {
    fn parse(target: &str) -> Self {
        let node_modules = path_has_node_modules(target);
        let Some((before, after)) = target.split_once('*') else {
            return Self {
                before: target.to_compact_string(),
                after: CompactString::default(),
                wildcard: false,
                node_modules,
            };
        };
        Self {
            before: before.to_compact_string(),
            after: after.to_compact_string(),
            wildcard: true,
            node_modules,
        }
    }

    fn substitute(&self, capture: &str) -> Option<CompactString> {
        if !self.wildcard {
            return Some(self.before.clone());
        }
        if self.after.contains('*') {
            return None;
        }
        Some(cstr!("{}{capture}{}", self.before, self.after))
    }
}

fn split_pattern(pattern: &str) -> (bool, CompactString, CompactString) {
    match pattern.split_once('*') {
        Some((prefix, suffix)) => (
            false,
            prefix.to_compact_string(),
            suffix.to_compact_string(),
        ),
        None => (true, pattern.to_compact_string(), CompactString::default()),
    }
}

fn path_has_node_modules(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

fn absolute_target(project: &VirtualProject, substituted: &str) -> PathBuf {
    let raw = Path::new(substituted);
    let absolute = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        project.project_root().join(raw)
    };
    lexical_normalize(&absolute)
}

fn resolve_partition_file(
    project: &VirtualProject,
    absolute: &Path,
    index_by_virtual: &FxHashMap<&Path, usize>,
    index_by_original: &FxHashMap<&Path, usize>,
) -> Option<usize> {
    if let Some(index) = probe(absolute, index_by_original) {
        return Some(index);
    }
    let relative = absolute.strip_prefix(project.project_root()).ok()?;
    let virtual_path = project.virtual_root().join(relative);
    if let Some(index) = resolve_virtual_import(&virtual_path, index_by_virtual) {
        return Some(index);
    }
    probe(&virtual_path, index_by_virtual)
}

fn probe(candidate: &Path, index: &FxHashMap<&Path, usize>) -> Option<usize> {
    if let Some(&found) = index.get(candidate) {
        return Some(found);
    }
    if has_known_extension(candidate) {
        return None;
    }
    let target_str = candidate.to_string_lossy();
    for suffix in EXTENSIONS {
        let path = PathBuf::from(cstr!("{target_str}{suffix}").as_str());
        if let Some(&found) = index.get(path.as_path()) {
            return Some(found);
        }
    }
    for name in INDEX_FILES {
        let path = candidate.join(name);
        if let Some(&found) = index.get(path.as_path()) {
            return Some(found);
        }
    }
    for suffix in [".vue.ts", ".vue.tsx"] {
        let path = PathBuf::from(cstr!("{target_str}{suffix}").as_str());
        if let Some(&found) = index.get(path.as_path()) {
            return Some(found);
        }
    }
    None
}

fn has_known_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "ts" | "tsx" | "mts" | "cts" | "vue" | "js" | "jsx" | "mjs" | "cjs" | "json"
            )
        })
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() && !normalized.has_root() {
                    normalized.push("..");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
