---
title: ファイル間ルール
---

# ファイル間ルール

プロジェクトの検査には解析対象のコンポーネント構成が必要です。各ページに共通ファイルと悪い例・良い例を示します。共通ファイルは両方の例で使ってください。

次の Vite+ 設定から始めてください。vp run lint は Vize と Oxlint を実行します。組み込みの vp lint は Vite+ 自身の検査を実行します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "cross-file": "warn" },
    },
  },
});
```

```sh
vp run lint
```

CLI では vize lint --cross-file で同じ検査を実行できます。表示コード vize:croquis/cf/* は、lint.vize.rules には vize: を除いた croquis/cf/* として指定します。information / hint は CLI では warning として表示されます。関連位置から提供元と使用側の関係を確認できます。

公開されている 60 のコードは対応範囲が異なります。19 は CLI の検査対象（18 の完全なソースの例と 1 つの参照構成の例）で、16 は実験的な Rust analyzer に実装があるものの CLI では個別コードとして生成されません。25 は現在の生成元がない公開契約です。ルール名を設定しても未対応の生成元は有効になりません。

## プロジェクト固有の lint ID

| ルール | 例 | 重大度 |
| --- | --- | --- |
| [`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | [悪い例](./project/ecosystem-vue-router-unknown-route.md#悪い) · [良い例](./project/ecosystem-vue-router-unknown-route.md#良い) | error |
| [`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | [悪い例](./project/ecosystem-vue-router-extra-param.md#悪い) · [良い例](./project/ecosystem-vue-router-extra-param.md#良い) | error |
| [`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | [悪い例](./project/ecosystem-vue-router-param-type.md#悪い) · [良い例](./project/ecosystem-vue-router-param-type.md#良い) | error |
| [`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | [悪い例](./project/ecosystem-vue-router-missing-param.md#悪い) · [良い例](./project/ecosystem-vue-router-missing-param.md#良い) | warning |
| [`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | [悪い例](./project/html-cross-component-nesting.md#悪い) · [良い例](./project/html-cross-component-nesting.md#良い) | warning |
| [`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | [悪い例](./project/vue-cross-file-attrs-fallthrough.md#悪い) · [良い例](./project/vue-cross-file-attrs-fallthrough.md#良い) | warning |

## 公開 analyzer コード

| コード | 例 | 対応状況 |
| --- | --- | --- |
| [`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | [悪い例](./project/vize-croquis-cf-array-mutation.md#悪い) · [良い例](./project/vize-croquis-cf-array-mutation.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | [悪い例](./project/vize-croquis-cf-async-boundary.md#悪い) · [良い例](./project/vize-croquis-cf-async-boundary.md#良い) | CLI |
| [`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | [悪い例](./project/vize-croquis-cf-async-no-suspense.md#悪い) · [良い例](./project/vize-croquis-cf-async-no-suspense.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | [悪い例](./project/vize-croquis-cf-browser-api-ssr.md#悪い) · [良い例](./project/vize-croquis-cf-browser-api-ssr.md#良い) | CLI |
| [`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | [悪い例](./project/vize-croquis-cf-circular-dep.md#悪い) · [良い例](./project/vize-croquis-cf-circular-dep.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | [悪い例](./project/vize-croquis-cf-circular-reactive-dependency.md#悪い) · [良い例](./project/vize-croquis-cf-circular-reactive-dependency.md#良い) | CLI |
| [`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | [悪い例](./project/vize-croquis-cf-closure-captures-reactive.md#悪い) · [良い例](./project/vize-croquis-cf-closure-captures-reactive.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | [悪い例](./project/vize-croquis-cf-composable-outside-setup.md#悪い) · [良い例](./project/vize-croquis-cf-composable-outside-setup.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | [悪い例](./project/vize-croquis-cf-computed-side-effects.md#悪い) · [良い例](./project/vize-croquis-cf-computed-side-effects.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | [悪い例](./project/vize-croquis-cf-deep-import.md#悪い) · [良い例](./project/vize-croquis-cf-deep-import.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | [悪い例](./project/vize-croquis-cf-destructuring-breaks-reactivity.md#悪い) · [良い例](./project/vize-croquis-cf-destructuring-breaks-reactivity.md#良い) | CLI |
| [`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | [悪い例](./project/vize-croquis-cf-di-outside-setup.md#悪い) · [良い例](./project/vize-croquis-cf-di-outside-setup.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | [悪い例](./project/vize-croquis-cf-dom-access-without-next-tick.md#悪い) · [良い例](./project/vize-croquis-cf-dom-access-without-next-tick.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | [悪い例](./project/vize-croquis-cf-duplicate-id.md#悪い) · [良い例](./project/vize-croquis-cf-duplicate-id.md#良い) | CLI |
| [`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | [悪い例](./project/vize-croquis-cf-event-listener-leak.md#悪い) · [良い例](./project/vize-croquis-cf-event-listener-leak.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | [悪い例](./project/vize-croquis-cf-event-modifier.md#悪い) · [良い例](./project/vize-croquis-cf-event-modifier.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | [悪い例](./project/vize-croquis-cf-hydration-risk.md#悪い) · [良い例](./project/vize-croquis-cf-hydration-risk.md#良い) | CLI |
| [`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | [悪い例](./project/vize-croquis-cf-inherit-attrs-unused.md#悪い) · [良い例](./project/vize-croquis-cf-inherit-attrs-unused.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | [悪い例](./project/vize-croquis-cf-inject-without-symbol.md#悪い) · [良い例](./project/vize-croquis-cf-inject-without-symbol.md#良い) | CLI |
| [`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | [悪い例](./project/vize-croquis-cf-injected-async-mutation-race.md#悪い) · [良い例](./project/vize-croquis-cf-injected-async-mutation-race.md#良い) | CLI |
| [`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | [悪い例](./project/vize-croquis-cf-lifecycle-outside-setup.md#悪い) · [良い例](./project/vize-croquis-cf-lifecycle-outside-setup.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | [悪い例](./project/vize-croquis-cf-lifecycle-without-cleanup.md#悪い) · [良い例](./project/vize-croquis-cf-lifecycle-without-cleanup.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | [悪い例](./project/vize-croquis-cf-missing-required-prop.md#悪い) · [良い例](./project/vize-croquis-cf-missing-required-prop.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | [悪い例](./project/vize-croquis-cf-missing-suspense.md#悪い) · [良い例](./project/vize-croquis-cf-missing-suspense.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | [悪い例](./project/vize-croquis-cf-module-scope-reactive.md#悪い) · [良い例](./project/vize-croquis-cf-module-scope-reactive.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | [悪い例](./project/vize-croquis-cf-multi-root-attrs.md#悪い) · [良い例](./project/vize-croquis-cf-multi-root-attrs.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | [悪い例](./project/vize-croquis-cf-mutated-after-escape.md#悪い) · [良い例](./project/vize-croquis-cf-mutated-after-escape.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | [悪い例](./project/vize-croquis-cf-non-reactive-provide.md#悪い) · [良い例](./project/vize-croquis-cf-non-reactive-provide.md#良い) | CLI |
| [`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | [悪い例](./project/vize-croquis-cf-non-unique-id.md#悪い) · [良い例](./project/vize-croquis-cf-non-unique-id.md#良い) | CLI |
| [`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | [悪い例](./project/vize-croquis-cf-object-identity-comparison.md#悪い) · [良い例](./project/vize-croquis-cf-object-identity-comparison.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | [悪い例](./project/vize-croquis-cf-pinia-getter.md#悪い) · [良い例](./project/vize-croquis-cf-pinia-getter.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | [悪い例](./project/vize-croquis-cf-prop-type-mismatch.md#悪い) · [良い例](./project/vize-croquis-cf-prop-type-mismatch.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | [悪い例](./project/vize-croquis-cf-provide-inject-type.md#悪い) · [良い例](./project/vize-croquis-cf-provide-inject-type.md#良い) | CLI |
| [`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | [悪い例](./project/vize-croquis-cf-provide-without-symbol.md#悪い) · [良い例](./project/vize-croquis-cf-provide-without-symbol.md#良い) | CLI |
| [`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | [悪い例](./project/vize-croquis-cf-reactive-export.md#悪い) · [良い例](./project/vize-croquis-cf-reactive-export.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | [悪い例](./project/vize-croquis-cf-reactivity-outside-setup.md#悪い) · [良い例](./project/vize-croquis-cf-reactivity-outside-setup.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | [悪い例](./project/vize-croquis-cf-reassignment-breaks-reactivity.md#悪い) · [良い例](./project/vize-croquis-cf-reassignment-breaks-reactivity.md#良い) | CLI |
| [`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | [悪い例](./project/vize-croquis-cf-reference-escapes-scope.md#悪い) · [良い例](./project/vize-croquis-cf-reference-escapes-scope.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | [悪い例](./project/vize-croquis-cf-setup-context-violation.md#悪い) · [良い例](./project/vize-croquis-cf-setup-context-violation.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | [悪い例](./project/vize-croquis-cf-shallow-deep-access.md#悪い) · [良い例](./project/vize-croquis-cf-shallow-deep-access.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | [悪い例](./project/vize-croquis-cf-spread-breaks-reactivity.md#悪い) · [良い例](./project/vize-croquis-cf-spread-breaks-reactivity.md#良い) | CLI |
| [`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | [悪い例](./project/vize-croquis-cf-suspense-no-fallback.md#悪い) · [良い例](./project/vize-croquis-cf-suspense-no-fallback.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | [悪い例](./project/vize-croquis-cf-template-ref-timing.md#悪い) · [良い例](./project/vize-croquis-cf-template-ref-timing.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | [悪い例](./project/vize-croquis-cf-toraw-mutation.md#悪い) · [良い例](./project/vize-croquis-cf-toraw-mutation.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | [悪い例](./project/vize-croquis-cf-uncaught-error.md#悪い) · [良い例](./project/vize-croquis-cf-uncaught-error.md#良い) | CLI |
| [`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | [悪い例](./project/vize-croquis-cf-undeclared-emit.md#悪い) · [良い例](./project/vize-croquis-cf-undeclared-emit.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | [悪い例](./project/vize-croquis-cf-undeclared-prop.md#悪い) · [良い例](./project/vize-croquis-cf-undeclared-prop.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | [悪い例](./project/vize-croquis-cf-undefined-slot.md#悪い) · [良い例](./project/vize-croquis-cf-undefined-slot.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | [悪い例](./project/vize-croquis-cf-unhandled-event.md#悪い) · [良い例](./project/vize-croquis-cf-unhandled-event.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | [悪い例](./project/vize-croquis-cf-unmatched-inject.md#悪い) · [良い例](./project/vize-croquis-cf-unmatched-inject.md#良い) | CLI |
| [`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | [悪い例](./project/vize-croquis-cf-unmatched-listener.md#悪い) · [良い例](./project/vize-croquis-cf-unmatched-listener.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | [悪い例](./project/vize-croquis-cf-unregistered-component.md#悪い) · [良い例](./project/vize-croquis-cf-unregistered-component.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | [悪い例](./project/vize-croquis-cf-unresolved-import.md#悪い) · [良い例](./project/vize-croquis-cf-unresolved-import.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | [悪い例](./project/vize-croquis-cf-unused-attrs.md#悪い) · [良い例](./project/vize-croquis-cf-unused-attrs.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | [悪い例](./project/vize-croquis-cf-unused-emit.md#悪い) · [良い例](./project/vize-croquis-cf-unused-emit.md#良い) | Rust analyzer。CLI は別の表示または未有効 |
| [`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | [悪い例](./project/vize-croquis-cf-unused-provide.md#悪い) · [良い例](./project/vize-croquis-cf-unused-provide.md#良い) | CLI |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | [悪い例](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md#悪い) · [良い例](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md#良い) | CLI |
| [`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | [悪い例](./project/vize-croquis-cf-watch-can-be-computed.md#悪い) · [良い例](./project/vize-croquis-cf-watch-can-be-computed.md#良い) | 契約のみ。現在の生成元なし |
| [`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | [悪い例](./project/vize-croquis-cf-watcheffect-async.md#悪い) · [良い例](./project/vize-croquis-cf-watcheffect-async.md#良い) | CLI |
| [`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | [悪い例](./project/vize-croquis-cf-watcher-outside-setup.md#悪い) · [良い例](./project/vize-croquis-cf-watcher-outside-setup.md#良い) | 契約のみ。現在の生成元なし |
