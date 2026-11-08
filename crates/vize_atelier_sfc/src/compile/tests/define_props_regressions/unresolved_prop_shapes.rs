use super::super::super::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn define_props_rejects_unresolved_object_shapes() {
    let cases = [
        ("P", "Cannot resolve macro type `P`."),
        ("Omit<T, 'x'>", "Cannot resolve macro type `T`."),
        ("Partial<T>", "Cannot resolve macro type `T`."),
        (
            "Record<MissingKeys, string>",
            "Cannot resolve macro type `MissingKeys`.",
        ),
    ];
    for (props, expected_message) in cases {
        let source = format!(
            r#"<script setup lang="ts">
defineProps<{props}>()
</script>"#
        );
        let descriptor =
            parse_sfc(&source, SfcParseOptions::default()).expect("parse reported SFC");
        let error = compile_sfc(&descriptor, SfcCompileOptions::default())
            .expect_err("compiler must reject an unresolved props object shape");

        assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
        assert_eq!(error.message, expected_message);
    }
}
