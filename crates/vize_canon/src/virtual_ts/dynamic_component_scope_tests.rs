//! A `<component :is>` alias is declared in the scope its `:is` is evaluated
//! in, so slot props and loop aliases it reads are in scope (#11491).
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_macros, reason = "insta formats its snapshot")]

use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_carton::{String, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "import Picker from './Picker.vue'\nimport Panel from './Panel.vue'\nconst map = { a: Picker, b: Picker }\nconst foo = 'a' as const\nconst items = [{ kind: 'a' as const }]\nconst take = (value: string) => value\n";

const TEMPLATE: &str = concat!(
    r#"<Panel v-slot:qux="{ kind, open }">"#,
    r#"<component :is="map[kind]" :x="1" @pick="(v) => take(v)"></component>"#,
    r#"<component v-if="open" :is="map[foo]" @pick="(v) => take(v)"></component>"#,
    r#"</Panel>"#,
    r#"<component v-for="item in items" :is="map[item.kind]" @pick="(v) => take(v)"></component>"#,
    r#"<component :is="map[foo]" @pick="(v) => take(v)"></component>"#,
);

fn generate_source(script: &str, template: &str) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "template errors: {errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    )
    .code
}

fn generate() -> String {
    generate_source(SCRIPT, TEMPLATE)
}

fn alias_for(template: &str, nth: usize) -> String {
    let start = template
        .match_indices("<component")
        .nth(nth)
        .map(|(start, _)| start)
        .expect("authored component");
    cstr!("__vize_dynamic_is_{start}")
}

fn alias(nth: usize) -> String {
    alias_for(TEMPLATE, nth)
}

#[test]
fn scope_bound_targets_declare_their_alias_inside_that_scope() {
    let code = generate();
    let template = &code[code
        .find(";(function __template()")
        .expect("template scope")..];
    // The slot and loop declare theirs once in the template pass and once in
    // the component-props pass; the root target keeps one root declaration.
    let slot = ["function _slot_qux_", "function _slot_props_qux_"];
    let item = ["for (const [item]", "for (const [item]"];
    for (nth, openers) in [(0, slot), (1, slot), (2, item)] {
        let alias = alias(nth);
        let declaration = cstr!("const {alias} = ");
        let declarations = template
            .match_indices(declaration.as_str())
            .collect::<Vec<_>>();
        // Both sit in a scope body, one level below the template root.
        let indents = declarations
            .iter()
            .map(|(at, _)| &template[template[..*at].rfind('\n').expect("line") + 1..*at])
            .collect::<Vec<_>>();
        assert_eq!(indents, ["    ", "    "], "{alias}");
        let mut from = 0;
        for ((declared, _), opener) in declarations.iter().zip(openers) {
            let opened = from + template[from..].find(opener).expect("scope opener");
            assert!(opened < *declared, "{alias} opens with its scope");
            from = opened + opener.len();
        }
        // Nothing before the scope's first declaration reads the alias.
        assert_eq!(
            template.find(alias.as_str()),
            Some(declarations[0].0 + "const ".len()),
            "{alias}"
        );
    }
    let root = alias(3);
    assert_eq!(
        template
            .matches(cstr!("\n  const {root} = (map[foo]);").as_str())
            .count(),
        1
    );
    insta::assert_snapshot!(template);
}

#[test]
fn outer_guard_keeps_its_binding_identity_across_a_shadowing_slot() {
    const SCRIPT: &str = "import Picker from './Picker.vue'\nimport Panel from './Panel.vue'\nconst map = { a: Picker }\nconst visible = true\n";
    const TEMPLATE: &str = concat!(
        r#"<Panel v-if="visible" v-slot:default="{ kind, open, visible }">"#,
        r#"<component :is="map[kind]"></component>"#,
        r#"<component v-if="open" :is="map[kind]"></component>"#,
        r#"</Panel>"#,
    );
    let code = generate_source(SCRIPT, TEMPLATE);
    let template = &code[code
        .find(";(function __template()")
        .expect("template scope")..];
    let guard_start = TEMPLATE.find("v-if=\"visible\"").expect("outer guard") + "v-if=\"".len();
    let guard = cstr!("__vize_dynamic_guard_{guard_start}");
    let outer_alias = alias_for(TEMPLATE, 0);
    let inner_guard_start = TEMPLATE.find("v-if=\"open\"").expect("inner guard") + "v-if=\"".len();
    let inner_guard = cstr!("__vize_dynamic_guard_{inner_guard_start}");
    let inner_alias = alias_for(TEMPLATE, 1);
    let capture = cstr!("const {guard} = ((visible));");
    let outer_declaration =
        cstr!("const {outer_alias} = ({guard}) ? (map[kind]) : (undefined as never);");
    let inner_capture = cstr!("const {inner_guard} = ({guard} && (open));");
    let inner_declaration =
        cstr!("const {inner_alias} = ({inner_guard}) ? (map[kind]) : (undefined as never);");
    let slot_start = template
        .find("function _slot_default_")
        .expect("slot scope");
    let capture_start = template.find(capture.as_str()).expect("guard capture");
    let alias_start = template
        .find(outer_declaration.as_str())
        .expect("captured alias declaration");

    assert!(capture_start < slot_start && slot_start < alias_start);
    assert_eq!(template.matches(capture.as_str()).count(), 1);
    assert_eq!(template.matches(outer_declaration.as_str()).count(), 2);
    assert_eq!(template.matches(inner_capture.as_str()).count(), 2);
    assert_eq!(template.matches(inner_declaration.as_str()).count(), 2);
    assert!(!template.contains("((visible)) ? (map[kind])"));
}
