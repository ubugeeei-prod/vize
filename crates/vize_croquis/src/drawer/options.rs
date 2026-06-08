/// Drawer options for controlling which facts are collected.
///
/// Use this to skip unnecessary passes for better performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct DrawerOptions {
    /// Draw script bindings (defineProps, defineEmits, etc.).
    pub analyze_script: bool,
    /// Draw template scopes (v-for, v-slot variables).
    pub analyze_template_scopes: bool,
    /// Track component and directive usage
    pub track_usage: bool,
    /// Detect undefined references (requires script + template).
    pub detect_undefined: bool,
    /// Collect hoisting opportunities.
    pub analyze_hoisting: bool,
    /// Collect template expressions for type checking
    pub collect_template_expressions: bool,
}

impl DrawerOptions {
    /// Full croquis (all features enabled).
    #[inline]
    pub const fn full() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: true,
            analyze_hoisting: true,
            collect_template_expressions: true,
        }
    }

    /// Minimal croquis for linting (fast).
    #[inline]
    pub const fn for_lint() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: true,
            analyze_hoisting: false,
            collect_template_expressions: false,
        }
    }

    /// Croquis for compilation (needs hoisting).
    #[inline]
    pub const fn for_compile() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: false,
            analyze_hoisting: true,
            collect_template_expressions: false,
        }
    }
}
