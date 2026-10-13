//! Borrow SFC text while the synchronous workspace-symbol collector owns its input.

use vize_croquis::sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_l0::String;

pub(super) fn parse<'source>(
    filename: &str,
    source: &'source str,
) -> Option<SfcDescriptor<'source>> {
    let options = SfcParseOptions {
        filename: String::from(filename),
        ..Default::default()
    };
    parse_sfc(source, options).ok()
}

#[cfg(test)]
mod tests;
