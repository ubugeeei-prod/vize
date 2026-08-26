use super::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, InspectorTemplateSyntax,
    build_agent_report, build_payload, serialize_agent_report,
};
use vize_s0::{FxHashSet, String, cstr};

#[test]
fn agent_report_preserves_semantic_scope_references() {
    let files = vec![InspectorSourceFile {
        path: cstr!("src/App.vue"),
        source: String::from(
            r#"<script setup lang="ts">
import Child from './Child.vue'
const groups = [{ id: 1, visible: true }]
</script>
<template>
  <section v-if="Math.max(groups.length, 0)" :class="$attrs.class">
    <Child v-for="group in groups" :key="group.id" v-slot="{ value }">
      {{ group.visible ? Math.max(value, group.id) : group.id }}
    </Child>
  </section>
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
    let report = build_agent_report(payload, cstr!("https://example.test"), files);
    let report_json = serialize_agent_report(&report).expect("report serializes");
    let report: serde_json::Value =
        serde_json::from_str(report_json.as_str()).expect("report is valid JSON");
    let snapshot = &report["semanticFiles"][0]["snapshot"];
    let scopes = snapshot["scopes"]
        .as_array()
        .expect("semantic scopes are serialized");
    let scope_ids = scopes
        .iter()
        .map(|scope| scope["id"].as_u64().expect("scope id"))
        .collect::<FxHashSet<_>>();
    let mut ambient_scope_count = 0;

    assert!(!scope_ids.is_empty());
    assert!(
        snapshot["templateExpressions"]
            .as_array()
            .is_some_and(|expressions| !expressions.is_empty())
    );
    for scope in scopes {
        let bindings = scope["bindings"].as_array().expect("scope bindings");
        assert_eq!(
            scope["bindingCount"].as_u64().expect("binding count"),
            bindings.len() as u64
        );
        if matches!(
            scope["kind"].as_str(),
            Some("univ" | "client" | "server" | "vue")
        ) {
            ambient_scope_count += 1;
            assert!(
                bindings.is_empty(),
                "ambient table should be compact: {scope}"
            );
        }
        for parent_id in scope["parentIds"].as_array().expect("scope parent ids") {
            assert!(
                scope_ids.contains(&parent_id.as_u64().expect("parent scope id")),
                "scope parent must resolve: {scope}"
            );
        }
    }
    assert!(ambient_scope_count > 0);

    for collection in ["componentUsages", "templateExpressions"] {
        for item in snapshot[collection]
            .as_array()
            .expect("semantic collection")
        {
            assert!(
                scope_ids.contains(&item["scopeId"].as_u64().expect("referenced scope id")),
                "{collection} scope must resolve: {item}"
            );
        }
    }
}
