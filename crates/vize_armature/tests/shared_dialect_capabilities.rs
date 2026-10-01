//! Compatibility names must expose the shared L1 capability owner.

#[cfg(feature = "legacy")]
#[test]
fn parser_exports_are_the_shared_l1_types() {
    use vize_armature::legacy::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};
    use vize_l0::config::VueVersion;

    for version in VueVersion::ALL {
        let parser_caps: vize_l1::dialect::LegacyDialectCapabilities =
            LegacyDialectCapabilities::for_dialect(version);
        let lowering_caps = vize_l1::dialect::LegacyCaps::for_version(version);
        assert_eq!(parser_caps.supports_filters, lowering_caps.supports_filters);
        assert_eq!(
            parser_caps.scoped_slot_attrs,
            lowering_caps.scoped_slot_attrs
        );
        assert_eq!(parser_caps.v2_event_sugar, lowering_caps.v2_event_sugar);
    }
    let _: vize_l1::dialect::LegacyVueVersion = LegacyVueVersion::V2;
    let _: vize_l1::dialect::DirectiveArgStyle = DirectiveArgStyle::Colon;
}
