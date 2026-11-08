//! Explicit native descriptor and document-print policy.

use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1::markup::SurfaceParseOptions;

use super::super::PrintOptions;

/// Explicit native directive formatting; the default remains strict refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcDirectivePolicy {
    /// Reject every directive value that would require expression formatting.
    Refuse,
    /// Format conditional directive expressions and refuse binding values.
    FormatConditionals,
    /// Format conditional expressions and admitted static binding values.
    FormatConditionalsAndStaticBindings,
}

/// The native family uses these policies directly, without legacy option conversion.
/// Authored outer bytes, literal spellings and physical line endings remain original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSfcOptions {
    /// Vue descriptor parsing and template-surface policy.
    pub descriptor: DescriptorOptions,
    /// Document layout and generated line-ending policy.
    pub print: PrintOptions,
    /// Native directive values that may participate in formatting.
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
