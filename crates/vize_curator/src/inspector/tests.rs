use super::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax,
    build_agent_report, build_diff, build_graph, build_line_diff, build_payload,
    build_playground_url, serialize_agent_report, serialize_payload,
};
use vize_carton::cstr;

#[test]
fn builds_inspector_payload_json_and_url() {
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Quirks,
        },
        vec![InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<template><div>msg</div></template>"),
        }],
    );
    let json = serialize_payload(&payload).expect("payload serializes");

    assert_eq!(
        json.as_str(),
        r#"{"version":1,"target":"dom","selectedFile":"src/App.vue","options":{"customRenderer":false,"templateSyntax":"quirks"},"files":[{"path":"src/App.vue","source":"<template><div>msg</div></template>"}]}"#
    );
    assert_eq!(
        build_playground_url("https://vizejs.dev/play/?foo=bar#old", json.as_str()).as_str(),
        "https://vizejs.dev/play/?foo=bar&tab=inspector#inspector=%7B%22version%22%3A1%2C%22target%22%3A%22dom%22%2C%22selectedFile%22%3A%22src%2FApp.vue%22%2C%22options%22%3A%7B%22customRenderer%22%3Afalse%2C%22templateSyntax%22%3A%22quirks%22%7D%2C%22files%22%3A%5B%7B%22path%22%3A%22src%2FApp.vue%22%2C%22source%22%3A%22%3Ctemplate%3E%3Cdiv%3Emsg%3C%2Fdiv%3E%3C%2Ftemplate%3E%22%7D%5D%7D"
    );
}

#[test]
fn builds_vapor_inspector_payload_json() {
    let payload = build_payload(
        InspectorTarget::Vapor,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        vec![InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("<template><div>msg</div></template>"),
        }],
    );
    let json = serialize_payload(&payload).expect("payload serializes");

    assert!(json.contains(r#""target":"vapor""#));
}

#[test]
fn builds_graph_edges_for_relative_imports() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("import Child from './Child.vue'\n"),
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
fn builds_line_diff_and_stats() {
    let diff = build_diff("one\ntwo\nthree", "one\nTWO\nthree\nfour");

    assert_eq!(diff.stats.additions, 2);
    assert_eq!(diff.stats.removals, 1);
    assert_eq!(diff.stats.unchanged, 2);
    assert_eq!(diff.lines.len(), 5);
    assert_eq!(diff.lines[0].kind, "same");
    assert_eq!(diff.lines[1].kind, "remove");
    assert_eq!(diff.lines[2].kind, "add");
    assert_eq!(diff.lines[4].right_line, Some(4));
}

#[test]
fn line_diff_prefers_content_matches_over_empty_line_anchors() {
    let left = "\
import { defineComponent as _defineComponent } from 'vue'
import { computed, watch } from 'vue'

// Reactive Props Destructure
export default {}";
    let right = "\
import { defineComponent as _defineComponent } from 'vue'
import {
  openBlock as _openBlock,
} from 'vue'

import { computed, watch } from 'vue'

export default {}";

    let diff = build_line_diff(left, right);
    let matched_import = diff
        .iter()
        .find(|line| line.text == "import { computed, watch } from 'vue'")
        .expect("matching import line exists");

    assert_eq!(matched_import.kind, "same");
    assert_eq!(matched_import.left_line, Some(2));
    assert_eq!(matched_import.right_line, Some(6));
    assert!(!diff.iter().any(|line| {
        line.kind == "remove" && line.text == "import { computed, watch } from 'vue'"
    }));
}

#[test]
fn builds_agent_report_with_payload_url_and_graph() {
    let files = vec![
        InspectorSourceFile {
            path: cstr!("src/App.vue"),
            source: cstr!("import Child from './Child'\n"),
        },
        InspectorSourceFile {
            path: cstr!("src/Child.vue"),
            source: cstr!("<template><span /></template>\n"),
        },
    ];
    let payload = build_payload(
        InspectorTarget::Ssr,
        InspectorOptions {
            custom_renderer: true,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files.clone(),
    );
    let json = serialize_payload(&payload).expect("payload serializes");
    let url = build_playground_url("https://vizejs.dev/play/", json.as_str());
    let report = build_agent_report(payload, url, files);
    let report_json = serialize_agent_report(&report).expect("report serializes");

    assert!(report_json.contains(r#""schema": "vize.inspector.agent""#));
    assert!(report_json.contains(r#""target": "ssr""#));
    assert!(report_json.contains(r#""to": "src/Child.vue""#));
}
