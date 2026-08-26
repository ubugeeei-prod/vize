//! Script-setup binding metadata shared by transform and codegen.

use vize_s0::{FxHashMap, String};

/// Binding metadata from script setup
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingMetadata {
    /// Setup bindings with their types
    pub bindings: FxHashMap<String, BindingType>,

    /// Props aliases (local name -> prop key)
    /// For destructured props with aliases like: const { foo: bar } = defineProps()
    /// This maps "bar" -> "foo"
    pub props_aliases: FxHashMap<String, String>,

    /// Whether these bindings are from script setup
    /// If false, components/directives won't be resolved from these bindings
    pub is_script_setup: bool,
}

/// Binding type from script setup.
///
/// Optimized with `#[repr(u8)]` for minimal memory footprint.
/// Each variant fits in a single byte, reducing cache pressure
/// when stored in large collections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum BindingType {
    /// Variable declared with let in setup
    SetupLet = 0,
    /// Const binding that may be a ref
    SetupMaybeRef = 1,
    /// Const binding that is definitely a ref
    SetupRef = 2,
    /// Reactive const binding (reactive(), shallowReactive())
    SetupReactiveConst = 3,
    /// Const binding (functions, classes, non-reactive values)
    SetupConst = 4,
    /// Binding from props
    Props = 5,
    /// Binding from props with alias
    PropsAliased = 6,
    /// Data binding from data()
    Data = 7,
    /// Options API binding (computed, methods, inject)
    Options = 8,
    /// Literal constant (string, number, boolean literals)
    LiteralConst = 9,
    /// Universal JavaScript global (works in all runtimes: console, Math, Object, Array, JSON, etc.)
    JsGlobalUniversal = 10,
    /// Browser-only JavaScript global (window, document, navigator, localStorage, etc.)
    /// WARNING: Not available in SSR server context
    JsGlobalBrowser = 11,
    /// Node.js-only JavaScript global (process, Buffer, __dirname, __filename, require, etc.)
    /// WARNING: Not available in browser context
    JsGlobalNode = 12,
    /// Deno-only JavaScript global (Deno namespace)
    JsGlobalDeno = 13,
    /// Bun-only JavaScript global (Bun namespace)
    JsGlobalBun = 14,
    /// Vue global ($refs, $emit, $slots, $attrs, $el, etc.)
    VueGlobal = 15,
    /// Imported from external module
    ExternalModule = 16,
}

impl BindingType {
    /// Short display code for VIR output (zero allocation)
    /// - st = state (ref, needs .value)
    /// - ist = implicit state (reactive, props - no .value needed)
    /// - drv = derived (computed)
    #[inline]
    pub const fn to_vir(self) -> &'static str {
        match self {
            Self::SetupLet => "let",
            Self::SetupMaybeRef => "st?",
            Self::SetupRef => "st",
            Self::SetupReactiveConst => "ist",
            Self::SetupConst => "c",
            Self::Props => "ist",        // props are implicit state (no .value)
            Self::PropsAliased => "ist", // aliased props too
            Self::Data => "data",
            Self::Options => "opt",
            Self::LiteralConst => "lit",
            Self::JsGlobalUniversal => "~js",
            Self::JsGlobalBrowser => "!js",
            Self::JsGlobalNode => "#js",
            Self::JsGlobalDeno => "#deno",
            Self::JsGlobalBun => "#bun",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "ext",
        }
    }

    /// Render-function prefix for this binding in non-inline (function) mode,
    /// matching `@vue/compiler-core`'s `rewriteIdentifier`:
    ///
    /// - setup-* bindings and `literal-const` → `$setup.`
    /// - `props` → `$props.`
    /// - `data` → `$data.`
    /// - `options` (computed / methods / inject) → `$options.`
    /// - `vue-global` (`$slots`, `$emit`, `$attrs`, ...) → `_ctx.`
    ///
    /// `props-aliased` also returns `$props.` here; the caller must still resolve
    /// the original key through the props-alias map, which rewrites the prefixed
    /// access to `$props['<original-key>']`. JavaScript globals are skipped
    /// before this call site, so their `$setup.` arm is unused in practice, and
    /// `external-module` bindings behave like setup bindings in function mode.
    #[inline]
    pub const fn non_inline_template_prefix(self) -> &'static str {
        match self {
            Self::SetupLet
            | Self::SetupMaybeRef
            | Self::SetupRef
            | Self::SetupReactiveConst
            | Self::SetupConst
            | Self::LiteralConst => "$setup.",
            Self::Props | Self::PropsAliased => "$props.",
            Self::Data => "$data.",
            Self::Options => "$options.",
            // Vue globals (`$slots`, `$emit`, `$attrs`, ...) live on the render
            // context, never on the setup object.
            Self::VueGlobal => "_ctx.",
            // JS globals are skipped before this call site (they need no prefix);
            // external-module bindings behave like setup bindings in function mode.
            Self::JsGlobalUniversal
            | Self::JsGlobalBrowser
            | Self::JsGlobalNode
            | Self::JsGlobalDeno
            | Self::JsGlobalBun
            | Self::ExternalModule => "$setup.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BindingMetadata, BindingType};

    #[test]
    fn binding_type_discriminants() {
        assert_eq!(BindingType::SetupLet as u8, 0);
        assert_eq!(BindingType::SetupMaybeRef as u8, 1);
        assert_eq!(BindingType::SetupRef as u8, 2);
        assert_eq!(BindingType::SetupReactiveConst as u8, 3);
        assert_eq!(BindingType::SetupConst as u8, 4);
        assert_eq!(BindingType::Props as u8, 5);
        assert_eq!(BindingType::PropsAliased as u8, 6);
        assert_eq!(BindingType::Data as u8, 7);
        assert_eq!(BindingType::Options as u8, 8);
        assert_eq!(BindingType::LiteralConst as u8, 9);
        assert_eq!(BindingType::JsGlobalUniversal as u8, 10);
        assert_eq!(BindingType::JsGlobalBrowser as u8, 11);
        assert_eq!(BindingType::JsGlobalNode as u8, 12);
        assert_eq!(BindingType::JsGlobalDeno as u8, 13);
        assert_eq!(BindingType::JsGlobalBun as u8, 14);
        assert_eq!(BindingType::VueGlobal as u8, 15);
        assert_eq!(BindingType::ExternalModule as u8, 16);
    }

    #[test]
    fn binding_type_to_vir() {
        assert_eq!(BindingType::SetupLet.to_vir(), "let");
        assert_eq!(BindingType::SetupMaybeRef.to_vir(), "st?");
        assert_eq!(BindingType::SetupRef.to_vir(), "st");
        assert_eq!(BindingType::SetupReactiveConst.to_vir(), "ist");
        assert_eq!(BindingType::SetupConst.to_vir(), "c");
        assert_eq!(BindingType::Props.to_vir(), "ist");
        assert_eq!(BindingType::PropsAliased.to_vir(), "ist");
        assert_eq!(BindingType::Data.to_vir(), "data");
        assert_eq!(BindingType::Options.to_vir(), "opt");
        assert_eq!(BindingType::LiteralConst.to_vir(), "lit");
        assert_eq!(BindingType::JsGlobalUniversal.to_vir(), "~js");
        assert_eq!(BindingType::JsGlobalBrowser.to_vir(), "!js");
        assert_eq!(BindingType::JsGlobalNode.to_vir(), "#js");
        assert_eq!(BindingType::JsGlobalDeno.to_vir(), "#deno");
        assert_eq!(BindingType::JsGlobalBun.to_vir(), "#bun");
        assert_eq!(BindingType::VueGlobal.to_vir(), "vue");
        assert_eq!(BindingType::ExternalModule.to_vir(), "ext");
    }

    #[test]
    fn non_inline_template_prefix_matches_vue_compiler_core() {
        // Mirrors `@vue/compiler-core`'s `rewriteIdentifier` else-branch:
        // setup-* / literal-const -> $setup., props -> $props.,
        // data -> $data., options -> $options.
        assert_eq!(
            BindingType::SetupLet.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(
            BindingType::SetupMaybeRef.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(
            BindingType::SetupRef.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(
            BindingType::SetupReactiveConst.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(
            BindingType::SetupConst.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(
            BindingType::LiteralConst.non_inline_template_prefix(),
            "$setup."
        );
        assert_eq!(BindingType::Props.non_inline_template_prefix(), "$props.");
        assert_eq!(
            BindingType::PropsAliased.non_inline_template_prefix(),
            "$props."
        );
        assert_eq!(BindingType::Data.non_inline_template_prefix(), "$data.");
        assert_eq!(
            BindingType::Options.non_inline_template_prefix(),
            "$options."
        );
    }

    #[test]
    fn non_inline_template_prefix_keeps_vue_globals_on_render_context() {
        // `$slots` / `$emit` and friends live on the render context, so function
        // mode must emit `_ctx.$slots`, never `$setup.$slots`.
        assert_eq!(BindingType::VueGlobal.non_inline_template_prefix(), "_ctx.");
    }

    #[test]
    fn binding_metadata_default() {
        let meta = BindingMetadata::default();
        assert!(meta.bindings.is_empty());
        assert!(meta.props_aliases.is_empty());
        assert!(!meta.is_script_setup);
    }

    #[test]
    fn binding_type_serde_roundtrip() {
        let all_types = [
            BindingType::SetupLet,
            BindingType::SetupMaybeRef,
            BindingType::SetupRef,
            BindingType::SetupReactiveConst,
            BindingType::SetupConst,
            BindingType::Props,
            BindingType::PropsAliased,
            BindingType::Data,
            BindingType::Options,
            BindingType::LiteralConst,
            BindingType::JsGlobalUniversal,
            BindingType::JsGlobalBrowser,
            BindingType::JsGlobalNode,
            BindingType::JsGlobalDeno,
            BindingType::JsGlobalBun,
            BindingType::VueGlobal,
            BindingType::ExternalModule,
        ];
        for bt in &all_types {
            let json = serde_json::to_string(bt).unwrap();
            let deserialized: BindingType = serde_json::from_str(&json).unwrap();
            assert_eq!(*bt, deserialized, "Roundtrip failed for {:?}", bt);
        }
    }
}
