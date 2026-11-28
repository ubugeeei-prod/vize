//! Template refs reach the runtime ref setter as the authored key and target.

use super::{VaporCompilerOptions, compile_vapor};
use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_carton::{Allocator, FxHashMap, String};

fn compile(source: &str) -> String {
    compile_with(source, VaporCompilerOptions::default())
}

fn compile_with(source: &str, options: VaporCompilerOptions) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, source, options);
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result.code
}

/// The generated lines that pass something to the ref setter.
fn ref_calls(code: &str) -> std::vec::Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once('(')
                .is_some_and(|(head, _)| head == "_setRef")
        })
        .collect()
}

/// The ref setter lines, including the ones a render effect wraps.
fn ref_lines(code: &str) -> std::vec::Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| line.contains("_setRef(n"))
        .collect()
}

fn script_setup(bindings: &[(&str, BindingType)]) -> VaporCompilerOptions {
    let mut table = FxHashMap::default();
    for (name, kind) in bindings {
        table.insert((*name).into(), *kind);
    }
    VaporCompilerOptions {
        binding_metadata: Some(BindingMetadata {
            bindings: table,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        }),
        ..Default::default()
    }
}

#[test]
fn static_ref_key_is_a_valid_string_literal() {
    let code = compile(r#"<div ref='a"b\c'></div>"#);
    assert_eq!(ref_calls(&code), [r#"_setRef(n0, "a\"b\\c")"#], "{code}");
}

#[test]
fn slot_outlet_ref_targets_the_outlet_instead_of_becoming_a_slot_prop() {
    let code = compile(r#"<slot ref="s" ref_key="k" name="x" :foo="1"></slot>"#);
    assert_eq!(ref_calls(&code), [r#"_setRef(n0, "s")"#], "{code}");
    let slot_lines: std::vec::Vec<&str> = code
        .lines()
        .filter(|line| line.split_once("_createSlot(").is_some())
        .collect();
    assert_eq!(slot_lines.len(), 1, "{code}");
    for line in slot_lines {
        assert_eq!(line.matches("ref").count(), 0, "{code}");
    }
}

#[test]
fn dynamic_ref_follows_its_value_in_a_render_effect() {
    let code = compile(r#"<div :ref="on ? 'a' : 'b'"></div>"#);
    assert_eq!(
        ref_lines(&code),
        [r#"_renderEffect(() => _setRef(n0, _ctx.on ? 'a' : 'b'))"#],
        "{code}"
    );
}

#[test]
fn dynamic_ref_joins_the_element_render_effect() {
    let code = compile(r#"<div :class="c" :ref="r"></div>"#);
    let effect: std::vec::Vec<&str> = code
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != "_renderEffect(() => {")
        .take(4)
        .collect();
    assert_eq!(
        effect,
        [
            "_renderEffect(() => {",
            "_setClass(n0, _ctx.c)",
            "_setRef(n0, _ctx.r)",
            "})"
        ],
        "{code}"
    );
}

#[test]
fn literal_refs_are_set_once() {
    let code = compile(r#"<a :ref="'a'"></a><b :ref="`b`"></b><i :ref="null"></i>"#);
    assert_eq!(
        ref_lines(&code),
        ["_setRef(n0, 'a')", "_setRef(n1, `b`)", "_setRef(n2, null)"],
        "{code}"
    );
}

#[test]
fn dynamic_loop_and_component_refs_follow_their_value() {
    let code = compile(r#"<div v-for="i in list" :ref="fn"></div><Comp :ref="r" />"#);
    assert_eq!(
        ref_lines(&code),
        [
            "_renderEffect(() => _setRef(n2, _ctx.fn, true))",
            "_renderEffect(() => _setRef(n3, _ctx.r))"
        ],
        "{code}"
    );
}

#[test]
fn ref_naming_a_setup_binding_binds_it_by_name() {
    let options = script_setup(&[
        ("el", BindingType::SetupRef),
        ("maybe", BindingType::SetupMaybeRef),
        ("mutable", BindingType::SetupLet),
        ("callback", BindingType::SetupConst),
    ]);
    let code = compile_with(
        r#"<a :ref="el"></a><b :ref="maybe"></b><i :ref="mutable"></i><p :ref="callback"></p>"#,
        options,
    );
    assert_eq!(
        ref_lines(&code),
        [
            r#"_setRef(n0, "el")"#,
            r#"_setRef(n1, "maybe")"#,
            r#"_setRef(n2, "mutable")"#,
            "_setRef(n3, _ctx.callback)",
        ],
        "{code}"
    );
}

#[test]
fn loop_alias_shadowing_a_setup_ref_stays_the_alias() {
    let options = script_setup(&[("el", BindingType::SetupRef)]);
    let code = compile_with(r#"<div v-for="el in list" :ref="el"></div>"#, options);
    assert_eq!(
        ref_lines(&code),
        ["_renderEffect(() => _setRef(n2, _for_item0.value, true))"],
        "{code}"
    );
}
