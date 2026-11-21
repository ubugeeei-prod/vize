use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::container::vue::DescriptorOptions;

pub(super) fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}
