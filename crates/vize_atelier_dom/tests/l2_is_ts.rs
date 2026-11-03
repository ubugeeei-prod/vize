//! P2-11 installment 87 witness: **`is_ts`**. Template expressions are
//! TypeScript, so the emitter type-erases each one before the identifier
//! pass reads it — and every later reader (the patch-flag static check,
//! the hoist decision, the hoisted text itself) sees the erased spelling,
//! not the authored bytes. Compared byte-for-byte with the shipped lane
//! in the three shapes production uses: plain, prefixed, and the full
//! module + prefix + bindings dev-server shape.

#![expect(
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use crate::support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_l0::FxHashMap;
use vize_l1_to_l2::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    // `as` assertions, the shape the detector fast-paths.
    ("interp_as", "<div>{{ count as number }}</div>"),
    (
        "interp_as_chain",
        "<div>{{ (raw as Wrapper).value as string }}</div>",
    ),
    ("bind_as", r#"<div :id="raw as string"></div>"#),
    (
        "bind_as_const_object",
        r#"<Card :meta="{ src: 'a.png', loading: 'lazy' as const }" />"#,
    ),
    ("bind_as_in_string", r#"<div :id="'kept as is'"></div>"#),
    // Non-null assertions, including the one that reads as logical NOT.
    ("interp_non_null", "<div>{{ maybe!.name }}</div>"),
    ("interp_non_null_call", "<div>{{ list()!.length }}</div>"),
    ("interp_non_null_index", "<div>{{ list[0]! + 1 }}</div>"),
    ("interp_logical_not", "<div>{{ !count }}</div>"),
    (
        "bind_non_null_math",
        r#"<div :n="Intl.NumberFormat().format((a ?? b)! * 1000)"></div>"#,
    ),
    // Generic calls and `satisfies`.
    ("interp_generic", "<div>{{ pick<Item>(list) }}</div>"),
    (
        "interp_generic_nested",
        "<div>{{ make<Map<string, number>>(seed) }}</div>",
    ),
    (
        "interp_satisfies",
        "<div>{{ payload satisfies Payload }}</div>",
    ),
    (
        "interp_satisfies_in_string",
        r#"<div>{{ 'payload satisfies Payload'.length }}</div>"#,
    ),
    // Annotated parameters in inline functions.
    (
        "handler_typed_param",
        r#"<div @click="(e: MouseEvent) => pick(e)"></div>"#,
    ),
    // A dependency-free arrow would be hoisted by the shipped lane through
    // a constness rule L2 does not carry yet — unrelated to `is_ts` (it
    // reproduces on plain JS), so the typed-parameter case here keeps a
    // runtime dependency and stays on the un-hoisted path.
    (
        "bind_typed_params",
        r#"<div :fn="(a: string, b: number) => pick(a, b)"></div>"#,
    ),
    (
        "interp_typed_filter",
        "<div>{{ list.filter((x: number) => x > count) }}</div>",
    ),
    // Handlers whose *shape* changes only after erasure.
    (
        "handler_as_reference",
        r#"<div @click="onPick as any"></div>"#,
    ),
    (
        "handler_non_null_reference",
        r#"<div @click="onPick!"></div>"#,
    ),
    ("handler_plain_reference", r#"<div @click="onPick"></div>"#),
    (
        "handler_statement_as",
        r#"<div @click="count = raw as number"></div>"#,
    ),
    // Structural directives.
    (
        "vfor_as",
        r#"<li v-for="item in (list as Item[])" :key="item.id">{{ item.n }}</li>"#,
    ),
    (
        "vfor_typed_alias",
        r#"<li v-for="(item, i) in list" :key="i">{{ (item as Item).n }}</li>"#,
    ),
    ("vif_as", r#"<p v-if="(flag as boolean)">{{ count }}</p>"#),
    ("vmodel_as", r#"<input v-model="(form as Form).name">"#),
    ("vshow_non_null", r#"<div v-show="visible!"></div>"#),
    ("vhtml_as", r#"<div v-html="raw as string"></div>"#),
    ("vtext_as", r#"<p v-text="raw as string"></p>"#),
    (
        "vmemo_as",
        r#"<div v-memo="[count as number]">{{ count }}</div>"#,
    ),
    // Slots and dynamic keys.
    (
        "slot_prop_as",
        r#"<Card #body="{ row }">{{ (row as Row).n }}</Card>"#,
    ),
    (
        "slot_outlet_as",
        r#"<slot :item="raw as Item">{{ count }}</slot>"#,
    ),
    // A dynamic argument cannot carry a `]`, so the TS-shaped value sits
    // on the right-hand side instead.
    (
        "dynamic_bind_key",
        r#"<div :[key]="count as number"></div>"#,
    ),
    ("dynamic_on_key", r#"<div @[key]="onPick as any"></div>"#),
    // Hoisting and patch flags read the erased text.
    (
        "hoisted_static_as",
        r#"<div><span :meta="{ a: 'b' as const }">x</span></div>"#,
    ),
    (
        "patch_flag_as_const",
        r#"<Card :meta="{ n: 1 } as const" :live="count" />"#,
    ),
    (
        "text_only_children",
        "<p>Hi {{ name as string }}, {{ count }}!</p>",
    ),
    // Text with `!` that is not TypeScript at all — the detector's own
    // false positive, whose round-trip the shipped lane also takes.
    (
        "bang_in_string_literal",
        r#"<Card :meta="{ text: 'Hello, Ant Design!' }" />"#,
    ),
    // Plain JavaScript under `is_ts`: the erasure must be a no-op.
    ("plain_member", "<div>{{ user.name }}</div>"),
    ("plain_call", r#"<div :id="pick(list, count)"></div>"#),
    ("plain_arrow", r#"<div @click="() => pick(count)"></div>"#),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("count", BindingType::SetupRef),
        ("raw", BindingType::SetupLet),
        ("list", BindingType::SetupConst),
        ("form", BindingType::SetupReactiveConst),
        ("flag", BindingType::SetupMaybeRef),
        ("visible", BindingType::SetupRef),
        ("maybe", BindingType::SetupRef),
        ("name", BindingType::Props),
        ("payload", BindingType::Props),
        ("seed", BindingType::LiteralConst),
        ("key", BindingType::SetupRef),
        ("pick", BindingType::SetupConst),
        ("make", BindingType::ExternalModule),
        ("onPick", BindingType::SetupConst),
        ("a", BindingType::SetupRef),
        ("b", BindingType::SetupRef),
        ("user", BindingType::SetupReactiveConst),
        ("Card", BindingType::SetupConst),
    ];
    let mut bindings = FxHashMap::default();
    for (name, kind) in entries {
        bindings.insert((*name).into(), *kind);
    }
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
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
        [],
        metadata.is_script_setup,
    )
}

#[test]
fn is_ts_alone_matches_the_shipped_dom_lane() {
    support::assert_l2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions {
            is_ts: true,
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            is_ts: true,
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn is_ts_with_prefixing_matches_the_shipped_dom_lane() {
    support::assert_l2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions {
            prefix_identifiers: true,
            is_ts: true,
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            prefix_identifiers: true,
            is_ts: true,
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn is_ts_in_the_production_shape_matches_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_l2_matches_shipped_with_options(
        BATTERY,
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            is_ts: true,
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            is_ts: true,
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// The erasure is not cosmetic: these spellings would all differ if the
/// emitter passed the authored bytes through.
#[test]
fn erased_spellings_are_pinned() {
    let allocator = vize_l0::Allocator::new();
    let emit = |src: &str| {
        vize_l1_to_l2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_l1_to_l2::LegacyCaps::VUE3,
            &DomEmitOptions {
                prefix_identifiers: true,
                is_ts: true,
                ..DomEmitOptions::DEFAULT
            },
        )
        .expect("is_ts witness must emit")
        .assembled()
    };
    let body = |src: &str| {
        let out = emit(src);
        out.lines()
            .find(|line| line.contains("return "))
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    assert_eq!(
        body("<div>{{ count as number }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(_ctx.count), 1 /* TEXT */))"
    );
    assert_eq!(
        body("<div>{{ maybe!.name }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(_ctx.maybe.name), 1 /* TEXT */))"
    );
    assert_eq!(
        body("<div>{{ pick<Item>(list) }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(_ctx.pick(_ctx.list)), 1 /* TEXT */))"
    );
    // `Intl` is seeded into the transform's root scope, so it stays bare
    // even though the prefixer's own allowlist does not name it.
    assert_eq!(
        body("<div>{{ Intl.NumberFormat().format(count!) }}</div>"),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(Intl.NumberFormat().format(_ctx.count)), 1 /* TEXT */))"
    );
}

/// TypeScript the detection scan does not recognize. The template lane
/// decides it by the parse, so each one still leaves as JavaScript.
const UNSCANNED: &[(&str, &str)] = &[
    (
        "return_type",
        r#"<div @click="(e): void => pick(e)"></div>"#,
    ),
    (
        "typed_function_expression",
        r#"<div @click="function (e: Event) { pick(e) }"></div>"#,
    ),
    (
        "this_param_and_return_type",
        r#"<div :fn="function (this: Window, x: number): number { return x + count }"></div>"#,
    ),
    ("instantiation", "<div>{{ pick<Item> }}</div>"),
    (
        "callback_return_type",
        "<div>{{ list.map((x): number => x * 2) }}</div>",
    ),
    (
        "typed_declaration_in_body",
        "<div>{{ (() => { const y: number = count; return y })() }}</div>",
    ),
    ("angle_assertion", "<div>{{ <any>count }}</div>"),
    (
        "optional_param",
        "<div>{{ ((x?: number) => x)(count) }}</div>",
    ),
    (
        "typed_statements",
        r#"<div @click="const n: number = count; pick(n)"></div>"#,
    ),
    (
        "typed_for_aliases",
        r#"<li v-for="(item: Item, i: number) in list" :key="i">{{ item }}</li>"#,
    ),
    (
        "typed_slot_props",
        r#"<Card #default="{ row }: { row: Row }">{{ row }}</Card>"#,
    ),
    (
        "typed_slot_param",
        r#"<Card v-slot="props: Props">{{ props }}</Card>"#,
    ),
];

/// The render module the SFC compiler receives, with its diagnostics.
fn sfc_render(src: &str, inline: bool) -> (Vec<vize_l0::String>, vize_l0::String) {
    let allocator = vize_l0::Allocator::new();
    let options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        is_ts: true,
        inline,
        binding_metadata: Some(metadata()),
        ..Default::default()
    };
    let (_, errors, result) =
        vize_atelier_dom::compile_template_with_options(&allocator, src, options);
    let errors = errors
        .iter()
        .map(|error| vize_l0::cstr!("{error:?}"))
        .collect();
    let code = vize_l0::cstr!("{}\n{}", result.preamble, result.code);
    (errors, code)
}

fn javascript_diagnostics(code: &str) -> Vec<vize_l0::String> {
    let allocator = oxc_allocator::Allocator::default();
    oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs())
        .parse()
        .diagnostics
        .iter()
        .map(|diagnostic| vize_l0::cstr!("{diagnostic}"))
        .collect()
}

fn render_return(code: &str) -> vize_l0::String {
    let mut out = vize_l0::String::default();
    let lines = code
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("return "))
        .take_while(|line| *line != "}");
    for (index, line) in lines.enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(line.trim());
    }
    out
}

#[test]
fn typescript_the_scan_misses_leaves_the_template_lane_as_javascript() {
    for inline in [false, true] {
        for (name, src) in UNSCANNED {
            let (errors, code) = sfc_render(src, inline);
            assert_eq!(
                errors,
                Vec::<vize_l0::String>::new(),
                "{name} (inline: {inline})"
            );
            assert_eq!(
                javascript_diagnostics(&code),
                Vec::<vize_l0::String>::new(),
                "{name} (inline: {inline}):\n{code}"
            );
        }
    }
}

/// The erased text is prefixed like any other JavaScript: before erasure
/// decided by the parse, these names reached the render code bare.
#[test]
fn unscanned_typescript_is_prefixed_after_erasure() {
    let body = |src: &str, inline: bool| render_return(&sfc_render(src, inline).1);
    assert_eq!(
        body("<div>{{ list.map((x): number => x * 2) }}</div>", false),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString($setup.list.map((x) => x * 2)), 1 /* TEXT */))"
    );
    assert_eq!(
        body(
            "<div>{{ (() => { const y: number = count; return y })() }}</div>",
            true
        ),
        "return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString((() => {\nconst y = count.value;\nreturn y;\n})()), 1 /* TEXT */))"
    );
    assert_eq!(
        body(r#"<div @click="(e): void => pick(e)"></div>"#, false),
        "return (_openBlock(), _createElementBlock(\"div\", {\nonClick: (e) => $setup.pick(e)\n}, null, 8 /* PROPS */, [\"onClick\"]))"
    );
    assert_eq!(
        body(
            r#"<div @click="const n: number = count; pick(n)"></div>"#,
            false
        ),
        "return (_openBlock(), _createElementBlock(\"div\", {\nonClick: $event => {const n = $setup.count;\n$setup.pick(n);}\n}, null, 8 /* PROPS */, [\"onClick\"]))"
    );
    assert_eq!(
        body(
            r#"<Card #default="{ row }: { row: Row }">{{ row }}</Card>"#,
            false
        ),
        "return (_openBlock(), _createBlock($setup[\"Card\"], null, {\ndefault: _withCtx(({ row }) => [\n_createTextVNode(_toDisplayString(row), 1 /* TEXT */)\n]),\n_: 1 /* STABLE */\n}))"
    );
    assert_eq!(
        body(
            r#"<li v-for="(item: Item, i: number) in list" :key="i">{{ item }}</li>"#,
            false
        ),
        "return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList($setup.list, (item, i) => {\nreturn (_openBlock(), _createElementBlock(\"li\", { key: i }, _toDisplayString(item), 1 /* TEXT */))\n}), 128 /* KEYED_FRAGMENT */))"
    );
}

/// Negative control: text that is neither JavaScript nor TypeScript is
/// still reported, never erased into something that compiles.
#[test]
fn text_that_is_not_typescript_still_reports() {
    let (errors, _) = sfc_render("<div>{{ count as }}</div>", false);
    assert_eq!(errors.len(), 1, "{errors:?}");
}
