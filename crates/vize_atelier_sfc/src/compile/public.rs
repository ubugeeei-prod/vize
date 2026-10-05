//! Public SFC compile entry points.

use super::entry;

pub use crate::compile_script::ScriptCompileResult;
#[expect(deprecated, reason = "kept exported until removal")]
pub use entry::compile_sfc_with_vue_parser_quirks;
pub use entry::{
    SfcScriptOutputMode, compile_sfc, compile_sfc_for_adapter,
    compile_sfc_for_adapter_with_experimental_options, compile_sfc_for_adapter_with_nuxt_page_meta,
    compile_sfc_for_adapter_with_stage_capture,
    compile_sfc_with_custom_elements_template_syntax_and_codegen_options,
    compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options,
    compile_sfc_with_template_syntax, compile_sfc_with_template_syntax_and_codegen_options,
    prepare_root_patterned_template,
};
