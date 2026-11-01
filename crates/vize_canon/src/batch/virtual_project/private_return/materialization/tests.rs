use super::*;
use crate::batch::virtual_project::private_return::HELPER_BODY;

fn helper(root: &Path) -> HelperFile {
    HelperFile {
        root: root.to_path_buf(),
        path: root.join("app/config/__private_return_abc.d.ts"),
        body: HELPER_BODY,
    }
}

fn fixture() -> (tempfile::TempDir, HelperFile) {
    let directory = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(directory.path()).unwrap();
    let file = helper(&root);
    (directory, file)
}

#[cfg(unix)]
fn importer(helper: &HelperFile) -> PathBuf {
    let path = helper.root.join("app/components/Consumer.vue.ts");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "export {};\n").unwrap();
    path
}

#[cfg(unix)]
#[test]
fn nested_owned_helper_is_exact_and_warm_write_preserves_it() {
    use sha2::{Digest, Sha256};

    assert_eq!(HELPER_BODY.len(), 230);
    assert_eq!(
        crate::batch::virtual_project::identity::encode_digest(Sha256::digest(HELPER_BODY)),
        "d43fa803a7d57f1436535a2ccb501378ea855e3e031471743b0308311f81d151"
    );
    let (_directory, helper) = fixture();
    let importer = importer(&helper);
    write_owned(&helper).unwrap();
    verify_owned(&helper, &[importer]).unwrap();
    assert_eq!(fs::read(&helper.path).unwrap(), HELPER_BODY.as_bytes());
    let modified = fs::metadata(&helper.path).unwrap().modified().unwrap();
    write_owned(&helper).unwrap();
    assert_eq!(
        fs::metadata(&helper.path).unwrap().modified().unwrap(),
        modified
    );
}

#[cfg(unix)]
#[test]
fn missing_wrong_or_nonregular_body_refuses_without_repair() {
    let (_directory, helper) = fixture();
    assert!(verify_owned(&helper, &[]).is_err());
    write_owned(&helper).unwrap();
    let mut changed = HELPER_BODY.as_bytes().to_vec();
    changed[0] = b'T';
    fs::write(&helper.path, &changed).unwrap();
    assert!(verify_owned(&helper, &[]).is_err());
    assert!(write_owned(&helper).is_err());
    assert_eq!(fs::read(&helper.path).unwrap(), changed);
    fs::remove_file(&helper.path).unwrap();
    fs::create_dir(&helper.path).unwrap();
    assert!(write_owned(&helper).is_err());
    assert!(helper.path.is_dir());
}

#[cfg(unix)]
#[test]
fn every_competing_extension_and_package_directory_refuses() {
    for suffix in [".ts", ".tsx", ".js", ".jsx", ".TS", ""] {
        let (_directory, helper) = fixture();
        write_owned(&helper).unwrap();
        let name = vize_carton::cstr!("__private_return_abc{suffix}");
        let competing = helper.path.with_file_name(name.as_str());
        if suffix.is_empty() {
            fs::create_dir(&competing).unwrap();
            fs::write(competing.join("package.json"), "{}\n").unwrap();
        } else {
            fs::write(&competing, "export {};\n").unwrap();
        }
        assert!(verify_owned(&helper, &[]).is_err(), "{suffix}");
        assert!(write_owned(&helper).is_err(), "{suffix}");
        assert_eq!(fs::read(&helper.path).unwrap(), HELPER_BODY.as_bytes());
    }
    let (_directory, helper) = fixture();
    write_owned(&helper).unwrap();
    fs::create_dir(helper.path.with_file_name("__private_return_abc.js")).unwrap();
    assert!(verify_owned(&helper, &[]).is_err());
}

#[cfg(unix)]
#[test]
fn case_collisions_and_unknown_unicode_spellings_refuse() {
    let (_directory, helper) = fixture();
    fs::create_dir_all(helper.path.parent().unwrap()).unwrap();
    let collision = helper.path.with_file_name("__PRIVATE_RETURN_ABC.d.ts");
    fs::write(&collision, HELPER_BODY).unwrap();
    assert!(write_owned(&helper).is_err());
    assert!(verify_owned(&helper, &[]).is_err());
    assert_eq!(fs::read(collision).unwrap(), HELPER_BODY.as_bytes());

    let (_directory, helper) = fixture();
    fs::create_dir(helper.root.join("APP")).unwrap();
    assert!(write_owned(&helper).is_err());

    let (_directory, helper) = fixture();
    write_owned(&helper).unwrap();
    fs::write(helper.path.with_file_name("\u{212a}.d.ts"), "export {};\n").unwrap();
    assert!(verify_owned(&helper, &[]).is_err());
}

#[cfg(unix)]
#[test]
fn out_of_root_parent_segments_invalid_suffix_and_missing_importer_refuse() {
    let (_directory, mut helper) = fixture();
    let outside = tempfile::tempdir().unwrap();
    helper.path = outside.path().join("outside.d.ts");
    assert!(write_owned(&helper).is_err());
    assert!(!helper.path.exists());
    helper.path = helper.root.join("app/../outside.d.ts");
    assert!(write_owned(&helper).is_err());
    helper.path = helper.root.clone();
    assert!(write_owned(&helper).is_err());
    for name in ["wrong.ts", ".d.ts", "..d.ts", "...d.ts"] {
        helper.path = helper.root.join(name);
        assert!(write_owned(&helper).is_err());
    }
    helper.path = helper.root.join("app/config/__private_return_abc.d.ts");
    write_owned(&helper).unwrap();
    assert!(verify_owned(&helper, &[outside.path().join("Consumer.ts")]).is_err());
    assert!(verify_owned(&helper, &[helper.root.join("Missing.vue.ts")]).is_err());
    assert!(verify_owned(&helper, &[helper.root.join("app/../Consumer.ts")]).is_err());
    let directory_importer = helper.root.join("Directory.vue.ts");
    fs::create_dir(&directory_importer).unwrap();
    assert!(verify_owned(&helper, &[directory_importer]).is_err());
}

#[cfg(unix)]
#[test]
fn owner_root_must_already_be_an_absolute_regular_directory() {
    let (_directory, mut helper) = fixture();
    let missing = helper.root.join("missing-root");
    helper.root = missing.clone();
    helper.path = missing.join("helper.d.ts");
    assert!(write_owned(&helper).is_err());
    assert!(!missing.exists());

    let (_directory, mut helper) = fixture();
    let file_root = helper.root.join("file-root");
    fs::write(&file_root, "unowned root bytes\n").unwrap();
    helper.root = file_root.clone();
    helper.path = file_root.join("helper.d.ts");
    assert!(write_owned(&helper).is_err());
    assert_eq!(
        fs::read_to_string(file_root).unwrap(),
        "unowned root bytes\n"
    );

    helper.root = PathBuf::from("relative-owner");
    helper.path = helper.root.join("helper.d.ts");
    assert!(write_owned(&helper).is_err());
}

#[cfg(unix)]
#[test]
fn leaf_parent_root_and_importer_links_never_authorize_writes() {
    use std::os::unix::fs::symlink;

    let (_directory, helper) = fixture();
    let outside = tempfile::tempdir().unwrap();
    let retained = outside.path().join("retained.d.ts");
    fs::write(&retained, "unowned bytes\n").unwrap();
    fs::create_dir_all(helper.path.parent().unwrap()).unwrap();
    symlink(&retained, &helper.path).unwrap();
    assert!(write_owned(&helper).is_err());
    assert!(verify_owned(&helper, &[]).is_err());
    assert!(
        fs::symlink_metadata(&helper.path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&retained).unwrap(), "unowned bytes\n");

    let (_directory, helper) = fixture();
    symlink(outside.path(), helper.root.join("app")).unwrap();
    assert!(write_owned(&helper).is_err());
    assert!(
        !outside
            .path()
            .join("config/__private_return_abc.d.ts")
            .exists()
    );

    let (_directory, helper) = fixture();
    write_owned(&helper).unwrap();
    let consumer = outside.path().join("Consumer.vue.ts");
    fs::write(&consumer, "export {};\n").unwrap();
    symlink(outside.path(), helper.root.join("imports")).unwrap();
    assert!(verify_owned(&helper, &[helper.root.join("imports/Consumer.vue.ts")]).is_err());
    let linked_importer = helper.root.join("Linked.vue.ts");
    symlink(&consumer, &linked_importer).unwrap();
    assert!(verify_owned(&helper, &[linked_importer]).is_err());

    let alias = outside.path().join("root-alias");
    symlink(&helper.root, &alias).unwrap();
    let linked_root = super::super::HelperFile {
        path: alias.join("app/config/__private_return_abc.d.ts"),
        root: alias,
        body: HELPER_BODY,
    };
    assert!(write_owned(&linked_root).is_err());
}

#[cfg(windows)]
#[test]
fn windows_volume_prefix_remains_outside_the_initial_owner_domain() {
    let (_directory, helper) = fixture();
    assert!(
        helper
            .root
            .components()
            .any(|component| matches!(component, Component::Prefix(_)))
    );
    assert!(write_owned(&helper).is_err());
    assert!(verify_owned(&helper, &[]).is_err());
    assert!(!helper.path.exists());
}
