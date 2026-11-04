//! Loose `<component :is>` listeners stay implicitly `any` (#11600).
//!
//! `Component`, `VNode`, `any`, and a string tag have no declared emit. Their
//! listener context is an empty parameter list, so `noImplicitAny` reports
//! TS7006 on `(v) => take(v)`. A child with `defineEmits` still supplies the
//! payload tuple through `__vizeResolveEmitProps`.

use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_carton::String;
use vize_croquis::{Analyzer, AnalyzerOptions};

mod current_capture;

fn generate(script: &str, template: &str) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "template errors: {errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let code = generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    )
    .code;
    current_capture::record(script, template, &code);
    code
}

#[test]
fn loose_is_listener_reports_ts7006_for_untyped_parameter() {
    let script = r#"type Component = new (...args: any[]) => { $emit: (...args: any[]) => void }
interface VNode { readonly __v_isVNode: true }
declare const target: Component
declare const node: VNode
declare const loose: any
declare const tag: string
const take = (value: unknown) => value
"#;
    let template = concat!(
        r#"<component :is="target" @pick="(v) => take(v)"></component>"#,
        r#"<component :is="node" @pick="(v) => take(v)"></component>"#,
        r#"<component :is="loose" @pick="(v) => take(v)"></component>"#,
        r#"<component :is="tag" @pick="(v) => take(v)"></component>"#,
        r#"<component :is="'span'" @pick="(v) => take(v)"></component>"#,
    );
    let code = generate(script, template);

    assert!(
        code.contains("type $LooseEmitProps = { readonly __vizeLooseListener: [] };"),
        "loose targets need an empty listener context:\n{code}"
    );
    assert!(
        !code.contains("(props: any) => {}"),
        "an explicit props any types the listener parameter and hides TS7006:\n{code}"
    );
    assert_eq!(
        code.matches("readonly __vizeLooseListener: infer __L extends unknown[]")
            .count(),
        5,
        "each loose listener must read the empty parameter list:\n{code}"
    );
    assert!(
        code.contains("$IsAny<__P> extends true ? never"),
        "any props must not union an explicit any into the handler:\n{code}"
    );
    assert_eq!(
        code.matches("(v) => take(v)").count(),
        5,
        "each loose listener keeps its authored parameter:\n{code}"
    );
    assert!(
        !code.contains("(v: any)") && !code.contains("(v:any)"),
        "the authored parameter must stay implicit any so TS7006 can report it:\n{code}"
    );
}

#[test]
fn loose_is_listener_typed_emit_keeps_declared_payload() {
    let child = generate(
        "const emit = defineEmits<{ pick: [value: string] }>()\n",
        "<button @click=\"emit('pick', 'ok')\"></button>",
    );
    assert!(
        child.contains("export type Emits = { pick: [value: string] };")
            && child.contains("__vizeEmitProps")
            && child.contains("readonly __vizeComponentMarker: true"),
        "a declared emit stays on the component signature:\n{child}"
    );
    assert!(
        !child.contains("infer __L extends unknown[]") && !child.contains("$LooseEmitProps"),
        "the component that declares the emit is not a loose listener target:\n{child}"
    );

    let parent = generate(
        "import Child from './Child.vue'\nconst take = (value: string) => value\n",
        r#"<Child @pick="(v) => take(v)"></Child>"#,
    );
    assert!(
        parent.contains(
            "C extends { readonly __vizeComponentMarker: true } ? $ConstructorEmitPropsFactory<"
        ) && parent.contains(
            "C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F"
        ),
        "a generated child and a declared resolver stay off the loose factory:\n{parent}"
    );
    assert!(
        parent.contains("(v) => take(v)") && !parent.contains("(v: any)"),
        "the typed listener is still the authored callback:\n{parent}"
    );
    assert!(
        !parent.contains("infer __L extends unknown[]"),
        "a declared emit must not take the loose empty parameter list:\n{parent}"
    );
}
