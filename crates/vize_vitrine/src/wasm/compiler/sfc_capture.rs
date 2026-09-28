//! Optional product capture beside the exact SFC adapter result.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
    SfcScriptOutputMode, compile_sfc_for_adapter_with_experimental_options as compile_ordinary,
    compile_sfc_for_adapter_with_stage_capture as compile_captured,
};
use vize_curator::inspector::{ProductCaptureSource, product_capture_value};
use vize_l0::Span;

pub(super) fn compile_sfc_product(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    custom_elements: CustomElementMatcher,
    codegen_options: CodegenOptions,
    script_output: SfcScriptOutputMode,
    experimental_options: SfcCompileExperimentalOptions,
    capture_stages: bool,
) -> Result<(SfcCompileResult, Option<serde_json::Value>), SfcError> {
    if !capture_stages {
        return compile_ordinary(
            descriptor,
            options,
            template_syntax,
            custom_elements,
            codegen_options,
            script_output,
            experimental_options,
        )
        .map(|result| (result, None));
    }

    let (result, capture) = compile_captured(
        descriptor,
        options,
        template_syntax,
        custom_elements,
        codegen_options,
        script_output,
        experimental_options,
    )?;
    let template = descriptor.template.as_ref();
    let authored_syntax = template
        .and_then(|block| block.lang.as_deref())
        .map(str::to_ascii_lowercase)
        .filter(|lang| lang != "html")
        .unwrap_or_else(|| "vue-template".to_string());
    let template_span = template.and_then(|block| {
        let start = u32::try_from(block.loc.start).ok()?;
        let end = u32::try_from(block.loc.end).ok()?;
        Some(Span::new(start, end))
    });
    let feed = product_capture_value(
        "compile-sfc",
        ProductCaptureSource {
            path: Some(descriptor.filename.as_ref()),
            container: "vue-sfc",
            authored_syntax: &authored_syntax,
            compiled_syntax: "vue-template",
            template_span,
        },
        &capture,
    );
    Ok((result, Some(feed)))
}
