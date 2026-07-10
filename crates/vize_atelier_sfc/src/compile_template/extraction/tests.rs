use super::slice_template_parts_full;
use crate::compile::output_module::{AtelierOutputSections, OutputModule};
use vize_atelier_core::atelier_output::AtelierRange;
use vize_atelier_ssr::SsrCodegenResult;

#[test]
fn slice_template_parts_full_uses_module_sections() {
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

    let (imports, hoisted, render_fn, render_fn_name) =
        slice_template_parts_full(&code, &sections, "ssrRender");

    assert_eq!(
        imports,
        "import { ssrRenderComponent as _ssrRenderComponent } from \"vue/server-renderer\"\n"
    );
    assert!(hoisted.is_empty());
    assert_eq!(render_fn_name, "ssrRender");
    assert_eq!(
        render_fn,
        "export function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(_ssrRenderComponent(_ctx.Foo, null, null, _parent))\n}\n"
    );
}

#[test]
fn slice_template_parts_uses_render_sections_without_scanning() {
    let code = r#"import { createVNode as _createVNode } from 'vue'

const _hoisted_1 = { class: "test" }

export function render(_ctx, _cache) {
  const _component_Foo = _resolveComponent("Foo")

  return _createVNode(_component_Foo, _hoisted_1, "Hello");
}
"#;
    let imports_end = "import { createVNode as _createVNode } from 'vue'\n".len();
    let hoisted_start = imports_end + 1;
    let hoisted_end = hoisted_start + "const _hoisted_1 = { class: \"test\" }\n".len();
    let assets_start = code.find("  const _component_Foo").unwrap();
    let assets_end = code.find("  return ").unwrap();
    let return_start = assets_end + "  return ".len();
    let return_expr = "_createVNode(_component_Foo, _hoisted_1, \"Hello\")";
    let return_end = return_start + return_expr.len();
    let sections = AtelierOutputSections {
        imports: AtelierRange::new(0, imports_end),
        hoisted: AtelierRange::new(hoisted_start, hoisted_end),
        assets: AtelierRange::new(assets_start, assets_end),
        return_expr: AtelierRange::new(return_start, return_end),
    };

    let (imports, hoisted, preamble, render_body, render_fn_name) =
        super::slice_template_parts(code, &sections);

    assert_eq!(
        imports,
        "import { createVNode as _createVNode } from 'vue'\n"
    );
    assert_eq!(hoisted, "const _hoisted_1 = { class: \"test\" }\n");
    assert_eq!(
        preamble,
        "const _component_Foo = _resolveComponent(\"Foo\")\n"
    );
    assert_eq!(render_body, return_expr);
    assert_eq!(render_fn_name, "render");
}
