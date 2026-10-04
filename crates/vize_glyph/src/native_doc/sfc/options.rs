//! Explicit native descriptor and document-print policy.

use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::SurfaceParseOptions;
use vize_l1::container::vue::DescriptorOptions;

use super::super::PrintOptions;

/// Explicit native directive formatting; the default remains strict refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcDirectivePolicy {
    Refuse,
    FormatConditionals,
}

/// The native family uses these policies directly, without legacy option conversion.
/// Authored outer bytes, literal spellings and physical line endings remain original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSfcOptions {
    pub descriptor: DescriptorOptions,
    pub print: PrintOptions,
    pub directives: NativeSfcDirectivePolicy,
}

impl Default for NativeSfcOptions {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            print: PrintOptions::default(),
            directives: NativeSfcDirectivePolicy::Refuse,
        }
    }
}
