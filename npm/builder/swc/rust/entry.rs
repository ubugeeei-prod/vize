use swc_core::{
    common::{Spanned, errors::HANDLER},
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

/// Compile the current SWC AST exactly once, then let SWC lower the target and modules.
#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    match super::transform(&mut program, &metadata) {
        Ok(Some(compiled)) => compiled,
        Ok(None) => program,
        Err(message) => {
            HANDLER.with(|handler| handler.struct_span_err(program.span(), &message).emit());
            program
        }
    }
}
