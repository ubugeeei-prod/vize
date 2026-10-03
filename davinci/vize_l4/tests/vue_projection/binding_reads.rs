//! Read-only template values do not acquire cross-expression control flow.

use super::*;
use vize_l2::resolution::Usage;

#[test]
fn genuine_template_binding_write_rows_are_refused_without_mutating_original_resolution() {
    for attribute in ["", " lang=ts"] {
        for (expression, expected_usage) in
            [("value=2", Usage::Write), ("value++", Usage::ReadWrite)]
        {
            let arena = Allocator::default();
            let source = cstr!(
                "<template>{{{{{expression}}}}}</template><script setup{attribute}>let value=1;</script>"
            );
            let observed = lower_sfc_native(&arena, &source, options());
            assert!(observed.admitted().is_some(), "{:?}", observed.issues());
            let embed = observed.template().unwrap().embeds().first().unwrap();
            let resolution = observed
                .file()
                .unwrap()
                .file()
                .expression(embed.node.unwrap())
                .unwrap();
            let rows = resolution.table().unwrap().occurrences();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].usage, expected_usage);
            assert!(
                observed
                    .file()
                    .unwrap()
                    .exposure(resolution.binding(rows[0].binding).unwrap())
                    .is_some()
            );
            assert!(matches!(
                project_vue(observed.admitted().unwrap()),
                Err(VueProjectionError::UnsupportedTemplateBindings)
            ));
            assert!(matches!(
                project_vue_no_links(observed.admitted().unwrap()),
                Err(VueProjectionError::UnsupportedTemplateBindings)
            ));
            assert_eq!(rows[0].usage, expected_usage);
            assert!(core::ptr::eq(
                resolution.table().unwrap().expression().ast,
                embed.syntax.expression().unwrap()
            ));
            assert_eq!(
                observed.scripts().first().unwrap().block().source(),
                "let value=1;"
            );
        }
    }
}
