//! A source that imports a plugin's module augmentation stays in every shard.

use super::{VirtualProject, fs};
use crate::batch::executor::cli::partition_virtual_files;

const PLUGIN: &str = "dayjs/plugin/relativeTime";

fn project(root: &std::path::Path) -> (VirtualProject, std::path::PathBuf) {
    let entry = root.join("main.ts");
    fs::write(
        &entry,
        "import dayjs from 'dayjs'\nimport relativeTime from 'dayjs/plugin/relativeTime'\ndayjs.extend(relativeTime)\nexport {}\n",
    )
    .unwrap();
    let mut project = VirtualProject::new(root).unwrap();
    project.register_path(&entry).unwrap();
    for name in ["A", "B", "C"] {
        let source = root.join(vize_carton::cstr!("{name}.vue").as_str());
        fs::write(
            &source,
            "<script setup lang=\"ts\">import dayjs from 'dayjs'\nconst at = dayjs().fromNow()</script><template>{{ at }}</template>",
        )
        .unwrap();
        project.register_path(&source).unwrap();
    }
    (project, entry)
}

#[test]
fn plugin_registering_sources_are_shared_by_every_shard() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let (mut project, entry) = project(&root);
    project.set_module_augmentation_probe(Box::new(|_, specifier| specifier == PLUGIN));

    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    assert_eq!(plan.owners.len(), 3);
    assert_eq!(plan.owners.get(&entry), None);
    let entry_virtual = &project.find_by_original(&entry).unwrap().virtual_path;
    for shard in &plan.shards {
        assert_eq!(
            shard
                .iter()
                .filter(|path| **path == entry_virtual.as_path())
                .count(),
            1
        );
    }
}

#[test]
fn sources_stay_partitioned_without_an_augmenting_import() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let (mut project, entry) = project(&root);
    project.set_module_augmentation_probe(Box::new(|_, _| false));

    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    assert_eq!(plan.owners.len(), 4);
    let owner = plan.owners.get(&entry).copied().unwrap();
    let entry_virtual = &project.find_by_original(&entry).unwrap().virtual_path;
    for (index, shard) in plan.shards.iter().enumerate() {
        assert_eq!(
            shard.iter().any(|path| *path == entry_virtual.as_path()),
            index == owner
        );
    }
}
