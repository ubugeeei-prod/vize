use vize_atelier_ssr::{Allocator, compile_ssr};

#[test]
fn nested_elements_omit_static_and_bound_vnode_metadata() {
    let allocator = Allocator::new();
    for source in [
        r#"<main><p key="row" ref="node" ref_for="" ref_key="local" id="kept">text</p></main>"#,
        r#"<main><p :key="key" :ref="reference" :ref_for="true" :ref_key="refKey" id="kept">text</p></main>"#,
    ] {
        let (_, errors, result) = compile_ssr(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(
            result.code.as_str(),
            "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<main${_ssrRenderAttrs(_attrs)}><p id=\"kept\">text</p></main>`)\n}\n"
        );
        assert_eq!(
            result.preamble.as_str(),
            "import { ssrRenderAttrs as _ssrRenderAttrs } from \"@vue/server-renderer\"\n"
        );
    }
}
