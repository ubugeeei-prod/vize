use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

const TEMPLATE: &str =
    r#"<el-badge><template #content="{ value }">{{ value.missing }}</template></el-badge>"#;
const SLOT_RESOLVER_DECLARATIONS: &str =
    include_str!("../helpers/declarations/slot-resolver.d.ts.txt");
const SLOT_PAYLOAD_DECLARATIONS: &str =
    include_str!("../helpers/declarations/slot-payload.d.ts.txt");

fn exact_type_declaration<'a>(declarations: &'a str, name: &str) -> &'a str {
    let expected_prefix = format!("type {name}");
    let mut matching = declarations.lines().filter(|line| {
        line.split_once('<')
            .is_some_and(|(prefix, _)| prefix == expected_prefix)
    });
    let declaration = matching
        .next()
        .unwrap_or_else(|| panic!("missing {name} declaration"));
    assert!(matching.next().is_none(), "duplicate {name} declaration");
    declaration
}

fn exact_line_count(code: &str, expected: &str) -> usize {
    code.lines().filter(|line| *line == expected).count()
}

fn slot_factory_owners(code: &str) -> Vec<&str> {
    code.split("$SlotsFactory<typeof ")
        .skip(1)
        .map(|suffix| {
            suffix
                .split_once('>')
                .expect("every slot factory reference must close its owner type")
                .0
        })
        .collect()
}

fn declared_const_lines<'a>(code: &'a str, name: &str) -> Vec<&'a str> {
    code.lines()
        .filter(|line| {
            let mut words = line.split_whitespace();
            words.next() == Some("declare")
                && words.next() == Some("const")
                && words.next().and_then(|token| token.strip_suffix(':')) == Some(name)
        })
        .collect()
}

fn local_const_lines<'a>(code: &'a str, name: &str) -> Vec<&'a str> {
    code.lines()
        .filter(|line| {
            let mut words = line.split_whitespace();
            words.next() == Some("const")
                && words.next().and_then(|token| token.strip_suffix(':')) == Some(name)
        })
        .collect()
}

/// Generate the embedded-preamble virtual TS for a template with no script.
fn generate(template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets(&summary, None, Some(&root), 0, 0, &Default::default()).code
}

/// The slot-payload aliases are roots that nothing else in the preamble
/// reaches, so a module-scope copy in a component that never annotates a slot
/// scope from them is dead code — `TS6196` for a `noUnusedLocals` consumer, and
/// what the exact-tsgo project gate reports. Each is declared exactly when the
/// document references it, and never otherwise.
#[test]
fn slot_payload_helpers_are_declared_only_where_they_are_referenced() {
    let slot_free = generate(r#"<div v-for="item in items">{{ item }}</div>"#);
    for (declarations, alias) in [
        (SLOT_RESOLVER_DECLARATIONS, "$SlotHostSlots"),
        (SLOT_RESOLVER_DECLARATIONS, "$SlotsFactory"),
        (SLOT_PAYLOAD_DECLARATIONS, "$SlotPayload"),
    ] {
        let declaration = exact_type_declaration(declarations, alias);
        assert_eq!(
            exact_line_count(&slot_free, declaration),
            0,
            "a component with no v-slot scope must not declare {alias}:\n{slot_free}",
        );
    }

    let static_name = generate(TEMPLATE);
    for (declarations, alias) in [
        (SLOT_RESOLVER_DECLARATIONS, "$SlotsFactory"),
        (SLOT_PAYLOAD_DECLARATIONS, "$SlotPayload"),
    ] {
        assert_eq!(
            exact_line_count(&static_name, exact_type_declaration(declarations, alias)),
            1,
            "{static_name}",
        );
    }

    let dynamic_name = generate(
        r#"<el-badge><template #[slotName]="{ value }">{{ value }}</template></el-badge>"#,
    );
    for (declarations, alias) in [
        (SLOT_RESOLVER_DECLARATIONS, "$SlotsFactory"),
        (SLOT_PAYLOAD_DECLARATIONS, "$SlotPayload"),
        (SLOT_PAYLOAD_DECLARATIONS, "$NamedSlotPayload"),
    ] {
        assert_eq!(
            exact_line_count(&dynamic_name, exact_type_declaration(declarations, alias)),
            1,
            "payload extraction must retain the exact {alias} declaration:\n{dynamic_name}",
        );
    }
}

#[test]
fn slot_constructor_calls_adapt_the_first_argument_without_replacing_the_signature() {
    let code = generate(TEMPLATE);
    for helper in [
        "type $SlotConstructorArgumentLists<C> =",
        "type $SlotConstructorHasArguments<C> =",
        "type $SlotConstructorWithoutArguments<C> =",
        "type $SlotConstructorWithArguments<C> =",
        "declare function __vizeSlotConstructorInstance<C>():",
    ] {
        assert_eq!(
            code.matches(helper).count(),
            1,
            "expected one exact {helper} declaration:\n{code}",
        );
    }
    assert_eq!(
        code.matches("__vizeSlotConstructorInstance<$SlotConstructor<typeof el_badge>>()(",)
            .count(),
        1,
        "the slot-host constructor must select the exact direct construction:\n{code}",
    );
    assert_eq!(
        code.matches("$SlotConstructorWithArguments<$SlotConstructor<typeof el_badge>>)(",)
            .count(),
        1,
        "the non-empty branch must preserve the direct props construction:\n{code}",
    );
}

#[test]
fn kebab_case_slot_host_uses_pascal_case_setup_binding() {
    let script = r#"import { ElBadge } from 'element-plus'"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert_eq!(
        slot_factory_owners(&output.code),
        // The slot body and restored canonical prop-check scope each keep the
        // same actual PascalCase constructor, as for configured globals below.
        ["ElBadge", "ElBadge"],
        "{}",
        output.code,
    );
    assert_eq!(
        declared_const_lines(&output.code, "el_badge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
}

#[test]
fn kebab_case_slot_host_uses_ambient_pascal_global_component() {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = super::VirtualTsOptions {
        external_template_bindings: vec!["ElBadge".into()],
        ..Default::default()
    };
    let output = generate_virtual_ts_with_offsets(&summary, None, Some(&root), 0, 0, &options);

    assert_eq!(
        slot_factory_owners(&output.code),
        ["ElBadge", "ElBadge"],
        "{}",
        output.code,
    );
    assert_eq!(
        declared_const_lines(&output.code, "ElBadge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
    assert_eq!(
        local_const_lines(&output.code, "el_badge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
    assert_eq!(
        declared_const_lines(&output.code, "el_badge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
}

#[test]
fn type_only_local_component_import_keeps_ambient_global_component_value() {
    let script = r#"import type { ElBadge } from '#components'
type BadgeInstance = typeof ElBadge
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = super::VirtualTsOptions {
        external_template_bindings: vec!["ElBadge".into()],
        ..Default::default()
    };
    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), Some(&root), 0, 0, &options);

    assert_eq!(
        declared_const_lines(&output.code, "$Component_el_badge"),
        [
            "declare const $Component_el_badge: $GlobalComponent<\"el-badge\", \"ElBadge\"> & $GlobalComponentFallback;"
        ],
        "{}",
        output.code,
    );
    assert_eq!(
        declared_const_lines(&output.code, "ElBadge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
    assert_eq!(
        slot_factory_owners(&output.code),
        ["$Component_el_badge", "$Component_el_badge"],
        "{}",
        output.code,
    );
    assert_eq!(
        local_const_lines(&output.code, "el_badge"),
        Vec::<&str>::new(),
        "{}",
        output.code,
    );
}

#[test]
fn unresolved_slot_host_uses_vue_global_components_fallback() {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, None, Some(&root), 0, 0, &Default::default());

    assert_eq!(
        declared_const_lines(&output.code, "el_badge"),
        [
            "declare const el_badge: $GlobalComponent<\"el-badge\", \"ElBadge\"> & $GlobalComponentFallback;"
        ],
        "{}",
        output.code,
    );
    assert_eq!(
        slot_factory_owners(&output.code),
        ["el_badge"],
        "{}",
        output.code,
    );
}

#[test]
fn editor_reference_paths_are_literal_and_deduplicated() {
    let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
    let options = super::VirtualTsOptions {
        reference_paths: vec![
            "/tmp/app&docs/components.d.ts".into(),
            "/tmp/app&docs/components.d.ts".into(),
            "invalid\npath.d.ts".into(),
        ],
        ..Default::default()
    };

    let output = generate_virtual_ts_with_offsets(&summary, None, None, 0, 0, &options);

    assert_eq!(
        output
            .code
            .lines()
            .filter(|line| {
                line.split_once(" path=")
                    .is_some_and(|(prefix, _)| prefix == "/// <reference")
            })
            .collect::<Vec<_>>(),
        ["/// <reference path=\"/tmp/app&docs/components.d.ts\" />"],
        "{}",
        output.code,
    );
}

/// An empty `v-slot` expression (`<template #text="">`) is a slot function
/// without parameters for Vue's compiler. The generated slot function declares
/// its synthetic parameter for it; an empty authored pattern would leave the
/// parameter without a name (`_slot_text_7(: $SlotPayload<…>)`), a syntax
/// error that aborts Corsa's semantic pass for every file in the project.
#[test]
fn empty_v_slot_expression_keeps_the_slot_function_parseable() {
    for template in [
        r#"<el-badge><template #content="">{{ label }}</template></el-badge>"#,
        r#"<el-badge><template #content=" ">{{ label }}</template></el-badge>"#,
    ] {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let scope = summary
            .scopes
            .iter()
            .find_map(|scope| match scope.data() {
                vize_croquis::ScopeData::VSlot(_) => Some(scope.id.as_u32()),
                _ => None,
            })
            .expect("the slot template must open a v-slot scope");
        let code = generate_virtual_ts_with_offsets(
            &summary,
            None,
            Some(&root),
            0,
            0,
            &Default::default(),
        )
        .code;
        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "the generated module must parse: {:?}\n{code}",
            parsed.diagnostics
        );
        let function = vize_carton::cstr!(
            "  void function _slot_content_{scope}(__vize_slot_props_{scope}: $SlotPayload<typeof __vize_slot_host_{scope}, \"content\">) {{"
        );
        let parameter = vize_carton::cstr!("    void __vize_slot_props_{scope};");
        for expected in [function, parameter] {
            assert!(
                code.lines().any(|line| line == expected.as_str()),
                "expected the exact line {expected:?} in:\n{code}"
            );
        }
    }
}

#[test]
fn hoisted_slot_constructor_branches_keep_the_component_owner() {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = super::generate_virtual_ts_with_offsets_and_checks(
        &summary,
        None,
        Some(&root),
        0,
        0,
        &super::VirtualTsOptions::default(),
        super::VirtualTsGenerationOptions {
            hoist_shared_preamble: true,
            ..Default::default()
        },
    )
    .code;

    let constructor = "import(\"vize:virtual-ts\").$SlotConstructor<typeof el_badge>";
    for helper in [
        "$SlotConstructorWithoutArguments",
        "$SlotConstructorWithArguments",
    ] {
        let reference = format!("import(\"vize:virtual-ts\").{helper}<{constructor}>");
        assert_eq!(
            code.matches(&reference).count(),
            1,
            "the hoisted direct branch must use one collision-proof {helper} reference:\n{code}",
        );
        assert_eq!(
            crate::virtual_ts::helpers::SHARED_PREAMBLE_DTS
                .matches(&format!("export type {helper}<"))
                .count(),
            1,
            "the hoisted helper module must export {helper}",
        );
    }
    assert_eq!(
        code.matches(&format!(
            "import(\"vize:virtual-ts\").$SlotConstructorInstance<{constructor}, __Z, __P>",
        ))
        .count(),
        1,
        "the branch selector must retain the exact component owner:\n{code}",
    );
    assert_eq!(
        crate::virtual_ts::helpers::SHARED_PREAMBLE_DTS
            .matches("export type $SlotConstructorInstance<")
            .count(),
        1,
        "the hoisted helper module must export the branch selector",
    );
    assert_eq!(
        code.matches("declare function __vizeSlotConstructorInstance")
            .count(),
        0,
        "a hoisted module must not redeclare the embedded selector:\n{code}",
    );
}
