//! Project a checked module using the render operations and host regions
//! already supplied by lowering.

use super::super::{
    MODULE_SUFFIX,
    output::{LinkRole, Out, TsDocument},
};
use super::{Cx, ModuleProjectionError, ops};
use vize_l2::component::{Exec, RenderScope};
use vize_l2::module::{ModuleDef, ModulePart};

pub fn project_module(module: &ModuleDef<'_>) -> Result<TsDocument, ModuleProjectionError> {
    for root in module.components() {
        if root.exec != Exec::ReRender {
            return Err(ModuleProjectionError::RunOnce);
        }
        if root.scope != RenderScope::Lexical {
            return Err(ModuleProjectionError::InstanceScope);
        }
    }
    let source = module.source();
    let mut cx = Cx {
        source,
        markup: module.protected_starts(),
        out: Out::with_capacity(source.len().saturating_mul(2)),
    };
    module.visit(|part| match part {
        ModulePart::Host { span, text } => {
            cx.out.verbatim(text, span, LinkRole::Script);
            Ok(())
        }
        ModulePart::Render(root) => ops::root(&mut cx, root),
    })?;
    cx.out.push(MODULE_SUFFIX);
    Ok(cx.out.finish())
}
