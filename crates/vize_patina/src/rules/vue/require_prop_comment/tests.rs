use crate::linter::Linter;
use crate::preset::LintPreset;

fn messages(source: &str) -> Vec<(u32, String)> {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/require-prop-comment".into()]))
        .lint_sfc(source, "Props.vue")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/require-prop-comment")
        .map(|diagnostic| (diagnostic.start, diagnostic.message.to_string()))
        .collect()
}

#[test]
fn an_inline_prop_and_a_local_interface_member_need_jsdoc() {
    let inline = "<script setup lang=\"ts\">\nconst { size } = defineProps<{ size: string }>();\n</script>\n";
    let interface = "<script setup lang=\"ts\">\ninterface Props {\n  size: string;\n}\n\nconst { size } = defineProps<Props>();\n</script>\n";
    let control = "<script setup lang=\"ts\">\nconst { size } = defineProps<{\n  /** The rendered size. */\n  size: string;\n}>();\n</script>\n";
    assert_eq!(
        messages(inline),
        vec![(
            inline.find("size: string").unwrap() as u32,
            "The \"size\" property should have a JSDoc comment.".into()
        )]
    );
    assert_eq!(
        messages(interface),
        vec![(
            interface.find("size: string").unwrap() as u32,
            "The \"size\" property should have a JSDoc comment.".into()
        )]
    );
    assert!(messages(control).is_empty(), "{control}");
}

#[test]
fn an_unresolved_import_is_skipped_and_a_runtime_prop_is_checked() {
    let imported = "<script setup lang=\"ts\">\nimport type { CardProps } from \"./types\";\nconst props = defineProps<CardProps>();\n</script>\n";
    assert!(messages(imported).is_empty());
    let runtime = "<script setup>\ndefineProps({ size: String })\n</script>\n";
    assert_eq!(messages(runtime).len(), 1);
    let commented = "<script>\nexport default { props: { /** The rendered size. */\nsize: String } }\n</script>\n";
    assert!(messages(commented).is_empty(), "{commented}");
}
