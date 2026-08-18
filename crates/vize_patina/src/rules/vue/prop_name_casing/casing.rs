//! The casing predicates `eslint-plugin-vue` applies to a prop name.
//!
//! Ported from `eslint-plugin-vue@10.9.2`'s `lib/utils/casing.ts`, which decides
//! by *rejection*: a name is in a casing unless it holds one of the shapes that
//! casing forbids. That is why a single lowercase word such as `count` is both
//! camelCase and snake_case, and why `$attrs` stays camelCase — `$` is not in
//! the symbol set upstream rejects.

/// The symbol set upstream's `hasSymbols` rejects, verbatim.
const SYMBOLS: &str = "!\"#%&'()*+,./:;<=>?@[\\]^`{|}";

fn has_symbols(name: &str) -> bool {
    name.chars().any(|character| SYMBOLS.contains(character))
}

fn has_upper(name: &str) -> bool {
    name.chars().any(|character| character.is_ascii_uppercase())
}

/// `!hasSymbols(str) && !/^[A-Z]/.test(str) && !/-|_|\s/.test(str)`
pub(super) fn is_camel_case(name: &str) -> bool {
    !has_symbols(name)
        && !name.starts_with(|character: char| character.is_ascii_uppercase())
        && !name.contains(['-', '_'])
        && !name.chars().any(char::is_whitespace)
}

/// `!hasUpper(str) && !hasSymbols(str) && !/-|__|\s/.test(str)`
pub(super) fn is_snake_case(name: &str) -> bool {
    !has_upper(name)
        && !has_symbols(name)
        && !name.contains('-')
        && !name.contains("__")
        && !name.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::{is_camel_case, is_snake_case};

    #[test]
    fn camel_case_accepts_lowercase_starts_without_separators() {
        assert!(is_camel_case("count"));
        assert!(is_camel_case("myProp"));
        assert!(is_camel_case("prop2"));
        assert!(is_camel_case("$attrs"));
    }

    #[test]
    fn camel_case_rejects_separators_and_leading_capitals() {
        assert!(!is_camel_case("my-prop"));
        assert!(!is_camel_case("my_prop"));
        assert!(!is_camel_case("MyProp"));
        assert!(!is_camel_case("my prop"));
        assert!(!is_camel_case("my.prop"));
    }

    #[test]
    fn snake_case_accepts_single_underscores_and_lowercase() {
        assert!(is_snake_case("count"));
        assert!(is_snake_case("my_prop"));
        assert!(is_snake_case("_leading"));
    }

    #[test]
    fn snake_case_rejects_uppercase_hyphens_and_doubled_underscores() {
        assert!(!is_snake_case("myProp"));
        assert!(!is_snake_case("my-prop"));
        assert!(!is_snake_case("my__prop"));
        assert!(!is_snake_case("my prop"));
    }
}
