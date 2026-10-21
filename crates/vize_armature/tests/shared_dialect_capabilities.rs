//! Compatibility names must expose the shared L1 capability owner.

#[cfg(feature = "legacy")]
#[test]
fn parser_exports_are_the_shared_l1_types() {
    use vize_armature::legacy::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};
    use vize_l0::config::VueVersion;

    for version in VueVersion::ALL {
        let parser_caps: vize_l1::dialect::LegacyDialectCapabilities =
            LegacyDialectCapabilities::for_dialect(version);
        let lowering_caps: vize_l1::dialect::LegacyCaps =
            vize_l1::dialect::LegacyCaps::for_version(version);
        assert_eq!(parser_caps, lowering_caps);
    }
    let _: vize_l1::dialect::LegacyVueVersion = LegacyVueVersion::V2;
    let _: vize_l1::dialect::DirectiveArgStyle = DirectiveArgStyle::Colon;
}
