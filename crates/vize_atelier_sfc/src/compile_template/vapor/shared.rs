//! Vapor compilation from a shared parse-only Relief product.

use vize_atelier_vapor::{
    VaporCompilerOptions, compile_vapor_root_with_template_syntax_and_diagnostics,
};
use vize_carton::{Bump, String, ToCompactString};
use vize_relief::ReliefArtifact;

use super::super::{TemplateBlockCompileResult, recoverable_template_warnings};
use super::{
    VaporTemplateCompileContext, record_unsupported_vapor_shape, transform_vapor_template_module,
};
use crate::{
    compile::output_module::AtelierOutputMaps,
    types::{BindingMetadata, SfcError, SfcTemplateBlock},
};

pub(crate) fn compile_template_block_vapor_from_syntax(
    template: &SfcTemplateBlock,
    syntax: &ReliefArtifact,
    context: VaporTemplateCompileContext<'_>,
    bindings: Option<&BindingMetadata>,
) -> Result<TemplateBlockCompileResult, SfcError> {
    let VaporTemplateCompileContext {
        scope_id,
        has_scoped,
        options,
        template_syntax,
        codegen_options,
    } = context;
    let allocator = Bump::new();
    let compiler_options = options.compiler_options.as_ref();
    let vapor_options = VaporCompilerOptions {
        prefix_identifiers: false,
        ssr: false,
        binding_metadata: bindings.cloned(),
        custom_renderer: options.custom_renderer,
        experimental_in_tag_comments: compiler_options
            .is_some_and(|options| options.experimental_in_tag_comments),
        experimental_patterned_template: compiler_options
            .is_some_and(|options| options.experimental_patterned_template),
        ..Default::default()
    };
    let (result, diagnostics) = compile_vapor_root_with_template_syntax_and_diagnostics(
        &allocator,
        syntax.snapshot().materialize(&allocator),
        syntax.parse_diagnostics().to_vec(),
        vapor_options,
        template_syntax,
    );
    if !result.error_messages.is_empty() {
        record_unsupported_vapor_shape();
        let mut message = String::from("Vapor template compilation errors: ");
        use std::fmt::Write as _;
        let _ = write!(&mut message, "{:?}", result.error_messages);
        return Err(SfcError {
            message,
            code: Some("VAPOR_TEMPLATE_ERROR".to_compact_string()),
            loc: Some(template.loc.clone()),
        });
    }
    let scope_attr = has_scoped.then(|| {
        let mut attribute = String::with_capacity(scope_id.len() + 7);
        attribute.push_str("data-v-");
        attribute.push_str(scope_id);
        attribute
    });
    let output = transform_vapor_template_module(
        &result.code,
        scope_attr.as_deref(),
        template,
        bindings,
        codegen_options.runtime_module_name.as_str(),
    )?;
    Ok(TemplateBlockCompileResult {
        code: output.code,
        warnings: recoverable_template_warnings(&diagnostics),
        sections: None,
        module_sections: Some(output.module_sections),
        maps: AtelierOutputMaps::default(),
    })
}
