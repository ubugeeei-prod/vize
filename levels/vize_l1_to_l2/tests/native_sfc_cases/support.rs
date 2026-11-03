use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};

pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
