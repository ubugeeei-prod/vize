//! Public descriptor compatibility over the L1-owned SFC container walk.
//!
//! L1 owns every opening tag, attribute, script/template/raw boundary and
//! source location. This module projects those original facts into the shipped
//! descriptor types without calling the retained Croquis SFC parser.

use crate::types::{SfcDescriptor, SfcError, SfcParseOptions};

mod native;

/// Parse a Vue SFC into the existing zero-copy descriptor.
#[inline]
pub fn parse_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    native::parse_sfc(source, options)
}

#[cfg(test)]
mod tests;
