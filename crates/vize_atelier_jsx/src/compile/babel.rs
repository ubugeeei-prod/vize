use vize_s0::{Allocator, String};

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
    /// Whether a lone identifier or call child of a component is checked at
    /// runtime for already being a slots object, matching Babel's
    /// `enableObjectSlots` (default `true`).
    pub enable_object_slots: bool,
}

impl Default for BabelJsxCustomizations<'_> {
    fn default() -> Self {
        Self {
            pragma: None,
            merge_props: true,
            is_custom_element: None,
            enable_object_slots: true,
        }
    }
}

/// Compile JSX/TSX with every additive Babel customization combined.
pub fn compile_jsx_with_babel_customizations(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    customizations: BabelJsxCustomizations<'_>,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations_inner(
        allocator,
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
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    pragma: Option<&str>,
    merge_props: bool,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        allocator,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations {
            pragma,
            merge_props,
            ..Default::default()
        },
    )
}

/// Compile JSX/TSX with explicit `@vue/babel-plugin-jsx` option compatibility.
pub fn compile_jsx_with_babel_options(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        allocator,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations::default(),
    )
}

/// Compile JSX/TSX with an optional Babel-compatible vnode factory pragma.
pub fn compile_jsx_with_babel_pragma(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    pragma: Option<&str>,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        allocator,
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
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    merge_props: bool,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        allocator,
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

/// Compile JSX/TSX with an explicit Babel-compatible `enableObjectSlots` value.
///
/// The plugin defaults this option to `true`: a lone identifier or call child
/// of a component may already be a slots object and is checked at runtime.
/// Passing `false` always wraps that value as the raw default-slot child.
pub fn compile_jsx_with_babel_object_slots(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    enable_object_slots: bool,
) -> JsxCompileOutput {
    compile_jsx_with_babel_customizations(
        allocator,
        source,
        lang,
        config,
        babel_options,
        BabelJsxCustomizations {
            enable_object_slots,
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
    let allocator = oxc_allocator::Allocator::default();
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

/// Collision-free local bindings used by Babel's object-slot discriminator.
pub(super) struct ObjectSlotHelpers {
    pub is_slot: String,
    pub is_vnode: String,
}

pub(super) fn collision_free_object_slot_helpers(source: &str) -> ObjectSlotHelpers {
    ObjectSlotHelpers {
        is_slot: collision_free_helper(source, "_isSlot"),
        is_vnode: collision_free_helper(source, "_isVNode"),
    }
}

fn collision_free_helper(source: &str, base: &str) -> String {
    let mut helper = String::from(base);
    while source.contains(helper.as_str()) {
        helper.push('_');
    }
    helper
}
