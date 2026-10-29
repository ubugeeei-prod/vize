//! Render ancestry requires both proven Vue factory and exact loader return.
use super::{ProjectFacts, RenderTree, RenderTreeReader};
use crate::{CrossFileAnalyzer, CrossFileOptions};
use std::path::Path;
use vize_carton::{CompactString, smallvec};
use vize_croquis::{AnalyzerOptions, ScopeId, analysis::ComponentUsage};

fn analysis(source: &str, tag: Option<&str>) -> vize_croquis::Croquis {
    let mut drawer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    drawer.analyze_script_setup(source);
    let mut result = drawer.finish();
    if let Some(tag) = tag {
        result.note_used_component(CompactString::new(tag));
        result.note_component_usage(ComponentUsage {
            name: CompactString::new(tag),
            start: 3,
            end: 25,
            props: smallvec![],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            spread_props: smallvec![],
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
    }
    result
}

fn assert_target(source: &str, tag: &str, expected: bool) {
    let mut project = CrossFileAnalyzer::new(CrossFileOptions::default());
    let target =
        project.add_file_with_analysis(Path::new("src/NoticeLink.vue"), "", analysis("", None));
    let parent = project.add_file_with_analysis(
        Path::new("src/Page.vue"),
        source,
        analysis(source, Some(tag)),
    );
    project.rebuild_component_edges();
    let mut facts = ProjectFacts::new(project.registry());
    let table = facts
        .prepare::<RenderTreeReader>()
        .get::<RenderTree>()
        .expect("render table");
    let actual: Vec<_> = table
        .iter()
        .map(|(key, _)| (key.caller, key.target, key.export_name.as_str()))
        .collect();
    let expected_rows = if expected {
        vec![(parent, target, "default")]
    } else {
        vec![]
    };
    assert_eq!(actual, expected_rows, "{source}");
}

#[test]
fn async_component_ancestry_accepts_alias_namespace_and_exact_loader_forms() {
    for source in [
        "import { defineAsyncComponent } from 'vue'; const Lazy = defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => import('./NoticeLink.vue'))",
        "import * as Vue from 'vue'; const Lazy = Vue.defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import * as Vue from 'vue'; const V = Vue; const Lazy = V.defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const factory = dac; const Lazy = factory(() => import('./NoticeLink.vue'))",
        "import * as Vue from 'vue'; const factory = Vue.defineAsyncComponent; const Lazy = factory(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const load = () => import('./NoticeLink.vue'); const alias = load; const Lazy = dac(alias)",
        "import { defineAsyncComponent as dac } from 'vue'; function load() { return import('./NoticeLink.vue'); } const Lazy = dac(load)",
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac({ loader: () => import('./NoticeLink.vue'), delay: 10 })",
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => { return import('./NoticeLink.vue'); })",
        "import { defineAsyncComponent as dac } from 'vue'; const Original = dac(() => import('./NoticeLink.vue')); const Lazy = Original",
    ] {
        assert_target(source, "Lazy", true);
    }
}

#[test]
fn async_component_ancestry_rejects_fake_opaque_nonliteral_and_uncalled_nested_loaders() {
    for source in [
        "import { defineAsyncComponent } from 'fake-vue'; const NoticeLink = defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "const defineAsyncComponent = (loader) => ({}); const NoticeLink = defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import type { defineAsyncComponent } from 'vue'; const NoticeLink = defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; function make(dac) { return dac(() => import('./NoticeLink.vue')); } const NoticeLink = make(fake)",
        "import * as Vue from 'vue'; function make(Vue) { return Vue.defineAsyncComponent(() => import('./NoticeLink.vue')); } const NoticeLink = make(fake)",
        "import { defineAsyncComponent as dac } from 'vue'; let NoticeLink = dac(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const path = './NoticeLink.vue'; const NoticeLink = dac(() => import(path))",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => { const nested = () => import('./NoticeLink.vue'); return {}; })",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => () => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const options = {loader: () => import('./NoticeLink.vue')}; const NoticeLink = dac(options)",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac({ ...opaque, loader: () => import('./NoticeLink.vue') })",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac({ loader: opaque, loader: () => import('./NoticeLink.vue') })",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => import('./NoticeLink.vue')); NoticeLink = opaque",
        "import * as Vue from 'vue'; Vue.defineAsyncComponent = fake; const NoticeLink = Vue.defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import * as Vue from 'vue'; const V = Vue; delete V.defineAsyncComponent; const NoticeLink = Vue.defineAsyncComponent(() => import('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const load = later; const later = () => import('./NoticeLink.vue'); const NoticeLink = dac(load)",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = unknown()",
        "const NoticeLink = () => import('./NoticeLink.vue')",
        "import Original from './Other.vue'; const NoticeLink = Original",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => import.source('./NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; function loader() { return import('./NoticeLink.vue'); } for (loader of [() => import('./Other.vue')]) {} const NoticeLink = dac(loader)",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => import('@/NoticeLink.vue'))",
        "import { defineAsyncComponent as dac } from 'vue'; const NoticeLink = dac(() => import('./NoticeLink'))",
    ] {
        assert_target(source, "NoticeLink", false);
    }
}

#[test]
fn async_component_ancestry_keeps_static_import_and_kebab_alias_controls() {
    assert_target("import Lazy from './NoticeLink.vue'", "Lazy", true);
    assert_target(
        "import { defineAsyncComponent as dac } from 'vue'; const Lazy = dac(() => import('./NoticeLink.vue'))",
        "lazy",
        true,
    );
}

#[test]
fn opaque_loader_reads_preserve_an_independent_proven_component() {
    let source = "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); const path = './NoticeLink.vue'; const NoticeLink = dac(() => import(path));";
    assert_target(source, "Healthy", true);
    assert_target(source, "NoticeLink", false);
}

#[test]
fn nested_constructor_and_template_writes_only_invalidate_the_target_loader() {
    for source in [
        "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); function loader() { return import('./NoticeLink.vue'); } new Wrapper(loader = () => import('./Other.vue')); const Changed = dac(loader);",
        "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); function loader() { return import('./NoticeLink.vue'); } new Wrapper(...[loader = () => import('./Other.vue')]); const Changed = dac(loader);",
        "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); function loader() { return import('./NoticeLink.vue'); } tag`${loader = () => import('./Other.vue')}`; const Changed = dac(loader);",
        "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); function loader() { return import('./NoticeLink.vue'); } (loader = tag)``; const Changed = dac(loader);",
        "import { defineAsyncComponent as dac } from 'vue'; const Healthy = dac(() => import('./NoticeLink.vue')); function loader() { return import('./NoticeLink.vue'); } `value ${loader = () => import('./Other.vue')}`; const Changed = dac(loader);",
    ] {
        assert_target(source, "Healthy", true);
        assert_target(source, "Changed", false);
    }
}
