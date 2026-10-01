//! TS-42 production alpha projection, using the same exporter as Maestro.

use std::path::{Path, PathBuf};

use vize_l2::summary::{Facet, SfcSummary};
use vize_resident::{ResidentDocuments, parse_descriptor};

use super::super::export_component_interface;
use super::edits::replace_member_type;

const ROOTS: [&str; 5] = [
    "tests/_fixtures/_git/splitpanes",
    "tests/_fixtures/_git/layoutit-grid",
    "tests/_fixtures/_git/cssgridgenerator",
    "tests/_fixtures/_projects",
    "tests/_fixtures/vue-language-tools",
];

fn clean(filename: &str, source: &str) -> Option<SfcSummary> {
    let descriptor = parse_descriptor(filename, source)?;
    let pages = export_component_interface(&descriptor, filename, true, false)?;
    SfcSummary::from_alpha(pages).ok()
}

fn read(docs: &mut ResidentDocuments, filename: &str, source: &str) -> Option<SfcSummary> {
    docs.interface(filename, filename, source, "strict", |descriptor| {
        export_component_interface(descriptor, filename, true, false)
    })
}

fn vue_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = ignore::WalkBuilder::new(root)
        .git_ignore(false)
        .hidden(false)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "vue"))
        .map(ignore::DirEntry::into_path)
        .collect();
    files.sort();
    files
}

#[test]
#[ignore = "requires the hydrated TS-42 corpus shard; Actions runs this explicitly"]
fn production_alpha_corpus_edits_match_clean() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut docs = ResidentDocuments::default();
    let mut compared = 0_u32;
    let mut body_edits = 0_u32;
    let mut rejected = 0_u32;
    for relative in ROOTS {
        let files = vue_files(&root.join(relative));
        assert!(!files.is_empty(), "TS-42 corpus root is empty: {relative}");
        let mut accepted = 0_u32;
        for path in files {
            let source = std::fs::read_to_string(&path).unwrap();
            let filename = path.to_string_lossy();
            let first = read(&mut docs, &filename, &source);
            assert_eq!(first, clean(&filename, &source), "initial {filename}");
            compared += 1;
            if first.is_none() {
                rejected += 1;
                continue;
            }
            accepted += 1;
            let descriptor = parse_descriptor(&filename, &source).unwrap();
            let Some(template) = descriptor
                .template
                .as_ref()
                .filter(|block| block.lang.is_none())
            else {
                continue;
            };
            let mut edited = source.clone();
            edited.insert_str(template.loc.end, "<!-- davinci body edit -->");
            let body = read(&mut docs, &filename, &edited);
            assert_eq!(body, clean(&filename, &edited), "body {filename}");
            assert_eq!(body, first, "body dirtied an interface: {filename}");
            body_edits += 1;
            let _before = docs.take_interface_stats();
            docs.invalidate_interfaces();
            assert_eq!(
                read(&mut docs, &filename, &edited),
                body,
                "config {filename}"
            );
            assert_eq!(docs.take_interface_stats().exports, 1);
            docs.close(&filename);
            assert_eq!(
                read(&mut docs, &filename, &source),
                first,
                "reopen {filename}"
            );
        }
        assert!(accepted > 0, "no accepted interfaces in {relative}");
    }
    assert!(compared > 0 && body_edits > 0);
    println!(
        "TS-42 production alpha: files={compared} body_edits={body_edits} rejected={rejected} mismatches=0"
    );
}

#[test]
fn production_alpha_interface_scripts_isolate_used_declarations() {
    let mut docs = ResidentDocuments::default();
    let initial = "<script setup lang='ts'>type Public = { value: string }; defineProps<{ label: string }>(); defineEmits<{ select: [value: number] }>(); defineSlots<{ header(props: { id: string }): any }>(); const count = ref(1); const item: Public = { value: 'a' }; defineExpose({ publicCount: count, item })</script><template><slot name='header'/></template>";
    let before = read(&mut docs, "Button.vue", initial).unwrap();
    let emit = before.fingerprint(Facet::Emit, "select");
    let slot = before.fingerprint(Facet::Slot, "header");
    let exposed = before.fingerprint(Facet::Reactivity, "publicCount");
    assert!(emit.is_some() && slot.is_some() && exposed.is_some());
    for edited in [
        replace_member_type(initial, "label", "number"),
        replace_member_type(initial, "label", "'a b'"),
        replace_member_type(initial, "label", "'ab'"),
    ] {
        let served = read(&mut docs, "Button.vue", &edited).unwrap();
        assert_eq!(Some(served.clone()), clean("Button.vue", &edited));
        assert_ne!(
            served.fingerprint(Facet::Prop, "label"),
            before.fingerprint(Facet::Prop, "label")
        );
        assert_eq!(served.fingerprint(Facet::Emit, "select"), emit);
        assert_eq!(served.fingerprint(Facet::Slot, "header"), slot);
        assert_eq!(
            served.fingerprint(Facet::Reactivity, "publicCount"),
            exposed
        );
    }
    let edited = replace_member_type(initial, "select", "[value: string]");
    let served = read(&mut docs, "Button.vue", &edited).unwrap();
    assert_eq!(Some(served.clone()), clean("Button.vue", &edited));
    assert_ne!(served.fingerprint(Facet::Emit, "select"), emit);
    assert_eq!(
        served.fingerprint(Facet::Prop, "label"),
        before.fingerprint(Facet::Prop, "label")
    );
    assert_eq!(served.fingerprint(Facet::Slot, "header"), slot);
    let edited = replace_member_type(initial, "value", "number");
    let served = read(&mut docs, "Button.vue", &edited).unwrap();
    assert_eq!(Some(served.clone()), clean("Button.vue", &edited));
    assert_ne!(
        served.fingerprint(Facet::Reactivity, "item"),
        before.fingerprint(Facet::Reactivity, "item"),
        "before item={:?}; after item={:?}; edited source={edited}",
        reactivity_contract(initial, "item"),
        reactivity_contract(&edited, "item")
    );
    assert_eq!(
        served.fingerprint(Facet::Reactivity, "publicCount"),
        exposed
    );
    assert_eq!(
        served.fingerprint(Facet::Prop, "label"),
        before.fingerprint(Facet::Prop, "label")
    );
    assert_eq!(served.fingerprint(Facet::Emit, "select"), emit);
    assert_eq!(served.fingerprint(Facet::Slot, "header"), slot);
}

fn reactivity_contract(source: &str, name: &str) -> Option<String> {
    let descriptor = parse_descriptor("Button.vue", source)?;
    export_component_interface(&descriptor, "Button.vue", true, false)?
        .reactivity
        .into_iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.contract.to_string())
}

#[test]
fn production_expanded_prop_tracks_its_external_module_helper() {
    let directory = tempfile::tempdir().expect("temporary type module");
    let types = directory.path().join("types.ts");
    let component = directory.path().join("Component.vue");
    let filename = component.to_str().expect("UTF8 fixture path");
    let source = "<script setup lang='ts'>import type { Public as Alias } from './types'; type Helper = boolean; defineProps<Alias>(); defineEmits<{ stable: [] }>();</script>";
    let module =
        |helper| format!("type Helper = {helper}; export type Public = {{ value: Helper }};");
    std::fs::write(&types, module("string")).expect("type source");
    let before = clean(filename, source).expect("initial interface");
    std::fs::write(&types, module("number")).expect("changed type source");
    let after = clean(filename, source).expect("changed interface");
    assert_ne!(
        before.fingerprint(Facet::Prop, "value"),
        after.fingerprint(Facet::Prop, "value")
    );
    assert_eq!(
        before.fingerprint(Facet::Emit, "stable"),
        after.fingerprint(Facet::Emit, "stable")
    );
    let descriptor = parse_descriptor(filename, source).expect("descriptor");
    let pages = export_component_interface(&descriptor, filename, true, false).expect("pages");
    let prop: vize_croquis::croquis::alpha::PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "value")
            .expect("expanded value")
            .contract,
    )
    .expect("prop contract");
    assert!(
        prop.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Helper"
                && dependency.body.as_deref() == Some("number")
                && dependency
                    .module
                    .as_deref()
                    .is_some_and(|module| module.ends_with("types.ts")))
    );
    assert!(
        !prop
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Helper"
                && dependency.body.as_deref() == Some("boolean"))
    );
}

#[test]
fn production_prop_catalog_completeness_preserves_known_defaults() {
    let source = "<script setup lang='ts'>withDefaults(defineProps<{ label: string; count?: number }>(), { count: 41 });</script>";
    let descriptor = parse_descriptor("Known.vue", source).expect("descriptor");
    let pages = export_component_interface(&descriptor, "Known.vue", true, false).expect("pages");
    let signature: vize_croquis::croquis::alpha::SignatureContract =
        serde_json::from_str(&pages.signature.params).expect("signature");
    assert!(signature.props_complete);
    assert_eq!(signature.with_defaults.as_deref(), Some("{ count: 41 }"));
    let count: vize_croquis::croquis::alpha::PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "count")
            .expect("count")
            .contract,
    )
    .expect("prop");
    assert_eq!(count.prop_type.as_deref(), Some("number"));
    assert_eq!(count.required, Some(false));
    assert_eq!(count.default_value.as_deref(), Some("41"));

    let source = "<script setup lang='ts'>defineProps<Missing>();</script>";
    let descriptor = parse_descriptor("Unknown.vue", source).expect("descriptor");
    let pages = export_component_interface(&descriptor, "Unknown.vue", true, false).expect("pages");
    let signature: vize_croquis::croquis::alpha::SignatureContract =
        serde_json::from_str(&pages.signature.params).expect("signature");
    assert!(!signature.props_complete);
    assert!(!signature.type_dependencies.complete);
    assert_eq!(signature.with_defaults, None);
}
