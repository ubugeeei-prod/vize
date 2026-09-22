//! [`ProjectSources`] — the project-level artifact providers read.
//!
//! One entry per file the host supplies: script modules (the `.ts`/`.js`
//! family) whole, Vue SFCs split into their `<script>` and `<template>`
//! blocks once, at insertion, by the parse-only splitter. Every offset a
//! provider or consumer reports is a **whole-file** byte offset, so a
//! diagnostic points into the file the author edits.

use std::path::{Component, Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashMap, SmallVec};
use vize_croquis::sfc::{SfcParseOptions, parse_sfc_without_css_vars};

/// A module's index in its [`ProjectSources`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u32);

impl ModuleId {
    /// The module's insertion index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// What kind of file a module is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    /// A plain script module.
    Script,
    /// A Vue single-file component.
    Sfc,
}

/// A script body: a whole script module, or one SFC `<script>` block.
#[derive(Debug, Clone, Copy)]
pub struct ScriptBlock<'s> {
    /// The script text.
    pub text: &'s str,
    /// Whole-file offset of `text`'s first byte.
    pub offset: u32,
    /// The OXC source type the text parses as.
    pub source_type: SourceType,
    /// Whether this is an SFC `<script setup>` block.
    pub setup: bool,
}

/// An SFC `<template>` block.
#[derive(Debug, Clone, Copy)]
pub struct TemplateBlock<'s> {
    /// The template text.
    pub text: &'s str,
    /// Whole-file offset of `text`'s first byte.
    pub offset: u32,
}

#[derive(Debug, Clone, Copy)]
struct ScriptRange {
    start: u32,
    end: u32,
    source_type: SourceType,
    setup: bool,
}

/// One file of the project.
#[derive(Debug)]
pub struct ProjectModule {
    path: CompactString,
    key: CompactString,
    source: CompactString,
    kind: ModuleKind,
    scripts: SmallVec<[ScriptRange; 2]>,
    template: Option<(u32, u32)>,
}

impl ProjectModule {
    /// The path as the host supplied it, without a leading `./`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The whole file.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Whether the module is a script module or an SFC.
    #[must_use]
    pub fn kind(&self) -> ModuleKind {
        self.kind
    }

    /// The module's script bodies, in source order.
    pub fn scripts(&self) -> impl Iterator<Item = ScriptBlock<'_>> {
        self.scripts.iter().map(|range| ScriptBlock {
            text: &self.source[range.start as usize..range.end as usize],
            offset: range.start,
            source_type: range.source_type,
            setup: range.setup,
        })
    }

    /// The SFC template block, if any.
    #[must_use]
    pub fn template(&self) -> Option<TemplateBlock<'_>> {
        self.template.map(|(start, end)| TemplateBlock {
            text: &self.source[start as usize..end as usize],
            offset: start,
        })
    }

    /// The file name without its extension (`UserCard` for
    /// `src/components/UserCard.vue`).
    #[must_use]
    pub fn stem(&self) -> &str {
        let name = self.key.rsplit('/').next().unwrap_or(&self.key);
        name.split('.').next().unwrap_or(name)
    }
}

/// The project artifact: every file a provider may read.
#[derive(Debug, Default)]
pub struct ProjectSources {
    modules: Vec<ProjectModule>,
    by_key: FxHashMap<CompactString, ModuleId>,
}

const SCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];
const RESOLVE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".vue",
];

impl ProjectSources {
    /// An empty project.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file. Returns `None` for a file that is neither a script module
    /// nor a `.vue` SFC, or a path already added.
    pub fn add(&mut self, path: &str, source: &str) -> Option<ModuleId> {
        let extension = Path::new(path).extension()?.to_str()?;
        let key = normalize(Path::new(path));
        if self.by_key.contains_key(&key) {
            return None;
        }
        let (kind, scripts, template) = if extension == "vue" {
            let (scripts, template) = split_sfc(source, path);
            (ModuleKind::Sfc, scripts, template)
        } else if SCRIPT_EXTENSIONS.contains(&extension) {
            let range = ScriptRange {
                start: 0,
                end: u32::try_from(source.len()).ok()?,
                source_type: SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts()),
                setup: false,
            };
            (ModuleKind::Script, SmallVec::from_iter([range]), None)
        } else {
            return None;
        };
        let id = ModuleId(u32::try_from(self.modules.len()).ok()?);
        self.modules.push(ProjectModule {
            path: CompactString::new(path.strip_prefix("./").unwrap_or(path)),
            key: key.clone(),
            source: CompactString::new(source),
            kind,
            scripts,
            template,
        });
        self.by_key.insert(key, id);
        Some(id)
    }

    /// Every module, in insertion order.
    pub fn modules(&self) -> impl ExactSizeIterator<Item = (ModuleId, &ProjectModule)> {
        self.modules
            .iter()
            .enumerate()
            .map(|(index, module)| (ModuleId(index as u32), module))
    }

    /// The module `id`.
    #[must_use]
    pub fn module(&self, id: ModuleId) -> &ProjectModule {
        &self.modules[id.index()]
    }

    /// How many modules the project holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether the project holds no modules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// The module a **relative** import `specifier` in `from` names, trying
    /// the specifier as written, then with each script/SFC extension, then
    /// as a directory `index`. Bare and aliased specifiers resolve to `None`:
    /// a provider treats what it cannot see as outside its domain.
    #[must_use]
    pub fn resolve(&self, from: ModuleId, specifier: &str) -> Option<ModuleId> {
        if !(specifier.starts_with("./") || specifier.starts_with("../")) {
            return None;
        }
        let base = Path::new(self.module(from).key.as_str()).parent()?;
        let joined = normalize(&base.join(specifier));
        if let Some(id) = self.by_key.get(&joined) {
            return Some(*id);
        }
        for extension in RESOLVE_EXTENSIONS {
            let mut candidate = joined.clone();
            candidate.push_str(extension);
            if let Some(id) = self.by_key.get(&candidate) {
                return Some(*id);
            }
        }
        RESOLVE_EXTENSIONS.iter().find_map(|extension| {
            let mut candidate = joined.clone();
            candidate.push_str("/index");
            candidate.push_str(extension);
            self.by_key.get(&candidate).copied()
        })
    }
}

fn split_sfc(source: &str, path: &str) -> (SmallVec<[ScriptRange; 2]>, Option<(u32, u32)>) {
    let options = SfcParseOptions {
        filename: path.into(),
        ..Default::default()
    };
    let Ok(descriptor) = parse_sfc_without_css_vars(source, options) else {
        return (SmallVec::new(), None);
    };
    let mut scripts = SmallVec::new();
    for script in [&descriptor.script, &descriptor.script_setup]
        .into_iter()
        .flatten()
    {
        let lang = script.lang.as_deref().unwrap_or("js");
        let source_type = SourceType::from_extension(lang).unwrap_or_else(|_| SourceType::ts());
        scripts.push(ScriptRange {
            start: script.loc.start as u32,
            end: script.loc.end as u32,
            source_type,
            setup: script.setup,
        });
    }
    scripts.sort_by_key(|range: &ScriptRange| range.start);
    let template = descriptor
        .template
        .as_ref()
        .filter(|template| template.lang.as_deref().is_none_or(|lang| lang == "html"))
        .map(|template| (template.loc.start as u32, template.loc.end as u32));
    (scripts, template)
}

/// A path as a `/`-separated key with `.` and `..` folded lexically.
fn normalize(path: &Path) -> CompactString {
    let mut parts: Vec<PathBuf> = Vec::new();
    let mut prefix = CompactString::default();
    for component in path.components() {
        match component {
            Component::Prefix(value) => prefix.push_str(&value.as_os_str().to_string_lossy()),
            Component::RootDir => prefix.push('/'),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    parts.push(PathBuf::from(".."));
                }
            }
            Component::Normal(value) => parts.push(PathBuf::from(value)),
        }
    }
    let mut key = prefix;
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            key.push('/');
        }
        key.push_str(&part.to_string_lossy());
    }
    key
}
