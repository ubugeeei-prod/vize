use std::fs;
use tower_lsp::lsp_types::Url;

use super::pinia_is_available;

#[test]
fn pinia_availability_tracks_ancestor_package_identity() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("src/nested/App.vue");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    assert!(!pinia_is_available(&uri));

    let sibling = root.path().join("other/node_modules/pinia");
    fs::create_dir_all(&sibling).unwrap();
    fs::write(sibling.join("package.json"), "{}").unwrap();
    assert!(!pinia_is_available(&uri));

    let package = root.path().join("node_modules/pinia");
    fs::create_dir_all(&package).unwrap();
    assert!(!pinia_is_available(&uri));
    fs::write(package.join("package.json"), "{}").unwrap();
    assert!(pinia_is_available(&uri));
    fs::remove_file(package.join("package.json")).unwrap();
    assert!(!pinia_is_available(&uri));
}

#[test]
fn non_file_documents_keep_the_existing_unknown_project_policy() {
    assert!(pinia_is_available(&Url::parse("untitled:App.vue").unwrap()));
}
