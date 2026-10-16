//! Shard roots must preserve the unsharded workspace dependency context (#9201).

use super::{VirtualProject, fs};
use crate::batch::executor::cli::partition_virtual_files;

#[test]
fn dependency_only_external_sources_do_not_become_shard_roots() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let app = root.join("app");
    fs::create_dir_all(&app).unwrap();
    let external = root.join("lib.ts");
    fs::write(&external, "export const value = 1;\n").unwrap();
    let declaration = root.join("globals.d.ts");
    fs::write(&declaration, "declare const workspaceGlobal: string;\n").unwrap();
    let mut project = VirtualProject::new(&app).unwrap();
    project.register_path(&external).unwrap();
    project.register_path(&declaration).unwrap();
    for name in ["A", "B", "C"] {
        let source = app.join(vize_carton::cstr!("{name}.vue").as_str());
        fs::write(&source, "<template><div /></template>").unwrap();
        project.register_path(&source).unwrap();
    }
    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    assert_eq!(plan.owners.len(), 3);
    assert!(!plan.owners.contains_key(&external));
    let external_virtual = &project.find_by_original(&external).unwrap().virtual_path;
    let declaration_virtual = &project.find_by_original(&declaration).unwrap().virtual_path;
    for shard in plan.shards {
        assert!(!shard.contains(&external_virtual.as_path()));
        assert!(shard.contains(&declaration_virtual.as_path()));
    }
    fs::write(
        &external,
        "export {}; declare global { interface Window { external: string } }\n",
    )
    .unwrap();
    project.register_path(&external).unwrap();
    assert!(
        partition_virtual_files(&project, 2).shards.is_empty(),
        "external program-wide types must remain visible to all roots"
    );
}
