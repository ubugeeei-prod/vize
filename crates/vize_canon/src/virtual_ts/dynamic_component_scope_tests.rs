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

fn generate() -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
    assert!(errors.is_empty(), "template errors: {errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    )
    .code
}

fn alias(nth: usize) -> String {
    let start = TEMPLATE
        .match_indices("<component")
        .nth(nth)
        .map(|(start, _)| start)
        .expect("authored component");
    cstr!("__vize_dynamic_is_{start}")
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
