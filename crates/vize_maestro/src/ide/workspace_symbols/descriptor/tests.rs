use std::borrow::Cow;

use super::parse;

fn assert_borrowed(value: Cow<'_, str>, source: &str) {
    let Cow::Borrowed(text) = value else {
        panic!("synchronous workspace symbol parsing must borrow the input");
    };
    assert!(!text.is_empty());
    assert!(
        source
            .as_bytes()
            .windows(text.len())
            .any(|slice| { std::ptr::eq(slice.as_ptr(), text.as_ptr()) })
    );
}

#[test]
fn symbol_descriptor_borrows_whole_source_and_every_block() {
    let source = "<template>\n<main>{{ value }}</main>\n</template>\n\
<script lang=\"ts\">\nexport default { name: '借用' }\n</script>\n\
<script setup lang=\"ts\">\nconst value = 1;\nconst emit = defineEmits(['save-item']);\n</script>\n\
<style scoped>\n.item { color: red; }\n</style>\n\
<style>\n#root { display: block; }\n</style>\n\
<i18n>\n{\"en\":{\"label\":\"owned names, borrowed source\"}}\n</i18n>\n";
    let descriptor = parse("/workspace/borrowed-component.vue", source).unwrap();
    assert_borrowed(descriptor.source.clone(), source);
    assert_borrowed(
        descriptor.template.as_ref().unwrap().content.clone(),
        source,
    );
    assert_borrowed(descriptor.script.as_ref().unwrap().content.clone(), source);
    assert_borrowed(
        descriptor.script_setup.as_ref().unwrap().content.clone(),
        source,
    );
    assert_eq!(descriptor.styles.len(), 2);
    for block in &descriptor.styles {
        assert_borrowed(block.content.clone(), source);
    }
    assert_eq!(descriptor.custom_blocks.len(), 1);
    for block in &descriptor.custom_blocks {
        assert_borrowed(block.content.clone(), source);
    }

    let owned =
        vize_incremental::parse_descriptor("/workspace/borrowed-component.vue", source).unwrap();
    assert!(matches!(owned.source, Cow::Owned(_)));
    assert!(matches!(
        owned.script_setup.as_ref().unwrap().content,
        Cow::Owned(_)
    ));
    assert_eq!(
        serde_json::to_value(&descriptor).unwrap(),
        serde_json::to_value(&*owned).unwrap(),
        "the same parser, filename and options preserve the whole descriptor"
    );
}

#[test]
fn borrowed_and_owned_paths_preserve_every_descriptor_and_rejection() {
    for source in [
        "",
        "<template><div /></template>",
        "<script setup>const count = 1;</script>",
        "<script setup lang=\"ts\">const emit = defineEmits<{ 'save-item': [] }>(); const slots = defineSlots<{ default: () => unknown }>();</script>",
        "<template lang=\"pug\">main\n  slot(name=\"résumé\")</template><style lang=\"scss\">.slot { color: v-bind(tone); }</style>",
        "<script setup>const \\u0066oo = 1;</script><style>.foo, #foo { color: red; }</style>",
        "<template>\r\n<span>😀</span>\r\n</template>\r\n<script setup>\r\nconst value = 'İ';\r\n</script>",
        "<script src=\"./external.ts\"></script><style src=\"./external.css\"></style>",
        "<script setup>const broken = ;</script>",
        "<script setup>const value = 1;</script><script setup>const value = 2;</script>",
        "<template><div></template>",
        "<i18n>{\"ja\":{\"label\":\"日本語\"}}</i18n><template><slot /></template>",
    ] {
        let current = parse("/workspace/encoded%20name.vue", source)
            .map(|descriptor| serde_json::to_value(descriptor).unwrap());
        let original = vize_incremental::parse_descriptor("/workspace/encoded%20name.vue", source)
            .map(|descriptor| serde_json::to_value(&*descriptor).unwrap());
        assert_eq!(
            current, original,
            "whole descriptor or parse rejection: {source}"
        );
    }
}
