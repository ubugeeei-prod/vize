---
title: "全 lint ルール"
description: "Vize のリンターに含まれるすべてのルールを、カテゴリごとにまとめています。各ルールのページで、検出内容・悪い例と良い例・有効にする設定を確認できます。"
---

# 全 lint ルール

Vize のリンターに含まれるすべてのルールを、カテゴリごとにまとめています。各ルールのページで、検出内容・悪い例と良い例・有効にする設定を確認できます。

プリセット列は、そのルールを有効にする最小のプリセットです。すべてのプリセットは各ルールのページに載っています。各プリセットの内容は[プリセット](../rules/index.md#プリセット)を参照してください。

## Vue ルール (120)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | 画像に代替テキストの alt 属性を指定します。 | — |  |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | コンポーネントの prop 属性名を設定した形式に揃えます。 | `happy-path` | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | テンプレートの属性を一定の順に並べます。 | `happy-path` |  |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | テンプレート内の診断抑制コメントを扱い、設定に応じて未使用のコメントを検出します。 | `essential` |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 | `happy-path` |  |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | テンプレート内のコンポーネント名を指定した形式に揃えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | button に有効な type を明示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | HTML で省略できる終了タグも明示するよう求めます。 | `happy-path` | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | HTML 属性値の引用符を揃えます。 | `happy-path` | ✓ |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | 要素の種類ごとに自己終了タグの形式を揃えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | JSX のタグで参照する変数を、コンポーネントの使用として扱います。 | `essential` |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | 宣言したコンポーネント名をファイル名と一致させます。 | — |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | コンポーネント自身のテンプレートの複雑度を制限します。 | — |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | コンポーネント名を複数の単語で構成します。 | `essential` |  |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | mustache 内の空白を揃えます。 | `happy-path` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | テンプレートに直接書かれた、国際化の対象となるテキストを検出します。 | — |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | HTML の真偽属性に不要な値を指定した箇所を検出します。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 | `essential` |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Vue 2 の互換性のため、コンポーネントの v-model に独自の修飾子を使う箇所を検出します。 | — |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Vue 2 のパイプによるフィルター構文を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | SFC の template で削除済みの functional 属性を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | 削除済みの inline-template 属性を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | router-link の削除済み tag prop を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | template の削除済み scope 属性を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | 削除済みの slot 属性を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | 削除済みの slot-scope 属性を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | 削除済みの v-bind の .sync 修飾子を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | 非推奨の v-is ディレクティブを検出します。 | `essential` |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | 削除済みの v-on の .native 修飾子を検出します。 | `essential` |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | v-on の削除済みの数値 keyCode 修飾子を検出します。 | `essential` | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | v-if / v-else-if の条件重複を検出します。 | `essential` |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | 同じ要素の属性重複を検出します。 | `essential` |  |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | 空の SFC ブロックを検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | インラインの style 属性を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 | `happy-path` |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | 不要な template 要素を検出します。 | `happy-path` |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | 連続する不要な空白を検出します。 | `happy-path` | ✓ |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | :class 配列内の複数のオブジェクト指定をまとめます。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | 単一ルートを要求するテンプレートで複数ルートを検出します。 | — |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 | `essential` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | v-else がある条件分岐の否定条件を反転して読みやすくします。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 | `essential` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | CSS プリプロセッサーより標準の CSS を使う方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | 同じスコープで ref の値をコピーし、更新に追従しなくなる箇所を検出します。 | — |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | 予約済みのコンポーネント名を検出します。 | `essential` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | script の非標準の lang 指定を避ける方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | setup の props をコピーし、更新に追従しなくなる箇所を検出します。 | — |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | SFC ブロックの外部 src 指定を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | 静的なインライン style 属性を検出します。 | — |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | v-for 用ではない template の key 指定を検出します。 | `essential` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | template の lang 指定を避ける方針を示します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | テンプレート変数が外側の名前を隠す箇所を検出します。 | `happy-path` |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | target=_blank の外部リンクに適切な rel を指定します。 | `happy-path` |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | textarea 内の mustache を検出し、v-model の使用を勧めます。 | `essential` |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | beforeRouteEnter で未生成のインスタンスを this で参照する箇所を検出します。 | — |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | テンプレート内の未定義変数参照を検出します。 | — |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | 危険なスキームを含む可能性がある URL 属性やバインディングを検出します。 | `essential` |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | iframe に sandbox 属性を指定します。 | `happy-path` |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 | `essential` |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | defineProps に宣言しているのに使わない prop を検出します。 | `happy-path` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | テンプレートに宣言しているのに参照しない ref を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | script setup に宣言しているのに読み取らない変数を検出します。 | — |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | v-for / v-slot に宣言しているのに使わない変数を検出します。 | `essential` |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 | `happy-path` |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | 同じ要素での v-if と v-for の併用を検出します。 | `essential` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | 文字列リテラルだけの不要な mustache を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | template 要素の効果がない属性を検出します。 | `essential` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | 文字列リテラルだけの不要な v-bind を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Vue 2 の互換性のため、template v-for の key を実体のある子に指定します。 | — |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | template v-for の key を子ではなく template に指定します。 | `essential` |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | XSS を防ぐため、v-html を使用した箇所に警告を出します。 | `essential` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | v-text の代わりに mustache を使う方針を適用します。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | コンポーネントでの v-text / v-html を検出します。 | `essential` |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | HTML の要素ごとのコンテンツモデルを検査します。 | `happy-path` |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | prop の型コンストラクター配列で Boolean を先頭に置きます。 | — |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Vue 3.4 の同名 prop バインディングの省略形を使います。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | true を指定するバインディングを真偽属性の省略形にします。 | `nuxt`, `opinionated` |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | 宣言する prop 名の形式を揃えます。 | `happy-path` |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | 動的 component 要素に :is を指定します。 | `essential` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | 使用するコンポーネントを import または登録します。 | `opinionated` |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Options API のコンポーネントが公開するプロパティを expose で明示します。 | — |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | style に scoped を指定する方針を適用します。 | `happy-path` |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | transition の子要素に表示を切り替える条件を指定します。 | `essential` |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | v-for に安定した :key を指定します。 | `essential` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | イベント名を context:event の形式に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | SFC のトップレベルブロックを設定した順に並べます。 | `happy-path` |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | SFC の style を一つのブロックにまとめます。 | `happy-path` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | 名前付きスロットを kebab-case に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | テンプレートで不要な this. 参照を検出します。 | `happy-path` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 静的 ID の代わりに useId() で一意な ID を生成します。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | 修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。 | `essential` |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | v-bind の表記形式を揃えます。 | `happy-path` | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 | `happy-path` | ✓ |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | イベントハンドラーの書き方を参照または関数の形式に揃えます。 | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | v-on の表記形式を揃えます。 | `happy-path` | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | v-slot の表記形式を揃えます。 | `happy-path` | ✓ |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | 有効な属性名を指定します。 | `essential` |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Vue 2 の model オプションに有効なキーを指定します。 | — |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。 | `essential` |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | v-bind の引数・値・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | v-cloak の引数・値・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | v-else の位置・引数・値を検査します。 | `essential` | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | v-else-if の位置・引数・値を検査します。 | `essential` |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | v-for の式と変数宣言を検査します。 | `essential` |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | v-html の値・引数・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | v-if に有効な条件式を指定します。 | `essential` |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | v-is の適用先・引数・値を検査します。 | `essential` |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | v-memo の値を配列の式にします。 | `essential` |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | v-model の値・引数・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | v-on のイベント名・式・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | v-once の引数・値・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | v-pre の引数・値・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | v-show に有効な条件式を指定します。 | `essential` |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | v-slot の適用先・宣言・修飾子を検査します。 | `essential` |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | v-text の値・引数・修飾子を検査します。 | `essential` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | SFC のカスタムブロックを検出します。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | 登録が必要なカスタムディレクティブを検出します。 | `nuxt`, `opinionated` |  |

## 型と script のルール (82)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | コンポーネントの name オプションを PascalCase に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | emit するカスタムイベント名を指定した形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | defineEmits を型による宣言形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | script setup のコンパイラーマクロを一定の順に宣言します。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | defineProps を型による宣言形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | defineProps の分割代入スタイルを指定した方針に揃えます。 | — |  |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Options API の watch ハンドラーにアロー関数を使う箇所を検出します。 | `essential` |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | computed の getter で非同期関数を使う箇所を検出します。 | `essential` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Boolean prop の冗長な default を検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | 初期化前の data() で算出プロパティを読む箇所を検出します。 | `essential` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | defineProps の深い分割代入を検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 | `essential` |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Vue 3 で削除された $set / $delete を検出します。 | `essential` |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。 | `essential` | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Vue 3 で $attrs に統合された $listeners を検出します。 | `essential` |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Vue 3 で $slots に統合された $scopedSlots を検出します。 | `essential` |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Vue 3 で削除された $on / $off / $once を検出します。 | `essential` |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Vue 3 で削除された model オプションを検出します。 | `essential` |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | prop の default / validator 内で使えなくなった this を検出します。 | `essential` |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | 削除された Vue.config.keyCodes を検出します。 | `essential` |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Options API の props / data / computed などのキー重複を検出します。 | `essential` |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 | `essential` |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | script setup 内の export 文を検出します。 | `essential` |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | await の後の expose() と defineExpose() を禁止します。 | `essential` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Vapor で null を返す getCurrentInstance() を検出します。 | `opinionated` |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | 自動的に使える Vue コンパイラーマクロの import を検出します。 | `essential` |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Vue 内部モジュールからの import を検出します。 | `essential` |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | setup() 内で await の後にライフサイクルフックを登録することを禁止します。 | `essential` |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | scoped slot 関数に複数の引数を渡す箇所を検出します。 | `happy-path` |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 | — |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Vapor で Options API を使用する箇所を検出します。 | `opinionated` |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Options API のオプション名の入力ミスを検出します。 | `essential` |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | reactive オブジェクトを分割代入し、リアクティビティを失う箇所を検出します。 | — |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | ref を演算の値として使う際に .value を参照します。 | `essential` |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | required: true と default を同時に持つ prop を検出します。 | `essential` |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Vue コンパイラーが予約した識別子の宣言を検出します。 | `essential` |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Options API のキーに Vue の予約名を使う箇所を検出します。 | `essential` |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | prop 宣言に Vue の予約名を使う箇所を検出します。 | `essential` |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | 設定で禁止した実行環境のグローバル参照を検出します。 | — |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | 設定で禁止した object.property へのアクセスを検出します。 | — |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Options API の computed getter 内の副作用を検出します。 | `essential` |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 | — |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 | `happy-path` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | 宣言したまま emit していないイベントを検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 | `essential` |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。 | `essential` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | 1 ファイルにコンポーネント定義が複数あることを検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | コンポーネントオプションのプロパティ順を既定の並びに揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 | — |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | template の $emit ではなく defineEmits から受け取った型付き関数を呼び出します。 | — | ✓ |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | 内部パッケージではなく vue から import します。 | `essential` | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | 状態管理に reactive() より ref() を使う方針を適用します。 | — |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | setup の context.attrs を useAttrs() に置き換えます。 | — |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | 一意な ID の生成に Vue 3.5 の useId() を使います。 | — |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | setup の context.slots を useSlots() に置き換えます。 | — |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | 省略可能な、Boolean 以外の prop に default を用意します。 | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | emit するイベントを defineEmits または emits に宣言します。 | `happy-path` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | useSlots() で使う slot を defineSlots の型で宣言します。 | `opinionated`, `nuxt` |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | 関数に戻り値の型注釈を指定します。 | — |  |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | prop の type に文字列ではなくコンストラクターを指定します。 | `essential` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | 各 prop の型を宣言します。 | `opinionated` |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | render 関数が値を返すことを必須にします。 | `essential` |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Vue 3 の $slots プロパティを関数として呼び出します。 | `essential` |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | provide / inject のキーに衝突しにくい Symbol を使います。 | — |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Object / Array の prop に具体的な型を指定します。 | `opinionated`, `nuxt` |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | 引数なし、null、undefined で呼び出す ref() に型引数を指定します。 | — |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | prop の default を宣言した型に合う値にします。 | `essential` |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | computed の getter に値を返す return を用意します。 | `essential` |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Options API の emits validator に戻り値を用意します。 | `essential` |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | defineEmits の重複や型と実行時引数の併用を検出します。 | `essential` |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | defineOptions の引数と使用回数を検査します。 | `essential` |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | defineProps の重複や型と実行時引数の併用を検出します。 | `essential` |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | nextTick() の完了を await、then、またはコールバックで扱います。 | `essential` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | 処理しないまま放置された Promise を検出します。 | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | リアクティブな値を代入や呼び出しで取り出し、元の更新に追従しないスナップショットになる箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | テンプレートで安全でない型の値を使用する箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | defineEmits に型定義を指定します。 | `happy-path` |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | defineProps に型定義を指定します。 | `happy-path` |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | script とテンプレートの条件式で安全な真偽判定を使います。 | — |  |

## HTML ルール (9)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | 非推奨の HTML 属性を検出します。 | `happy-path` |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | 非推奨の HTML 要素を検出します。 | `happy-path` |  |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | 同じテンプレート内の ID 重複を検出します。 | `essential` |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | 連続する br 要素による余白指定を検出します。 | `happy-path` |  |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | 静的 style 属性内のプロパティ重複を検出します。 | `nuxt`, `opinionated` |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | 静的 class 属性内のクラス名重複を検出します。 | `happy-path` |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | dl 内の dt の名前重複を検出します。 | `happy-path` |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | 可視コンテンツを期待する要素が空の場合に検出します。 | `happy-path` |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | time 要素に機械可読の datetime を指定します。 | `happy-path` |  |

## アクセシビリティのルール (30)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | 画像などのメディアに代替テキストを用意します。 | `happy-path` |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | リンクに支援技術で読める内容を用意します。 | `happy-path` |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | リンクの href に有効な移動先を指定します。 | `happy-path` |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | 存在しない ARIA 属性を検出します。 | `happy-path` |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | 抽象ロールを除く、有効な ARIA role を指定します。 | `happy-path` |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | ARIA 属性を使用できない要素への指定を検出します。 | `happy-path` |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | クリックで操作する要素にキーボード操作も用意します。 | `happy-path` |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | フォーム部品に関連付けられたラベルを用意します。 | `happy-path` |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | 見出しに支援技術で読める内容を用意します。 | `happy-path` |  |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | 見出しの階層を飛ばした指定を検出します。 | `nuxt`, `opinionated` |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | iframe に内容を説明する title を指定します。 | `happy-path` |  |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 | — |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | 操作可能な role の要素をフォーカス可能にします。 | `happy-path` |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | label を対象のフォーム部品と関連付けます。 | `happy-path` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | ランドマーク role の配置と重複を検査します。 | `nuxt`, `opinionated` |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | 音声・動画に字幕を用意します。 | `happy-path` |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | マウス操作と対応する focus / blur 操作を用意します。 | `happy-path` |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | 環境のショートカットと衝突し得る accesskey を検出します。 | `happy-path` |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。 | `happy-path` |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | 意図せずフォーカスを移動させる autofocus を検出します。 | `happy-path` |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | marquee や blink などの注意をそらす要素を検出します。 | `happy-path` |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 | `happy-path` |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | 要素本来の意味と重複する ARIA role を検出します。 | `happy-path` | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | 文書内に存在しない ID への参照を検出します。 | `happy-path` |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。 | `happy-path` |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | 操作部品ではない要素へのイベント指定を検出します。 | `happy-path` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | required の select を値が空のプレースホルダー option から始めます。 | `nuxt`, `opinionated` |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | ARIA role が必要とする属性を指定します。 | `happy-path` |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | 通常のフォーカス順序を変える正の tabindex を検出します。 | `happy-path` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | 箇条書きに見えるテキストをリスト要素で表現します。 | `nuxt`, `opinionated` |  |

## SSR ルール (2)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 | `happy-path` |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | サーバーとクライアントで一致しないテンプレート値を検出します。 | `happy-path` |  |

## Vapor ルール (4)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Vapor で削除済みの inline-template 属性を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。 | `happy-path` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | 文字列リテラルの :class を静的 class に置き換えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Vapor 向けの script setup に vapor 属性を付ける規約を示します。 | `nuxt`, `opinionated` |  |

## エコシステムと Nuxt のルール (13)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Nuxt の内部リンクに NuxtLink を使います。 | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Pinia store の分割代入に storeToRefs() を使います。 | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | RouterLink / NuxtLink に to を指定します。 | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Void Vue の Link に href を指定します。 | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Void Vue の Link に有効な静的 method を指定します。 | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | SFC 内の翻訳データに存在しない静的キーを検出します。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Vue Router のプログラムによる移動に名前付きルートを使います。 | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | wrapper.html() 全体のスナップショットに依存するテストを検出します。 | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。 | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Nuxt 設定のプロパティを推奨順に並べます。 | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 | `nuxt` | ✓ |

## Musea と CSS のルール (16)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | 表示切り替えに display: none を使う箇所で v-show を検討します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | CSS に直接記述した値を CSS 変数にまとめます。 | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | 詳細度が高い CSS の ID セレクターを検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | 通常のカスケードを上書きする !important を検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | コンポーネント内でユーティリティクラスを定義する箇所を検出します。 | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | CSS v-bind() の実行時コストを検討するための警告です。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | 書字方向に対応する CSS の論理プロパティを使います。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | 子孫セレクターを CSS のネスト構文でまとめます。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | @font-face に font-display を指定します。 | `opinionated`, `nuxt` |  |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | 内容のない variant ブロックを検出します。 | — |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。 | — |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | art ブロックに対象の component を指定します。 | — |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | art ブロックに title を指定します。 | — |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | 同じ Art ファイル内の variant 名を一意にします。 | — |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | variant ブロックに name を指定します。 | — |  |

## petite-vue ルール (3)

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | petite-vue が対応しないディレクティブを検出します。 | — |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | v-effect に空でない式を指定します。 | — |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | v-scope に値を指定する場合は、オブジェクトリテラルにします。 | — |  |

## ファイル間ルール (66)

`vue.linter` に `crossFile: true` を指定するか、`vize lint --cross-file` を実行すると有効になります。診断は `vize:croquis/cf/<name>` と表示されます。`vue.linter.rules` では `vize:` を除いた `croquis/cf/<name>` で設定します。

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
