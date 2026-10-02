//! The historical lowering API keeps the compact L1 capability type.

use vize_l0::config::VueVersion;

#[test]
fn lowering_exports_the_shared_compact_projection() {
    for version in VueVersion::ALL {
        let caps: vize_l1::dialect::LegacyCaps = vize_l1_to_l2::LegacyCaps::for_version(version);
        assert_eq!(caps, vize_l1::dialect::LegacyCaps::for_version(version));
        assert_eq!(core::mem::size_of_val(&caps), 3);
    }
}
