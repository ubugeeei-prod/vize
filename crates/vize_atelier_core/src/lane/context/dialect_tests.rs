#[cfg(test)]
mod tests {
    use crate::lane::TransformContext;
    use crate::options::TransformOptions;
    use vize_l0::Allocator;
    use vize_l0::config::VueVersion;

    #[test]
    fn legacy_capabilities_follow_current_options_without_stale_context_state() {
        let allocator = Allocator::new();
        let mut context = TransformContext::new(&allocator, "", TransformOptions::default());
        assert_eq!(context.options.dialect, VueVersion::V3);
        for (dialect, filters, events) in [
            (VueVersion::V3, false, false),
            (VueVersion::V2, true, true),
            (VueVersion::V2_7, true, true),
            (VueVersion::V1, true, false),
            (VueVersion::V0_11, true, false),
            (VueVersion::V0_10, true, false),
            (VueVersion::V3, false, false),
        ] {
            context.options.dialect = dialect;
            assert_eq!(
                (
                    context.supports_filters(),
                    context.supports_v2_event_sugar()
                ),
                (filters, events),
                "{dialect:?}"
            );
        }
    }
}
