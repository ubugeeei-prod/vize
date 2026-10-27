//! Source-owned sites cannot resolve through an outer binding they shadow.

use super::{ProjectFacts, RenderTree, RenderTreeReader, component_usage_targets};
use crate::{CrossFileAnalyzer, CrossFileDiagnosticKind, CrossFileOptions};
use std::path::Path;
use vize_carton::{Allocator, CompactString, String};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[derive(Debug, PartialEq, Eq)]
struct Edge {
    caller: u32,
    target: u32,
    export: CompactString,
    sites: Vec<(u32, u32)>,
}

fn analysis(script: &str, template: &str) -> vize_croquis::Croquis {
    let allocator = Allocator::default();
    let (root, errors) = vize_armature::Parser::new(&allocator, template).parse();
    assert!(errors.is_empty(), "{template}: {errors:?}");
    let mut drawer = Analyzer::with_options(AnalyzerOptions::full());
    drawer.analyze_script_setup(script);
    drawer.analyze_template(&root);
    drawer.finish()
}

fn project(
    script: &str,
    template: &str,
    child_script: &str,
    child_name: &str,
) -> CrossFileAnalyzer {
    let mut project = CrossFileAnalyzer::new(
        CrossFileOptions::default()
            .with_props_validation(true)
            .with_provide_inject(true),
    );
    project.add_file_with_analysis(
        Path::new(child_name),
        child_script,
        analysis(child_script, "<p></p>"),
    );
    let mut source = String::from("<script setup>");
    source.push_str(script);
    source.push_str("</script><template>");
    source.push_str(template);
    source.push_str("</template>");
    project.add_file_with_analysis(
        Path::new("Page.vue"),
        source.as_str(),
        analysis(script, template),
    );
    project.rebuild_component_edges();
    project
}

fn edge_packet(project: &CrossFileAnalyzer) -> Vec<Edge> {
    let mut facts = ProjectFacts::new(project.registry());
    let table = facts
        .prepare::<RenderTreeReader>()
        .get::<RenderTree>()
        .expect("render table");
    table
        .iter()
        .map(|(edge, sites)| Edge {
            caller: edge.caller.as_u32(),
            target: edge.target.as_u32(),
            export: edge.export_name.clone(),
            sites: sites.iter().map(|site| (site.start, site.end)).collect(),
        })
        .collect()
}

#[test]
fn template_shadowed_async_and_static_sites_do_not_revive_name_fallbacks() {
    for script in [
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => import('./NoticeLink.vue')); const items = [];",
        "import Lazy from './NoticeLink.vue'; const items = [];",
    ] {
        for template in [
            "<Lazy v-for=\"Lazy in items\"/>",
            "<template v-for=\"Lazy in items\"><Lazy/></template>",
            "<Provider v-slot=\"{ Lazy }\"><Lazy/></Provider>",
            "<component :is=\"Lazy\" v-for=\"Lazy in items\"/>",
            "<lazy v-for=\"Lazy in items\"/>",
        ] {
            let project = project(script, template, "", "NoticeLink.vue");
            assert_eq!(edge_packet(&project), vec![], "{script} {template}");
            assert_eq!(
                component_usage_targets(project.registry(), crate::FileId::new(1)),
                vec![],
                "{script} {template}"
            );
        }
    }
}

#[test]
fn mixed_template_sites_keep_only_the_unshadowed_root_site() {
    for script in [
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => import('./NoticeLink.vue')); const items = [];",
        "import Lazy from './NoticeLink.vue'; const items = [];",
    ] {
        let project = project(
            script,
            "<Lazy/><template v-for=\"Lazy in items\"><Lazy/></template>",
            "",
            "NoticeLink.vue",
        );
        assert_eq!(
            edge_packet(&project),
            vec![Edge {
                caller: 1,
                target: 0,
                export: "default".into(),
                sites: vec![(0, 7)]
            }]
        );
        assert_eq!(
            component_usage_targets(project.registry(), crate::FileId::new(1)),
            vec![crate::FileId::new(0)]
        );
    }
}

#[test]
fn shadowed_prop_sites_do_not_reuse_the_root_component_contract() {
    let mut project = project(
        "import Lazy from './NoticeLink.vue'; const items = [];",
        "<Lazy title=\"ok\"/><template v-for=\"Lazy in items\"><Lazy :title=\"123\"/></template>",
        "defineProps<{ title: string }>()",
        "NoticeLink.vue",
    );
    assert!(project.analyze().props_validation_issues.is_empty());
}

#[test]
fn shadowed_global_component_sites_do_not_create_provide_ancestry() {
    for (template, expected_matches, expected_unmatched) in [
        ("<Lazy/>", vec![(1, 0, "token")], vec![]),
        (
            "<Lazy v-for=\"Lazy in items\"/>",
            vec![],
            vec![(0, "token")],
        ),
        (
            "<Provider v-slot=\"{ Lazy }\"><Lazy/></Provider>",
            vec![],
            vec![(0, "token")],
        ),
    ] {
        let mut project = project(
            "import { provide } from 'vue'; provide('token', 1); const items = [];",
            template,
            "import { inject } from 'vue'; const value = inject('token');",
            "Lazy.vue",
        );
        let result = project.analyze();
        let matches: Vec<_> = result
            .provide_inject_matches
            .iter()
            .map(|row| {
                (
                    row.provider.as_u32(),
                    row.consumer.as_u32(),
                    row.key.as_str(),
                )
            })
            .collect();
        assert_eq!(matches, expected_matches, "{template}");
        let missing: Vec<_> = result
            .diagnostics
            .iter()
            .filter_map(|diagnostic| match &diagnostic.kind {
                CrossFileDiagnosticKind::UnmatchedInject { key } => {
                    Some((diagnostic.primary_file.as_u32(), key.as_str()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(missing, expected_unmatched, "{template}");
    }
}

#[test]
fn own_slot_scope_does_not_shadow_its_host_component() {
    let template = "<Lazy v-slot=\"{ Lazy }\"><Lazy/></Lazy>";
    for script in [
        "import Lazy from './NoticeLink.vue';",
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => import('./NoticeLink.vue'));",
    ] {
        let project = project(script, template, "", "NoticeLink.vue");
        assert_eq!(
            edge_packet(&project),
            vec![Edge {
                caller: 1,
                target: 0,
                export: "default".into(),
                sites: vec![(0, template.len() as u32)],
            }],
            "{script}"
        );
        assert_eq!(
            component_usage_targets(project.registry(), crate::FileId::new(1)),
            vec![crate::FileId::new(0)]
        );
    }
}
