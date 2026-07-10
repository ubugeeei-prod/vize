use super::*;
use vize_atelier_core::{
    atelier_output::{AtelierOutputChunk, AtelierTarget},
    codegen::{CodegenResult, CodegenSections},
};

#[test]
fn dom_codegen_sections_are_ranges_into_flattened_output() {
    let imports = String::from("import { openBlock as _openBlock } from \"vue\"\n");
    let hoists = String::from("const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"div\")\n");
    let code = String::from(
        "export function render(_ctx, _cache) {\n  const _component_Foo = _resolveComponent(\"Foo\")\n\n  return _openBlock()\n}",
    );
    let asset = "const _component_Foo = _resolveComponent(\"Foo\")";
    let return_expr = "_openBlock()";
    let assets_start = code.find(asset).expect("asset statement should exist");
    let return_expr_start = code
        .find(return_expr)
        .expect("return expression should exist");
    let result = CodegenResultWithSections {
        result: CodegenResult {
            preamble: {
                let mut preamble = imports.clone();
                preamble.push('\n');
                preamble.push_str(&hoists);
                preamble
            },
            code,
            map: Some(String::from("{\"version\":3}")),
        },
        sections: Some(CodegenSections {
            imports_len: imports.len(),
            assets_start,
            assets_end: assets_start + asset.len(),
            return_expr_start,
            return_expr_end: return_expr_start + return_expr.len(),
        }),
    };

    let output = OutputModule::from_dom_codegen(result);
    let sections = output.sections.expect("DOM sections should be retained");
    let (code, maps) = output.into_code_and_maps();

    assert_eq!(maps.source_map(), Some("{\"version\":3}"));
    assert_eq!(&code[sections.imports.start..sections.imports.end], imports);
    assert_eq!(&code[sections.hoisted.start..sections.hoisted.end], hoists);
    assert_eq!(
        &code[sections.assets.start..sections.assets.end],
        "const _component_Foo = _resolveComponent(\"Foo\")"
    );
    assert_eq!(
        &code[sections.return_expr.start..sections.return_expr.end],
        "_openBlock()"
    );
}

#[test]
fn ssr_codegen_uses_the_same_flattening_boundary() {
    let output = OutputModule::from_ssr_codegen(SsrCodegenResult {
        preamble: String::from(
            "import { ssrInterpolate as _ssrInterpolate } from \"vue/server-renderer\"\n",
        ),
        code: String::from(
            "export function ssrRender(_ctx, _push) {\n  _push(_ssrInterpolate(_ctx.msg))\n}",
        ),
    });
    let sections = output.module_sections();
    let imports = output.imports.clone();
    let functions = output.functions.clone();

    let code = output.into_code();

    assert_eq!(
        code,
        "import { ssrInterpolate as _ssrInterpolate } from \"vue/server-renderer\"\n\nexport function ssrRender(_ctx, _push) {\n  _push(_ssrInterpolate(_ctx.msg))\n}\n"
    );
    assert_eq!(&code[sections.imports.start..sections.imports.end], imports);
    assert_eq!(
        &code[sections.functions.start..sections.functions.end],
        functions
    );
}

#[test]
fn output_module_exposes_borrowed_output_view() {
    let output = OutputModule {
        imports: String::from("import { h } from \"vue\"\n"),
        hoists: String::from("const _hoisted_1 = null\n"),
        functions: String::from("function render() {\n  return null\n}"),
        exports: String::from("export default _sfc_main\n"),
        sections: None,
        maps: AtelierOutputMaps::from_source_map(Some(String::from("{\"version\":3}"))),
    };

    let view = output.as_output_view(AtelierTarget::Dom);

    assert_eq!(view.target, AtelierTarget::Dom);
    assert_eq!(
        view.chunk(AtelierOutputChunk::Imports),
        output.imports.as_str()
    );
    assert_eq!(
        view.chunk(AtelierOutputChunk::Functions),
        output.functions.as_str()
    );
    assert_eq!(view.source_map, Some("{\"version\":3}"));
    assert_eq!(view.module_sections, output.module_sections());
}
