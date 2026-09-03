//! P2-11 installment 86 witness: **binding metadata** in non-inline mode.
//! With `prefix_identifiers` and a binding table, free identifiers resolve
//! to `$setup.` / `$props.` / `$data.` / `$options.` (`_ctx.` for Vue
//! globals and unknown names), components and directives that name a
//! script binding read `$setup` members instead of runtime resolution,
//! Options API method handlers are guarded references, destructured prop
//! aliases project onto their keys, and module mode carries the
//! six-argument render signature — all compared byte-for-byte with the
//! shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s0::FxHashMap;
use vize_s1_to_s2::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    ("setup_ref", "<div>{{ count }}</div>"),
    ("setup_let", "<div>{{ msg }}</div>"),
    ("setup_const", "<div>{{ handler }}</div>"),
    ("setup_reactive", "<div>{{ state.value }}</div>"),
    ("props", "<div>{{ title }}</div>"),
    ("props_alias", "<div>{{ label }}</div>"),
    ("data_option", "<div>{{ d }} {{ method }}</div>"),
    ("literal_const", "<div>{{ LIMIT }}</div>"),
    ("vue_global", "<div>{{ $slots.default }}</div>"),
    ("external_module", "<div>{{ helper(count) }}</div>"),
    ("unknown", "<div>{{ other }}</div>"),
    ("mixed_text", "<p>Hi {{ title }}, {{ other }}!</p>"),
    (
        "shorthand_object",
        r#"<div :foo="{ count, title, other }"></div>"#,
    ),
    (
        "bind_member",
        r#"<div :id="state.id" :class="{ a: count }"></div>"#,
    ),
    ("bind_style", r#"<div :style="{ color: theme }"></div>"#),
    (
        "bind_arrow",
        r#"<div :fn="(e) => handler(e, count)"></div>"#,
    ),
    (
        "bind_globals",
        r#"<div :a="Math.max(count, LIMIT)" :b="undefined"></div>"#,
    ),
    ("handler_setup_const", r#"<div @click="handler"></div>"#),
    ("handler_setup_ref", r#"<div @click="count"></div>"#),
    ("handler_options_method", r#"<div @click="method"></div>"#),
    ("handler_options_padded", r#"<div @click=" method "></div>"#),
    (
        "handler_options_member",
        r#"<div @click="method.bind"></div>"#,
    ),
    (
        "handler_options_modifiers",
        r#"<div @click.stop="method" @keyup.enter="method"></div>"#,
    ),
    ("handler_inline_update", r#"<div @click="count++"></div>"#),
    (
        "handler_inline_call",
        r#"<div @click="handler(count, other)"></div>"#,
    ),
    (
        "handler_inline_assign",
        r#"<div @click="msg = title"></div>"#,
    ),
    (
        "handler_arrow",
        r#"<div @focus="() => handler(count)"></div>"#,
    ),
    ("handler_multi_statement", r#"<div @keyup="a; b"></div>"#),
    ("handler_props", r#"<div @click="title"></div>"#),
    ("model_native", r#"<input v-model="msg">"#),
    ("model_native_ref", r#"<input v-model="count">"#),
    ("model_native_member", r#"<input v-model="state.value">"#),
    ("model_component", r#"<MyComp v-model="msg" />"#),
    ("model_component_arg", r#"<MyComp v-model:title="title" />"#),
    (
        "vfor_alias_shadows",
        r#"<li v-for="count in items">{{ count }} {{ other }}</li>"#,
    ),
    (
        "vfor_destructured",
        r#"<li v-for="({ title }, i) in items" :key="i">{{ title }} {{ count }}</li>"#,
    ),
    (
        "slot_param_shadows",
        r#"<MyComp v-slot="{ count }">{{ count }} {{ msg }}</MyComp>"#,
    ),
    (
        "slot_default_value",
        r#"<MyComp #default="{ x = count }">{{ x }}</MyComp>"#,
    ),
    (
        "dynamic_key_simple",
        r#"<div :[count]="1" @[method]="handler"></div>"#,
    ),
    ("dynamic_key_member", r#"<div :[state.key]="1"></div>"#),
    ("dynamic_key_template", "<div :[`k-${count}`]=\"1\"></div>"),
    ("component_setup_const", "<MyComp />"),
    ("component_kebab", "<my-comp />"),
    ("component_external", "<FooBar />"),
    ("component_props_named", "<Icon />"),
    ("component_dotted", "<Ns.Item />"),
    ("component_unknown", "<Other />"),
    (
        "component_mixed",
        "<div><MyComp /><Other /><Ns.Item /></div>",
    ),
    (
        "component_children",
        "<MyComp><span>{{ count }}</span></MyComp>",
    ),
    (
        "component_props",
        r#"<MyComp :title="title" @update="handler" />"#,
    ),
    ("component_dynamic", r#"<component :is="view" />"#),
    ("directive_setup", r#"<div v-focus></div>"#),
    ("directive_setup_value", r#"<div v-focus="count"></div>"#),
    ("directive_unknown", r#"<div v-other="count"></div>"#),
    ("directive_mixed", r#"<div v-focus v-other></div>"#),
    ("directive_kebab", r#"<div v-my-dir></div>"#),
    (
        "show_if",
        r#"<div v-if="count"><span v-show="msg"></span></div>"#,
    ),
    (
        "html_text",
        r#"<div v-html="rawHtml"></div><p v-text="title"></p>"#,
    ),
    ("v_once", "<div v-once>{{ count }}</div>"),
    ("v_memo", r#"<div v-memo="[count]">{{ msg }}</div>"#),
    (
        "slot_outlet",
        r#"<slot :item="count" name="x">{{ msg }}</slot>"#,
    ),
    (
        "bind_object_spread",
        r#"<div v-bind="attrs" v-on="listeners"></div>"#,
    ),
    (
        "props_alias_object",
        r#"<div :foo="{ label }" @click="label = 1"></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("handler", BindingType::SetupConst),
        ("state", BindingType::SetupReactiveConst),
        ("theme", BindingType::SetupMaybeRef),
        ("title", BindingType::Props),
        ("label", BindingType::PropsAliased),
        ("d", BindingType::Data),
        ("method", BindingType::Options),
        ("LIMIT", BindingType::LiteralConst),
        ("$slots", BindingType::VueGlobal),
        ("helper", BindingType::ExternalModule),
        ("FooBar", BindingType::ExternalModule),
        ("MyComp", BindingType::SetupConst),
        ("Icon", BindingType::Props),
        ("Ns", BindingType::SetupConst),
        ("vFocus", BindingType::SetupConst),
        ("vMyDir", BindingType::SetupLet),
        ("view", BindingType::SetupRef),
        ("rawHtml", BindingType::SetupRef),
        ("attrs", BindingType::SetupConst),
        ("listeners", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
    ];
    let mut bindings = FxHashMap::default();
    for (name, kind) in entries {
        bindings.insert((*name).into(), *kind);
    }
    let mut props_aliases = FxHashMap::default();
    props_aliases.insert("label".into(), "aria-label".into());
    BindingMetadata {
        bindings,
        props_aliases,
        is_script_setup: true,
    }
}

fn binding_kind(kind: BindingType) -> BindingKind {
    match kind {
        BindingType::SetupLet => BindingKind::SetupLet,
        BindingType::SetupMaybeRef => BindingKind::SetupMaybeRef,
        BindingType::SetupRef => BindingKind::SetupRef,
        BindingType::SetupReactiveConst => BindingKind::SetupReactiveConst,
        BindingType::SetupConst => BindingKind::SetupConst,
        BindingType::Props => BindingKind::Props,
        BindingType::PropsAliased => BindingKind::PropsAliased,
        BindingType::Data => BindingKind::Data,
        BindingType::Options => BindingKind::Options,
        BindingType::LiteralConst => BindingKind::LiteralConst,
        BindingType::JsGlobalUniversal => BindingKind::JsGlobalUniversal,
        BindingType::JsGlobalBrowser => BindingKind::JsGlobalBrowser,
        BindingType::JsGlobalNode => BindingKind::JsGlobalNode,
        BindingType::JsGlobalDeno => BindingKind::JsGlobalDeno,
        BindingType::JsGlobalBun => BindingKind::JsGlobalBun,
        BindingType::VueGlobal => BindingKind::VueGlobal,
        BindingType::ExternalModule => BindingKind::ExternalModule,
    }
}

fn table(metadata: &BindingMetadata) -> BindingTable {
    BindingTable::new(
        metadata
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), binding_kind(*kind))),
        metadata
            .props_aliases
            .iter()
            .map(|(local, key)| (local.as_str(), key.as_str())),
        metadata.is_script_setup,
    )
}

fn shipped_options(mode: CodegenMode) -> DomCompilerOptions {
    DomCompilerOptions {
        mode,
        prefix_identifiers: true,
        binding_metadata: Some(metadata()),
        ..Default::default()
    }
}

#[test]
fn binding_metadata_in_function_mode_matches_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &shipped_options(CodegenMode::Function),
        &CodegenOptions::default(),
        &DomEmitOptions {
            prefix_identifiers: true,
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn binding_metadata_in_module_mode_matches_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &shipped_options(CodegenMode::Module),
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// Exact spellings the dual-run above would also accept from a lane that
/// ignored the table entirely if the shipped side regressed alongside it:
/// the six-argument module signature, the `$setup.` read, the `$setup`
/// component tag, the `$setup["vFocus"]` directive and the guarded
/// Options API handler.
#[test]
fn binding_metadata_spellings_are_pinned() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let options = DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        bindings: Some(&table),
        ..DomEmitOptions::DEFAULT
    };
    let emit = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &options,
        )
        .expect("binding witness must emit")
        .assembled()
    };
    assert_eq!(
        emit("<div>{{ count }}</div>"),
        "import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString($setup.count), 1 /* TEXT */))\n}"
    );
    assert_eq!(
        emit(r#"<MyComp v-focus @click="method" />"#),
        "import { withDirectives as _withDirectives, openBlock as _openBlock, createBlock as _createBlock } from \"vue\"\n\nexport function render(_ctx, _cache, $props, $setup, $data, $options) {\n  const _directive_focus = $setup[\"vFocus\"]\n  \n  return _withDirectives((_openBlock(), _createBlock($setup.MyComp, { onClick: (...args) => (_ctx.method && _ctx.method(...args)) }, null, 8 /* PROPS */, [\"onClick\"])), [\n    [_directive_focus]\n  ])\n}"
    );
}
