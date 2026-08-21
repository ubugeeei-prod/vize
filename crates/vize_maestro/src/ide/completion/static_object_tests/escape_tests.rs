//! Receiver escapes through less common JavaScript syntax channels.

use super::{
    CompletionService, IdeContext, assert_falls_back, completion_labels, state_with_document,
};

#[test]
fn tagged_templates_class_fields_and_exports_fall_back() {
    for (name, escape) in [
        ("TaggedTemplateTag.vue", "probe`tag`"),
        ("TaggedTemplateValue.vue", "String.raw`${probe}`"),
        (
            "ClassFieldReceiver.vue",
            "class Holder { value = probe }\nnew Holder()",
        ),
        ("NamedExportReceiver.vue", "export { probe }"),
        ("DefaultExportReceiver.vue", "export default probe"),
    ] {
        let source = [
            "<script setup lang=\"ts\">\nconst probe = { initial: 1 }\n",
            escape,
            "\nconst chosen = probe.initial\n</script>\n",
        ]
        .concat();
        assert_falls_back(name, &source, "probe.initial");
    }
}

#[test]
fn source_reexport_does_not_escape_the_local_receiver() {
    let source = r#"<script setup lang="ts">
const probe = { initial: 1 }
export { probe } from './dependency'
const chosen = probe.initial
</script>
"#;
    let (state, uri) = state_with_document("SourceReexport.vue", source);
    let offset = source.find("probe.initial").unwrap() + "probe.".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert_eq!(
        completion_labels(CompletionService::complete(&ctx).unwrap()),
        ["initial"],
    );
}
