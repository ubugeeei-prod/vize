//! The historical lowering API names the shared L1 capability descriptor.

use vize_l0::config::VueVersion;

#[test]
fn lowering_exports_the_shared_descriptor() {
    for version in VueVersion::ALL {
        let caps: vize_l1::dialect::LegacyCaps = vize_l1_to_l2::LegacyCaps::for_version(version);
        let parser_caps = vize_l1::dialect::LegacyDialectCapabilities::for_dialect(version);
        assert_eq!(caps, parser_caps);
        assert_eq!(
            core::mem::size_of_val(&caps),
            core::mem::size_of_val(&parser_caps)
        );
    }
}
