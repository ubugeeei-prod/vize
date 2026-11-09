//! Validated component scope metadata; source/target providers own its policy.

use super::AssemblyError;

/// A Vue `data-v-*` scope identity safe in both CSS and a JS string literal.
/// Its private construction prevents caller-supplied punctuation from reaching
/// prepared module/CSS structure. This does not itself admit a scoped product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeId<'a> {
    value: &'a str,
}
impl<'a> ScopeId<'a> {
    pub fn new(value: &'a str) -> Result<Self, AssemblyError> {
        if value.strip_prefix("data-v-").is_some_and(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        }) {
            Ok(Self { value })
        } else {
            Err(AssemblyError::InvalidScopeId)
        }
    }
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{AssemblyError, ScopeId};
    #[test]
    fn scope_identity_has_one_unescaped_css_and_js_spelling() {
        for source in ["data-v-1234abcd", "data-v-Style_1", "data-v-x-y"] {
            assert_eq!(ScopeId::new(source).unwrap().as_str(), source);
        }
        for source in [
            "",
            "data-v-",
            "abcd",
            "data-v-雪",
            r#"data-v-a";alert(1)//"#,
            "data-v-a]",
            "data-v-a b",
            "data-v-a\\62",
        ] {
            assert_eq!(ScopeId::new(source), Err(AssemblyError::InvalidScopeId));
        }
    }
}
