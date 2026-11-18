use crate::linter::Linter;
use crate::preset::LintPreset;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{ScriptBlock, TypeResolver};

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

#[test]
fn local_members_survive_unresolved_intersections_and_interface_heritage() {
    let intersection = r#"<script setup lang="ts">
import type { ExternalProps } from "./external";
type LocalProps = { local: string };
defineProps<ExternalProps & LocalProps & { inline: number }>();
</script>
"#;
    assert_eq!(
        messages(intersection)
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>(),
        vec![
            "The \"local\" property should have a JSDoc comment.",
            "The \"inline\" property should have a JSDoc comment.",
        ]
    );

    let heritage = r#"<script setup lang="ts">
interface BaseProps { inherited: string }
interface Props extends BaseProps { own: string }
defineProps<Props>();
</script>
"#;
    assert_eq!(
        messages(heritage)
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>(),
        vec![
            "The \"inherited\" property should have a JSDoc comment.",
            "The \"own\" property should have a JSDoc comment.",
        ]
    );
}

#[test]
fn sibling_script_types_keep_their_authored_comments_and_spans() {
    let source = r#"<script lang="ts">
import type { ExternalProps } from "./external";
interface LocalBase { inherited: string }
export interface Props<T> extends ExternalProps, LocalBase {
  /** The visible value. */
  value: T;
  label?: string;
}
</script>

<script setup lang="ts" generic="T extends string">
defineProps<Props<T>>();
</script>
"#;
    assert_eq!(
        messages(source),
        vec![
            (
                source.find("inherited: string").unwrap() as u32,
                "The \"inherited\" property should have a JSDoc comment.".into(),
            ),
            (
                source.find("label?: string").unwrap() as u32,
                "The \"label\" property should have a JSDoc comment.".into(),
            ),
        ]
    );
}

#[test]
fn sibling_setup_declarations_win_over_same_named_module_types() {
    let source = r#"<script lang="ts">
interface Props { moduleOnly: string }
</script>

<script setup lang="ts">
interface Props { setupOnly: string }
defineProps<Props>();
</script>
"#;
    assert_eq!(
        messages(source),
        vec![(
            source.find("setupOnly: string").unwrap() as u32,
            "The \"setupOnly\" property should have a JSDoc comment.".into(),
        )]
    );
}

#[test]
fn repeated_alias_dags_and_cycles_resolve_each_authored_prop_once() {
    let mut source = String::from(
        r#"<script setup lang="ts">
type P0 = { leaf: string };
"#,
    );
    // Stay inside the rule's existing depth budget while still constructing
    // the 64-path DAG that used to repeat the same leaf diagnostic.
    for depth in 1..=6 {
        source.push_str(&format!(
            "type P{depth} = P{} & P{};\n",
            depth - 1,
            depth - 1
        ));
    }
    source.push_str(
        r#"interface CycleA extends CycleB { alpha: string }
interface CycleB extends CycleA { beta: string }
defineProps<P6 & CycleA>();
</script>
"#,
    );
    assert_eq!(
        messages(&source)
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>(),
        vec![
            "The \"leaf\" property should have a JSDoc comment.",
            "The \"alpha\" property should have a JSDoc comment.",
            "The \"beta\" property should have a JSDoc comment.",
        ]
    );
}

#[test]
fn cyclic_interface_expansions_are_independent_of_lookup_order() {
    let allocator = Allocator::default();
    let source = "interface CycleA extends CycleB { alpha: string }\ninterface CycleB extends CycleA { beta: string }";
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let blocks = [ScriptBlock {
        program: &parsed.program,
        source,
        offset: 0,
    }];
    let mut resolver = TypeResolver::new(&blocks);
    let names = |props: Vec<super::Prop>| {
        props
            .into_iter()
            .map(|prop| prop.name.to_string())
            .collect::<Vec<_>>()
    };

    assert_eq!(
        names(resolver.lookup("CycleA", 16, 0).unwrap()),
        ["alpha", "beta"]
    );
    assert_eq!(
        names(resolver.lookup("CycleB", 15, 0).unwrap()),
        ["beta", "alpha"]
    );
}

#[test]
fn repeated_interface_heritage_is_expanded_once() {
    let source = r#"<script setup lang="ts">
interface Shared { leaf: string }
interface Props extends Shared, Shared { own: string }
defineProps<Props>();
</script>"#;
    assert_eq!(
        messages(source)
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>(),
        [
            "The \"leaf\" property should have a JSDoc comment.",
            "The \"own\" property should have a JSDoc comment.",
        ]
    );
}

#[test]
fn reports_cover_the_whole_prop_like_eslint_plugin_vue() {
    let ranges = |source: &str| -> Vec<String> {
        Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec!["vue/require-prop-comment".into()]))
            .lint_sfc(source, "Props.vue")
            .diagnostics
            .iter()
            .filter_map(|diagnostic| source.get(diagnostic.start as usize..diagnostic.end as usize))
            .map(String::from)
            .collect()
    };
    let typed = "<script setup lang=\"ts\">\ntype Props = {\n  multiple?: false | undefined;\n  modelValue?: string,\n  pick(value: string): void\n}\ndefineProps<Props>()\n</script>\n";
    assert_eq!(
        ranges(typed),
        // A TS member spans its delimiter, as in typescript-estree.
        [
            "multiple?: false | undefined;",
            "modelValue?: string,",
            "pick(value: string): void"
        ]
    );
    let runtime = "<script>\nexport default {\n  props: { size: { type: String, default: 'm' }, tone: String, ['label']: String },\n}\n</script>\n";
    assert_eq!(
        ranges(runtime),
        [
            "size: { type: String, default: 'm' }",
            "tone: String",
            "['label']: String"
        ]
    );
    let array = "<script setup>\ndefineProps(['size'])\n</script>\n";
    assert_eq!(ranges(array), ["'size'"]);
}
