//! L2 residual-dialect registry.
//!
//! L2 remains neutral by default. The registry admits only residual meanings
//! that cannot be legalized away at the L1→L2 boundary.

use vize_l0::FileDialect;

/// Whether the existing native scoped-template semantic view is registered.
#[must_use]
pub fn native_scoped_template(dialect: FileDialect) -> bool {
    vize_l1::registry::is_standard_vue3_component(dialect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::{Lang, TemplateSyntaxMode, config::VueVersion, dialect::VueDialect};

    #[test]
    fn scoped_template_meaning_keeps_its_existing_profile() {
        let vue3 = FileDialect::component(
            VueVersion::V3,
            VueDialect::Vue,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        let quirks = FileDialect::component(
            VueVersion::V3,
            VueDialect::Vue,
            TemplateSyntaxMode::Quirks,
            Lang::Js,
        );
        assert!(native_scoped_template(vue3));
        assert!(!native_scoped_template(quirks));
    }
}
