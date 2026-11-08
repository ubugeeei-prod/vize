use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};

use super::super::PrintOptions;

/// Dedicated native policies, without legacy option conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeVue2SfcOptions {
    /// Vue 2 descriptor parsing and template-surface policy.
    pub descriptor: DescriptorOptions,
    /// Document layout and generated line-ending policy.
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
