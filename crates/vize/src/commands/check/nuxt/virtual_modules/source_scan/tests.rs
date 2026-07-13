use vize_atelier_sfc::{SfcDescriptorProduct, SfcParseOptions, parse_sfc};
use vize_atlas::{ObservationKind, ProductStatus};

use super::*;

fn entries() -> Vec<(PathBuf, String)> {
    vec![
        (
            PathBuf::from("app/pages/index.vue"),
            r##"<script setup lang="ts">
import { useI18n, type Breakpoint } from "#imports"
import { VFButton } from "#components"
</script>"##
                .into(),
        ),
        (
            PathBuf::from("app/pages/broken.vue"),
            "<template /><template /><script>import { ignored } from '#imports'</script>".into(),
        ),
        (
            PathBuf::from("server/router.ts"),
            "import router, { useRoute } from '@typed-router'".into(),
        ),
    ]
}

#[test]
fn all_vue_sources_share_one_snapshot_and_descriptor_cache() {
    let graph = SourceScanGraph::new(entries());
    let vue_sources: Vec<_> = graph
        .sources
        .iter()
        .filter_map(|source| source.sfc)
        .collect();
    assert_eq!(vue_sources.len(), 2);
    assert_eq!(graph.snapshot.sources().iter().count(), 2);

    let first = graph.descriptor(vue_sources[0]).unwrap();
    let second = graph.descriptor(vue_sources[0]).unwrap();
    assert_eq!(first.status(), ProductStatus::Executed);
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert!(first.trace().executed::<SfcDescriptorProduct>());
    assert!(second.trace().cache_hit::<SfcDescriptorProduct>());
    assert_eq!(
        first
            .execution()
            .observations()
            .iter()
            .filter(|observation| observation.kind() == ObservationKind::Fallback)
            .count(),
        0
    );
}

#[test]
fn atlas_scan_preserves_legacy_import_stub_inputs() {
    let entries = entries();
    let actual = SourceScanGraph::new(entries.clone()).collect();
    let mut expected = FxHashMap::default();
    for (path, source) in entries {
        if is_vue_source(&path) {
            let Ok(descriptor) = parse_sfc(
                source.as_str(),
                SfcParseOptions {
                    filename: path.to_string_lossy().as_ref().into(),
                    ..Default::default()
                },
            ) else {
                continue;
            };
            for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
                .into_iter()
                .flatten()
            {
                collect_script(
                    script.content.as_ref(),
                    source_type_for_script_lang(script.lang.as_deref()),
                    &mut expected,
                );
            }
        } else {
            collect_script(&source, source_type_for_path(&path), &mut expected);
        }
    }

    assert_eq!(actual, expected);
    assert_eq!(
        super::super::render_module_stub("#components", &actual["#components"])
            .unwrap()
            .as_str(),
        "declare module \"#components\" {\n  export const VFButton: any;\n  export type VFButton<T = any, T1 = any, T2 = any, T3 = any> = any;\n}\n"
    );
}

#[test]
fn malformed_sfc_is_cached_and_contributes_no_imports() {
    let graph = SourceScanGraph::new(entries());
    let broken = graph.sources[1].sfc.unwrap();
    let first = graph.descriptor(broken).unwrap();
    let imports = graph.collect();
    let second = graph.descriptor(broken).unwrap();

    assert!(first.value().diagnostic().is_some());
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert!(!imports["#imports"].named.contains("ignored"));
}
