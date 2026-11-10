//! Keep generated helper bindings separate from authored module bindings.

use vize_carton::{String, cstr};
use vize_croquis::Croquis;

pub(in crate::virtual_ts) fn authored_binding(summary: &Croquis, name: &str) -> bool {
    let definitions = summary.types.definitions();
    definitions.type_aliases.contains_key(name)
        || definitions.interfaces.contains_key(name)
        || definitions.imported_types.contains_key(name)
        || summary
            .type_exports
            .iter()
            .any(|export| export.name.as_str() == name)
        || crate::virtual_ts::script_facts::contains_binding(summary, name)
}

fn literal_keys_alias(summary: &Croquis) -> String {
    let mut alias = String::from("__vize$LiteralKeys");
    let mut suffix = 0;
    while authored_binding(summary, &alias) {
        suffix += 1;
        alias = cstr!("__vize$LiteralKeys_{suffix}");
    }
    alias
}

/// Only generated text is renamed; the shared module's exported name stays put.
pub(in crate::virtual_ts) fn emit_type_helpers(
    output: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    helpers: &str,
    imported: bool,
) {
    if !authored_binding(summary, "$LiteralKeys") {
        emit_helper_text(output, summary, helpers);
        return;
    }

    let alias = literal_keys_alias(summary);
    let rewritten = if imported {
        helpers.replace("  $LiteralKeys,\n", &cstr!("  $LiteralKeys as {alias},\n"))
    } else {
        helpers.replace("$LiteralKeys", &alias)
    };
    emit_helper_text(output, summary, &rewritten);
}

/// `PropertyKey` is an authored lexical name too; keyword types cannot capture it.
pub(in crate::virtual_ts) fn emit_inference_helpers(
    output: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    helpers: &str,
) {
    if authored_binding(summary, "PropertyKey") {
        let rewritten = helpers.replace(
            "O extends PropertyKey",
            "O extends string | number | symbol",
        );
        emit_helper_text(output, summary, &rewritten);
    } else {
        emit_helper_text(output, summary, helpers);
    }
}

/// Inline helpers share the SFC module. An authored `Pick` therefore captures
/// unqualified lib instantiations (`TS2314` when that alias has one parameter).
/// Lib `Pick` is `{ [P in K]: T[P] }` with `K extends keyof T`, so a fresh
/// homomorphic alias keeps modifiers, unions, and constructor props. Shared
/// helper modules are separate scopes and keep the builtin name.
pub(in crate::virtual_ts) fn emit_helper_text(
    output: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    helpers: &str,
) {
    if !authored_binding(summary, "Pick") || !helpers.contains("Pick<") {
        crate::virtual_ts::generated_names::push_lib_text(output, helpers);
        return;
    }
    let alias = owned_pick_alias(summary);
    let declaration = cstr!("type {alias}<T, K extends keyof T> = {{ [P in K]: T[P] }};\n");
    if !output.contains(declaration.as_str()) {
        output.push_str(&declaration);
    }
    crate::virtual_ts::generated_names::push_lib_text(
        output,
        &helpers.replace("Pick<", &cstr!("{alias}<")),
    );
}

fn owned_pick_alias(summary: &Croquis) -> String {
    let mut alias = String::from("__vizePick");
    let mut suffix = 0;
    while authored_binding(summary, &alias) {
        suffix += 1;
        alias = cstr!("__vizePick_{suffix}");
    }
    alias
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_ts::helpers::{
        CONTENT_MAPPER_TYPE_HELPERS, EMIT_OVERLOAD_HELPERS, VUE_TYPE_HELPERS,
    };
    use oxc_syntax::identifier::is_identifier_part;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    const INFERENCE: &str = include_str!("declarations/emit-inference.d.ts.txt");
    const LEGACY_TYPE_HELPERS: &str = concat!(
        include_str!("declarations/core-types.d.ts.txt"),
        include_str!("declarations/vue2-types.d.ts.txt"),
        include_str!("declarations/prop-keys.d.ts.txt"),
        include_str!("declarations/vue2-component-and-list.d.ts.txt")
    );
    const RAW_LEGACY_HELPERS: &str = concat!(
        include_str!("declarations/vue2-define-component.d.ts.txt"),
        include_str!("declarations/vue2-instance.d.ts.txt")
    );
    const INLINE_LIB_TYPES: &[&str] = &[
        "ArrayConstructor",
        "BooleanConstructor",
        "Capitalize",
        "Date",
        "DateConstructor",
        "Element",
        "Exclude",
        "Extract",
        "Function",
        "FunctionConstructor",
        "GlobalEventHandlersEventMap",
        "Iterable",
        "Lowercase",
        "NonNullable",
        "NumberConstructor",
        "Object",
        "ObjectConstructor",
        "Omit",
        "Parameters",
        "Partial",
        "Pick",
        "PropertyKey",
        "Readonly",
        "Record",
        "Required",
        "StringConstructor",
        "ThisType",
    ];

    fn summary(script: &str) -> Croquis {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.finish()
    }

    fn exact_unqualified_count(source: &str, name: &str) -> usize {
        source
            .match_indices(name)
            .filter(|&(start, matched)| {
                let before = source
                    .get(..start)
                    .and_then(|head| head.chars().next_back());
                let after = source
                    .get(start + matched.len()..)
                    .and_then(|tail| tail.chars().next());
                before.is_none_or(|c| c != '.' && c != '$' && !is_identifier_part(c))
                    && after.is_none_or(|c| c != '$' && !is_identifier_part(c))
            })
            .count()
    }

    fn exact_qualified_count(source: &str, name: &str) -> usize {
        let qualified = format!("globalThis.{name}");
        source
            .match_indices(qualified.as_str())
            .filter(|&(start, matched)| {
                source
                    .get(start + matched.len()..)
                    .and_then(|tail| tail.chars().next())
                    .is_none_or(|c| c != '$' && !is_identifier_part(c))
            })
            .count()
    }

    #[test]
    fn every_inline_lib_type_avoids_authored_declarations_and_imports() {
        let declarations = INLINE_LIB_TYPES
            .iter()
            .map(|name| format!("type {name} = never;"))
            .collect::<Vec<_>>()
            .join("\n");
        let imports = format!(
            "import type {{ {} }} from './lib-types';",
            INLINE_LIB_TYPES.join(", ")
        );
        let helper_chunks = [
            VUE_TYPE_HELPERS,
            LEGACY_TYPE_HELPERS,
            EMIT_OVERLOAD_HELPERS,
            INFERENCE,
            include_str!("pattern_matching.d.ts"),
            include_str!("declarations/slot-resolver.d.ts.txt"),
            include_str!("declarations/slot-payload.d.ts.txt"),
        ];
        let original = helper_chunks
            .iter()
            .copied()
            .chain([RAW_LEGACY_HELPERS])
            .collect::<std::string::String>();

        for script in [declarations.as_str(), imports.as_str()] {
            let summary = summary(script);
            let _binding = crate::virtual_ts::generated_names::bind(None, &summary);
            let mut output = crate::virtual_ts::TextBuf::default();
            for helpers in helper_chunks {
                emit_helper_text(&mut output, &summary, helpers);
            }
            crate::virtual_ts::generated_names::push_lib_text(&mut output, RAW_LEGACY_HELPERS);

            for name in INLINE_LIB_TYPES {
                let original_count = exact_unqualified_count(&original, name);
                assert_ne!(original_count, 0, "the audit must activate {name}");
                assert_eq!(
                    exact_unqualified_count(&output, name),
                    0,
                    "authored {name} captured an inline helper:\n{output}"
                );
                if *name == "Pick" {
                    assert_eq!(
                        output.matches("__vizePick<").count(),
                        original_count + 1,
                        "the fresh alias declaration is emitted once beside every rewritten use"
                    );
                } else {
                    assert_eq!(
                        exact_qualified_count(&output, name),
                        original_count,
                        "every generated {name} reference must be qualified"
                    );
                }
            }
        }
    }

    #[test]
    fn noncolliding_authored_contracts_keep_complete_helper_bytes() {
        for script in [
            "defineProps<{ value: string }>(); defineEmits<{ select: [value: number] }>();",
            "const note = '$LiteralKeys PropertyKey'; function local<$LiteralKeys, PropertyKey>() {}",
        ] {
            let summary = summary(script);
            assert_eq!(
                [
                    authored_binding(&summary, "$LiteralKeys"),
                    authored_binding(&summary, "PropertyKey"),
                ],
                [false, false],
                "{script}"
            );
            for (helpers, imported) in [
                (VUE_TYPE_HELPERS, false),
                (CONTENT_MAPPER_TYPE_HELPERS, true),
            ] {
                let mut output = crate::virtual_ts::TextBuf::default();
                emit_type_helpers(&mut output, &summary, helpers, imported);
                assert_eq!(output, helpers);
            }
            let mut output = crate::virtual_ts::TextBuf::default();
            emit_inference_helpers(&mut output, &summary, INFERENCE);
            assert_eq!(output, INFERENCE);
        }
    }

    fn assert_complete_alias_change(summary: &Croquis, alias: &str) {
        assert_eq!(VUE_TYPE_HELPERS.matches("$LiteralKeys").count(), 2);
        let mut embedded = crate::virtual_ts::TextBuf::default();
        emit_type_helpers(&mut embedded, summary, VUE_TYPE_HELPERS, false);
        assert_eq!(embedded.matches(alias).count(), 2);
        assert_eq!(
            embedded.replacen(alias, "$LiteralKeys", 2),
            VUE_TYPE_HELPERS
        );

        const ORIGINAL: &str = "  $LiteralKeys,\n";
        let replacement = cstr!("  $LiteralKeys as {alias},\n");
        assert_eq!(CONTENT_MAPPER_TYPE_HELPERS.matches(ORIGINAL).count(), 1);
        let mut shared = crate::virtual_ts::TextBuf::default();
        emit_type_helpers(&mut shared, summary, CONTENT_MAPPER_TYPE_HELPERS, true);
        assert_eq!(shared.matches(replacement.as_str()).count(), 1);
        assert_eq!(shared.matches(alias).count(), 1);
        assert_eq!(
            shared.replacen(replacement.as_str(), ORIGINAL, 1),
            CONTENT_MAPPER_TYPE_HELPERS,
            "every other import and helper byte remains exact"
        );
    }

    #[test]
    fn authored_type_aliases_choose_a_fresh_generated_binding_in_both_modes() {
        let summary = summary(
            "type $LiteralKeys<T> = never; type __vize$LiteralKeys<T> = T; interface __vize$LiteralKeys_1 {}",
        );
        assert_complete_alias_change(&summary, "__vize$LiteralKeys_2");
    }

    #[test]
    fn parsed_alias_interface_import_and_export_bindings_are_all_protected() {
        for script in [
            "type $LiteralKeys<T> = never; type PropertyKey = never;",
            "interface $LiteralKeys {} interface PropertyKey {}",
            "import type { Key as $LiteralKeys, Key as PropertyKey } from './types';",
            "import type * as $LiteralKeys from './keys'; import type PropertyKey from './keys';",
            "export type $LiteralKeys<T> = never; export type PropertyKey = never;",
        ] {
            let summary = summary(script);
            assert_eq!(
                [
                    authored_binding(&summary, "$LiteralKeys"),
                    authored_binding(&summary, "PropertyKey"),
                ],
                [true, true],
                "{script}"
            );
            assert_complete_alias_change(&summary, "__vize$LiteralKeys");
            assert_complete_intrinsic_constraints(&summary);
        }
    }

    fn assert_complete_intrinsic_constraints(summary: &Croquis) {
        assert_eq!(INFERENCE.matches("O extends PropertyKey").count(), 3);
        let mut output = crate::virtual_ts::TextBuf::default();
        emit_inference_helpers(&mut output, summary, INFERENCE);
        assert_eq!(
            output.matches("O extends string | number | symbol").count(),
            3
        );
        assert_eq!(output.matches("O extends PropertyKey").count(), 0);
        assert_eq!(
            output.replacen(
                "O extends string | number | symbol",
                "O extends PropertyKey",
                3
            ),
            INFERENCE,
            "owned inputs, required fields and native constructor inference stay complete"
        );
    }

    #[test]
    fn authored_property_key_cannot_erase_owned_emit_constraints() {
        assert_complete_intrinsic_constraints(&summary("type PropertyKey = never;"));
    }
}
