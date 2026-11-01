use super::super::SessionStorage;
use std::{fs, path::Path};

fn assert_base(storage: &SessionStorage, expected: &Path) {
    assert_eq!(
        storage
            .root()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap(),
        expected.canonicalize().unwrap()
    );
}

pub(super) fn fallback_preserves_rejected_trees() {
    let fixture = tempfile::tempdir().unwrap();
    let project = fixture.path().join("project");
    let local_temp = project.join(".tmp");
    let package_temp = fixture.path().join("node_modules/package/tmp");
    let cache = fixture.path().join("cache");
    for directory in [&local_temp, &package_temp, &cache] {
        fs::create_dir_all(directory).unwrap();
    }
    fs::write(project.join("source.ts"), b"authored bytes").unwrap();
    let missing_cache = cache.join("missing/deep");
    let storage = SessionStorage::new_with_bases(
        &project,
        [
            local_temp.clone(),
            package_temp.clone(),
            missing_cache.clone(),
        ],
    )
    .unwrap();
    assert_base(&storage, &cache);
    assert_eq!(fs::read_dir(&local_temp).unwrap().count(), 0);
    assert_eq!(fs::read_dir(&package_temp).unwrap().count(), 0);
    assert_eq!(
        fs::read(project.join("source.ts")).unwrap(),
        b"authored bytes"
    );
    assert!(!missing_cache.exists());
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 1);
    drop(storage);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
}

pub(super) fn aliases_and_unusable_bases() {
    let fixture = tempfile::tempdir().unwrap();
    let project = fixture.path().join("project");
    let local_temp = project.join(".tmp");
    let cache = fixture.path().join("cache");
    fs::create_dir_all(&local_temp).unwrap();
    fs::create_dir(&cache).unwrap();
    let alias = fixture.path().join("temp-alias");
    super::link(&local_temp, &alias);
    let storage = SessionStorage::new_with_bases(&project, [alias, cache.clone()]).unwrap();
    assert_base(&storage, &cache);
    assert_eq!(fs::read_dir(&local_temp).unwrap().count(), 0);
    drop(storage);

    // Existing aliases keep the original physical-base policy, including
    // lexical project/package paths whose targets are safely external.
    for safe_alias in [
        local_temp.join("external-cache"),
        fixture.path().join("node_modules/external-cache"),
    ] {
        fs::create_dir_all(safe_alias.parent().unwrap()).unwrap();
        super::link(&cache, &safe_alias);
        let storage = SessionStorage::new_with_bases(&project, [safe_alias]).unwrap();
        assert_base(&storage, &cache);
        drop(storage);
    }

    let file = fixture.path().join("not-a-directory");
    fs::write(&file, b"unrelated bytes").unwrap();
    let storage = SessionStorage::new_with_bases(&project, [file.clone(), cache.clone()]).unwrap();
    assert_base(&storage, &cache);
    assert_eq!(fs::read(&file).unwrap(), b"unrelated bytes");
    drop(storage);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
}

pub(super) fn all_unsafe_bases_refuse() {
    let fixture = tempfile::tempdir().unwrap();
    let project = fixture.path().join("project");
    let local_temp = project.join(".tmp");
    let package_temp = fixture.path().join("node_modules/package/tmp");
    fs::create_dir_all(&local_temp).unwrap();
    fs::create_dir_all(&package_temp).unwrap();
    let Err(error) = SessionStorage::new_with_bases(
        &project,
        [
            local_temp.clone(),
            package_temp.clone(),
            project.join("missing/deep"),
        ],
    ) else {
        panic!("all unsafe bases must refuse");
    };
    assert_eq!(
        error.as_str(),
        concat!(
            "No usable private Corsa storage base: ",
            "Private Corsa storage cannot be inside the authored or package tree; ",
            "Private Corsa storage cannot be inside the authored or package tree; ",
            "Private Corsa storage cannot be inside the authored or package tree"
        )
    );
    assert_eq!(fs::read_dir(&local_temp).unwrap().count(), 0);
    assert_eq!(fs::read_dir(&package_temp).unwrap().count(), 0);
    assert!(!project.join("missing").exists());
}
