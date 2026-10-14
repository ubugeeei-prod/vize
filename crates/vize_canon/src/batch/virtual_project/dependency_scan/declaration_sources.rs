//! A declaration barrel that owns source edges must join the source mirror.
//!
//! Terminal ambient declarations keep their authored identity. A generated
//! declaration that re-exports project TS/SFC source cannot stay there: native
//! would load the original TS, bypass its lowered `.vue` imports, and lose types.
#![expect(
    clippy::disallowed_types,
    reason = "effective alias API uses std strings"
)]

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet};

use super::{VirtualProject, canonical_key, inside_node_modules, is_declaration_file};

pub(super) fn alias_reaches_project_source(
    project: &VirtualProject,
    target: &str,
    aliases: &[(std::string::String, std::string::String)],
    overlays: &FxHashMap<PathBuf, &str>,
    cache: &mut FxHashMap<PathBuf, bool>,
) -> bool {
    let Some(target) = super::resolution::probe_candidates(&project.project_root.join(target))
        .and_then(|path| canonical_key(&path))
    else {
        return false;
    };
    is_declaration_file(&target)
        && !inside_node_modules(&target)
        && source_backed(project, &target, aliases, overlays, cache)
}

pub(super) fn source_backed(
    project: &VirtualProject,
    declaration: &Path,
    aliases: &[(std::string::String, std::string::String)],
    overlays: &FxHashMap<PathBuf, &str>,
    cache: &mut FxHashMap<PathBuf, bool>,
) -> bool {
    if let Some(answer) = cache.get(declaration) {
        return *answer;
    }
    let mut pending = vec![declaration.to_path_buf()];
    let mut visited = FxHashSet::default();
    let mut source_backed = false;
    while let Some(path) = pending.pop() {
        if let Some(answer) = cache.get(&path) {
            if *answer {
                source_backed = true;
                break;
            }
            continue;
        }
        if !visited.insert(path.clone()) {
            continue;
        }
        let content = overlays
            .get(&path)
            .map(|content| (*content).into())
            .or_else(|| std::fs::read_to_string(&path).ok());
        let Some(content) = content else { continue };
        let Some(directory) = path.parent() else {
            continue;
        };
        for (specifier, _) in project
            .rewriter()
            .collect_all_specifier_occurrences(&content, SourceType::ts())
        {
            let Some(target) =
                super::resolve_dependency(&specifier, directory, &project.project_root, aliases)
                    .and_then(|path| canonical_key(&path))
            else {
                continue;
            };
            if inside_node_modules(&target) {
                continue;
            }
            if is_declaration_file(&target) {
                pending.push(target);
            } else if project.source_file_policy().accepts_project_source(&target) {
                source_backed = true;
                break;
            }
        }
        if source_backed {
            break;
        }
    }
    if source_backed {
        // Early success proves the queried root, not every visited branch.
        cache.insert(declaration.to_path_buf(), true);
    } else {
        // A complete negative closure proves each visited node. Subsequent
        // aliases of the same cycle must not parse that closure again.
        cache.extend(visited.into_iter().map(|path| (path, false)));
    }
    source_backed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_cycles_cache_the_complete_closure_and_fresh_walks_observe_overlays() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path().canonicalize().unwrap();
        let first = root.join("first.d.ts");
        let second = root.join("second.d.ts");
        let third = root.join("third.d.ts");
        std::fs::write(&first, "export * from './second';\n").unwrap();
        std::fs::write(&second, "export * from './third';\n").unwrap();
        std::fs::write(&third, "export * from './first';\n").unwrap();
        std::fs::write(root.join("ready.ts"), "export const ready = 1;\n").unwrap();
        let project = VirtualProject::new(&root).unwrap();
        let empty = FxHashMap::default();
        let mut cache = FxHashMap::default();
        assert!(!source_backed(&project, &first, &[], &empty, &mut cache));
        assert_eq!(
            cache,
            [first.clone(), second.clone(), third.clone()]
                .into_iter()
                .map(|path| (path, false))
                .collect()
        );
        // A different root reuses the negative child closure in this walk.
        let outer = root.join("outer.d.ts");
        std::fs::write(&outer, "export * from './second';\n").unwrap();
        assert!(!source_backed(&project, &outer, &[], &empty, &mut cache));
        assert_eq!(cache.len(), 4);
        // Cache scope is one walk: the next walk admits unsaved source edges.
        let overlays = FxHashMap::from_iter([(second, "export { ready } from './ready';\n")]);
        let mut fresh = FxHashMap::default();
        assert!(source_backed(&project, &first, &[], &overlays, &mut fresh));
        assert_eq!(fresh, FxHashMap::from_iter([(first, true)]));
    }

    #[test]
    fn declaration_cycles_reaching_source_join_the_mirror_but_terminal_ambient_types_do_not() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join(".generated")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        let entry = root.join("src/entry.ts");
        let first = root.join(".generated/first.d.ts");
        let second = root.join(".generated/second.d.ts");
        let ambient = root.join(".generated/ambient.d.ts");
        std::fs::write(
            &entry,
            "import { ready } from '#imports';\nimport '../.generated/ambient';\nexport { ready };\n",
        )
        .unwrap();
        std::fs::write(&first, "export * from './second';\n").unwrap();
        std::fs::write(
            &second,
            "export * from './first';\nexport { ready } from '../src/ready';\n",
        )
        .unwrap();
        std::fs::write(root.join("src/ready.ts"), "export const ready = 1;\n").unwrap();
        std::fs::write(
            &ambient,
            "declare module 'terminal-types' { export const value: string; }\n",
        )
        .unwrap();
        std::fs::write(root.join("tsconfig.json"), r##"{"compilerOptions":{"paths":{"#imports":[".generated/first.d.ts"]}},"include":["src/**/*"]}"##).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.set_tsconfig_path(Some(root.join("tsconfig.json")));
        let aliases = project.dependency_alias_map();
        let mut cache = FxHashMap::default();
        assert!(alias_reaches_project_source(
            &project,
            ".generated/first.d.ts",
            &aliases,
            &FxHashMap::default(),
            &mut cache
        ));
        assert!(!source_backed(
            &project,
            &ambient,
            &aliases,
            &FxHashMap::default(),
            &mut cache
        ));
        project.register_path(&entry).unwrap();
        project.register_reachable_dependencies().unwrap();
        project.finalize_package_routes().unwrap();
        project.materialize().unwrap();
        assert!(project.find_by_original(&first).is_some());
        assert!(project.find_by_original(&second).is_some());
        assert!(
            project
                .find_by_original(&root.join("src/ready.ts"))
                .is_some()
        );
        assert!(project.find_by_original(&ambient).is_none());
        assert_eq!(
            project.find_by_original(&entry).unwrap().content.as_str(),
            format!(
                "import {{ ready }} from '#imports';\nimport '{}';\nexport {{ ready }};\n",
                root.join(".generated/ambient").display()
            )
        );
        let code =
            std::fs::read_to_string(project.virtual_root().join(".generated/first.d.ts")).unwrap();
        assert_eq!(
            code,
            format!(
                "export * from '{}';\n",
                project.virtual_root().join(".generated/second").display()
            )
        );
    }
}
