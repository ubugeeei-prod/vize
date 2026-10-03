//! Genuine original SFC owners through the checker emission and range mapper.

use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    cstr,
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l4::targets::ts::{
    MappingError, SourceKind,
    vue::{VueProjectionError, project_vue, project_vue_no_links},
};

#[path = "vue_projection/binding_reads.rs"]
mod binding_reads;

#[path = "vue_projection/jsdoc_reads.rs"]
mod jsdoc_reads;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn original_setup_and_each_template_read_keep_exact_owners_text_kind_and_links() {
    for (attribute, kind) in [
        ("", SourceKind::JavaScript),
        (" lang=ts", SourceKind::TypeScript),
    ] {
        let arena = Allocator::default();
        let source = cstr!(
            "\r\n<template><div title='static'>{{{{日本語.missing}}}}<span>{{{{日本語 + 2}}}}</span></div></template>\r\n<script setup{attribute}>/*😀*/ const 日本語 = 1; // tail</script>"
        );
        let observed = lower_sfc_native(&arena, &source, options());
        let projection = project_vue(observed.admitted().unwrap()).unwrap();
        let plain = project_vue_no_links(observed.admitted().unwrap()).unwrap();
        let original_file = observed.file().unwrap().file();
        assert!(core::ptr::eq(projection.file(), original_file));
        assert!(core::ptr::eq(
            projection.original().observation(),
            &observed
        ));
        assert_eq!(projection.source_kind(), kind);
        assert_eq!(
            projection.document().as_str(),
            "/*😀*/ const 日本語 = 1; // tail\n;\nexport {};\nvoid (\n日本語.missing\n);\nvoid (\n日本語 + 2\n);\n"
        );
        assert_eq!(projection.document().as_str(), plain.document().as_str());
        assert_eq!(projection.document().links().len(), 3);
        assert!(!plain.document().is_recording());
        assert!(plain.document().links().is_empty());
        assert_eq!(
            plain.map_span(Span::new(0, 0)),
            Err(MappingError::Unrecorded)
        );
        for link in projection.document().links() {
            assert_eq!(projection.map_span(link.generated), Ok(link.authored));
            assert_eq!(
                projection
                    .document()
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize),
                source.get(link.authored.start as usize..link.authored.end as usize)
            );
        }
        let script = observed.scripts().first().unwrap();
        let actual_program = script.syntax().unwrap().admitted_program().unwrap();
        assert_eq!(
            actual_program.program().source_text,
            script.block().source()
        );
        assert_eq!(observed.template().unwrap().embeds().len(), 2);
        for embed in observed.template().unwrap().embeds() {
            let resolution = projection.file().expression(embed.node.unwrap()).unwrap();
            assert!(core::ptr::eq(
                resolution.table().unwrap().expression().ast,
                embed.syntax.expression().unwrap()
            ));
            assert_eq!(
                resolution.scope(),
                observed.file().unwrap().setup().map(|setup| setup.scope())
            );
        }
    }
}

#[test]
fn diagnostic_ranges_use_original_decoding_atoms_utf16_and_generated_boundaries() {
    let arena = Allocator::default();
    let source = "\r\n<template>{{'😀&acE;&amp;日本語'.missing}}</template>\r\n<script setup lang=ts>/*😀*/ const unused = 1;</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let projection = project_vue(observed.admitted().unwrap()).unwrap();
    let text = projection.document().as_str();
    let expression = observed
        .template()
        .unwrap()
        .embeds()
        .first()
        .unwrap()
        .syntax
        .source();
    assert_eq!(expression.text(), "'😀∾̳&日本語'.missing");
    let start = u32::try_from(text.find(expression.text()).unwrap()).unwrap();
    let atom = text.find('∾').unwrap();
    let atom_utf16 = u32::try_from(text.get(..atom).unwrap().encode_utf16().count()).unwrap();
    let authored = source.find("&acE;").unwrap() as u32;
    // The five-byte decoded atom and five-byte authored entity are still atomic.
    assert_eq!(
        projection.map_utf16(atom_utf16, 1),
        Ok(Span::new(authored, authored + 5))
    );
    assert_eq!(
        projection.map_utf16(atom_utf16 + 1, 0),
        Ok(Span::new(authored, authored + 5))
    );
    let emoji = text.find('😀').unwrap();
    let middle = u32::try_from(text.get(..emoji).unwrap().encode_utf16().count()).unwrap() + 1;
    assert_eq!(
        projection.map_utf16(middle, 0),
        Err(MappingError::InvalidUtf16Boundary)
    );
    assert_eq!(
        projection.map_utf16(u32::MAX, 1),
        Err(MappingError::InvalidUtf16Boundary)
    );
    assert_eq!(
        projection.map_span(Span::new(start - 1, start + 1)),
        Err(MappingError::CrossesBoundary)
    );
    let end = start + expression.text().len() as u32;
    assert_eq!(
        projection.map_span(Span::new(end, end)),
        Ok(Span::new(expression.span().end, expression.span().end))
    );
    assert_eq!(
        projection.map_span(Span::new(end, end + 1)),
        Err(MappingError::CrossesBoundary)
    );
    assert_eq!(
        projection.map_span(Span::new(end + 1, end + 2)),
        Err(MappingError::GeneratedOnly)
    );
    assert_eq!(
        projection.map_span(Span::new(start + 2, start + 2)),
        Err(MappingError::InvalidRange)
    );
    let links = projection.document().links();
    assert_eq!(links.len(), 5);
    let entity_link = links
        .iter()
        .find(|link| link.authored == Span::new(authored, authored + 5))
        .unwrap();
    assert_eq!(entity_link.generated.end - entity_link.generated.start, 5);
    assert_eq!(
        projection.map_span(entity_link.generated),
        Ok(entity_link.authored)
    );
}

#[test]
fn scriptless_literals_and_line_comments_keep_complete_native_sources() {
    for source in [
        "<template>{{'literal'}}</template>",
        "<template>{{ 1 // raw tail\n + 2}}</template>",
        "<script setup>let value=1;</script>",
    ] {
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, source, options());
        let projection = project_vue(observed.admitted().unwrap()).unwrap();
        assert!(core::ptr::eq(
            projection.file(),
            observed.file().unwrap().file()
        ));
        assert_eq!(projection.source_kind(), SourceKind::JavaScript);
        if source.contains("raw tail") {
            assert_eq!(
                projection.document().as_str(),
                "\n;\nexport {};\nvoid (\n1 // raw tail\n + 2\n);\n"
            );
        }
        assert_eq!(observed.issues().len(), 0);
    }
}

#[test]
fn template_bindings_require_original_whole_unit_primitive_proof_without_restricting_script_copy() {
    for attribute in ["", " lang=ts"] {
        for script in [
            "import {counter} from './counter'; let value=counter;",
            "let value=1; value=2;",
            "let value={n:1};",
        ] {
            let arena = Allocator::default();
            let source = cstr!(
                "<template>{{{{value.toFixed}}}}</template><script setup{attribute}>{script}</script>"
            );
            let observed = lower_sfc_native(&arena, &source, options());
            assert!(observed.admitted().is_some(), "{:?}", observed.issues());
            let file = observed.file().unwrap().file();
            let embed = observed.template().unwrap().embeds().first().unwrap();
            let resolution = file.expression(embed.node.unwrap()).unwrap();
            let occurrences = resolution.table().unwrap().occurrences();
            assert_eq!(occurrences.len(), 1);
            let actual_binding = resolution.binding(occurrences[0].binding).unwrap();
            assert!(observed.file().unwrap().exposure(actual_binding).is_some());
            assert!(matches!(
                project_vue(observed.admitted().unwrap()),
                Err(VueProjectionError::UnsupportedTemplateBindings)
            ));
            assert!(matches!(
                project_vue_no_links(observed.admitted().unwrap()),
                Err(VueProjectionError::UnsupportedTemplateBindings)
            ));
            assert!(core::ptr::eq(
                resolution.table().unwrap().expression().ast,
                embed.syntax.expression().unwrap()
            ));
            assert_eq!(observed.scripts().first().unwrap().block().source(), script);

            // The genuine original import and complete Program still receive
            // checker projection when no template read needs Vue ref semantics.
            for template in ["", "<template>{{1}}</template>"] {
                let source = cstr!("{template}<script setup{attribute}>{script}</script>");
                let observed = lower_sfc_native(&arena, &source, options());
                let projection = project_vue(observed.admitted().unwrap()).unwrap();
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
                assert!(core::ptr::eq(
                    projection.original().observation(),
                    &observed
                ));
            }
        }
    }
}

#[test]
fn original_sfc_call_refusals_keep_the_actual_program_for_script_and_template_sources() {
    for attribute in ["", " lang=ts"] {
        for template in ["", "<template>{{value}}</template>"] {
            let arena = Allocator::default();
            let script = "let value=Math.abs(-1);";
            let source = cstr!("{template}<script setup{attribute}>{script}</script>");
            let observed = lower_sfc_native(&arena, &source, options());
            assert!(observed.admitted().is_none());
            let retained = observed.scripts().first().unwrap();
            assert_eq!(retained.block().source(), script);
            assert_eq!(
                retained
                    .syntax()
                    .unwrap()
                    .admitted_program()
                    .unwrap()
                    .program()
                    .source_text,
                script
            );
            assert!(core::ptr::eq(
                retained.block().source(),
                retained.syntax().unwrap().source().text()
            ));
            assert!(!observed.issues().is_empty());
        }
    }
}

#[test]
fn completed_owners_refuse_unchecked_sfc_type_families_without_losing_observations() {
    for (source, error) in [
        (
            "<template>{{1}}</template><script>const outer=1;</script>",
            VueProjectionError::UnsupportedScripts,
        ),
        (
            "<template>{{value}}</template><script>const outer=1;</script><script setup>const value=2;</script>",
            VueProjectionError::UnsupportedScripts,
        ),
        (
            "<template>{{1}}</template><style>.a{color:red}</style>",
            VueProjectionError::UnsupportedStyles,
        ),
        (
            "<template><div :title='value'>{{value}}</div></template><script setup>const value=1;</script>",
            VueProjectionError::UnsupportedTemplate,
        ),
        (
            "<template><search>{{1}}</search></template>",
            VueProjectionError::UnsupportedTemplate,
        ),
    ] {
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, source, options());
        assert!(
            observed.admitted().is_some(),
            "{source}: {:?}",
            observed.issues()
        );
        assert!(
            matches!(project_vue(observed.admitted().unwrap()), Err(actual) if actual == error)
        );
        assert!(observed.file().unwrap().file().is_complete());
        assert!(observed.template().unwrap().component().is_some());
    }
    for source in [
        "<template>{{ 1 // raw tail\n}}</template>",
        "<template><Child>{{1}}</Child></template>",
        "<template>{{missing}}</template>",
        "<template>{{value}}</template><script setup>const value: number=1;</script>",
        "<template><div v-if='value'>{{value}}</div></template><script setup>const value=1;</script>",
    ] {
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, source, options());
        assert!(observed.admitted().is_none(), "{source}");
        assert!(observed.template().unwrap().component().is_some());
        assert!(!observed.template().unwrap().embeds().is_empty() || !observed.issues().is_empty());
    }
}
