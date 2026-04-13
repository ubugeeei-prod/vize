//! Manual import-path rewriting for file renames.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

use ignore::{WalkBuilder, WalkState};
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, ImportExpression, Statement, TSImportType};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{FileRename, Range, TextEdit, Url, WorkspaceEdit};

use crate::{ide::offset_to_position, server::ServerState};

const SCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];
const RESOLVABLE_SCRIPT_EXTENSIONS: &[&str] =
    &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "vue"];

#[derive(Clone)]
struct RenameTarget {
    old_path: PathBuf,
    new_path: PathBuf,
}

#[derive(Clone, Copy)]
enum ImporterKind {
    Vue,
    Script,
}

#[derive(Clone, Copy)]
enum RenderStyle {
    Explicit,
    Extensionless,
    DirectoryIndex,
}

#[derive(Clone)]
struct SpecifierCandidate {
    resolved: PathBuf,
    style: RenderStyle,
}

#[derive(Clone)]
struct SpecifierOccurrence {
    start: usize,
    end: usize,
    specifier: std::string::String,
}

struct ScriptEditContext<'a> {
    state: &'a ServerState,
    current_path: &'a Path,
    future_path: &'a Path,
    full_source: &'a str,
    rename_targets: &'a [RenameTarget],
}

#[derive(Default)]
struct ModuleSpecifierCollector {
    specifiers: Vec<SpecifierOccurrence>,
}

impl ModuleSpecifierCollector {
    fn push(&mut self, start: u32, end: u32, specifier: &str) {
        self.specifiers.push(SpecifierOccurrence {
            start: start as usize,
            end: end as usize,
            specifier: specifier.to_string(),
        });
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_program(&mut self, program: &oxc_ast::ast::Program<'a>) {
        for statement in &program.body {
            match statement {
                Statement::ImportDeclaration(decl) => {
                    self.push(
                        decl.source.span.start + 1,
                        decl.source.span.end - 1,
                        decl.source.value.as_str(),
                    );
                }
                Statement::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source {
                        self.push(
                            source.span.start + 1,
                            source.span.end - 1,
                            source.value.as_str(),
                        );
                    }
                }
                Statement::ExportAllDeclaration(decl) => {
                    self.push(
                        decl.source.span.start + 1,
                        decl.source.span.end - 1,
                        decl.source.value.as_str(),
                    );
                }
                _ => {}
            }
        }

        walk::walk_program(self, program);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        if let Expression::StringLiteral(literal) = &expression.source {
            self.push(
                literal.span.start + 1,
                literal.span.end - 1,
                literal.value.as_str(),
            );
        }

        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if let Expression::Identifier(identifier) = &expression.callee {
            if identifier.name.as_str() == "require" {
                if let Some(oxc_ast::ast::Argument::StringLiteral(literal)) =
                    expression.arguments.first()
                {
                    self.push(
                        literal.span.start + 1,
                        literal.span.end - 1,
                        literal.value.as_str(),
                    );
                }
            }
        }

        walk::walk_call_expression(self, expression);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push(
            import_type.source.span.start + 1,
            import_type.source.span.end - 1,
            import_type.source.value.as_str(),
        );

        walk::walk_ts_import_type(self, import_type);
    }
}

pub(super) fn collect_import_rename_edits(
    state: &ServerState,
    renames: &[FileRename],
    only_vue_importers: bool,
) -> Option<WorkspaceEdit> {
    let rename_targets = rename_targets(renames);
    if rename_targets.is_empty() {
        return None;
    }

    let workspace_root = workspace_root(state);
    let changes = Mutex::new(HashMap::new());
    let seen_paths = Mutex::new(HashSet::new());

    WalkBuilder::new(&workspace_root)
        .standard_filters(true)
        .hidden(true)
        .build_parallel()
        .run(|| {
            let changes = &changes;
            let seen_paths = &seen_paths;
            let rename_targets = &rename_targets;

            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Continue;
                };

                let path = entry.path();
                let Some(kind) = importer_kind(path, only_vue_importers) else {
                    return WalkState::Continue;
                };

                if let Ok(mut seen) = seen_paths.lock() {
                    seen.insert(path.to_path_buf());
                }

                if let Some((uri, edits)) = process_importer_path(state, path, kind, rename_targets)
                {
                    if let Ok(mut changes) = changes.lock() {
                        changes.insert(uri, edits);
                    }
                }

                WalkState::Continue
            })
        });

    let seen_paths = seen_paths.into_inner().unwrap_or_default();
    for document in state.documents.iter() {
        let uri = document.key().clone();
        let Ok(path) = uri.to_file_path() else {
            continue;
        };
        let Some(kind) = importer_kind(&path, only_vue_importers) else {
            continue;
        };
        if seen_paths.contains(&path) {
            continue;
        }

        if let Some((uri, edits)) = process_source(
            state,
            uri.clone(),
            &path,
            &document.value().text(),
            kind,
            &rename_targets,
        ) {
            if let Ok(mut changes) = changes.lock() {
                changes.insert(uri, edits);
            }
        }
    }

    let changes = changes.into_inner().unwrap_or_default();
    if changes.is_empty() {
        None
    } else {
        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

pub(super) fn rename_open_documents(
    state: &ServerState,
    renames: &[FileRename],
) -> Vec<(Url, Url)> {
    let rename_targets = rename_targets(renames);
    if rename_targets.is_empty() {
        return Vec::new();
    }

    let mut renamed_documents = Vec::new();
    let open_uris = state.documents.uris();

    for old_uri in open_uris {
        let Some(new_uri) = apply_all_uri_renames(&old_uri, &rename_targets) else {
            continue;
        };

        if new_uri == old_uri {
            continue;
        }

        if state.documents.rename(&old_uri, new_uri.clone()) {
            state.remove_virtual_docs(&old_uri);

            if let Some(document) = state.documents.get(&new_uri) {
                let content = document.text();
                drop(document);
                state.update_virtual_docs(&new_uri, &content);
            }

            renamed_documents.push((old_uri, new_uri));
        }
    }

    renamed_documents
}

fn process_importer_path(
    state: &ServerState,
    path: &Path,
    kind: ImporterKind,
    rename_targets: &[RenameTarget],
) -> Option<(Url, Vec<TextEdit>)> {
    let uri = Url::from_file_path(path).ok()?;
    let source = read_workspace_source(state, path)?;
    process_source(state, uri, path, &source, kind, rename_targets)
}

fn process_source(
    state: &ServerState,
    uri: Url,
    path: &Path,
    source: &str,
    kind: ImporterKind,
    rename_targets: &[RenameTarget],
) -> Option<(Url, Vec<TextEdit>)> {
    let future_path =
        apply_all_path_renames(path, rename_targets).unwrap_or_else(|| normalize_path_buf(path));

    let mut edits = match kind {
        ImporterKind::Vue => collect_vue_edits(state, path, &future_path, source, rename_targets),
        ImporterKind::Script => {
            collect_script_file_edits(state, path, &future_path, source, rename_targets)
        }
    };

    if edits.is_empty() {
        return None;
    }

    edits.sort_by(|left, right| {
        left.range
            .start
            .line
            .cmp(&right.range.start.line)
            .then(left.range.start.character.cmp(&right.range.start.character))
    });

    Some((uri, edits))
}

fn collect_vue_edits(
    state: &ServerState,
    path: &Path,
    future_path: &Path,
    source: &str,
    rename_targets: &[RenameTarget],
) -> Vec<TextEdit> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: path.to_string_lossy().to_string().into(),
        ..Default::default()
    };

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
        return Vec::new();
    };

    let mut edits = Vec::new();
    let edit_context = ScriptEditContext {
        state,
        current_path: path,
        future_path,
        full_source: source,
        rename_targets,
    };

    if let Some(script) = descriptor.script.as_ref() {
        edits.extend(collect_script_content_edits(
            &edit_context,
            script.content.as_ref(),
            script_source_type(script.lang.as_deref()),
            script.loc.start,
        ));
    }

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        edits.extend(collect_script_content_edits(
            &edit_context,
            script_setup.content.as_ref(),
            script_source_type(script_setup.lang.as_deref()),
            script_setup.loc.start,
        ));
    }

    edits
}

fn collect_script_file_edits(
    state: &ServerState,
    path: &Path,
    future_path: &Path,
    source: &str,
    rename_targets: &[RenameTarget],
) -> Vec<TextEdit> {
    let Some(source_type) = SourceType::from_path(path).ok() else {
        return Vec::new();
    };

    let edit_context = ScriptEditContext {
        state,
        current_path: path,
        future_path,
        full_source: source,
        rename_targets,
    };

    collect_script_content_edits(&edit_context, source, source_type, 0)
}

fn collect_script_content_edits(
    context: &ScriptEditContext<'_>,
    script_source: &str,
    source_type: SourceType,
    base_offset: usize,
) -> Vec<TextEdit> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script_source, source_type).parse();

    let mut collector = ModuleSpecifierCollector::default();
    collector.visit_program(&parsed.program);

    let Some(current_dir) = context.current_path.parent() else {
        return Vec::new();
    };
    let Some(future_dir) = context.future_path.parent() else {
        return Vec::new();
    };

    collector
        .specifiers
        .into_iter()
        .filter_map(|specifier| {
            let new_text = rewrite_relative_specifier(
                context.state,
                current_dir,
                future_dir,
                &specifier.specifier,
                context.rename_targets,
            )?;

            if new_text == specifier.specifier {
                return None;
            }

            let start_offset = base_offset + specifier.start;
            let end_offset = base_offset + specifier.end;
            let range = offset_range(context.full_source, start_offset, end_offset)?;

            Some(TextEdit { range, new_text })
        })
        .collect()
}

fn rewrite_relative_specifier(
    state: &ServerState,
    current_importer_dir: &Path,
    future_importer_dir: &Path,
    specifier: &str,
    rename_targets: &[RenameTarget],
) -> Option<std::string::String> {
    let (specifier_path, suffix) = split_specifier_suffix(specifier);
    if !specifier_path.starts_with("./") && !specifier_path.starts_with("../") {
        return None;
    }

    let mut selected = None;
    for candidate in specifier_candidates(current_importer_dir, specifier_path) {
        if candidate_exists(state, &candidate.resolved)
            || apply_all_path_renames(&candidate.resolved, rename_targets).is_some()
        {
            selected = Some(candidate);
            break;
        }
    }

    let selected = selected?;
    let future_target = apply_all_path_renames(&selected.resolved, rename_targets)
        .unwrap_or_else(|| normalize_path_buf(&selected.resolved));

    if future_target == normalize_path_buf(&selected.resolved)
        && normalize_path_buf(current_importer_dir) == normalize_path_buf(future_importer_dir)
    {
        return None;
    }

    let mut rewritten =
        render_module_specifier(future_importer_dir, &future_target, selected.style)?;
    rewritten.push_str(suffix);
    Some(rewritten)
}

fn render_module_specifier(
    importer_dir: &Path,
    target_path: &Path,
    style: RenderStyle,
) -> Option<std::string::String> {
    let rendered_target = match style {
        RenderStyle::Explicit => normalize_path_buf(target_path),
        RenderStyle::Extensionless => strip_extension(target_path),
        RenderStyle::DirectoryIndex => {
            if is_index_file(target_path) {
                normalize_path_buf(target_path.parent()?)
            } else {
                strip_extension(target_path)
            }
        }
    };

    relative_module_path(importer_dir, &rendered_target)
}

fn relative_module_path(from_dir: &Path, to_path: &Path) -> Option<std::string::String> {
    let from_dir = normalize_path_buf(from_dir);
    let to_path = normalize_path_buf(to_path);

    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_path.components().collect::<Vec<_>>();

    let mut common = 0usize;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    if common == 0
        && matches!(from_components.first(), Some(Component::Prefix(_)))
        && matches!(to_components.first(), Some(Component::Prefix(_)))
    {
        return None;
    }

    let mut parts = Vec::new();
    for _ in common..from_components.len() {
        parts.push("..".to_string());
    }
    for component in &to_components[common..] {
        let part = match component {
            Component::Normal(value) => value.to_string_lossy().to_string(),
            Component::CurDir => ".".to_string(),
            Component::ParentDir => "..".to_string(),
            Component::RootDir | Component::Prefix(_) => continue,
        };
        parts.push(part);
    }

    let joined = if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    };

    if joined.starts_with("../") || joined == ".." {
        Some(joined)
    } else if joined == "." {
        Some("./".to_string())
    } else if joined.starts_with("./") {
        Some(joined)
    } else {
        let mut prefixed = std::string::String::from("./");
        prefixed.push_str(&joined);
        Some(prefixed)
    }
}

fn specifier_candidates(importer_dir: &Path, specifier: &str) -> Vec<SpecifierCandidate> {
    let resolved = normalize_path_buf(&importer_dir.join(specifier));
    let mut candidates = Vec::new();

    if Path::new(specifier).extension().is_none() {
        for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
            candidates.push(SpecifierCandidate {
                resolved: resolved.with_extension(extension),
                style: RenderStyle::Extensionless,
            });
        }

        for extension in RESOLVABLE_SCRIPT_EXTENSIONS {
            let mut index_name = std::string::String::from("index.");
            index_name.push_str(extension);
            candidates.push(SpecifierCandidate {
                resolved: resolved.join(index_name),
                style: RenderStyle::DirectoryIndex,
            });
        }

        candidates.push(SpecifierCandidate {
            resolved,
            style: RenderStyle::Explicit,
        });
    } else {
        candidates.push(SpecifierCandidate {
            resolved,
            style: RenderStyle::Explicit,
        });
    }

    candidates
}

fn rename_targets(renames: &[FileRename]) -> Vec<RenameTarget> {
    renames
        .iter()
        .filter_map(|rename| {
            let old_path = Url::parse(&rename.old_uri).ok()?.to_file_path().ok()?;
            let new_path = Url::parse(&rename.new_uri).ok()?.to_file_path().ok()?;

            Some(RenameTarget {
                old_path: normalize_path_buf(&old_path),
                new_path: normalize_path_buf(&new_path),
            })
        })
        .collect()
}

fn apply_all_uri_renames(uri: &Url, renames: &[RenameTarget]) -> Option<Url> {
    let path = uri.to_file_path().ok()?;
    let path = apply_all_path_renames(&path, renames)?;
    Url::from_file_path(path).ok()
}

fn apply_all_path_renames(path: &Path, renames: &[RenameTarget]) -> Option<PathBuf> {
    let mut updated = normalize_path_buf(path);
    let mut changed = false;

    for rename in renames {
        if let Some(next) = apply_path_rename(&updated, rename) {
            updated = next;
            changed = true;
        }
    }

    if changed {
        Some(updated)
    } else {
        None
    }
}

fn apply_path_rename(path: &Path, rename: &RenameTarget) -> Option<PathBuf> {
    let normalized = normalize_path_buf(path);
    if normalized == rename.old_path {
        return Some(rename.new_path.clone());
    }

    let suffix = normalized.strip_prefix(&rename.old_path).ok()?;
    if suffix.as_os_str().is_empty() {
        Some(rename.new_path.clone())
    } else {
        Some(normalize_path_buf(&rename.new_path.join(suffix)))
    }
}

fn candidate_exists(state: &ServerState, path: &Path) -> bool {
    if path.exists() {
        return true;
    }

    let Ok(uri) = Url::from_file_path(path) else {
        return false;
    };

    state.documents.contains(&uri)
}

fn importer_kind(path: &Path, only_vue_importers: bool) -> Option<ImporterKind> {
    if path.extension().is_some_and(|extension| extension == "vue") {
        return Some(ImporterKind::Vue);
    }

    if only_vue_importers {
        return None;
    }

    let extension = path.extension()?.to_str()?;
    if SCRIPT_EXTENSIONS.contains(&extension) {
        Some(ImporterKind::Script)
    } else {
        None
    }
}

fn read_workspace_source(state: &ServerState, path: &Path) -> Option<std::string::String> {
    if let Ok(uri) = Url::from_file_path(path) {
        if let Some(document) = state.documents.get(&uri) {
            return Some(document.text());
        }
    }

    fs::read_to_string(path).ok()
}

fn split_specifier_suffix(specifier: &str) -> (&str, &str) {
    let split_at = specifier.find(['?', '#']).unwrap_or(specifier.len());
    (&specifier[..split_at], &specifier[split_at..])
}

fn offset_range(source: &str, start: usize, end: usize) -> Option<Range> {
    let (start_line, start_character) = offset_to_position(source, start);
    let (end_line, end_character) = offset_to_position(source, end);

    Some(Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    })
}

fn script_source_type(lang: Option<&str>) -> SourceType {
    match lang.unwrap_or("js") {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}

fn normalize_path_buf(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

fn strip_extension(path: &Path) -> PathBuf {
    let mut stripped = normalize_path_buf(path);
    if stripped.extension().is_some() {
        stripped.set_extension("");
    }
    stripped
}

fn is_index_file(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem == "index")
}

#[cfg(feature = "native")]
fn workspace_root(state: &ServerState) -> PathBuf {
    state
        .get_workspace_root()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg(not(feature = "native"))]
fn workspace_root(_state: &ServerState) -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(all(test, feature = "native"))]
#[allow(clippy::disallowed_macros)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
    };

    use insta::assert_snapshot;
    use tower_lsp::lsp_types::{FileRename, Url, WorkspaceEdit};

    use super::{collect_import_rename_edits, rename_open_documents};
    use crate::server::ServerState;

    fn test_dir() -> tempfile::TempDir {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("__agent_only");
        fs::create_dir_all(&base).unwrap();
        tempfile::tempdir_in(base).unwrap()
    }

    fn file_uri(path: &Path) -> std::string::String {
        Url::from_file_path(path).unwrap().to_string()
    }

    fn normalize_edit(root: &Path, edit: &WorkspaceEdit) -> serde_json::Value {
        let mut files = BTreeMap::<std::string::String, Vec<serde_json::Value>>::new();

        for (uri, edits) in edit.changes.as_ref().unwrap() {
            let path = uri.to_file_path().unwrap();
            let label = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");

            let items = edits
                .iter()
                .map(|edit| {
                    serde_json::json!({
                        "range": {
                            "start": {
                                "line": edit.range.start.line,
                                "character": edit.range.start.character,
                            },
                            "end": {
                                "line": edit.range.end.line,
                                "character": edit.range.end.character,
                            }
                        },
                        "newText": edit.new_text
                    })
                })
                .collect::<Vec<_>>();

            files.insert(label, items);
        }

        serde_json::json!(files)
    }

    #[test]
    fn rewrites_vue_imports_for_component_rename() {
        let dir = test_dir();
        let root = dir.path();
        let src_dir = root.join("src");
        let components_dir = src_dir.join("components");
        fs::create_dir_all(&components_dir).unwrap();

        let app_path = src_dir.join("App.vue");
        let old_component = components_dir.join("Foo.vue");
        let new_component = components_dir.join("Bar.vue");

        fs::write(
            &app_path,
            r#"<script setup lang="ts">
import Foo from "./components/Foo.vue";
const Lazy = () => import("./components/Foo.vue");
type FooModule = typeof import("./components/Foo.vue");
</script>
"#,
        )
        .unwrap();
        fs::write(&old_component, "<template><div>foo</div></template>").unwrap();

        let state = ServerState::new();
        state.set_workspace_root(root.to_path_buf());

        let edit = collect_import_rename_edits(
            &state,
            &[FileRename {
                old_uri: file_uri(&old_component),
                new_uri: file_uri(&new_component),
            }],
            true,
        )
        .unwrap();

        assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
        {
          "src/App.vue": [
            {
              "newText": "./components/Bar.vue",
              "range": {
                "end": {
                  "character": 37,
                  "line": 1
                },
                "start": {
                  "character": 17,
                  "line": 1
                }
              }
            },
            {
              "newText": "./components/Bar.vue",
              "range": {
                "end": {
                  "character": 47,
                  "line": 2
                },
                "start": {
                  "character": 27,
                  "line": 2
                }
              }
            },
            {
              "newText": "./components/Bar.vue",
              "range": {
                "end": {
                  "character": 52,
                  "line": 3
                },
                "start": {
                  "character": 32,
                  "line": 3
                }
              }
            }
          ]
        }
        "###);
    }

    #[test]
    fn rewrites_extensionless_ts_imports_without_corsa() {
        let dir = test_dir();
        let root = dir.path();
        let src_dir = root.join("src");
        let util_dir = src_dir.join("util");
        fs::create_dir_all(&util_dir).unwrap();

        let entry_path = src_dir.join("entry.ts");
        let old_module = util_dir.join("foo.ts");
        let new_module = util_dir.join("bar.ts");

        fs::write(
            &entry_path,
            "import { value } from \"./util/foo\";\nconst lazy = require(\"./util/foo\");\n",
        )
        .unwrap();
        fs::write(&old_module, "export const value = 1;\n").unwrap();

        let state = ServerState::new();
        state.set_workspace_root(root.to_path_buf());

        let edit = collect_import_rename_edits(
            &state,
            &[FileRename {
                old_uri: file_uri(&old_module),
                new_uri: file_uri(&new_module),
            }],
            false,
        )
        .unwrap();

        assert_snapshot!(serde_json::to_string_pretty(&normalize_edit(root, &edit)).unwrap(), @r###"
        {
          "src/entry.ts": [
            {
              "newText": "./util/bar",
              "range": {
                "end": {
                  "character": 33,
                  "line": 0
                },
                "start": {
                  "character": 23,
                  "line": 0
                }
              }
            },
            {
              "newText": "./util/bar",
              "range": {
                "end": {
                  "character": 32,
                  "line": 1
                },
                "start": {
                  "character": 22,
                  "line": 1
                }
              }
            }
          ]
        }
        "###);
    }

    #[test]
    fn renames_open_documents_inside_renamed_folder() {
        let dir = test_dir();
        let root = dir.path();
        let old_dir = root.join("src/pages");
        let new_dir = root.join("src/views");
        let file_path = old_dir.join("Home.vue");
        fs::create_dir_all(&old_dir).unwrap();
        fs::write(&file_path, "<template><div>home</div></template>").unwrap();

        let state = ServerState::new();
        state.documents.open(
            Url::from_file_path(&file_path).unwrap(),
            "<template><div>home</div></template>".to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(
            &Url::from_file_path(&file_path).unwrap(),
            "<template><div>home</div></template>",
        );

        let renamed = rename_open_documents(
            &state,
            &[FileRename {
                old_uri: file_uri(&old_dir),
                new_uri: file_uri(&new_dir),
            }],
        );

        assert_eq!(renamed.len(), 1);
        let new_uri = Url::from_file_path(new_dir.join("Home.vue")).unwrap();
        assert!(state.documents.contains(&new_uri));
        assert!(state.get_virtual_docs(&new_uri).is_some());
    }
}
