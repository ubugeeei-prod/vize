//! `RenderTree` — cross-file component edges (P4-3b).
//!
//! The key is the caller file plus the component file the module graph
//! resolves, and the name that file exports. A local alias (`Foo as Bar`)
//! is not the key. Recomputing the table drops edges the current files no
//! longer support.

use std::collections::BTreeMap;
use std::path::Path;

use vize_carton::CompactString;
use vize_croquis::facts::{FactGroup, component_identity};
use vize_davinci::fact::{Demand, FactProducer, FactTable, FactTableBuilder, FactView};
use vize_davinci::pass::AnalysisId;

use crate::module_paths::import_candidates;
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};

/// Caller file, resolved component file, and the name that file exports.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderEdgeKey {
    pub caller: FileId,
    pub target: FileId,
    pub export_name: CompactString,
}

/// One template use of that component, in the caller's template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSite {
    pub start: u32,
    pub end: u32,
}

/// The project render tree.
pub struct RenderTree;

impl FactGroup for RenderTree {
    const ID: AnalysisId = vize_davinci::fact::ids::RENDER_TREE;
    const NAME: &'static str = "render-tree";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = RenderEdgeKey;
    type Value = Vec<RenderSite>;
}

impl FactProducer<ModuleRegistry> for RenderTree {
    fn produce(registry: &ModuleRegistry, _: &FactView<'_>) -> FactTable<Self> {
        evaluate(registry)
    }
}

/// Naive evaluator: one pass, no cache, import path before a unique name.
pub fn evaluate(registry: &ModuleRegistry) -> FactTable<RenderTree> {
    let mut entries: Vec<&ModuleEntry> = registry.iter().collect();
    entries.sort_by_key(|entry| entry.id);
    let mut builder = FactTableBuilder::default();
    for entry in entries {
        for (key, sites) in edges_of(registry, entry) {
            builder.insert(key, sites);
        }
    }
    builder.finish()
}

/// Component-usage targets of `caller`, through the render-tree group.
///
/// The returned order follows the file's component names, which is what the
/// dependency graph recorded before this group existed. The fact table itself
/// stays sorted by [`RenderEdgeKey`].
pub fn component_usage_targets(registry: &ModuleRegistry, caller: FileId) -> Vec<FileId> {
    let Some(entry) = registry.get(caller) else {
        return Vec::new();
    };
    let mut facts = super::ProjectFacts::new(registry);
    let table = facts
        .prepare::<super::RenderTreeReader>()
        .get::<RenderTree>()
        .expect("render tree demand");
    let mut targets = Vec::new();
    for tag in source_tags(entry) {
        let Some((target, export_name)) = resolve_tag(registry, entry, tag.as_str()) else {
            continue;
        };
        let key = RenderEdgeKey {
            caller,
            target,
            export_name,
        };
        if table.contains_key(&key) && !targets.contains(&target) {
            targets.push(target);
        }
    }
    targets
}

fn source_tags(entry: &ModuleEntry) -> Vec<CompactString> {
    let mut tags = Vec::new();
    for usage in vize_croquis::facts::component_usage_list(&entry.analysis) {
        if !tags
            .iter()
            .any(|tag: &CompactString| names_match(tag.as_str(), usage.name.as_str()))
        {
            tags.push(usage.name);
        }
    }
    for name in vize_croquis::facts::used_component_name_list(&entry.analysis) {
        if !tags
            .iter()
            .any(|tag: &CompactString| names_match(tag.as_str(), name.as_str()))
        {
            tags.push(name);
        }
    }
    tags
}

fn edges_of(
    registry: &ModuleRegistry,
    entry: &ModuleEntry,
) -> BTreeMap<RenderEdgeKey, Vec<RenderSite>> {
    let mut grouped = BTreeMap::new();
    for usage in vize_croquis::facts::component_usage_list(&entry.analysis) {
        push_edge(
            registry,
            entry,
            usage.name.as_str(),
            Some(RenderSite {
                start: usage.start,
                end: usage.end,
            }),
            &mut grouped,
        );
    }
    for name in vize_croquis::facts::used_component_name_list(&entry.analysis) {
        let covered = vize_croquis::facts::component_usage_list(&entry.analysis)
            .iter()
            .any(|usage| names_match(usage.name.as_str(), name.as_str()));
        if !covered {
            push_edge(registry, entry, name.as_str(), None, &mut grouped);
        }
    }
    for sites in grouped.values_mut() {
        sites.sort_by_key(|site| (site.start, site.end));
    }
    grouped
}

fn push_edge(
    registry: &ModuleRegistry,
    entry: &ModuleEntry,
    tag: &str,
    site: Option<RenderSite>,
    grouped: &mut BTreeMap<RenderEdgeKey, Vec<RenderSite>>,
) {
    let Some((target, export_name)) = resolve_tag(registry, entry, tag) else {
        return;
    };
    let key = RenderEdgeKey {
        caller: entry.id,
        target,
        export_name,
    };
    grouped.entry(key).or_default().extend(site);
}

/// The file a tag renders through this file's imports, with re-exports
/// followed. A project-wide unique name is not a resolution.
pub fn imported_render_target(
    registry: &ModuleRegistry,
    caller: FileId,
    tag: &str,
) -> Option<FileId> {
    let entry = registry.get(caller)?;
    resolve_imported(registry, entry, tag).map(|(file, _)| file)
}

fn resolve_tag(
    registry: &ModuleRegistry,
    entry: &ModuleEntry,
    tag: &str,
) -> Option<(FileId, CompactString)> {
    resolve_imported(registry, entry, tag).or_else(|| {
        unique_component(registry, tag).map(|target| (target, CompactString::new("default")))
    })
}

fn resolve_imported(
    registry: &ModuleRegistry,
    entry: &ModuleEntry,
    tag: &str,
) -> Option<(FileId, CompactString)> {
    let identity = component_identity(&entry.analysis, tag);
    let module = identity.module.as_deref()?;
    let resolved = resolve_module(registry, module, entry.path.parent())?;
    follow_forwards(registry, resolved, identity.export_name, 0)
}

fn follow_forwards(
    registry: &ModuleRegistry,
    file: FileId,
    export_name: CompactString,
    depth: u8,
) -> Option<(FileId, CompactString)> {
    if depth == 8 {
        return Some((file, export_name));
    }
    let Some(entry) = registry.get(file) else {
        return Some((file, export_name));
    };
    let Some(forward) = entry
        .analysis
        .re_export_forwards
        .iter()
        .find(|forward| forward.exported.as_str() == export_name.as_str())
    else {
        return Some((file, export_name));
    };
    let Some(next) = resolve_module(registry, forward.source.as_str(), entry.path.parent()) else {
        return Some((file, export_name));
    };
    if next == file {
        return Some((file, export_name));
    }
    follow_forwards(registry, next, forward.imported.clone(), depth + 1)
}

pub(crate) fn resolve_module(
    registry: &ModuleRegistry,
    specifier: &str,
    from_dir: Option<&Path>,
) -> Option<FileId> {
    for candidate in import_candidates(specifier, from_dir) {
        if let Some(entry) = registry.get_by_path(&candidate) {
            return Some(entry.id);
        }
    }
    if is_relative(specifier) {
        return None;
    }
    let basename = specifier
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())?;
    let mut matches: Vec<&ModuleEntry> = registry
        .iter()
        .filter(|entry| filename_matches(&entry.filename, basename))
        .collect();
    matches.sort_by(|left, right| left.path.cmp(&right.path));
    match matches.as_slice() {
        [only] => Some(only.id),
        _ => None,
    }
}

fn filename_matches(filename: &str, basename: &str) -> bool {
    if filename == basename {
        return true;
    }
    let mut vue = vize_carton::String::from(basename);
    vue.push_str(".vue");
    if filename == vue.as_str() {
        return true;
    }
    let mut ts = vize_carton::String::from(basename);
    ts.push_str(".ts");
    filename == ts.as_str()
}

fn is_relative(specifier: &str) -> bool {
    specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier == "."
        || specifier == ".."
}

fn unique_component(registry: &ModuleRegistry, tag: &str) -> Option<FileId> {
    let mut found = None;
    for entry in registry.iter() {
        let Some(name) = component_label(entry) else {
            continue;
        };
        if names_match(name, tag) {
            if found.is_some() {
                return None;
            }
            found = Some(entry.id);
        }
    }
    found
}

fn component_label(entry: &ModuleEntry) -> Option<&str> {
    entry
        .component_name
        .as_deref()
        .or_else(|| entry.filename.strip_suffix(".vue"))
}

fn names_match(left: &str, right: &str) -> bool {
    left == right
        || vize_croquis::naming::to_pascal_case(left) == vize_croquis::naming::to_pascal_case(right)
}
