use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};

pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
