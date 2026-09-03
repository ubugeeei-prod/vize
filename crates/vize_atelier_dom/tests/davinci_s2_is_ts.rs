//! P2-11 installment 87 witness: **`is_ts`**. Template expressions are
//! TypeScript, so the emitter type-erases each one before the identifier
//! pass reads it — and every later reader (the patch-flag static check,
//! the hoist decision, the hoisted text itself) sees the erased spelling,
//! not the authored bytes. Compared byte-for-byte with the shipped lane
//! in the three shapes production uses: plain, prefixed, and the full
//! module + prefix + bindings dev-server shape.

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
    // a constness rule S2 does not carry yet — unrelated to `is_ts` (it
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
    support::assert_s2_matches_shipped_with_options(
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
    support::assert_s2_matches_shipped_with_options(
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
    support::assert_s2_matches_shipped_with_options(
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
    let allocator = vize_s0::Allocator::new();
    let emit = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
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
