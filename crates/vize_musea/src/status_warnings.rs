//! Unknown `status` diagnostics for Art files.
//!
//! `parse_art` keeps `ArtDescriptor` field-stable, so these warnings are
//! collected through a separate entry rather than a new descriptor field.

use crate::parse::art_block::{find_art_block, parse_metadata};
use vize_s0::{Allocator, Vec};

/// Return unknown-status warnings for `source`, or an empty list if the Art
/// block is missing or metadata cannot be parsed.
pub fn parse_art_status_warnings<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> Vec<'a, &'a str> {
    let Ok(block) = find_art_block(source.as_bytes(), source) else {
        return Vec::new_in(&allocator);
    };
    match parse_metadata(allocator, &block, None, filename) {
        Ok((_, warnings)) => warnings,
        Err(_) => Vec::new_in(&allocator),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_art_status_warnings;
    use vize_s0::Allocator;

    #[test]
    fn unknown_status_returns_the_shared_warning() {
        let allocator = Allocator::new();
        let source = r#"<art title="Button" status="wip"></art>"#;
        assert_eq!(
            parse_art_status_warnings(&allocator, source, "button.art.vue").as_slice(),
            [
                "button.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
            ]
        );
    }

    #[test]
    fn known_status_and_missing_art_return_no_warnings() {
        let allocator = Allocator::new();
        let ready = r#"<art title="Button" status="ready"></art>"#;
        assert_eq!(
            parse_art_status_warnings(&allocator, ready, "button.art.vue").as_slice(),
            [] as [&str; 0]
        );
        assert_eq!(
            parse_art_status_warnings(&allocator, "<div></div>", "button.art.vue").as_slice(),
            [] as [&str; 0]
        );
        assert_eq!(
            parse_art_status_warnings(&allocator, r#"<art title="Button" status="wip"></art>"#, "")
                .as_slice(),
            [
                "anonymous.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
            ]
        );
    }
}
