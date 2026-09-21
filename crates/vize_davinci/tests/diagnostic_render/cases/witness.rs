//! A proven diagnostic: its witness chain renders as "why" notes, one per
//! link, in proof order — a link with source text quotes it, a link keyed
//! only by name names it.

use super::{Case, help, primary, span, tr};
use vize_davinci::diagnostic::{Diagnostic, Stage, WitnessChain, WitnessKey, WitnessLink};
use vize_davinci::pass::AnalysisId;
use vize_s0::Span;

const WITNESS_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue"

const total = ref(0)
</script>

<template>
  <p>Nothing reads the total.</p>
</template>
"#;

pub const WITNESS_WHY: Case =
    Case {
        name: "witness_why",
        path: "src/components/Totals.vue",
        source: WITNESS_SOURCE,
        diagnostics: |locale| {
            let binding = span(WITNESS_SOURCE, "total");
            let chain = WitnessChain::new(WitnessLink::new(
                AnalysisId::new(3),
                binding,
                WitnessKey::Name("total".into()),
            ))
            .then(WitnessLink::new(
                AnalysisId::new(7),
                Span::new(0, 0),
                WitnessKey::Name("total".into()),
            ));
            let message = tr(
                locale,
                "`total` is declared but never read.",
                "`total` は宣言されていますが、一度も読まれていません。",
                "`total` 已声明但从未被读取。",
            );
            let produced = Diagnostic::proven(Stage::Semantic, binding, message, chain)
            .with_part(primary(
                binding,
                tr(locale, "declared here", "ここで宣言されています", "在此处声明"),
            ))
            .with_part(help(
                binding,
                tr(
                    locale,
                    "remove the binding, or prefix it with `_` if it is intentionally unused",
                    "この変数を削除するか、意図的に使わないなら名前の先頭に `_` を付けてください",
                    "请删除该绑定；如确实有意不使用，请在名称前加上 `_`",
                ),
            ));
            vec![(Some("vue/no-unused-vars"), produced)]
        },
    };
