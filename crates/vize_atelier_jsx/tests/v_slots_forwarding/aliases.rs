//! Whole-module and diagnostic custody for equivalent slot spellings.

use crate::*;

fn assert_camel_alias(source: &str, config: &JsxCompileConfig, errors: usize) {
    let camel = source.replace("v-slots", "vSlots");
    assert_ne!(camel, source);
    let bump = Allocator::new();
    let expected = compile_jsx(&bump, source, JsxLang::Jsx, config);
    let actual = compile_jsx(&bump, &camel, JsxLang::Jsx, config);
    assert_eq!(actual.module_code(), expected.module_code(), "{source}");
    assert_eq!(
        expected.diagnostics.iter().filter(|d| d.is_error()).count(),
        errors,
        "{source}: {:?}",
        expected.diagnostics
    );
    let diagnostics = |input: &str, output: &vize_atelier_jsx::JsxCompileOutput| {
        output
            .diagnostics
            .iter()
            .map(|d| {
                let range = input
                    .get(d.start as usize..d.end as usize)
                    .expect("diagnostic must name a valid original source range");
                (
                    d.severity,
                    d.message.to_string(),
                    range.replace("vSlots", "v-slots"),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        diagnostics(&camel, &actual),
        diagnostics(source, &expected),
        "{source}"
    );
}

#[test]
fn camel_slots_forwarding_keeps_whole_modules_and_backend_diagnostics() {
    for source in [
        "const A = () => <B v-slots={slots}/>;",
        "const A = () => <B v-slots={slots}><div>A</div></B>;",
        "const A = () => <B v-slots:default={slots}/>;",
        "const A = () => <ul>{items.map(i => <B key={i} v-slots={slots}/>)}</ul>;",
    ] {
        for config in [
            JsxCompileConfig::default(),
            JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
            JsxCompileConfig {
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            },
            JsxCompileConfig {
                ssr: true,
                ..Default::default()
            },
        ] {
            // The original expression-bodied SSR profile remains an exact
            // refusal case; alias normalization must preserve that same range.
            assert_camel_alias(source, &config, usize::from(config.ssr));
        }
    }
}

#[test]
fn camel_inline_slots_compile_on_every_backend() {
    for source in [
        "const A = () => <B v-slots={{default: () => <i/>, bar: () => <b/>}}/>;",
        "const A = () => <B v-slots={{bar: () => <b/>}}><div>A</div></B>;",
    ] {
        for config in [
            JsxCompileConfig::default(),
            JsxCompileConfig {
                compat: JsxCompatMode::Babel,
                ..Default::default()
            },
            JsxCompileConfig {
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            },
            JsxCompileConfig {
                ssr: true,
                ..Default::default()
            },
        ] {
            // The original expression-bodied SSR profile remains an exact
            // refusal case; alias normalization must preserve that same range.
            assert_camel_alias(source, &config, usize::from(config.ssr));
        }
    }
}

#[test]
fn camel_slots_values_keep_existing_validation_and_source_ranges() {
    for source in [
        "const A = () => <B v-slots/>;",
        "const A = () => <B v-slots={1}/>;",
        "const A = () => <B v-slots={() => <i/>}/>;",
        "const A = () => <div v-slots={{default: () => <i/>}}/>;",
        "const A = () => <B v-slots={{default: () => <i/>}}><div>A</div></B>;",
    ] {
        assert_camel_alias(source, &JsxCompileConfig::default(), 1);
    }
}

#[test]
fn mixed_slots_aliases_are_one_duplicate_builtin() {
    let expected = vec!["v-slots is given more than once on this element; babel keeps only the last one, so merge them into a single slots object.".to_string()];
    for source in [
        "const A = () => <B vSlots={a} v-slots={b}/>;",
        "const A = () => <B v-slots={a} vSlots={b}/>;",
        "const A = () => <B vSlots={a} vSlots={b}/>;",
    ] {
        let bump = Allocator::new();
        let output = compile_jsx(&bump, source, JsxLang::Jsx, &JsxCompileConfig::default());
        assert_eq!(
            output
                .diagnostics
                .iter()
                .map(|d| d.message.to_string())
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        let diagnostic = &output.diagnostics[0];
        let second = source.split_once("={a} ").expect("second attribute").1;
        let second = second.strip_suffix("/>;").expect("component end");
        assert_eq!(
            source.get(diagnostic.start as usize..diagnostic.end as usize),
            Some(second)
        );
        assert!(diagnostic.is_error());
    }
}

#[test]
fn slots_alias_lookalikes_remain_ordinary_props() {
    for name in ["vslots", "VSlots", "vSlot", "vSlotsLike", "vSlots_foo"] {
        let bump = Allocator::new();
        let source = format!("const A = () => <B {name}={{slots}}/>;");
        let root = common::lower_one(&bump, &source);
        let element = common::root_element(&root);
        assert_eq!(element.props.len(), 1, "{name}");
        let prop = common::as_directive(&element.props[0]);
        assert_eq!(prop.name, "bind", "{name}");
        assert_eq!(
            common::simple_content(prop.arg.as_ref().expect("prop name")),
            name
        );
        assert_eq!(
            common::simple_content(prop.exp.as_ref().expect("prop value")),
            "slots"
        );
    }
}

/// The original namespace spelling was accepted by legacy lowering, which
/// discarded its argument. This preserves that behavior, not slot selection.
#[test]
fn legacy_namespace_spellings_forward_the_same_whole_ssr_module() {
    let allocator = Allocator::new();
    let expected = compile_to_ssr(
        &allocator,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    assert!(expected.diagnostics.is_empty());
    assert_eq!(expected.components.len(), 1);
    let expected = expected
        .components
        .first()
        .expect("golden-pinned bare module");
    for source in [
        "const A = () => <B v-slots:default={slots}/>;",
        "const A = () => <B vSlots:default={slots}/>;",
    ] {
        let output = compile_to_ssr(
            &allocator,
            source,
            JsxLang::Jsx,
            SsrCompileOptions::default(),
        );
        assert!(output.diagnostics.is_empty(), "{source}");
        assert_eq!(output.components.len(), 1, "{source}");
        assert_eq!(
            output.components.first().expect("one SSR module").code,
            expected.code,
            "{source}"
        );
    }
}

#[test]
fn camel_slot_aliases_compile_for_stateful_ssr_component_roles() {
    let config = JsxCompileConfig {
        ssr: true,
        ..Default::default()
    };
    for source in [
        "const A = () => {return <B v-slots={slots}/>;};",
        "const A = () => {return <B v-slots={{default: () => <i/>, bar: () => <b/>}}/>;};",
        "const A = () => {return <ul>{items.map(i => <B key={i} v-slots={slots}/>)}</ul>;};",
    ] {
        assert_camel_alias(source, &config, 0);
    }
}
