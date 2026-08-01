use crate::RuntimeHelper;

use super::CodegenContext;

impl CodegenContext {
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
