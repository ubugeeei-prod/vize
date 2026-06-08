use super::super::{Drawer, DrawerOptions};
use vize_carton::append;

// ========== Snapshot Tests ==========

#[test]
fn test_full_croquis_snapshot() {
    use insta::assert_snapshot;

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script(
        r#"import { ref, computed, inject, provide } from 'vue'
import MyComponent from './MyComponent.vue'

const props = defineProps<{
    msg: string
    count?: number
}>()

const emit = defineEmits<{
    (e: 'update', value: string): void
    (e: 'delete'): void
}>()

const model = defineModel<string>()

const counter = ref(0)
const doubled = computed(() => counter.value * 2)
const theme = inject('theme')

provide('counter', counter)

function increment() {
    counter.value++
    emit('update', String(counter.value))
}

export type UserProps = { name: string }
"#,
    );

    let summary = drawer.finish();

    // Build a readable snapshot
    let mut output = String::new();
    output.push_str("=== Bindings ===\n");
    for (name, ty) in summary.bindings.iter() {
        append!(output, "  {name}: {:?}\n", ty);
    }

    output.push_str("\n=== Macros ===\n");
    append!(output, "  props: {}\n", summary.macros.props().len());
    append!(output, "  emits: {}\n", summary.macros.emits().len());
    append!(output, "  models: {}\n", summary.macros.models().len());

    output.push_str("\n=== Reactivity ===\n");
    for source in summary.reactivity.sources() {
        append!(
            output,
            "  {}: kind={:?}, needs_value={}\n",
            source.name,
            source.kind,
            source.kind.needs_value_access()
        );
    }

    output.push_str("\n=== Provide/Inject ===\n");
    append!(
        output,
        "  provides: {}\n",
        summary.provide_inject.provides().len()
    );
    append!(
        output,
        "  injects: {}\n",
        summary.provide_inject.injects().len()
    );

    output.push_str("\n=== Type Exports ===\n");
    for te in &summary.type_exports {
        append!(output, "  {}: {:?}\n", te.name, te.kind);
    }

    assert_snapshot!(output);
}

#[test]
fn test_props_emits_snapshot() {
    use insta::assert_snapshot;

    let mut drawer = Drawer::for_lint();
    drawer.draw_script(
        r#"
const props = defineProps({
    title: String,
    count: { type: Number, required: true },
    items: { type: Array, default: () => [] }
})

const emit = defineEmits(['update', 'delete', 'select'])
"#,
    );

    let summary = drawer.finish();

    let mut output = String::new();
    output.push_str("=== Props ===\n");
    for prop in summary.macros.props() {
        append!(
            output,
            "  {}: required={}, has_default={}\n",
            prop.name,
            prop.required,
            prop.default_value.is_some()
        );
    }

    output.push_str("\n=== Emits ===\n");
    for emit in summary.macros.emits() {
        append!(output, "  {}\n", emit.name);
    }

    assert_snapshot!(output);
}

#[test]
fn test_provide_inject_snapshot() {
    use insta::assert_snapshot;

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script(
        r#"import { provide, inject } from 'vue'

// Simple provide
provide('theme', 'dark')

// Provide with ref
const counter = ref(0)
provide('counter', counter)

// Provide with Symbol key
const KEY = Symbol('key')
provide(KEY, { value: 42 })

// Simple inject
const theme = inject('theme')

// Inject with default
const locale = inject('locale', 'en')

// Inject with destructure
const { name, id } = inject('user') as { name: string; id: number }
"#,
    );

    let summary = drawer.finish();

    let mut output = String::new();
    output.push_str("=== Provides ===\n");
    for p in summary.provide_inject.provides() {
        append!(output, "  key: {:?}\n", p.key);
    }

    output.push_str("\n=== Injects ===\n");
    for i in summary.provide_inject.injects() {
        append!(
            output,
            "  key: {:?}, has_default: {}, pattern: {:?}\n",
            i.key,
            i.default_value.is_some(),
            i.pattern
        );
    }

    assert_snapshot!(output);
}

#[test]
fn test_vif_guard_in_template() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let template = r#"<div>
            <p v-if="todo.description">{{ unwrapDescription(todo.description) }}</p>
            <span>{{ todo.title }}</span>
        </div>"#;

    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "Template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();

    // Find the interpolation expressions
    let expressions: Vec<_> = summary
        .template_expressions
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                crate::croquis::TemplateExpressionKind::Interpolation
            )
        })
        .collect();

    insta::assert_debug_snapshot!(expressions);
}
