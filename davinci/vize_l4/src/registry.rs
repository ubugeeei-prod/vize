//! L4 output-policy registry.
//!
//! A resolved file dialect becomes target options once, before an emitter is
//! selected. Emitters consume these booleans and never rematch input variants.

use vize_l0::{
    FileDialect, ProfileKind, TemplateSyntaxMode, config::VueVersion, dialect::VueDialect,
};

/// Target capabilities selected for one resolved input descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetOptions {
    vue_projection: bool,
    vue2_runtime: bool,
}

impl TargetOptions {
    /// Resolve L4 output policy once for a file.
    #[must_use]
    pub const fn for_file(dialect: FileDialect) -> Self {
        let standard_component = matches!(dialect.framework(), VueDialect::Vue)
            && matches!(dialect.syntax(), TemplateSyntaxMode::Standard)
            && matches!(dialect.profile(), ProfileKind::Component);
        Self {
            vue_projection: standard_component && matches!(dialect.version(), VueVersion::V3),
            vue2_runtime: standard_component
                && matches!(dialect.version(), VueVersion::V2 | VueVersion::V2_7),
        }
    }

    #[must_use]
    pub const fn vue_projection(self) -> bool {
        self.vue_projection
    }

    #[must_use]
    pub const fn vue2_runtime(self) -> bool {
        self.vue2_runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::Lang;

    #[test]
    fn output_policy_keeps_vue3_projection_and_vue2_runtime_distinct() {
        let file = |version| {
            FileDialect::component(
                version,
                VueDialect::Vue,
                TemplateSyntaxMode::Standard,
                Lang::Ts,
            )
        };
        assert_eq!(
            TargetOptions::for_file(file(VueVersion::V3)),
            TargetOptions {
                vue_projection: true,
                vue2_runtime: false,
            }
        );
        assert_eq!(
            TargetOptions::for_file(file(VueVersion::V2_7)),
            TargetOptions {
                vue_projection: false,
                vue2_runtime: true,
            }
        );
    }
}
