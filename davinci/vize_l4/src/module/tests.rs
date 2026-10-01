use vize_l0::Span;

use super::{AssemblyError, ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble};
use crate::runtime::{Helper, Vocabulary};
use crate::write::{LinkSink, NoLinks, Recorded, SpanLink, Writer};

const VOCABULARY: Vocabulary = Vocabulary {
    module: "vue",
    names: &["createVNode", "ref", "toDisplayString"],
};

fn fragment<L: LinkSink>(text: &str, authored: Span, helpers: &[u8]) -> Writer<L> {
    let mut writer = Writer::default();
    writer.push_linked(text, authored);
    for &index in helpers {
        writer.use_helper(Helper::from_index(index).unwrap());
    }
    writer
}

fn bare<L: LinkSink>() -> ModuleParts<'static, L> {
    ModuleParts {
        vocabulary: &VOCABULARY,
        prelude: None,
        script: None,
        render: None,
        placement: RenderPlacement::None,
        component: "_sfc_main",
    }
}

#[test]
fn empty_component_has_one_default_export_without_runtime_imports() {
    let emitted = assemble(bare::<NoLinks>()).unwrap();
    assert_eq!(
        emitted.text.as_str(),
        "const _sfc_main = {}\nexport default _sfc_main\n"
    );
    assert!(emitted.helpers.is_empty());
    assert!(emitted.into_document().links().is_empty());
}

#[test]
fn script_only_module_preserves_prepared_script_and_line_boundaries() {
    let mut parts = bare::<NoLinks>();
    parts.component = "component";
    parts.script = Some(ScriptPart::Body(fragment(
        "const component = { name: 'App' } // trailing comment",
        Span::new(0, 50),
        &[],
    )));
    let emitted = assemble(parts).unwrap();
    assert_eq!(
        emitted.text.as_str(),
        "const component = { name: 'App' } // trailing comment\nexport default component\n"
    );
}

fn normal_module<L: LinkSink>() -> ModuleParts<'static, L> {
    let mut parts = bare();
    parts.prelude = Some(fragment(
        "const _hoisted_1 = _createVNode('p');\nconst _cache = []",
        Span::new(0, 3),
        &[0],
    ));
    parts.script = Some(ScriptPart::Body(fragment(
        "const _sfc_main = { setup() { return { count: _ref(0) } } }",
        Span::new(4, 10),
        &[1],
    )));
    parts.render = Some(fragment(
        "function _sfc_render() { return _toDisplayString(this.count) }",
        Span::new(11, 16),
        &[2, 0],
    ));
    parts.placement = RenderPlacement::Function {
        binding: "_sfc_render",
        property: RenderProperty::Render,
    };
    parts
}

#[test]
fn linked_module_rebases_hoists_script_and_template_after_complete_preamble() {
    let recorded = assemble(normal_module::<Recorded>()).unwrap();
    let unrecorded = assemble(normal_module::<NoLinks>()).unwrap();
    assert_eq!(recorded.text, unrecorded.text);
    assert_eq!(recorded.helpers, unrecorded.helpers);
    assert_eq!(
        recorded.text.as_str(),
        concat!(
            "import { createVNode as _createVNode, ref as _ref, toDisplayString as _toDisplayString } from \"vue\"\n",
            "const _hoisted_1 = _createVNode('p');\nconst _cache = []\n",
            ";\n",
            "const _sfc_main = { setup() { return { count: _ref(0) } } }\n",
            ";\n",
            "function _sfc_render() { return _toDisplayString(this.count) }\n",
            "_sfc_main.render = _sfc_render\nexport default _sfc_main\n",
        )
    );
    assert_eq!(
        recorded.links.links(),
        &[
            SpanLink {
                generated: Span::new(100, 155),
                authored: Span::new(0, 3),
                name: None,
                segment: true
            },
            SpanLink {
                generated: Span::new(158, 217),
                authored: Span::new(4, 10),
                name: None,
                segment: true
            },
            SpanLink {
                generated: Span::new(220, 282),
                authored: Span::new(11, 16),
                name: None,
                segment: true
            },
        ]
    );
    let document = recorded.into_document();
    assert!(document.is_recording());
    let source_map = document.source_map("App.vue", "<p>\n<script>\n{{x}}");
    assert!(source_map.contains("\"sources\":[\"App.vue\"]"));
    assert!(!unrecorded.into_document().is_recording());
}

#[test]
fn server_function_attaches_to_ssr_render_and_uses_its_supplied_vocabulary() {
    const SERVER: Vocabulary = Vocabulary {
        module: "vue/server-renderer",
        names: &["ssrInterpolate"],
    };
    let mut parts = bare::<NoLinks>();
    parts.vocabulary = &SERVER;
    parts.render = Some(fragment(
        "function ssrRender(_ctx, _push) { _push(_ssrInterpolate(_ctx.msg)) }",
        Span::new(0, 1),
        &[0],
    ));
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    let emitted = assemble(parts).unwrap();
    assert!(emitted.text.starts_with(
        "import { ssrInterpolate as _ssrInterpolate } from \"vue/server-renderer\"\n"
    ));
    assert!(
        emitted
            .text
            .ends_with("_sfc_main.ssrRender = ssrRender\nexport default _sfc_main\n")
    );
}

fn inline_module<L: LinkSink>() -> ModuleParts<'static, L> {
    let mut parts = bare();
    parts.script = Some(ScriptPart::Inline {
        before_render: fragment(
            "const _sfc_main = { setup() { const n = _ref(1); return ",
            Span::new(0, 6),
            &[1],
        ),
        after_render: fragment(" } }", Span::new(7, 10), &[2]),
    });
    parts.render = Some(fragment(
        "() => _createVNode('p', null, _toDisplayString(n.value))",
        Span::new(11, 16),
        &[0, 2],
    ));
    parts.placement = RenderPlacement::Inline;
    parts
}

#[test]
fn inline_setup_insertion_preserves_return_semantics_and_fragment_link_order() {
    let emitted = assemble(inline_module::<Recorded>()).unwrap();
    assert_eq!(
        emitted.text,
        assemble(inline_module::<NoLinks>()).unwrap().text
    );
    assert!(emitted.text.contains("return () => _createVNode('p', null, _toDisplayString(n.value)) } }\nexport default _sfc_main\n"));
    assert_eq!(
        emitted.helpers.in_use_order(),
        &[
            Helper::from_index(1).unwrap(),
            Helper::from_index(0).unwrap(),
            Helper::from_index(2).unwrap()
        ]
    );
    assert_eq!(
        emitted
            .links
            .links()
            .iter()
            .map(|link| link.authored)
            .collect::<Vec<_>>(),
        &[Span::new(0, 6), Span::new(11, 16), Span::new(7, 10)]
    );
    for link in emitted.links.links() {
        assert!(link.generated.end <= emitted.text.len() as u32);
    }
}

#[test]
fn module_specifier_is_quoted_as_a_string_literal() {
    const ESCAPED: Vocabulary = Vocabulary {
        module: "module\"with\\escapes\n",
        names: &["ref"],
    };
    let mut parts = bare::<NoLinks>();
    parts.vocabulary = &ESCAPED;
    parts.script = Some(ScriptPart::Body(fragment(
        "const _sfc_main = _ref({})",
        Span::new(0, 1),
        &[0],
    )));
    assert!(
        assemble(parts)
            .unwrap()
            .text
            .starts_with("import { ref as _ref } from \"module\\\"with\\\\escapes\\n\"\n")
    );
}

#[test]
fn unknown_helpers_are_reported_without_a_partial_import_or_panic() {
    let mut parts = bare::<NoLinks>();
    parts.prelude = Some(fragment(
        "const hoist = _missing()",
        Span::new(0, 1),
        &[127],
    ));
    assert_eq!(
        assemble(parts).unwrap_err(),
        AssemblyError::UnknownHelper(Helper::from_index(127).unwrap())
    );
}

#[test]
fn inconsistent_render_contracts_are_rejected() {
    let mut parts = bare::<NoLinks>();
    parts.render = Some(Writer::default());
    assert_eq!(
        assemble(parts).unwrap_err(),
        AssemblyError::UnexpectedRender
    );

    let mut parts = bare::<NoLinks>();
    parts.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    assert_eq!(assemble(parts).unwrap_err(), AssemblyError::MissingRender);

    let mut parts = bare::<NoLinks>();
    parts.placement = RenderPlacement::Inline;
    parts.render = Some(Writer::default());
    assert_eq!(
        assemble(parts).unwrap_err(),
        AssemblyError::MissingInlineScript
    );

    let mut parts = inline_module::<NoLinks>();
    parts.placement = RenderPlacement::None;
    parts.render = None;
    assert_eq!(
        assemble(parts).unwrap_err(),
        AssemblyError::UnexpectedInlineScript
    );

    let mut parts = inline_module::<NoLinks>();
    parts.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    assert_eq!(
        assemble(parts).unwrap_err(),
        AssemblyError::UnexpectedInlineScript
    );
}

#[test]
fn complete_statement_fragments_cannot_continue_a_previous_initializer() {
    for prelude in ["const h = make()", "const h = make() // trailing comment"] {
        let mut parts = bare::<NoLinks>();
        parts.component = "component";
        parts.prelude = Some(fragment(prelude, Span::new(0, 1), &[]));
        parts.script = Some(ScriptPart::Body(fragment(
            "(() => sideEffect())(); const component = {}",
            Span::new(2, 3),
            &[],
        )));
        let emitted = assemble(parts).unwrap();
        assert!(emitted.text.contains("\n;\n(() => sideEffect())()"));
        assert!(emitted.text.ends_with("export default component\n"));
    }
}
