//! Original JSDoc presence refuses only the unfinished JS template binding read.

use super::*;

#[test]
fn original_jsdoc_read_refusal_retains_complete_owner_and_script_literal_copies() {
    for (attribute, kind) in [
        ("", SourceKind::JavaScript),
        (" lang=ts", SourceKind::TypeScript),
    ] {
        for comment in [
            "/** @type {import('vue').Ref<number>} */",
            "/** @license original @type {import('vue').Ref<number>} */",
        ] {
            let arena = Allocator::default();
            let script = cstr!("{comment} let value=1;");
            let source = cstr!(
                "<script setup{attribute}>{script}</script><template>{{{{value.toFixed}}}}</template>"
            );
            let observed = lower_sfc_native(&arena, &source, options());
            assert!(observed.admitted().is_some(), "{:?}", observed.issues());
            let syntax = observed.scripts().first().unwrap().syntax().unwrap();
            let program = syntax.admitted_program().unwrap();
            assert!(program.has_jsdoc_comments());
            assert_eq!(program.source(), script.as_str());
            let embed = observed.template().unwrap().embeds().first().unwrap();
            let resolution = observed
                .file()
                .unwrap()
                .file()
                .expression(embed.node.unwrap())
                .unwrap();
            let ast = resolution.table().unwrap().expression().ast;
            assert!(core::ptr::eq(ast, embed.syntax.expression().unwrap()));
            if kind == SourceKind::JavaScript {
                assert!(matches!(
                    project_vue(observed.admitted().unwrap()),
                    Err(VueProjectionError::UnsupportedTemplateBindings)
                ));
                assert!(matches!(
                    project_vue_no_links(observed.admitted().unwrap()),
                    Err(VueProjectionError::UnsupportedTemplateBindings)
                ));
            } else {
                let projection = project_vue(observed.admitted().unwrap()).unwrap();
                assert_eq!(projection.source_kind(), kind);
                assert_eq!(
                    projection.document().as_str(),
                    cstr!("{script}\n;\nexport {{}};\nvoid (\nvalue.toFixed\n);\n")
                );
            }
            assert_eq!(syntax.admitted_program().unwrap().source(), script.as_str());
            assert!(core::ptr::eq(ast, embed.syntax.expression().unwrap()));
            assert!(observed.file().unwrap().file().is_complete());

            for template in ["", "<template>{{1}}</template>"] {
                let source = cstr!("<script setup{attribute}>{script}</script>{template}");
                let observed = lower_sfc_native(&arena, &source, options());
                let projection = project_vue(observed.admitted().unwrap()).unwrap();
                let plain = project_vue_no_links(observed.admitted().unwrap()).unwrap();
                assert_eq!(projection.source_kind(), kind);
                assert_eq!(
                    projection.document().as_str(),
                    cstr!(
                        "{script}\n;\nexport {{}};\n{}",
                        if template.is_empty() {
                            ""
                        } else {
                            "void (\n1\n);\n"
                        }
                    )
                );
                assert_eq!(projection.document().as_str(), plain.document().as_str());
                assert!(plain.document().links().is_empty());
                assert!(core::ptr::eq(
                    projection.file(),
                    observed.file().unwrap().file()
                ));
                let first = projection.document().links().first().unwrap();
                assert_eq!(projection.map_span(first.generated), Ok(first.authored));
                assert_eq!(
                    &source[first.authored.start as usize..first.authored.end as usize],
                    script.as_str()
                );
            }
        }
    }
}

#[test]
fn original_ordinary_comments_and_quoted_doc_text_keep_js_binding_reads() {
    for script in [
        "/* ordinary */ let value=1; // ordinary\n",
        "/* @type {import('vue').Ref<number>} */ let value=1;",
        "// /** @type {import('vue').Ref<number>} */\nlet value=1;",
        "let value='/** @type {import(\"vue\").Ref<number>} */';",
    ] {
        let arena = Allocator::default();
        let source =
            cstr!("<script setup>{script}</script><template>{{{{value.missing}}}}</template>");
        let observed = lower_sfc_native(&arena, &source, options());
        assert!(observed.admitted().is_some(), "{:?}", observed.issues());
        let original = observed
            .scripts()
            .first()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .unwrap();
        assert!(!original.has_jsdoc_comments());
        let projection = project_vue(observed.admitted().unwrap()).unwrap();
        assert_eq!(projection.source_kind(), SourceKind::JavaScript);
        assert_eq!(
            projection.document().as_str(),
            cstr!("{script}\n;\nexport {{}};\nvoid (\nvalue.missing\n);\n")
        );
        assert_eq!(original.source(), script);
    }
}
