//! A hoisted module reads every listener contract through the shared helpers.
use super::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use crate::virtual_ts::helpers::SHARED_PREAMBLE_DTS;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = r#"import Child from './Child.vue'
const onSave = (value: number) => value
const onFooBar = () => undefined
"#;
const TEMPLATE: &str = r#"<div>
  <Child @save="onSave" @foo-bar="onFooBar" />
  <Child @save="onSave" @click="onFooBar" />
</div>"#;

fn generate(hoist_shared_preamble: bool) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            hoist_shared_preamble,
            ..Default::default()
        },
    )
    .code
    .to_string()
}

fn helper(name: &str) -> String {
    format!("import(\"vize:virtual-ts-listeners\").{name}<")
}

#[test]
fn hoisted_listeners_read_component_and_event_contracts_from_shared_helpers() {
    let code = generate(true);
    for name in [
        "$ListenerLibrary",
        "$ListenerPropsBox",
        "$ListenerEmitPropsBox",
        "$ListenerNativeFallthroughBox",
    ] {
        assert_eq!(
            code.matches(&format!("{}typeof Child>;", helper(name)))
                .count(),
            1,
            "one component contract per component: {name}\n{code}"
        );
    }
    // `save` and `click` are camel keys; only the authored `foo-bar` spelling
    // admits its raw `onFoo-bar` key.
    for (name, count) in [
        ("$ListenerPropArgs", 2),
        ("$ListenerPropArgsRaw", 1),
        ("$ListenerFallthroughArgs", 2),
        ("$ListenerFallthroughArgsRaw", 1),
        ("$ListenerStaticEmitArgs", 3),
        ("$ListenerNativeReturn", 3),
        ("$ListenerReturn", 3),
    ] {
        assert_eq!(code.matches(&helper(name)).count(), count, "{name}\n{code}");
    }
    assert_eq!(code.matches("\"onFoo-bar\">;").count(), 2, "{code}");
    // No per-event or per-component copy of the helper text remains.
    for inline in [
        "{ $props: infer __P }",
        "__vizeFallthroughProps?: infer __F",
        "__vizeEmitProps?: infer __EP",
    ] {
        assert_eq!(code.matches(inline).count(), 0, "{inline}\n{code}");
    }
}

#[test]
fn every_hoisted_listener_helper_is_exported_by_the_shared_module() {
    let code = generate(true);
    let mut referenced = 0;
    for (at, _) in code.match_indices("import(\"vize:virtual-ts-listeners\").$Listener") {
        let name = &code[at + "import(\"vize:virtual-ts-listeners\").".len()..];
        let name = &name[..name.find('<').unwrap()];
        assert_eq!(
            SHARED_PREAMBLE_DTS
                .matches(&format!("export type {name}<"))
                .count(),
            1,
            "{name}"
        );
        referenced += 1;
    }
    assert!(referenced > 0, "{code}");
}

#[test]
fn standalone_listeners_keep_their_inline_contracts() {
    let code = generate(false);
    assert_eq!(code.matches("$Listener").count(), 0, "{code}");
    assert_eq!(
        code.matches(
            "typeof Child extends { new (...args: any[]): { $props: infer __P } } ? [__P]"
        )
        .count(),
        1,
        "{code}"
    );
}
