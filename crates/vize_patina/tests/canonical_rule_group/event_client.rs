//! Standalone consumer of the original public L2 node and event APIs.
use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

mod regions;
mod group {
    pub(super) use super::regions::original_preorder as preorder;
}
