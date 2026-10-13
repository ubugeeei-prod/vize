use super::TypeSourceSnapshot;
use std::sync::Arc;

#[test]
fn overlay_text_is_shared_and_a_snapshot_freezes_disk_reads_and_misses() {
    let dir = tempfile::tempdir().unwrap();
    let overlay = dir.path().join("open.ts");
    let disk = dir.path().join("saved.ts");
    let missing = dir.path().join("missing.ts");
    let text: Arc<str> = Arc::from("export type Open = number");
    let sources = TypeSourceSnapshot::new([(overlay.clone(), Arc::clone(&text))]);
    let read = sources.read(&overlay).unwrap();
    assert!(Arc::ptr_eq(&text, &read));
    std::fs::write(&disk, "export type Saved = string").unwrap();
    assert_eq!(
        sources.read(&disk).as_deref(),
        Some("export type Saved = string")
    );
    assert_eq!(sources.read(&missing).as_deref(), None);
    std::fs::write(&disk, "export type Saved = number").unwrap();
    std::fs::write(&missing, "export type New = boolean").unwrap();
    assert_eq!(
        sources.read(&disk).as_deref(),
        Some("export type Saved = string")
    );
    assert_eq!(sources.read(&missing).as_deref(), None);
    let fresh = TypeSourceSnapshot::default();
    assert_eq!(
        fresh.read(&disk).as_deref(),
        Some("export type Saved = number")
    );
    assert_eq!(
        fresh.read(&missing).as_deref(),
        Some("export type New = boolean")
    );
}

#[test]
fn new_unsaved_modules_follow_disk_extension_precedence_and_js_substitution() {
    let dir = tempfile::tempdir().unwrap();
    // An unsaved module keeps its canonical parent identity, so compare from
    // the physical temp root (macOS reaches it through the `/var` symlink).
    let base = dir.path().canonicalize().unwrap();
    let root = base.join("App.vue");
    let existing = base.join("api.ts");
    let unsaved_lower = base.join("api.tsx");
    let unsaved_new = base.join("new.ts");
    std::fs::write(&existing, "export type Existing = number").unwrap();
    let sources = TypeSourceSnapshot::new([
        (
            unsaved_lower,
            Arc::<str>::from("export type Lower = string"),
        ),
        (
            unsaved_new.clone(),
            Arc::<str>::from("export type New = boolean"),
        ),
    ]);
    assert_eq!(
        sources.resolve_import(&root, "./api"),
        Some(existing.canonicalize().unwrap())
    );
    assert_eq!(sources.resolve_import(&root, "./new.js"), Some(unsaved_new));
}
