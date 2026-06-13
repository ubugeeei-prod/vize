use super::{extract_template_parts_full, slice_template_parts_full};
use crate::compile::output_module::OutputModule;
use vize_atelier_ssr::SsrCodegenResult;

#[test]
fn slice_template_parts_full_matches_ssr_line_scanner() {
    let output_module = OutputModule::from_ssr_codegen(SsrCodegenResult {
        preamble: String::from(
            "import { ssrRenderComponent as _ssrRenderComponent } from \"vue/server-renderer\"\n",
        )
        .into(),
        code: String::from(
            "export function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(_ssrRenderComponent(_ctx.Foo, null, null, _parent))\n}",
        )
        .into(),
    });
    let sections = output_module.module_sections();
    let code = output_module.into_code();

    let sliced = slice_template_parts_full(&code, &sections, "ssrRender");
    let scanned = extract_template_parts_full(&code);

    assert_eq!(sliced, scanned);
}
