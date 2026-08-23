//! Shared classification for `defineArt` / `<art status>` values.

use crate::types::ArtStatus;
use vize_carton::{Allocator, cstr};

#[inline]
pub(crate) fn classify_status(value: &str) -> ArtStatus {
    if value.eq_ignore_ascii_case("draft") {
        ArtStatus::Draft
    } else if value.eq_ignore_ascii_case("ready") {
        ArtStatus::Ready
    } else if value.eq_ignore_ascii_case("deprecated") {
        ArtStatus::Deprecated
    } else {
        ArtStatus::Draft
    }
}

#[inline]
pub(crate) fn is_unknown_status(value: &str) -> bool {
    !value.eq_ignore_ascii_case("draft")
        && !value.eq_ignore_ascii_case("ready")
        && !value.eq_ignore_ascii_case("deprecated")
}

pub(crate) fn unknown_status_warning<'a>(
    allocator: &'a Allocator,
    filename: &str,
    status: &str,
) -> &'a str {
    let name = if filename.is_empty() {
        "anonymous.art.vue"
    } else {
        filename
    };
    allocator.alloc_str(&cstr!(
        "{name}: unknown status \"{status}\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
    ))
}

#[cfg(test)]
mod tests {
    use super::{classify_status, is_unknown_status, unknown_status_warning};
    use crate::types::ArtStatus;
    use vize_carton::Allocator;

    #[test]
    fn classifies_known_and_unknown_status() {
        assert_eq!(classify_status("draft"), ArtStatus::Draft);
        assert_eq!(classify_status("READY"), ArtStatus::Ready);
        assert_eq!(classify_status("Deprecated"), ArtStatus::Deprecated);
        assert_eq!(classify_status("wip"), ArtStatus::Draft);
        assert!(is_unknown_status("wip"));
        assert!(!is_unknown_status("ready"));
    }

    #[test]
    fn formats_unknown_status_warning() {
        let allocator = Allocator::new();
        assert_eq!(
            unknown_status_warning(&allocator, "button.art.vue", "wip"),
            "button.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
        );
        assert_eq!(
            unknown_status_warning(&allocator, "", "wip"),
            "anonymous.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
        );
    }
}
