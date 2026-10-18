//! Filesystem and overlay mutations remain authoritative with warm syntax facts.

use super::*;

#[test]
fn warm_stamp_facts_observe_missing_priority_content_config_and_overlay_changes() {
    let fixture = Fixture::new();
    let overlays = FxHashMap::default();
    let requested = fixture.requested(false);
    let mut facts = ModuleSpecifierFactsCache::default();
    let context = fixture.build(&overlays, &requested, &mut facts);
    let mut previous = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);

    let late = fixture.root.join("src/late.ts");
    assert!(!late.exists());
    write(&late, "export const late = 1;");
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    let appeared = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    assert_ne!(previous, appeared);
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    assert!(
        fresh
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&late)
            .is_some()
    );
    previous = appeared;
    std::fs::remove_file(&late).unwrap();
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    let deleted = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    assert_ne!(previous, deleted);
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    assert!(
        fresh
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&late)
            .is_none()
    );
    previous = deleted;
    write(&late, "export const late = 2;");
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    previous = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    assert!(
        fresh
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&late)
            .unwrap()
            .content
            .contains("late = 2")
    );

    // A `.ts` appearing beside an existing `.js` wins TypeScript extension
    // substitution. Reusing syntax must still rerun every failed probe.
    let priority = fixture.root.join("src/priority.ts");
    assert!(!priority.exists());
    write(&priority, "export const priority = 2;");
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    let changed = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    assert_ne!(previous, changed);
    assert_eq!(
        crate::batch::virtual_project::dependency_scan::resolve_dependency(
            "./priority.js",
            &fixture.root.join("src"),
            &fixture.root,
            &context.aliases,
        ),
        Some(priority.clone())
    );
    previous = changed;
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    let mirror = fresh.mirror.as_ref().unwrap();
    assert!(mirror.find_by_original(&priority).is_some());
    assert!(
        mirror
            .find_by_original(&fixture.root.join("src/priority.js"))
            .is_none()
    );

    let base = fixture.root.join("src/base.ts");
    let modified = std::fs::metadata(&base).unwrap().modified().unwrap();
    let old_content = std::fs::read_to_string(&base).unwrap();
    let edited = old_content.replace("1", "2");
    assert_eq!(old_content.len(), edited.len());
    write(&base, &edited);
    std::fs::File::options()
        .write(true)
        .open(&base)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    let changed = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    assert_ne!(previous, changed);
    previous = changed;
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    assert!(
        fresh
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&base)
            .unwrap()
            .content
            .contains("base = 2")
    );

    write(
        &fixture.root.join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"Bundler","customConditions":["development"],"paths":{"@/*":["./src/*"]}}}"#,
    );
    assert!(previous.iter().any(|stamp| !stamp.is_current()));
    assert_ne!(
        previous,
        stamp_equal(&fixture, &context, &overlays, &requested, &mut facts)
    );
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    let bindings = fresh.mirror.as_ref().unwrap().package_routes_snapshot();
    assert!(!bindings.is_empty());
    assert!(
        bindings
            .iter()
            .all(|binding| binding.context.active_conditions == ["development"])
    );

    let child = fixture.root.join("src/Child.vue");
    let overlay_source = r#"<script setup lang="ts">import Only from './OverlayOnly.vue'; defineProps<{ value: number }>()</script><template><Only />{{ value }}</template>"#;
    let overlay = FxHashMap::from_iter([(child.clone(), overlay_source)]);
    write(&fixture.root.join("src/OverlayOnly.vue"), "<template />");
    let mut overlay_facts = ModuleSpecifierFactsCache::default();
    let overlay_context = fixture.build(&overlay, &requested, &mut overlay_facts);
    let mut uncached = ModuleSpecifierFactsCache::uncached();
    let reference = fixture.build(&overlay, &requested, &mut uncached);
    assert_context_equal(&overlay_context, &reference);
    stamp_equal(
        &fixture,
        &overlay_context,
        &overlay,
        &requested,
        &mut overlay_facts,
    );
    assert_ne!(
        fixture.fingerprint(&overlays, &requested).identity(),
        fixture.fingerprint(&overlay, &requested).identity()
    );
    assert_eq!(std::fs::read_to_string(&child).unwrap(), CHILD);
    assert!(
        overlay_context
            .mirror
            .as_ref()
            .unwrap()
            .find_by_original(&fixture.root.join("src/OverlayOnly.vue"))
            .is_some()
    );
}

#[cfg(unix)]
#[test]
fn warm_stamp_facts_reobserve_a_retargeted_symlink() {
    let fixture = Fixture::new();
    let overlays = FxHashMap::default();
    let requested = fixture.requested(false);
    let mut facts = ModuleSpecifierFactsCache::default();
    let context = fixture.build(&overlays, &requested, &mut facts);
    let before = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
    let link = fixture.root.join("src/link.ts");
    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink("linked-b.ts", &link).unwrap();
    assert!(before.iter().any(|stamp| !stamp.is_current()));
    assert_ne!(
        before,
        stamp_equal(&fixture, &context, &overlays, &requested, &mut facts)
    );
    let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
    let mirror = fresh.mirror.as_ref().unwrap();
    assert!(
        mirror
            .find_by_original(&fixture.root.join("src/linked-b.ts"))
            .is_some()
    );
    assert!(
        mirror
            .find_by_original(&fixture.root.join("src/linked-a.ts"))
            .is_none()
    );
}
