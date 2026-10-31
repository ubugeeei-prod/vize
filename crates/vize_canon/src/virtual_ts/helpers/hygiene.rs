//! Keep generated helper bindings separate from authored module bindings.

use vize_carton::{String, cstr};
use vize_croquis::Croquis;

fn authored_binding(summary: &Croquis, name: &str) -> bool {
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
    output: &mut String,
    summary: &Croquis,
    helpers: &str,
    imported: bool,
) {
    if !authored_binding(summary, "$LiteralKeys") {
        output.push_str(helpers);
        return;
    }

    let alias = literal_keys_alias(summary);
    if imported {
        output.push_str(
            &helpers.replace("  $LiteralKeys,\n", &cstr!("  $LiteralKeys as {alias},\n")),
        );
    } else {
        output.push_str(&helpers.replace("$LiteralKeys", &alias));
    }
}

/// `PropertyKey` is an authored lexical name too; keyword types cannot capture it.
pub(in crate::virtual_ts) fn emit_inference_helpers(
    output: &mut String,
    summary: &Croquis,
    helpers: &str,
) {
    if authored_binding(summary, "PropertyKey") {
        output.push_str(&helpers.replace(
            "O extends PropertyKey",
            "O extends string | number | symbol",
        ));
    } else {
        output.push_str(helpers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_ts::helpers::{CONTENT_MAPPER_TYPE_HELPERS, VUE_TYPE_HELPERS};
    use vize_croquis::{Analyzer, AnalyzerOptions};

    const INFERENCE: &str = include_str!("declarations/emit-inference.d.ts.txt");

    fn summary(script: &str) -> Croquis {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.finish()
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
                let mut output = String::default();
                emit_type_helpers(&mut output, &summary, helpers, imported);
                assert_eq!(output, helpers);
            }
            let mut output = String::default();
            emit_inference_helpers(&mut output, &summary, INFERENCE);
            assert_eq!(output, INFERENCE);
        }
    }

    fn assert_complete_alias_change(summary: &Croquis, alias: &str) {
        assert_eq!(VUE_TYPE_HELPERS.matches("$LiteralKeys").count(), 2);
        let mut embedded = String::default();
        emit_type_helpers(&mut embedded, summary, VUE_TYPE_HELPERS, false);
        assert_eq!(embedded.matches(alias).count(), 2);
        assert_eq!(
            embedded.replacen(alias, "$LiteralKeys", 2),
            VUE_TYPE_HELPERS
        );

        const ORIGINAL: &str = "  $LiteralKeys,\n";
        let replacement = cstr!("  $LiteralKeys as {alias},\n");
        assert_eq!(CONTENT_MAPPER_TYPE_HELPERS.matches(ORIGINAL).count(), 1);
        let mut shared = String::default();
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
        assert_eq!(INFERENCE.matches("O extends PropertyKey").count(), 2);
        let mut output = String::default();
        emit_inference_helpers(&mut output, summary, INFERENCE);
        assert_eq!(
            output.matches("O extends string | number | symbol").count(),
            2
        );
        assert_eq!(output.matches("O extends PropertyKey").count(), 0);
        assert_eq!(
            output.replacen(
                "O extends string | number | symbol",
                "O extends PropertyKey",
                2
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
