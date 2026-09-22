//! Witness-derived why notes for the two producers that name a fact chain
//! (P4-14c): P4-3c's `UnusedBindings` and P4-11b's composed nesting
//! (`HtmlElements`, then `HtmlComposedNesting`). One `= note: because …`
//! footer per link, in proof order, ahead of help.

use super::{Case, help, primary, secondary, span, tr};
use vize_davinci::diagnostic::{Diagnostic, Stage, WitnessChain, WitnessKey, WitnessLink};
use vize_davinci::fact::ids;

const UNUSED_SOURCE: &str = r#"<script setup lang="ts">
const total = ref(0)
</script>

<template>
  <p>Nothing reads the total.</p>
</template>
"#;

/// P4-3c: one link, the unread script-setup binding.
pub const UNUSED_BINDING: Case = Case {
    name: "witness_unused_binding",
    path: "src/components/Totals.vue",
    source: UNUSED_SOURCE,
    diagnostics: |locale| {
        let binding = span(UNUSED_SOURCE, "total");
        let chain = WitnessChain::new(WitnessLink::new(
            ids::UNUSED_BINDINGS,
            binding,
            WitnessKey::Name("total".into()),
        ));
        let message = tr(
            locale,
            "`total` is declared in `<script setup>` and nothing reads it.",
            "`total` は `<script setup>` で宣言されていますが、どこからも読まれていません。",
            "`total` 在 `<script setup>` 中声明，但没有任何地方读取它。",
        );
        let produced = Diagnostic::proven(Stage::Semantic, binding, message, chain)
            .with_part(primary(
                binding,
                tr(
                    locale,
                    "declared here",
                    "ここで宣言されています",
                    "在此处声明",
                ),
            ))
            .with_part(help(
                binding,
                tr(
                    locale,
                    "remove the binding, or read it from the template",
                    "この束縛を削除するか、テンプレートから読んでください",
                    "请删除该绑定，或在模板中读取它",
                ),
            ));
        vec![(Some("vue/no-unused-setup-bindings"), produced)]
    },
};

const COMPOSED_SOURCE: &str = r#"<script setup lang="ts">
import InfoCard from "./components/InfoCard.vue";
</script>

<template>
  <p>
    <InfoCard title="Composed" />
  </p>
</template>
"#;

/// P4-11b: the ancestor element, then the violation at the usage site.
pub const COMPOSED_NESTING: Case = Case {
    name: "witness_composed_nesting",
    path: "src/App.vue",
    source: COMPOSED_SOURCE,
    diagnostics: |locale| {
        let ancestor = span(COMPOSED_SOURCE, "<p");
        let usage = span(COMPOSED_SOURCE, "<InfoCard");
        let chain = WitnessChain::new(WitnessLink::new(
            ids::HTML_ELEMENTS,
            ancestor,
            WitnessKey::Index(0),
        ))
        .then(WitnessLink::new(
            ids::HTML_COMPOSED_NESTING,
            usage,
            WitnessKey::Index(0),
        ));
        let message = tr(
            locale,
            "<div> (rendered by <InfoCard> at src/components/InfoCard.vue:6:3) closes the open <p>: the browser ends the paragraph before it, so the rendered DOM will not match this template",
            "<div>（<InfoCard> が src/components/InfoCard.vue:6:3 で描画）は開いている <p> を閉じます。ブラウザはその手前で段落を終了するため、描画される DOM はこのテンプレートと一致しません",
            "<div>（由 <InfoCard> 在 src/components/InfoCard.vue:6:3 渲染）会关闭已打开的 <p>：浏览器会在它之前结束段落，渲染出的 DOM 与此模板不一致",
        );
        let produced = Diagnostic::proven(Stage::Semantic, usage, message, chain)
            .with_part(secondary(
                ancestor,
                tr(locale, "<p> is open here", "<p> はここで開かれています", "<p> 在此处打开"),
            ))
            .with_part(help(
                usage,
                tr(
                    locale,
                    "render the component where its root is permitted, or change the parent element",
                    "ルート要素が許可される位置で描画するか、親要素を変えてください",
                    "请在允许其根元素的位置渲染该组件，或更换父元素",
                ),
            ));
        vec![(Some("html/cross-component-nesting"), produced)]
    },
};
