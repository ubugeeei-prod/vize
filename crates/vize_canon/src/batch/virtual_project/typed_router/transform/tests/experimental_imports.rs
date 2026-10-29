use super::{FILE, transform, unchanged};

#[test]
fn published_experimental_page_import_keeps_complete_code_and_authored_anchors() {
    for newline in ["\n", "\r\n"] {
        let source = "import { definePage } from 'vue-router/experimental';\n// 雪😀\ndefinePage({ params: { path: { missing: 'int' } } }); const after = 1;".replace('\n', newline);
        let (output, imported) = transform(&source, false);
        assert!(!imported);
        assert_eq!(
            output.code,
            source.replace("definePage(", &vize_l0::cstr!("definePage<{FILE}>("))
        );
        for token in ["missing", "after"] {
            let authored = source.find(token).unwrap();
            let generated = output.code.find(token).unwrap();
            assert_eq!(output.mapping.to_generated(authored), Some(generated));
            assert_eq!(output.mapping.to_authored(generated), Some(authored));
            assert_eq!(
                output
                    .mapping
                    .diagnostic_range_to_authored(generated, generated + token.len()),
                Some((authored, authored + token.len()))
            );
        }
        unchanged(&source, true);
    }
}

#[test]
fn experimental_entry_does_not_authorize_foreign_or_unpublished_value_bindings() {
    for source in [
        "import { useRoute } from 'vue-router/experimental'; useRoute(); type Route = typeof useRoute;",
        "import { definePage } from 'other/experimental'; definePage({});",
        "import type { definePage } from 'vue-router/experimental'; definePage({});",
        "import { type definePage } from 'vue-router/experimental'; definePage({});",
        "import type definePage from 'vue-router/experimental'; definePage({});",
        "import type * as definePage from 'vue-router/experimental'; definePage({});",
        "import type { other as definePage } from 'other'; definePage({});",
        "import type { useRoute } from 'vue-router'; useRoute(); type Route = typeof useRoute;",
        "import { type useRoute } from 'vue-router'; useRoute(); type Route = typeof useRoute;",
        "import { definePage as page } from 'vue-router/experimental'; page({});",
        "import { other as definePage } from 'vue-router/experimental'; definePage({});",
        "import * as page from 'vue-router/experimental'; page.definePage({});",
    ] {
        unchanged(source, false);
    }
}

#[test]
fn imported_page_macro_does_not_rewrite_shadowed_calls_or_explicit_generics() {
    let source = "import { definePage } from 'vue-router/experimental'; definePage({}); function scoped(definePage: (value: object) => void) { definePage({}); } definePage<'known'>({});";
    let (output, imported) = transform(source, false);
    assert!(!imported);
    assert_eq!(
        output.code,
        source.replacen("definePage(", &vize_l0::cstr!("definePage<{FILE}>("), 1)
    );
}

#[test]
fn a_type_alias_does_not_hide_the_auto_imported_value_macro() {
    let source = "type definePage = object; definePage({});";
    let (output, imported) = transform(source, false);
    assert_eq!(imported, false);
    assert_eq!(
        output.code,
        source.replace("definePage(", &vize_l0::cstr!("definePage<{FILE}>("))
    );
}
