//! Actual Croquis exports share the production descriptor revision and firewall.

#![expect(clippy::unwrap_used, reason = "test assertions")]

use vize_croquis::{Drawer, DrawerOptions};
use vize_incremental::{ResidentDocuments, SharedDescriptor};
use vize_l2::summary::{AlphaPages, Facet, SfcSummary};

fn export(descriptor: &SharedDescriptor) -> Option<AlphaPages> {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    if let Some(script) = &descriptor.script_setup {
        drawer.draw_script_setup(&script.content);
    }
    drawer.finish().alpha_pages(&descriptor.filename, None).ok()
}

const FIRST: &str = "<script setup lang='ts'>defineProps<{ label: string }>(); defineEmits<{ select: [value: number] }>(); const n = 1</script><template><p>{{n}}</p></template>";
const BODY: &str = "<script setup lang='ts'>defineProps<{ label: string }>(); defineEmits<{ select: [value: number] }>(); const n = 2</script><template><p>{{n + 1}}</p></template>";
const PROP: &str = "<script setup lang='ts'>defineProps<{ label: number }>(); defineEmits<{ select: [value: number] }>(); const n = 2</script><template><p>{{n + 1}}</p></template>";

fn read(docs: &mut ResidentDocuments, text: &str) -> SfcSummary {
    docs.interface("button", "Button.vue", text, "strict", export)
        .unwrap()
}

#[test]
fn body_edits_execute_zero_production_metadata_consumers() {
    let mut docs = ResidentDocuments::default();
    let first = docs
        .component_surface("button", "Button.vue", FIRST, "strict", export)
        .unwrap();
    let stats = docs.take_interface_stats();
    assert_eq!(
        (stats.exports, stats.consumers, stats.consumer_reuses),
        (1, 1, 0)
    );
    assert_eq!(
        docs.component_surface("button", "Button.vue", BODY, "strict", export),
        Some(first.clone())
    );
    let stats = docs.take_interface_stats();
    assert_eq!(
        (stats.exports, stats.consumers, stats.consumer_reuses),
        (1, 0, 1)
    );
    assert_eq!(
        docs.component_surface("button", "Button.vue", BODY, "strict", export),
        Some(first)
    );
    let stats = docs.take_interface_stats();
    assert_eq!(
        (stats.exports, stats.consumers, stats.consumer_reuses),
        (0, 0, 0)
    );
    assert!(
        docs.component_surface("button", "Button.vue", PROP, "strict", export)
            .is_some()
    );
    let stats = docs.take_interface_stats();
    assert_eq!(
        (stats.exports, stats.consumers, stats.consumer_reuses),
        (1, 1, 0)
    );
}

#[test]
fn editor_source_world_refreshes_at_buffer_durability_and_failed_exports_stay_stale() {
    let mut docs = ResidentDocuments::default();
    let first = docs
        .component_surface("button", "Button.vue", FIRST, "strict", export)
        .unwrap();
    let before = docs
        .interface_fingerprint("button", Facet::Prop, "label")
        .unwrap();
    let _ = docs.take_interface_stats();
    docs.set_source_world_revision(1);
    assert_eq!(
        docs.component_surface("button", "Button.vue", FIRST, "strict", export),
        Some(first)
    );
    let refreshed = docs.take_interface_stats();
    assert_eq!(
        (
            refreshed.exports,
            refreshed.consumers,
            refreshed.consumer_reuses
        ),
        (1, 0, 1)
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        Some(before)
    );
    docs.set_source_world_revision(2);
    assert_eq!(
        docs.interface("button", "Button.vue", FIRST, "strict", |_| None),
        None
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        None
    );
    assert_eq!(
        read(&mut docs, FIRST).fingerprint(Facet::Prop, "label"),
        Some(before)
    );
}

#[test]
fn production_exports_refresh_once_and_preserve_declaration_firewalls() {
    let mut docs = ResidentDocuments::default();
    let first = read(&mut docs, FIRST);
    let first_prop = docs
        .interface_fingerprint("button", Facet::Prop, "label")
        .unwrap();
    let first_emit = docs
        .interface_fingerprint("button", Facet::Emit, "select")
        .unwrap();
    assert_eq!(docs.take_interface_stats().exports, 1);
    for _ in 0..8 {
        assert_eq!(read(&mut docs, FIRST), first);
    }
    assert_eq!(docs.take_interface_stats().exports, 0);
    let body = read(&mut docs, BODY);
    assert_eq!(body, first);
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        Some(first_prop)
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Emit, "select"),
        Some(first_emit)
    );
    assert_eq!(docs.take_interface_stats().exports, 1);
    let prop = read(&mut docs, PROP);
    assert_ne!(prop, first);
    assert_ne!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        Some(first_prop)
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Emit, "select"),
        Some(first_emit)
    );
    assert_eq!(docs.take_interface_stats().exports, 1);
}

#[test]
fn project_notifications_refresh_every_provider_lazily() {
    let mut docs = ResidentDocuments::default();
    for key in ["a", "b"] {
        assert!(docs.interface(key, key, FIRST, "strict", export).is_some());
    }
    assert_eq!(docs.take_interface_stats().exports, 2);
    docs.invalidate_interfaces();
    for key in ["a", "b"] {
        assert!(docs.interface(key, key, FIRST, "strict", export).is_some());
        assert_eq!(docs.take_interface_stats().exports, 1);
    }
    assert!(
        docs.interface("a", "a", FIRST, "new tsconfig", export)
            .is_some()
    );
    assert_eq!(docs.take_interface_stats().exports, 1);
    assert!(
        docs.interface("b", "b", FIRST, "new tsconfig", export)
            .is_some()
    );
    assert_eq!(docs.take_interface_stats().exports, 1);
}

#[test]
fn rejected_revisions_never_return_a_previous_interface() {
    let mut docs = ResidentDocuments::default();
    let _first = read(&mut docs, FIRST);
    assert!(
        docs.interface("button", "Button.vue", "<template>", "strict", export)
            .is_none()
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        None
    );
    assert!(
        docs.interface("button", "Button.vue", BODY, "strict", |_| None)
            .is_none()
    );
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        None
    );
    assert!(
        docs.interface("button", "Button.vue", BODY, "strict", export)
            .is_some()
    );
    docs.close("button");
    assert_eq!(
        docs.interface_fingerprint("button", Facet::Prop, "label"),
        None
    );
    assert!(
        docs.interface("button", "Renamed.vue", BODY, "strict", export)
            .is_some()
    );
    assert_eq!(docs.take_interface_stats().exports, 3);
}
