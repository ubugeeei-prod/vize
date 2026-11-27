//! Pure component filename casing policy shared by configured lint backends.

pub(crate) fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Single-letter and all-uppercase names are valid PascalCase. Keep
    // punctuation and separators out so dotted or underscored stems still
    // follow the component filename conventions.
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// A hyphen-separated lowercase name: `my-component`, `job-board-2`.
///
/// Single-segment names never reach here — the caller returns early on an
/// all-lowercase stem — so a lone `-` boundary is what this actually decides.
pub(crate) fn is_kebab_case(s: &str) -> bool {
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return false;
    }
    // A leading, trailing, or doubled hyphen leaves an empty segment.
    let mut segments = s.split('-');
    segments.next().is_some_and(|first| {
        first.starts_with(|c: char| c.is_ascii_lowercase())
            && segments.all(|segment| !segment.is_empty())
    })
}

pub(crate) fn is_nuxt_route_file(filename: &str) -> bool {
    filename
        .replace('\\', "/")
        .split('/')
        .any(|segment| segment == "pages")
}

/// The component name a `.vue` filename defines: the basename without `.vue`
/// and without Nuxt's mode suffixes.
///
/// Nuxt reads `.client`, `.server`, `.global` and `.island` as rendering
/// modes, not as part of the name: `ClientBox.client.vue` registers
/// `ClientBox`. This strips them the way Nuxt's component scanner does (one
/// optional `.client`/`.server` followed by any `.global`/`.island`). Any other
/// dotted stem, such as `page.block`, is kept whole so it stays reported.
pub(crate) fn definition_name_stem(filename: &str) -> &str {
    let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let mut name = basename.trim_end_matches(".vue");
    while let Some(rest) = name
        .strip_suffix(".global")
        .or_else(|| name.strip_suffix(".island"))
    {
        name = rest;
    }
    name.strip_suffix(".client")
        .or_else(|| name.strip_suffix(".server"))
        .unwrap_or(name)
}

/// Common exception filenames that don't need PascalCase
pub(crate) const EXCEPTION_NAMES: &[&str] = &["App"];
