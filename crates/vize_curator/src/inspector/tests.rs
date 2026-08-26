use super::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax,
    build_agent_report, build_diff, build_graph, build_line_diff, build_payload,
    build_playground_url, serialize_agent_report, serialize_payload,
};
use vize_s0::{String, cstr};

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

    // The `spolvero` member (P2-18) rides in the payload as an embedded
    // JSON value, so its keys serialize alphabetically here; the canonical
    // byte form stays `SpolveroFeed::to_json`, pinned in
    // `tests/spolvero_payload.rs`.
    assert_eq!(
        json.as_str(),
        r#"{"version":1,"target":"dom","selectedFile":"src/App.vue","options":{"customRenderer":false,"templateSyntax":"quirks"},"files":[{"path":"src/App.vue","source":"<template><div>msg</div></template>"}],"spolvero":{"command":"inspector","pages":[{"pass":"parse","path":"src/App.vue","stage":"s1","text":"<div>msg</div>"}],"schema_version":1}}"#
    );
    assert_eq!(
        build_playground_url("https://vizejs.dev/play/?foo=bar#old", json.as_str()).as_str(),
        "https://vizejs.dev/play/?foo=bar&tab=inspector#inspector=%7B%22version%22%3A1%2C%22target%22%3A%22dom%22%2C%22selectedFile%22%3A%22src%2FApp.vue%22%2C%22options%22%3A%7B%22customRenderer%22%3Afalse%2C%22templateSyntax%22%3A%22quirks%22%7D%2C%22files%22%3A%5B%7B%22path%22%3A%22src%2FApp.vue%22%2C%22source%22%3A%22%3Ctemplate%3E%3Cdiv%3Emsg%3C%2Fdiv%3E%3C%2Ftemplate%3E%22%7D%5D%2C%22spolvero%22%3A%7B%22command%22%3A%22inspector%22%2C%22pages%22%3A%5B%7B%22pass%22%3A%22parse%22%2C%22path%22%3A%22src%2FApp.vue%22%2C%22stage%22%3A%22s1%22%2C%22text%22%3A%22%3Cdiv%3Emsg%3C%2Fdiv%3E%22%7D%5D%2C%22schema_version%22%3A1%7D%7D"
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
            source: cstr!("<script setup>import Child from './Child'</script>\n"),
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

#[test]
fn agent_report_includes_semantic_summary_counts() {
    let files = vec![InspectorSourceFile {
        path: cstr!("src/App.vue"),
        source: String::from(
            r#"<script setup>
const count = 0
provide('count', count)
</script>
<template>
  <button id="save" @click="count++">{{ count }}</button>
</template>"#,
        ),
    }];
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files.clone(),
    );
    let json = serialize_payload(&payload).expect("payload serializes");
    let url = build_playground_url("https://vizejs.dev/play/", json.as_str());
    let report = build_agent_report(payload, url, files);
    let report_json = serialize_agent_report(&report).expect("report serializes");

    assert!(report_json.contains(r#""semantic": {"#));
    assert!(report_json.contains(r#""semanticFiles": ["#));
    assert!(report_json.contains(r#""snapshot": {"#));
    assert!(report_json.contains(r#""provides": ["#));
    assert!(report_json.contains(r#""analyzedFiles": 1"#));
    assert!(report_json.contains(r#""provideCount": 1"#));
    assert!(report_json.contains(r#""elementIdCount": 1"#));
}
