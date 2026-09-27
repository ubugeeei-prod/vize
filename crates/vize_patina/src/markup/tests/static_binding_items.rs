//! Full hook-trace witnesses for elements with no attached binding ops.

#[cfg(test)]
mod static_binding_items_tests {
    use crate::ir::TemplateSyntax;
    use crate::markup::differential::trace_document;
    use crate::markup::{L2Markup, L2Template, MarkupDocument};
    use vize_atelier_jsx::JsxLang;
    use vize_l0::Allocator;

    fn assert_template_trace(source: &str) {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let (root, errors) = vize_armature::Parser::new(&allocator, source).parse();
        assert_eq!(errors.len(), 0);
        let lowered = L2Template::lower(&allocator, source);
        assert_eq!(lowered.lowered().diagnostics.len(), 0);
        let markup = lowered.markup();
        let actual = trace_document(
            &MarkupDocument::from_l2(&markup, TemplateSyntax::Vue),
            source,
        );
        let reference = trace_document(&MarkupDocument::new(&root, TemplateSyntax::Vue), source);
        assert_eq!(actual, reference);
    }

    #[test]
    fn static_attributes_keep_authored_order_values_and_ranges() {
        assert_template_trace(
            r#"<div title="a &amp; b" disabled id="last"><img alt="photo" src="/p" /></div>"#,
        );
    }

    #[test]
    fn consumed_slot_name_and_scope_carrier_items_are_restored() {
        assert_template_trace(
            r#"<slot name="fallback" id="outlet"><span class="fallback">Text</span></slot>"#,
        );
        assert_template_trace(
            r#"<div v-if="visible" key="first" id="branch">One</div><div v-else key="second">Two</div>"#,
        );
        assert_template_trace(
            r#"<template v-for="(value, key, index) in entries" id="carrier"><span title="item">{{ value }}</span></template>"#,
        );
    }

    #[test]
    fn mixed_dynamic_items_keep_exact_callbacks_and_order() {
        assert_template_trace(
            r#"<button type="button" :title="label" id="middle" @click.stop="run" aria-label="end">Run</button>"#,
        );
    }

    #[test]
    fn projected_static_and_mixed_attributes_keep_exact_hook_trace() {
        let source = r#"const View = () => <section id="outer"><img alt="photo" src="/p" /><button type="button" onClick={run} title="end">Run</button></section>;"#;
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let lowered =
            vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
        assert_eq!(lowered.diagnostics.len(), 0);
        assert_eq!(lowered.roots.len(), 1);
        for root in &lowered.roots {
            let Ok(projected) = root.l2.as_ref() else {
                panic!("the admitted JSX root must project to L2");
            };
            let markup = L2Markup::from_projected_root(projected);
            let actual = trace_document(
                &MarkupDocument::from_l2(&markup, TemplateSyntax::Vue),
                source,
            );
            let reference = trace_document(
                &MarkupDocument::new(&root.root, TemplateSyntax::Vue),
                source,
            );
            assert_eq!(actual, reference);
        }
    }
}
