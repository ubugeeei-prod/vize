//! Project a checked module using the render operations and host regions
//! already supplied by lowering.

use super::super::{
    MODULE_SUFFIX,
    output::{LinkRole, Out, TsDocument},
};
use super::{Cx, ModuleProjectionError, ops};
use vize_l2::component::{Exec, RenderScope};
use vize_l2::module::{ModuleDef, ModulePart};

// Expand both entry points at the call site so the existing module-only build
// retains its writer and allocation code even across codegen units.
macro_rules! emit {
    ($module:ident, $components:expr) => {{
        for root in $module.components() {
            if root.exec != Exec::ReRender {
                return Err(ModuleProjectionError::RunOnce);
            }
            if root.scope != RenderScope::Lexical {
                return Err(ModuleProjectionError::InstanceScope);
            }
        }
        let source = $module.source();
        let mut cx = Cx {
            source,
            markup: $module.protected_starts(),
            #[cfg(feature = "component-projection")]
            component_checks: $components,
            out: Out::with_capacity(source.len().saturating_mul(2)),
        };
        $module.visit(|part| match part {
            ModulePart::Host { span, text } => {
                cx.out.verbatim(text, span, LinkRole::Script);
                Ok(())
            }
            ModulePart::Render(root) => ops::root(&mut cx, root),
        })?;
        cx.out.push(MODULE_SUFFIX);
        Ok(cx.out.finish())
    }};
}

pub fn project_module(module: &ModuleDef<'_>) -> Result<TsDocument, ModuleProjectionError> {
    emit!(module, None)
}

/// Project native checks and lexical component prop objects from the same L2 roots.
/// An unrepresented owner, binding or slot produces a refusal for the entire document.
#[cfg(feature = "component-projection")]
pub fn project_module_with_component_checks(
    module: &ModuleDef<'_>,
) -> Result<TsDocument, ModuleProjectionError> {
    let sites = module
        .component_sites()
        .ok_or(ModuleProjectionError::MissingComponentSites)?;
    emit!(module, Some(sites))
}
