//! Whole handler outputs around the identifier shape fast path.

#[cfg(test)]
mod tests {
    use super::super::generate_event_handler;
    use crate::codegen::context::CodegenContext;
    use crate::options::CodegenOptions;
    use crate::{ExpressionNode, SimpleExpressionNode, SourceLocation};
    use vize_l0::{Allocator, Box, Span};

    #[test]
    fn whole_handler_reference_and_non_reference_outputs_keep_cache_semantics() {
        let cases = [
            ("handler00", false, "handler00"),
            (
                "handler00",
                true,
                "(...args) => (handler00 && handler00(...args))",
            ),
            ("_处理$", false, "_处理$"),
            (
                "_ctx.handler",
                true,
                "(...args) => (_ctx.handler && _ctx.handler(...args))",
            ),
            ("e => save(e)", true, "e => save(e)"),
            ("count++", false, "$event => (count++)"),
            ("first(); second()", false, "$event => {first(); second()}"),
        ];
        for (source, cached, expected) in cases {
            let allocator = Allocator::new();
            let expression = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(source, false, SourceLocation::STUB),
                &allocator,
            ));
            for mapped in [false, true] {
                let mut ctx = CodegenContext::new(CodegenOptions {
                    source_map: mapped,
                    ..CodegenOptions::default()
                });
                generate_event_handler(&mut ctx, &expression, cached);
                assert_eq!(ctx.into_code().as_str(), expected, "{source}, {cached}");
            }
        }
    }

    #[test]
    fn cached_identifier_keeps_both_complete_authored_expression_links() {
        let allocator = Allocator::new();
        let mut location = SourceLocation::STUB;
        location.span = Span::new(3, 12);
        let expression = ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new("handler00", false, location),
            &allocator,
        ));
        let mut ctx = CodegenContext::new(CodegenOptions {
            source_map: true,
            ..CodegenOptions::default()
        });
        // The source is intentionally independent of the emitted parameter/wrapper.
        ctx.source = "xx handler00".into();
        generate_event_handler(&mut ctx, &expression, true);
        assert_eq!(
            ctx.out.as_str(),
            "(...args) => (handler00 && handler00(...args))"
        );
        let links: Vec<_> = ctx
            .out
            .links()
            .iter()
            .map(|link| {
                (
                    link.generated,
                    link.authored,
                    link.name.as_deref(),
                    link.segment,
                )
            })
            .collect();
        assert_eq!(
            links,
            vec![
                (Span::new(13, 22), Span::new(3, 12), None, true),
                (Span::new(26, 35), Span::new(3, 12), None, true),
            ]
        );
    }
}
