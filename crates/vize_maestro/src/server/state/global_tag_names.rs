//! Cache authored GlobalComponents declaration names for tag completion.
use super::ServerState;
use futures::{channel::oneshot, lock::Mutex as AsyncMutex};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, TSInterfaceDeclaration, TSModuleDeclaration, TSModuleDeclarationName, TSSignature,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use parking_lot::RwLock;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::SystemTime,
};
use tower_lsp::lsp_types::Url;
use vize_carton::source_io as fs;
use vize_l0::{String, ToCompactString};

#[derive(PartialEq, Eq)]
enum Stamp {
    Open(u64),
    Disk(u64, Option<SystemTime>),
}
struct CachedNames {
    stamp: Stamp,
    names: Vec<String>,
}
#[derive(Default)]
pub(super) struct GlobalTagNamesCache {
    names: RwLock<BTreeMap<PathBuf, CachedNames>>,
    scan_lock: AsyncMutex<()>,
}

impl GlobalTagNamesCache {
    pub(super) fn clear(&self) {
        self.names.write().clear();
    }
}

impl ServerState {
    pub(crate) async fn global_component_tag_names(&self) -> Vec<String> {
        let mut paths = self.global_component_reference_paths().await;
        for document in self.documents.iter() {
            if let Ok(path) = document.key().to_file_path()
                && path.to_str().is_some_and(|path| {
                    [".d.ts", ".d.mts", ".d.cts"]
                        .iter()
                        .any(|suffix| path.ends_with(suffix))
                })
                && !vize_carton::path::is_git_metadata_path(&path)
            {
                paths.push(path);
            }
        }
        paths.sort();
        paths.dedup();
        let cache = &self.global_component_references.tag_names;
        let _scan = cache.scan_lock.lock().await;
        let mut result = BTreeSet::new();
        let mut missing = Vec::new();
        for path in &paths {
            let document = Url::from_file_path(path)
                .ok()
                .and_then(|uri| self.documents.get(&uri));
            let stamp = if let Some(document) = &document {
                Stamp::Open(document.revision())
            } else if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > 4 * 1024 * 1024 {
                    continue;
                }
                Stamp::Disk(metadata.len(), metadata.modified().ok())
            } else {
                continue;
            };
            if let Some(cached) = cache
                .names
                .read()
                .get(path)
                .filter(|cached| cached.stamp == stamp)
            {
                result.extend(cached.names.clone());
                continue;
            }
            if document
                .as_ref()
                .is_some_and(|document| document.content.len_bytes() > 4 * 1024 * 1024)
            {
                cache.names.write().insert(
                    path.clone(),
                    CachedNames {
                        stamp,
                        names: Vec::new(),
                    },
                );
                continue;
            }
            let source = document.map(|document| String::from(document.text()));
            missing.push((path.clone(), stamp, source));
        }
        if !missing.is_empty() {
            let (sender, receiver) = oneshot::channel();
            let spawned = std::thread::Builder::new()
                .name("vize-global-tag-names".to_string())
                .spawn(move || {
                    let parsed = missing
                        .into_iter()
                        .filter_map(|(path, stamp, source)| {
                            let source = source
                                .or_else(|| fs::read_to_string(&path).ok().map(String::from))?;
                            let names = declared_names(&source);
                            Some((path, CachedNames { stamp, names }))
                        })
                        .collect::<Vec<_>>();
                    let _ = sender.send(parsed);
                });
            if let Err(error) = spawned {
                tracing::warn!("failed to spawn global tag name parsing: {error}");
            } else if let Ok(parsed) = receiver.await {
                let mut cached = cache.names.write();
                for (path, entry) in parsed {
                    result.extend(entry.names.iter().cloned());
                    cached.insert(path, entry);
                }
            }
        }
        cache
            .names
            .write()
            .retain(|path, _| paths.binary_search(path).is_ok());
        result.into_iter().collect()
    }
}

#[derive(Default)]
struct Interface {
    names: Vec<String>,
    extends: Vec<String>,
}
#[derive(Default)]
struct Declarations {
    vue_module: bool,
    interfaces: BTreeMap<String, Interface>,
    roots: Vec<String>,
}

impl<'a> Visit<'a> for Declarations {
    fn visit_ts_module_declaration(&mut self, module: &TSModuleDeclaration<'a>) {
        let previous = self.vue_module;
        self.vue_module = matches!(&module.id, TSModuleDeclarationName::StringLiteral(name) if matches!(name.value.as_str(), "vue" | "@vue/runtime-core"));
        walk::walk_ts_module_declaration(self, module);
        self.vue_module = previous;
    }
    fn visit_ts_interface_declaration(&mut self, interface: &TSInterfaceDeclaration<'a>) {
        let name = interface.id.name.as_str().to_compact_string();
        if name == "GlobalComponents" && self.vue_module {
            self.roots.push(name.clone());
        }
        if name == "GlobalComponents" && !self.vue_module {
            return;
        }
        let info = self.interfaces.entry(name).or_default();
        for member in &interface.body.body {
            if let TSSignature::TSPropertySignature(property) = member
                && !property.computed
                && let Some(name) = property.key.static_name()
            {
                info.names.push(name.as_ref().to_compact_string());
            }
        }
        for heritage in &interface.extends {
            if let Expression::Identifier(identifier) = &heritage.expression {
                info.extends
                    .push(identifier.name.as_str().to_compact_string());
            }
        }
    }
}

fn declared_names(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let mut declarations = Declarations::default();
    declarations.visit_program(&parsed.program);
    let mut pending = declarations.roots;
    let mut visited = BTreeSet::new();
    let mut result = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        if let Some(interface) = declarations.interfaces.get(&name) {
            result.extend(interface.names.iter().cloned());
            pending.extend(interface.extends.iter().cloned());
        }
    }
    result.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::declared_names;
    use vize_l0::String;
    #[test]
    fn collects_vue_module_members_and_inheritance_without_unrelated_interfaces() {
        let source = "interface Shared { Extra: unknown }\ninterface GlobalComponents { Wrong: unknown }\ndeclare module 'vue' { export interface GlobalComponents extends Shared { Child: unknown; 'el-button': unknown; [computed]: unknown } }\ndeclare module '@vue/runtime-core' { interface GlobalComponents { Other: unknown } }";
        assert_eq!(
            declared_names(source),
            ["Child", "Extra", "Other", "el-button"].map(String::from)
        );
    }

    #[test]
    fn cached_disk_names_follow_unsaved_declarations_and_reopened_documents() {
        let root = tempfile::tempdir().expect("workspace");
        let path = root.path().join("components.d.ts");
        std::fs::write(
            &path,
            "declare module 'vue' { interface GlobalComponents { DiskCard: unknown } }",
        )
        .expect("declaration");
        let state = super::ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        let names = || crate::runtime::block_on(state.global_component_tag_names());
        assert_eq!(names(), [String::from("DiskCard")]);
        assert_eq!(names(), [String::from("DiskCard")]);
        let uri = tower_lsp::lsp_types::Url::from_file_path(&path).expect("URI");
        state.documents.open(
            uri.clone(),
            "declare module 'vue' { interface GlobalComponents { UnsavedCard: unknown } }".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(names(), [String::from("UnsavedCard")]);
        state.documents.close(&uri);
        state.documents.open(
            uri.clone(),
            "declare module 'vue' { interface GlobalComponents { ReopenedCard: unknown } }".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(names(), [String::from("ReopenedCard")]);
        state
            .documents
            .open(uri, " ".repeat(4 * 1024 * 1024 + 1), 2, "typescript".into());
        assert_eq!(names(), Vec::<String>::new());
        assert_eq!(names(), Vec::<String>::new());
    }
}
