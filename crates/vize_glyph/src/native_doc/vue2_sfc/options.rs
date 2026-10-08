use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};

use super::super::PrintOptions;

/// Dedicated native policies, without legacy option conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeVue2SfcOptions {
    pub descriptor: DescriptorOptions,
    pub print: PrintOptions,
}

impl Default for NativeVue2SfcOptions {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: VueVersion::V2,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            print: PrintOptions::default(),
        }
    }
}
