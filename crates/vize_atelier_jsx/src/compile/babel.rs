use oxc_allocator::Allocator;
use vize_carton::{Bump, String};

use super::{
    BabelJsxOptions, JsxCompileConfig, JsxCompileOutput,
    compile_jsx_with_babel_customizations_inner,
};
use crate::{JsxDiagnostic, JsxLang};

/// Thread-safe predicate matching Babel's `isCustomElement` option.
pub type BabelIsCustomElement = dyn Fn(&str) -> bool + Send + Sync;

/// Additive Babel options that carry borrowed values or must compose together.
#[derive(Clone, Copy)]
pub struct BabelJsxCustomizations<'a> {
    /// Optional vnode factory supplied by Babel's `pragma` option.
    pub pragma: Option<&'a str>,
    /// Whether prop segments are combined through Vue's `mergeProps` helper.
    pub merge_props: bool,
    /// Project-specific custom-element classifier.
    pub is_custom_element: Option<&'a BabelIsCustomElement>,
}

impl Default for BabelJsxCustomizations<'_> {
    fn default() -> Self {
        Self {
            pragma: None,
            merge_props: true,
            is_custom_element: None,
        }
    }
}

/// Compile JSX/TSX with every additive Babel customization combined.
pub fn compile_jsx_with_babel_customizations(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    customizations: BabelJsxCustomizations<'_>,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations_inner(
        bump,
        source,
        lang,
        config,
        babel_options,
        customizations,
    )
}

/// Compile with the additive Babel `pragma` and `mergeProps` options combined.
#[doc(hidden)]
pub fn compile_jsx_with_babel_pragma_and_merge_props(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    pragma: Option<&str>,
    merge_props: bool,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        bump,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations {
            pragma,
            merge_props,
            is_custom_element: None,
        },
    )
}

/// Compile JSX/TSX with explicit `@vue/babel-plugin-jsx` option compatibility.
pub fn compile_jsx_with_babel_options(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        bump,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations::default(),
    )
}

/// Compile JSX/TSX with an optional Babel-compatible vnode factory pragma.
pub fn compile_jsx_with_babel_pragma(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    pragma: Option<&str>,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        bump,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations {
            pragma,
            ..Default::default()
        },
    )
}

/// Compile JSX/TSX with an explicit Babel-compatible `mergeProps` value.
pub fn compile_jsx_with_babel_merge_props(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    merge_props: bool,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        bump,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations {
            merge_props,
            ..Default::default()
        },
    )
}

pub(super) fn resolve_vnode_factory<'a>(
    pragma: Option<&'a str>,
    active: bool,
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> Option<&'a str> {
    pragma
        .map(str::trim)
        .filter(|pragma| !pragma.is_empty())
        .filter(|_| active)
        .and_then(|pragma| {
            if valid_pragma_expression(pragma) {
                Some(pragma)
            } else {
                diagnostics.push(JsxDiagnostic::error(
                    "Babel JSX pragma must be a valid JavaScript expression",
                    0,
                    0,
                ));
                None
            }
        })
}

fn valid_pragma_expression(pragma: &str) -> bool {
    let mut probe = String::from("const __vize_pragma = (");
    probe.push_str(pragma);
    probe.push_str("\n);");
    let allocator = Allocator::default();
    !crate::parse_module(&allocator, probe.as_str(), JsxLang::Jsx).has_errors()
}

/// Babel allocates a fresh helper binding when the source already declares
/// `_transformOn`. A conservative source-text check gives the generated module
/// the same collision safety without coupling this option to a second semantic
/// analysis pass.
pub(super) fn collision_free_transform_on_helper(source: &str) -> String {
    let mut helper = String::from("_transformOn");
    while source.contains(helper.as_str()) {
        helper.push('_');
    }
    helper
}
