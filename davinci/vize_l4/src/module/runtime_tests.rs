use vize_l0::Span;

use super::{AssemblyError, ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble};
use crate::runtime::Runtime;
use crate::write::{LinkSink, NoLinks, Recorded, Writer};

fn ssr<L: LinkSink>() -> ModuleParts<'static, L> {
    let mut parts =
        ModuleParts::for_runtime(Runtime::VueServerRenderer, "3.5.35", "component").unwrap();
    let mut script = Writer::default();
    script.use_helper(parts.vocabulary.helper("ref").unwrap());
    script.use_helper(parts.vocabulary.helper("unref").unwrap());
    script.push_linked(
        "const component = { setup: () => ({ value: _ref(1) }) }",
        Span::new(0, 1),
    );
    let mut render = Writer::default();
    render.use_helper(parts.vocabulary.helper("ssrRenderAttrs").unwrap());
    render.use_helper(parts.vocabulary.helper("unref").unwrap());
    render.use_helper(parts.vocabulary.helper("ssrInterpolate").unwrap());
    render.use_helper(parts.vocabulary.helper("ssrRenderAttrs").unwrap());
    render.push_linked(
        "function ssrRender(_ctx, _push) { _push(_ssrInterpolate(_unref(_ctx.value))) }",
        Span::new(2, 3),
    );
    parts.script = Some(ScriptPart::Body(script));
    parts.render = Some(render);
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    parts
}

#[test]
fn real_ssr_vocabulary_imports_both_modules_without_alias_collision_or_reordering() {
    let emitted = assemble(ssr::<Recorded>()).unwrap();
    let plain = assemble(ssr::<NoLinks>()).unwrap();
    assert_eq!(emitted.text, plain.text);
    assert_eq!(emitted.helpers, plain.helpers);
    let imports = concat!(
        "import { ssrRenderAttrs as _ssrRenderAttrs, ssrInterpolate as _ssrInterpolate } from \"@vue/server-renderer\"\n",
        "import { ref as _ref, unref as _unref } from \"vue\"\n",
    );
    assert!(emitted.text.starts_with(imports));
    assert_eq!(emitted.text.matches("unref as _unref").count(), 1);
    let vocabulary = ssr::<NoLinks>().vocabulary;
    assert_eq!(
        emitted
            .helpers
            .in_use_order()
            .iter()
            .map(|&helper| vocabulary.name(helper).unwrap())
            .collect::<Vec<_>>(),
        ["ref", "unref", "ssrRenderAttrs", "ssrInterpolate"]
    );
    assert_eq!(
        emitted.links.links()[0].generated.start as usize,
        imports.len()
    );
    assert_eq!(emitted.links.links()[0].authored, Span::new(0, 1));
    assert_eq!(emitted.links.links()[1].authored, Span::new(2, 3));
    for link in emitted.links.links() {
        assert!(link.generated.end <= emitted.text.len() as u32);
    }
}

#[test]
fn real_dom_and_vapor_tables_are_consumed_by_the_same_module_assembler() {
    for (runtime, version, helper, body) in [
        (
            Runtime::VueDom,
            "3.5.35",
            "createElementVNode",
            "function render() { return _createElementVNode('p') }",
        ),
        (
            Runtime::VueVapor,
            "3.6.0-rc.9",
            "template",
            "const _tmpl = _template('<p>ok</p>'); function render() { return _tmpl() }",
        ),
    ] {
        let mut parts = ModuleParts::for_runtime(runtime, version, "component").unwrap();
        let mut render: Writer<NoLinks> = Writer::default();
        render.use_helper(parts.vocabulary.helper(helper).unwrap());
        render.push(body);
        parts.render = Some(render);
        parts.placement = RenderPlacement::Function {
            binding: "render",
            property: RenderProperty::Render,
        };
        let emitted = assemble(parts).unwrap();
        let import = vize_l0::cstr!("import {{ {helper} as _{helper} }} from \"vue\"\n");
        assert!(emitted.text.starts_with(import.as_str()));
        assert!(emitted.text.contains(body));
    }
}

#[test]
fn unsupported_runtime_versions_are_rejected_before_any_fragments_are_assembled() {
    assert_eq!(
        ModuleParts::<NoLinks>::for_runtime(Runtime::VueVapor, "3.5.35", "component").unwrap_err(),
        AssemblyError::UnsupportedRuntimeVersion
    );
    assert_eq!(
        ModuleParts::<NoLinks>::for_runtime(Runtime::VueDom, "2.7.16", "component").unwrap_err(),
        AssemblyError::UnsupportedRuntimeVersion
    );
}
