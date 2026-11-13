---
title: "Vue ルール"
description: "SFC のテンプレート・ディレクティブ・コンポーネント・各ブロックについて、正しさ・書き方の統一・Vue 3 への移行を検査します。"
---

# Vue ルール

SFC のテンプレート・ディレクティブ・コンポーネント・各ブロックについて、正しさ・書き方の統一・Vue 3 への移行を検査します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 基本 (53)

`essential` プリセットに含まれる、正しさを守るルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | テンプレート内の診断抑制コメントを扱い、設定に応じて未使用のコメントを検出します。 |  |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | JSX のタグで参照する変数を、コンポーネントの使用として扱います。 |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | コンポーネント名を複数の単語で構成します。 |  |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Vue 2 のパイプによるフィルター構文を検出します。 |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | SFC の template で削除済みの functional 属性を検出します。 |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | 削除済みの inline-template 属性を検出します。 |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | router-link の削除済み tag prop を検出します。 |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | template の削除済み scope 属性を検出します。 |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | 削除済みの slot 属性を検出します。 |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | 削除済みの slot-scope 属性を検出します。 |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | 削除済みの v-bind の .sync 修飾子を検出します。 |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | 非推奨の v-is ディレクティブを検出します。 |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | 削除済みの v-on の .native 修飾子を検出します。 |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | v-on の削除済みの数値 keyCode 修飾子を検出します。 | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | v-if / v-else-if の条件重複を検出します。 |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | 同じ要素の属性重複を検出します。 |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | 予約済みのコンポーネント名を検出します。 |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | v-for 用ではない template の key 指定を検出します。 |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | 危険なスキームを含む可能性がある URL 属性やバインディングを検出します。 |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | 同じ要素での v-if と v-for の併用を検出します。 |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | template 要素の効果がない属性を検出します。 |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | template v-for の key を子ではなく template に指定します。 |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | XSS を防ぐため、v-html を使用した箇所に警告を出します。 |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | コンポーネントでの v-text / v-html を検出します。 |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | 動的 component 要素に :is を指定します。 |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | transition の子要素に表示を切り替える条件を指定します。 |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | v-for に安定した :key を指定します。 |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | 修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。 |  |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | 有効な属性名を指定します。 |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。 |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | v-bind の引数・値・修飾子を検査します。 |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | v-cloak の引数・値・修飾子を検査します。 |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | v-else の位置・引数・値を検査します。 | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | v-else-if の位置・引数・値を検査します。 |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | v-for の式と変数宣言を検査します。 |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | v-html の値・引数・修飾子を検査します。 |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | v-if に有効な条件式を指定します。 |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | v-is の適用先・引数・値を検査します。 |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | v-memo の値を配列の式にします。 |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | v-model の値・引数・修飾子を検査します。 |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | v-on のイベント名・式・修飾子を検査します。 |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | v-once の引数・値・修飾子を検査します。 |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | v-pre の引数・値・修飾子を検査します。 |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | v-show に有効な条件式を指定します。 |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | v-slot の適用先・宣言・修飾子を検査します。 |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | v-text の値・引数・修飾子を検査します。 |  |

## 既定 (27)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | コンポーネントの prop 属性名を設定した形式に揃えます。 | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | テンプレートの属性を一定の順に並べます。 |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | HTML で省略できる終了タグも明示するよう求めます。 | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | HTML 属性値の引用符を揃えます。 | ✓ |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | mustache 内の空白を揃えます。 | ✓ |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | 不要な template 要素を検出します。 |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | 連続する不要な空白を検出します。 | ✓ |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | テンプレート変数が外側の名前を隠す箇所を検出します。 |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | target=_blank の外部リンクに適切な rel を指定します。 |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | beforeRouteEnter で未生成のインスタンスを this で参照する箇所を検出します。 |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | テンプレート内の未定義変数参照を検出します。 |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | iframe に sandbox 属性を指定します。 |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | defineProps に宣言しているのに使わない prop を検出します。 |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | script setup に宣言しているのに読み取らない変数を検出します。 |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | HTML の要素ごとのコンテンツモデルを検査します。 |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | 宣言する prop 名の形式を揃えます。 |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | style に scoped を指定する方針を適用します。 |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | SFC のトップレベルブロックを設定した順に並べます。 |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | SFC の style を一つのブロックにまとめます。 |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | テンプレートで不要な this. 参照を検出します。 |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | v-bind の表記形式を揃えます。 | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 | ✓ |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | v-on の表記形式を揃えます。 | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | v-slot の表記形式を揃えます。 | ✓ |

## より厳しいプリセット (29)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | テンプレート内のコンポーネント名を指定した形式に揃えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | button に有効な type を明示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | 要素の種類ごとに自己終了タグの形式を揃えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | HTML の真偽属性に不要な値を指定した箇所を検出します。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | 空の SFC ブロックを検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | インラインの style 属性を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | :class 配列内の複数のオブジェクト指定をまとめます。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | v-else がある条件分岐の否定条件を反転して読みやすくします。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | CSS プリプロセッサーより標準の CSS を使う方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | script の非標準の lang 指定を避ける方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | SFC ブロックの外部 src 指定を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | template の lang 指定を避ける方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | テンプレートに宣言しているのに参照しない ref を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-script-vars"></span>[`vue/no-unused-script-vars`](./reference/vue-no-unused-script-vars.md) | スクリプトで宣言しているのに使わない変数、インポート、引数を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | 文字列リテラルだけの不要な mustache を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | 文字列リテラルだけの不要な v-bind を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | v-text の代わりに mustache を使う方針を適用します。 | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-const"></span>[`vue/prefer-const`](./reference/vue-prefer-const.md) | スクリプトとテンプレートが再代入しないバインディングを const にします。 | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Vue 3.4 の同名 prop バインディングの省略形を使います。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | true を指定するバインディングを真偽属性の省略形にします。 | `nuxt`, `opinionated` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | 使用するコンポーネントを import または登録します。 | `opinionated` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | イベント名を context:event の形式に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | 描画する slot outlet の静的な名前を設定した形式に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 静的 ID の代わりに useId() で一意な ID を生成します。 | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | イベントハンドラーをメソッド参照の形式に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | SFC のカスタムブロックを検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | 登録が必要なカスタムディレクティブを検出します。 | `nuxt`, `opinionated` |  |

## 個別に有効化 (60)

どのプリセットにも含まれません。`withVue` に渡す `linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="max-lines"></span>[`max-lines`](./reference/max-lines.md) | ファイルの行数が上限を超えた箇所を検出します。 |  |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | 画像に代替テキストの alt 属性を指定します。 |  |
| <span id="vue-block-lang"></span>[`vue/block-lang`](./reference/vue-block-lang.md) | SFC ブロックの lang を設定した言語に揃えます。 |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | テンプレート式の識別子を camelCase に揃えます。 |  |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | テンプレート式で静的なプロパティ参照にドット記法を使います。 |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | style ブロックを許可した属性モードに揃えます。 |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | テンプレート式で厳密等価演算子を使います。 |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | 宣言したコンポーネント名をファイル名と一致させます。 |  |
| <span id="vue-max-lines-per-block"></span>[`vue/max-lines-per-block`](./reference/vue-max-lines-per-block.md) | SFC ブロックの行数が上限を超えた箇所を検出します。 |  |
| <span id="vue-max-props"></span>[`vue/max-props`](./reference/vue-max-props.md) | Vue コンポーネントの props 数が上限を超えた箇所を検出します。 |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | コンポーネント自身のテンプレートの複雑度を制限します。 |  |
| <span id="vue-max-template-depth"></span>[`vue/max-template-depth`](./reference/vue-max-template-depth.md) | テンプレートの入れ子が上限を超えた箇所を検出します。 |  |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | nextTick を Promise 形式で使います。 |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | テンプレートに直接書かれた、国際化の対象となるテキストを検出します。 |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | テンプレート式から console を参照する箇所を検出します。 |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | テンプレート式の定数条件を検出します。 |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | SFC のトップレベルブロック外にある内容を検出します。 |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Vue 2 の互換性のため、コンポーネントの v-model に独自の修飾子を使う箇所を検出します。 |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | defineModel に既定値を指定する箇所を検出します。 |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | テンプレート式の空の分割代入パターンを検出します。 |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | テンプレート式の暗黙的な型変換を検出します。 |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | テンプレート式の不規則な空白文字を検出します。 |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | v-bind のオブジェクト・配列・関数リテラルを検出します。 |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | 精度を失う数値リテラルを検出します。 |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | 単一ルートを要求するテンプレートで複数ルートを検出します。 |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | 否定から始まる三項条件を検出します。 |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | script setup で prop と同名の宣言を検出します。 |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | 同じスコープで ref の値をコピーし、更新に追従しなくなる箇所を検出します。 |  |
| <span id="vue-no-restricted-block"></span>[`vue/no-restricted-block`](./reference/vue-no-restricted-block.md) | 設定したブロックを使う箇所を検出します。 |  |
| <span id="vue-no-restricted-call-after-await"></span>[`vue/no-restricted-call-after-await`](./reference/vue-no-restricted-call-after-await.md) | await の後に制限されたメソッドを呼ぶ箇所を検出します。 |  |
| <span id="vue-no-restricted-class"></span>[`vue/no-restricted-class`](./reference/vue-no-restricted-class.md) | 設定した class を使う箇所を検出します。 |  |
| <span id="vue-no-restricted-component-names"></span>[`vue/no-restricted-component-names`](./reference/vue-no-restricted-component-names.md) | 設定したコンポーネント名を使う箇所を検出します。 |  |
| <span id="vue-no-restricted-component-options"></span>[`vue/no-restricted-component-options`](./reference/vue-no-restricted-component-options.md) | 設定したコンポーネントオプションを使う箇所を検出します。 |  |
| <span id="vue-no-restricted-custom-event"></span>[`vue/no-restricted-custom-event`](./reference/vue-no-restricted-custom-event.md) | 設定したカスタムイベントを使う箇所を検出します。 |  |
| <span id="vue-no-restricted-html-elements"></span>[`vue/no-restricted-html-elements`](./reference/vue-no-restricted-html-elements.md) | 設定した要素を使う箇所を検出します。 |  |
| <span id="vue-no-restricted-props"></span>[`vue/no-restricted-props`](./reference/vue-no-restricted-props.md) | 設定した props を使う箇所を検出します。 |  |
| <span id="vue-no-restricted-static-attribute"></span>[`vue/no-restricted-static-attribute`](./reference/vue-no-restricted-static-attribute.md) | 設定した属性を使う箇所を検出します。 |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | 設定した構文をテンプレート式で使う箇所を検出します。 |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | 制限対象の v-bind 引数を検出します。 |  |
| <span id="vue-no-restricted-v-on"></span>[`vue/no-restricted-v-on`](./reference/vue-no-restricted-v-on.md) | 設定した v-on 引数を使う箇所を検出します。 |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | setup の props をコピーし、更新に追従しなくなる箇所を検出します。 |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | ネイティブイベントと衝突するコンポーネントイベント名を検出します。 |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | テンプレート式の疎な配列を検出します。 |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | 静的なインライン style 属性を検出します。 |  |
| <span id="vue-no-unsupported-features"></span>[`vue/no-unsupported-features`](./reference/vue-no-unsupported-features.md) | 指定した Vue.js の版で未対応の構文を検出します。 |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | 文字列リテラル同士の不要な連結を検出します。 |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Vue 2 の互換性のため、template v-for の key を実体のある子に指定します。 |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | テンプレート式のオブジェクトに短縮プロパティ記法を使います。 |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | prop の型コンストラクター配列で Boolean を先頭に置きます。 |  |
| <span id="vue-prefer-single-event-payload"></span>[`vue/prefer-single-event-payload`](./reference/vue-prefer-single-event-payload.md) | カスタムイベントのペイロードを1つにまとめます。 |  |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | 文字列連結の代わりにテンプレートリテラルを使います。 |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | 対応する bind と update リスナーを v-model にまとめます。 |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | 不要な引用符付きオブジェクトプロパティを検出します。 |  |
| <span id="vue-require-default-export"></span>[`vue/require-default-export`](./reference/vue-require-default-export.md) | コンポーネントの script をデフォルトエクスポートにします。 |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Options API のコンポーネントが公開するプロパティを expose で明示します。 |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | コンパイラーマクロの変数名を既定の名前に揃えます。 |  |
| <span id="vue-require-prop-comment"></span>[`vue/require-prop-comment`](./reference/vue-require-prop-comment.md) | 宣言した各 prop に JSDoc コメントを付けます。 |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | コンポーネントオプション内のキーを設定した順序に並べます。 |  |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | 条件分岐で繰り返すコンポーネントに key を指定します。 |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Vue 2 の model オプションに有効なキーを指定します。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
