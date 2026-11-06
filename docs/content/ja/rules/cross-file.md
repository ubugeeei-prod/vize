---
title: "ファイル間ルール"
description: "複数のファイルを見る必要がある検査です。親子コンポーネント、provide と inject、ルーターとリンクの関係を確認します。"
---

# ファイル間ルール

複数のファイルを見る必要がある検査です。親子コンポーネント、provide と inject、ルーターとリンクの関係を確認します。

`withVue` に渡す `linter` に `crossFile: true` を指定するか、`vize lint --cross-file` を実行すると有効になります。診断は `vize:croquis/cf/<name>` と表示されます。`linter.rules` では `vize:` を除いた `croquis/cf/<name>` で設定します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

| ルール | 検出する内容 | 対応状況 |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | 対象のルートが宣言していないナビゲーションパラメーターを報告します。 | `vp run lint` で診断 |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | 必須のルートパラメーターを省略した名前付き遷移を報告します。 | `vp run lint` で診断 |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | 宣言されたパスが受け付けないルートパラメーター値を報告します。 | `vp run lint` で診断 |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | インストール済みのルーターにない遷移名を報告します。 | `vp run lint` で診断 |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | import した子コンポーネントの要素を合成して HTML の入れ子を検査します。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | Vue 2.7 の配列のインデックス更新に、通知される操作を使います。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | reactive な状態が async 境界を越え、古い値が見えることがあります。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | async コンポーネントが Suspense 境界なしで描画されています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | サーバーでも描画され得る場所で、ブラウザ専用 API を使っています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | コンポーネントの import が循環しています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | reactive な計算が互いに循環して依存しています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | クロージャが reactive な値を捕捉しており、その後の更新が見えません。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | composable が `setup` の外で呼ばれています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | computed の getter が状態を書いたり、他の副作用を起こしています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | import の連鎖がプロジェクトの許す深さより深くなっています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | reactive オブジェクトの分割代入はフィールドをコピーし、追跡を失います。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` または `inject` が `setup` の外で呼ばれています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | Vue が更新を反映する前に DOM を読んでいます。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | 同じ要素 id が複数のコンポーネントで使われています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | イベントリスナーが登録されたまま、解除されていません。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | emit が対応しない修飾子を、イベントリスナーが使っています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | サーバーとクライアントで描画結果が食い違う可能性があります。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | `inheritAttrs: false` が設定されているのに、属性が読まれていません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` が `InjectionKey` のシンボルではなく素のキーを使っています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | inject した値が、競合し得る async タスクから変更されています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | ライフサイクルフックが `setup` の外で登録されています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | ライフサイクルフックが後片付けなしで処理を始めています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | 必須 prop が渡されていません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | async な依存が Suspense 境界の外で使われています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | reactive な状態がモジュールスコープで作られ、すべての呼び出し元で共有されます。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | 複数ルートのコンポーネントが属性を受け取りますが、付ける場所がありません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | reactive オブジェクトが、所有者から漏れた後に変更されています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | provide した値が reactive ではないため、子孫は更新を観測できません。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | ループ内の要素 id が項目ごとに一意ではありません。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | オブジェクトと Proxy が表すレコードの同一性を、安定した ID で比較します。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | setup で Pinia の getter を更新に追従しない値へコピーする操作を避けます。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | 渡した prop の値が、宣言された型と一致しません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | provide した値と inject の型が一致しません。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` が `InjectionKey` のシンボルではなく素のキーを使っています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | reactive な状態がモジュールから export されています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | reactive API が `setup` の外で呼ばれています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | reactive な束縛を再代入すると、ただの値に置き換わります。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | reactive な参照が、寿命を持つスコープの外へ漏れています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | インスタンスごとに持つリアクティブな状態を setup 内で作ります。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | `shallowReactive` または `shallowRef` の深いプロパティを、追跡されるものとして読んでいます。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | reactive オブジェクトのスプレッドは値をコピーし、追跡を失います。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` に fallback がありません。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | テンプレート ref が、コンポーネントのマウント前に読まれています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | `toRaw` した生のオブジェクトを変更しています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | コンポーネントが例外を投げても、受け止めるエラー境界がありません。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | 宣言されていないイベントを emit しています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | 親が、子が宣言していない prop を渡しています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | 親が、子が公開していないスロットを埋めています。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | 子が emit したイベントを、どの親も処理していません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` が、どの祖先も provide していないキーを指定しています。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | 親が、子が emit しないイベントを購読しています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | テンプレートが、登録も import もされていないコンポーネントを使っています。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | import がモジュールへ解決できません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | 複数ルートのコンポーネントに渡した fallthrough 属性が使われていません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | 宣言した emit が一度も使われていません。 | 実験的。CLI ではまだ診断されません |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | provide したキーが一度も inject されていません。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | reactive な値をローカルへ取り出すと、その後の更新が失われます。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | ウォッチャが値を状態へ写しているだけで、computed にできます。 | 予定。まだ診断されません |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | 非同期の watchEffect で無効になった処理を中止し、古い応答を反映しないようにします。 | `vp run lint` で診断 |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` または `watchEffect` が `setup` の外で呼ばれています。 | 予定。まだ診断されません |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | 属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。 | `vp run lint` で診断 |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
