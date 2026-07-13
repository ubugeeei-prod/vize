use vize_armature::parse;
use vize_atlas::{Compilation, ProductId, ProductStatus};
use vize_carton::Bump;
use vize_croquis::{CroquisDocumentProduct, CroquisSemanticSnapshot};
use vize_relief::ReliefProduct;
use vize_relief::RootNode;

use super::*;
use crate::{SfcDescriptor, SfcDescriptorProduct, SfcParseOptions, parse_sfc};

const SOURCE: &str = r#"<script>
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }} {{ $route }}</template>"#;

fn direct_snapshot(
    mode: SfcCroquisMode,
    descriptor: &SfcDescriptor<'_>,
    root: &RootNode<'_>,
) -> CroquisSemanticSnapshot {
    let analysis = match mode {
        SfcCroquisMode::Full => {
            analyze_sfc_descriptor(descriptor, Some(root), SfcCroquisOptions::full())
        }
        SfcCroquisMode::OptionsApi => {
            analyze_sfc_descriptor_with_context_options_api(
                descriptor,
                Some(root),
                SfcCroquisOptions::full(),
            )
            .croquis
        }
        SfcCroquisMode::LegacyVue2 => {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                descriptor,
                Some(root),
                SfcCroquisOptions::full(),
            )
            .croquis
        }
        SfcCroquisMode::Declaration => {
            analyze_sfc_descriptor(descriptor, None, SfcCroquisOptions::for_declaration())
        }
    };
    analysis.semantic_snapshot()
}

#[test]
fn source_mode_matches_direct_analysis_and_invalidates_only_croquis() {
    let descriptor = parse_sfc(SOURCE, SfcParseOptions::default()).unwrap();
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, &descriptor.template.as_ref().unwrap().content);
    assert!(errors.is_empty());

    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("Options.vue", SOURCE).unwrap();
    let mut snapshots = Vec::new();

    for mode in [
        SfcCroquisMode::Full,
        SfcCroquisMode::OptionsApi,
        SfcCroquisMode::LegacyVue2,
    ] {
        let mut settings = SfcCroquisSettings::default();
        settings.insert(source, mode);
        settings.install(&mut compilation).unwrap();
        let outcome = compilation.query::<CroquisDocumentProduct>(source).unwrap();
        let snapshot = outcome.value().analysis().semantic_snapshot();
        assert_eq!(outcome.status(), ProductStatus::Executed);
        assert_eq!(snapshot, direct_snapshot(mode, &descriptor, &root));
        snapshots.push(snapshot);
    }

    assert_ne!(snapshots[0], snapshots[1]);
    assert_ne!(snapshots[1], snapshots[2]);
    assert_eq!(snapshots[0].summary.undefined_ref_count, 2);
    assert_eq!(snapshots[1].summary.undefined_ref_count, 1);
    assert_eq!(snapshots[2].summary.undefined_ref_count, 0);
    assert_eq!(
        compilation
            .counters()
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        3
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
}

#[test]
fn resolved_filename_matches_imported_heritage_analysis_and_invalidates_only_croquis() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let project = std::env::temp_dir().join(
        vize_carton::cstr!(
            "vize-sfc-atlas-resolved-props-{}-{nonce}",
            std::process::id()
        )
        .as_str(),
    );
    let src = project.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        src.join("types.ts"),
        "export interface RootProps { side?: 'left' | 'right'; resizable?: boolean }",
    )
    .unwrap();
    let component = src.join("Resolved.vue");
    let source_text = r#"<script setup lang="ts">
import type { RootProps } from './types'
interface Props extends Pick<RootProps, 'side' | 'resizable'> { label?: string }
defineProps<Props>()
</script>
<template>{{ side }} {{ resizable }} {{ label }}</template>"#;

    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(component.to_string_lossy().as_ref(), source_text)
        .unwrap();

    let unresolved = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    assert!(
        unresolved
            .value()
            .analysis()
            .undefined_refs
            .iter()
            .any(|reference| reference.name == "side")
    );

    let mut settings = SfcCroquisSettings::new(SfcCroquisMode::Full);
    settings.insert_resolved_filename(source, component.to_string_lossy().as_ref());
    let invalidation = settings.install(&mut compilation).unwrap();
    assert!(
        invalidation
            .iter()
            .flat_map(|report| report.evicted())
            .any(|entry| entry.product == ProductId::of::<CroquisDocumentProduct>())
    );
    let resolved = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let analysis = resolved.value().analysis();
    for prop in ["side", "resizable", "label"] {
        assert!(analysis.bindings.bindings.contains_key(prop));
        assert!(analysis.macros.props().iter().any(|item| item.name == prop));
        assert!(
            !analysis
                .undefined_refs
                .iter()
                .any(|reference| reference.name == prop)
        );
    }
    assert_eq!(resolved.status(), ProductStatus::Executed);

    let mut compatibility = SfcCroquisSettings::new(SfcCroquisMode::Full);
    compatibility.insert_resolved_filename_with_policy(
        source,
        component.to_string_lossy().as_ref(),
        SfcResolvedPropsPolicy::PreserveCanonAfterTemplate,
    );
    compatibility.install(&mut compilation).unwrap();
    let compatibility = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let compatibility = compatibility.value().analysis();
    for prop in ["side", "resizable", "label"] {
        assert!(compatibility.bindings.bindings.contains_key(prop));
        assert!(
            compatibility
                .macros
                .props()
                .iter()
                .any(|item| item.name == prop)
        );
    }
    assert!(
        compatibility
            .undefined_refs
            .iter()
            .any(|reference| reference.name == "side"),
        "Canon compatibility deliberately retains the old post-template guard",
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        3
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    let _ = std::fs::remove_dir_all(project);
}
