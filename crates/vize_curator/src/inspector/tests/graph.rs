use super::super::{InspectorSourceFile, build_graph};
use vize_carton::{String, cstr};

#[test]
fn builds_graph_edges_for_relative_imports() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<script setup>import Child from './Child.vue'</script>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Child.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];

    let graph = build_graph(&files);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from.as_str(), "src/App.vue");
    assert_eq!(graph.edges[0].to.as_str(), "src/Child.vue");
    assert_eq!(graph.edges[0].kind, "import");
}

#[test]
fn builds_graph_component_edges_for_used_imports() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("./src/App.vue"),
            source: cstr!(
                "<script setup>import ChildCard from './ChildCard.vue'</script><template><child-card /></template>"
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/ChildCard.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];

    let graph = build_graph(&files);

    assert_eq!(graph.nodes[0].path.as_str(), "src/App.vue");
    assert_eq!(graph.edges.len(), 2);
    assert_eq!(graph.edges[0].kind, "component");
    assert_eq!(graph.edges[1].kind, "import");
}

#[test]
fn builds_component_edges_for_jsx_namespace_members() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.tsx"),
            source: cstr!(
                "import * as Cards from './Cards.tsx'; export const App = () => <Cards.Button />;"
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/Cards.tsx"),
            source: cstr!("export const Button = () => <button />;"),
        },
    ];

    let graph = build_graph(&files);
    let component_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "component")
        .collect();

    assert_eq!(component_edges.len(), 1);
    assert_eq!(component_edges[0].to.as_str(), "src/Cards.tsx");
}

#[test]
fn graph_ignores_import_meta_and_type_only_component_imports() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!(
                "<script setup lang=\"ts\">
const mode = import.meta.env.MODE;
import type TypeOnly from './TypeOnly.vue';
import RuntimeOnly from './RuntimeOnly.vue';
</script>
<template><TypeOnly /><RuntimeOnly /></template>"
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/TypeOnly.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/RuntimeOnly.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];

    let graph = build_graph(&files);
    let component_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "component")
        .collect();

    assert_eq!(component_edges.len(), 1);
    assert_eq!(component_edges[0].to.as_str(), "src/RuntimeOnly.vue");
}

#[test]
fn graph_ignores_inline_type_only_import_specifiers() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.tsx"),
            source: String::from(
                "import { type Child } from './Child.vue'; export const App = () => <main />;",
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/Child.vue"),
            source: cstr!("<template><span /></template>"),
        },
    ];

    assert!(build_graph(&files).edges.is_empty());
}

#[test]
fn graph_uses_ast_for_imports_and_template_component_usage() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!(
                r#"<script setup>
// import Ghost from './Ghost.vue'
const example = "import Phantom from './Phantom.vue'; <Hidden />";
import RuntimeOnly from './RuntimeOnly.vue';
import Hidden from './Hidden.vue';
const Lazy = () => import('./Lazy.vue');
</script>
<template>
  <RuntimeOnly />
</template>"#
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/RuntimeOnly.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Hidden.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Lazy.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Ghost.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Phantom.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];

    let graph = build_graph(&files);
    let mut edges: Vec<_> = graph
        .edges
        .iter()
        .map(|edge| (edge.kind, edge.to.as_str()))
        .collect();
    edges.sort_unstable();

    assert!(edges.contains(&("component", "src/RuntimeOnly.vue")));
    assert!(edges.contains(&("dynamic-import", "src/Lazy.vue")));
    assert!(edges.contains(&("import", "src/Hidden.vue")));
    assert!(edges.contains(&("import", "src/RuntimeOnly.vue")));
    assert!(!edges.contains(&("component", "src/Hidden.vue")));
    assert!(!edges.iter().any(|(_, to)| *to == "src/Ghost.vue"));
    assert!(!edges.iter().any(|(_, to)| *to == "src/Phantom.vue"));
}

#[test]
fn graph_resolves_modern_script_module_extensions() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!(
                r#"<script setup>
import EntryPanel from './entry';
import config from './config.cts';
const loadServer = () => import('./server');
</script>
<template><EntryPanel /></template>"#
            ),
        },
        InspectorSourceFile {
            path: cstr!("src/entry/index.tsx"),
            source: cstr!("export default null;\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/config.cts"),
            source: cstr!("export default true;\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/server/index.mjs"),
            source: cstr!("export default true;\n"),
        },
    ];

    let graph = build_graph(&files);
    let mut edges: Vec<_> = graph
        .edges
        .iter()
        .map(|edge| (edge.kind, edge.to.as_str()))
        .collect();
    edges.sort_unstable();

    assert!(edges.contains(&("component", "src/entry/index.tsx")));
    assert!(edges.contains(&("dynamic-import", "src/server/index.mjs")));
    assert!(edges.contains(&("import", "src/config.cts")));
    assert!(edges.contains(&("import", "src/entry/index.tsx")));
}
