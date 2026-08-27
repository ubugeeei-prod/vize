use vize_s0::{FxHashSet, String, ToCompactString};

use crate::codegen::helpers::default_helper_alias;
use crate::codegen::source_map::SourceMapBuilder;
use crate::options::CodegenOptions;
use crate::runtime_helpers::RuntimeHelpers;
use crate::{Namespace, RuntimeHelper};

use super::CodegenContext;

impl CodegenContext {
    /// Create a new codegen context.
    pub fn new(options: CodegenOptions) -> Self {
        Self::new_with_vnode_factory_and_merge_props(options, None, true)
    }

    pub(in crate::codegen) fn new_with_vnode_factory_and_merge_props(
        options: CodegenOptions,
        vnode_factory: Option<&str>,
        merge_props: bool,
    ) -> Self {
        let map_builder = options.source_map.then(SourceMapBuilder::new);
        Self {
            code: String::with_capacity(4096),
            indent_level: 0,
            ssr: options.ssr,
            helper_alias: default_helper_alias,
            vnode_factory: vnode_factory.map(normalize_vnode_factory),
            runtime_global_name: options.runtime_global_name.to_compact_string(),
            runtime_module_name: options.runtime_module_name.to_compact_string(),
            options,
            merge_props,
            pure: false,
            used_helpers: RuntimeHelpers::default(),
            cache_index: 0,
            slot_params: FxHashSet::default(),
            skip_is_prop: false,
            skip_scope_id: false,
            skip_normalize: false,
            in_v_for: false,
            skip_v_memo: false,
            props_is_plain_element: false,
            parent_ns: Namespace::Html,
            static_cache: false,
            in_cached_static: false,
            v_if_branch_counter: 0,
            map_builder,
            source: String::default(),
        }
    }

    /// Name used for vnode creation helpers under the optional JSX factory.
    pub(in crate::codegen) fn vnode_helper(&self, helper: RuntimeHelper) -> &str {
        debug_assert!(is_vnode_factory_helper(helper));
        match (&self.vnode_factory, helper) {
            // A custom factory has no block-tracking API. Keep the compiler's
            // comma-expression shape with an inert call and avoid importing
            // Vue's `openBlock` solely for this bookkeeping operation.
            (Some(_), RuntimeHelper::OpenBlock) => "_openBlock",
            (Some(factory), _) => factory.as_str(),
            (None, _) => self.helper(helper),
        }
    }

    /// Emit a vnode creation helper and track its default runtime import.
    pub(in crate::codegen) fn push_vnode_helper(&mut self, helper: RuntimeHelper) {
        debug_assert!(is_vnode_factory_helper(helper));
        if let Some(factory) = &self.vnode_factory {
            if helper == RuntimeHelper::OpenBlock {
                self.code.push_str("_openBlock");
            } else {
                self.code.push_str(factory);
            }
        } else {
            self.used_helpers.add(helper);
            self.code.push_str((self.helper_alias)(helper));
        }
    }

    /// Whether a helper still needs a Vue runtime import.
    pub(in crate::codegen) fn should_import_helper(&self, helper: RuntimeHelper) -> bool {
        self.vnode_factory.is_none() || !is_vnode_factory_helper(helper)
    }

    /// Whether custom-factory output needs the inert block-tracking shim.
    pub(in crate::codegen) fn needs_open_block_shim(&self) -> bool {
        self.vnode_factory.is_some() && self.used_helpers.contains(RuntimeHelper::OpenBlock)
    }
}

/// Store a JSX factory pragma so it can be emitted directly as a call callee.
///
/// The pragma may be any valid JavaScript expression, so anything that is not a
/// plain identifier or dotted member chain is parenthesized once here to keep
/// callee precedence: `a || b` must emit `(a || b)("div")`.
fn normalize_vnode_factory(factory: &str) -> String {
    if is_member_expression_chain(factory) {
        return String::from(factory);
    }
    let mut wrapped = String::with_capacity(factory.len() + 3);
    wrapped.push('(');
    wrapped.push_str(factory);
    // Keep the closing parenthesis outside a trailing line comment.
    wrapped.push('\n');
    wrapped.push(')');
    wrapped
}

fn is_member_expression_chain(factory: &str) -> bool {
    !factory.is_empty()
        && factory
            .split('.')
            .all(crate::codegen::helpers::is_valid_js_identifier)
}

fn is_vnode_factory_helper(helper: RuntimeHelper) -> bool {
    matches!(
        helper,
        RuntimeHelper::OpenBlock
            | RuntimeHelper::CreateBlock
            | RuntimeHelper::CreateElementBlock
            | RuntimeHelper::CreateVNode
            | RuntimeHelper::CreateElementVNode
    )
}
