---
title: Vue ルール
---

# Vue ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | [悪い例](./reference/vue-a11y-img-alt.md#悪い) · [良い例](./reference/vue-a11y-img-alt.md#良い) | 画像に代替テキストの alt 属性を指定します。 |
| [`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | [悪い例](./reference/vue-attribute-hyphenation.md#悪い) · [良い例](./reference/vue-attribute-hyphenation.md#良い) | コンポーネントの prop 属性名を設定した形式に揃えます。 |
| [`vue/attribute-order`](./reference/vue-attribute-order.md) | [悪い例](./reference/vue-attribute-order.md#悪い) · [良い例](./reference/vue-attribute-order.md#良い) | テンプレートの属性を一定の順に並べます。 |
| [`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | [悪い例](./reference/vue-component-definition-name-casing.md#悪い) · [良い例](./reference/vue-component-definition-name-casing.md#良い) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |
| [`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | [悪い例](./reference/vue-component-name-in-template-casing.md#悪い) · [良い例](./reference/vue-component-name-in-template-casing.md#良い) | テンプレート内のコンポーネント名を指定した形式に揃えます。 |
| [`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | [悪い例](./reference/vue-html-button-has-type.md#悪い) · [良い例](./reference/vue-html-button-has-type.md#良い) | button に有効な type を明示します。 |
| [`vue/html-quotes`](./reference/vue-html-quotes.md) | [悪い例](./reference/vue-html-quotes.md#悪い) · [良い例](./reference/vue-html-quotes.md#良い) | HTML 属性値の引用符を揃えます。 |
| [`vue/html-self-closing`](./reference/vue-html-self-closing.md) | [悪い例](./reference/vue-html-self-closing.md#悪い) · [良い例](./reference/vue-html-self-closing.md#良い) | 要素の種類ごとに自己終了タグの形式を揃えます。 |
| [`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | [悪い例](./reference/vue-max-template-complexity.md#悪い) · [良い例](./reference/vue-max-template-complexity.md#良い) | コンポーネント自身のテンプレートの複雑度を制限します。 |
| [`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | [悪い例](./reference/vue-multi-word-component-names.md#悪い) · [良い例](./reference/vue-multi-word-component-names.md#良い) | コンポーネント名を複数の単語で構成します。 |
| [`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | [悪い例](./reference/vue-mustache-interpolation-spacing.md#悪い) · [良い例](./reference/vue-mustache-interpolation-spacing.md#良い) | mustache 内の空白を揃えます。 |
| [`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | [悪い例](./reference/vue-no-array-index-key.md#悪い) · [良い例](./reference/vue-no-array-index-key.md#良い) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 |
| [`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | [悪い例](./reference/vue-no-bare-strings-in-template.md#悪い) · [良い例](./reference/vue-no-bare-strings-in-template.md#良い) | 国際化すべきテンプレートの直接指定テキストを検出します。 |
| [`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | [悪い例](./reference/vue-no-boolean-attr-value.md#悪い) · [良い例](./reference/vue-no-boolean-attr-value.md#良い) | HTML の boolean 属性に不要な値を指定した箇所を検出します。 |
| [`vue/no-child-content`](./reference/vue-no-child-content.md) | [悪い例](./reference/vue-no-child-content.md#悪い) · [良い例](./reference/vue-no-child-content.md#良い) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |
| [`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | [悪い例](./reference/vue-no-deprecated-filter.md#悪い) · [良い例](./reference/vue-no-deprecated-filter.md#良い) | Vue 2 の pipe による filter 構文を検出します。 |
| [`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | [悪い例](./reference/vue-no-deprecated-functional-template.md#悪い) · [良い例](./reference/vue-no-deprecated-functional-template.md#良い) | SFC の template で削除済みの functional 属性を検出します。 |
| [`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | [悪い例](./reference/vue-no-deprecated-html-element-is.md#悪い) · [良い例](./reference/vue-no-deprecated-html-element-is.md#良い) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 |
| [`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | [悪い例](./reference/vue-no-deprecated-inline-template.md#悪い) · [良い例](./reference/vue-no-deprecated-inline-template.md#良い) | 削除済みの inline-template 属性を検出します。 |
| [`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | [悪い例](./reference/vue-no-deprecated-router-link-tag-prop.md#悪い) · [良い例](./reference/vue-no-deprecated-router-link-tag-prop.md#良い) | router-link の削除済み tag prop を検出します。 |
| [`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | [悪い例](./reference/vue-no-deprecated-scope-attribute.md#悪い) · [良い例](./reference/vue-no-deprecated-scope-attribute.md#良い) | template の削除済み scope 属性を検出します。 |
| [`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | [悪い例](./reference/vue-no-deprecated-slot-attribute.md#悪い) · [良い例](./reference/vue-no-deprecated-slot-attribute.md#良い) | 削除済みの slot 属性を検出します。 |
| [`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | [悪い例](./reference/vue-no-deprecated-slot-scope-attribute.md#悪い) · [良い例](./reference/vue-no-deprecated-slot-scope-attribute.md#良い) | 削除済みの slot-scope 属性を検出します。 |
| [`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | [悪い例](./reference/vue-no-deprecated-v-bind-sync.md#悪い) · [良い例](./reference/vue-no-deprecated-v-bind-sync.md#良い) | 削除済みの v-bind の .sync modifier を検出します。 |
| [`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | [悪い例](./reference/vue-no-deprecated-v-on-native-modifier.md#悪い) · [良い例](./reference/vue-no-deprecated-v-on-native-modifier.md#良い) | 削除済みの v-on の .native modifier を検出します。 |
| [`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | [悪い例](./reference/vue-no-deprecated-v-on-number-modifiers.md#悪い) · [良い例](./reference/vue-no-deprecated-v-on-number-modifiers.md#良い) | v-on の削除済み数値 keyCode modifier を検出します。 |
| [`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | [悪い例](./reference/vue-no-dupe-v-else-if.md#悪い) · [良い例](./reference/vue-no-dupe-v-else-if.md#良い) | v-if / v-else-if の条件重複を検出します。 |
| [`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | [悪い例](./reference/vue-no-duplicate-attributes.md#悪い) · [良い例](./reference/vue-no-duplicate-attributes.md#良い) | 同じ要素の属性重複を検出します。 |
| [`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | [悪い例](./reference/vue-no-empty-component-block.md#悪い) · [良い例](./reference/vue-no-empty-component-block.md#良い) | 空の SFC ブロックを検出します。 |
| [`vue/no-inline-style`](./reference/vue-no-inline-style.md) | [悪い例](./reference/vue-no-inline-style.md#悪い) · [良い例](./reference/vue-no-inline-style.md#良い) | インラインの style 属性を検出します。 |
| [`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | [悪い例](./reference/vue-no-invalid-html-attribute.md#悪い) · [良い例](./reference/vue-no-invalid-html-attribute.md#良い) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 |
| [`vue/no-lone-template`](./reference/vue-no-lone-template.md) | [悪い例](./reference/vue-no-lone-template.md#悪い) · [良い例](./reference/vue-no-lone-template.md#良い) | 不要な template 要素を検出します。 |
| [`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | [悪い例](./reference/vue-no-multi-spaces.md#悪い) · [良い例](./reference/vue-no-multi-spaces.md#良い) | 連続する不要な空白を検出します。 |
| [`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | [悪い例](./reference/vue-no-multiple-objects-in-class.md#悪い) · [良い例](./reference/vue-no-multiple-objects-in-class.md#良い) | :class 配列内の複数のオブジェクト指定をまとめます。 |
| [`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | [悪い例](./reference/vue-no-multiple-template-root.md#悪い) · [良い例](./reference/vue-no-multiple-template-root.md#良い) | 単一ルートを要求するテンプレートで複数ルートを検出します。 |
| [`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | [悪い例](./reference/vue-no-mutating-props.md#悪い) · [良い例](./reference/vue-no-mutating-props.md#良い) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |
| [`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | [悪い例](./reference/vue-no-negated-v-if-condition.md#悪い) · [良い例](./reference/vue-no-negated-v-if-condition.md#良い) | v-else がある条件分岐の否定条件を反転して読みやすくします。 |
| [`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | [悪い例](./reference/vue-no-non-component-keep-alive-child.md#悪い) · [良い例](./reference/vue-no-non-component-keep-alive-child.md#良い) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |
| [`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | [悪い例](./reference/vue-no-preprocessor-lang.md#悪い) · [良い例](./reference/vue-no-preprocessor-lang.md#良い) | CSS preprocessor より標準の CSS を使う方針を適用します。 |
| [`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | [悪い例](./reference/vue-no-reserved-component-names.md#悪い) · [良い例](./reference/vue-no-reserved-component-names.md#良い) | 予約済みのコンポーネント名を検出します。 |
| [`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | [悪い例](./reference/vue-no-root-v-if.md#悪い) · [良い例](./reference/vue-no-root-v-if.md#良い) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 |
| [`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | [悪い例](./reference/vue-no-script-non-standard-lang.md#悪い) · [良い例](./reference/vue-no-script-non-standard-lang.md#良い) | script の非標準 lang 指定を検出します。 |
| [`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | [悪い例](./reference/vue-no-src-attribute.md#悪い) · [良い例](./reference/vue-no-src-attribute.md#良い) | SFC ブロックの外部 src 指定を検出します。 |
| [`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | [悪い例](./reference/vue-no-static-inline-styles.md#悪い) · [良い例](./reference/vue-no-static-inline-styles.md#良い) | 静的なインライン style 属性を検出します。 |
| [`vue/no-template-key`](./reference/vue-no-template-key.md) | [悪い例](./reference/vue-no-template-key.md#悪い) · [良い例](./reference/vue-no-template-key.md#良い) | v-for 用ではない template の key 指定を検出します。 |
| [`vue/no-template-lang`](./reference/vue-no-template-lang.md) | [悪い例](./reference/vue-no-template-lang.md#悪い) · [良い例](./reference/vue-no-template-lang.md#良い) | template の lang 指定を検出します。 |
| [`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | [悪い例](./reference/vue-no-template-shadow.md#悪い) · [良い例](./reference/vue-no-template-shadow.md#良い) | テンプレート変数が外側の名前を隠す箇所を検出します。 |
| [`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | [悪い例](./reference/vue-no-template-target-blank.md#悪い) · [良い例](./reference/vue-no-template-target-blank.md#良い) | target=_blank の外部リンクに適切な rel を指定します。 |
| [`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | [悪い例](./reference/vue-no-textarea-mustache.md#悪い) · [良い例](./reference/vue-no-textarea-mustache.md#良い) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |
| [`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | [悪い例](./reference/vue-no-undefined-refs.md#悪い) · [良い例](./reference/vue-no-undefined-refs.md#良い) | テンプレート内の未定義変数参照を検出します。 |
| [`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | [悪い例](./reference/vue-no-unsafe-url.md#悪い) · [良い例](./reference/vue-no-unsafe-url.md#良い) | 危険なスキームになり得る URL 属性やバインディングを検出します。 |
| [`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | [悪い例](./reference/vue-no-unsandboxed-iframe.md#悪い) · [良い例](./reference/vue-no-unsandboxed-iframe.md#良い) | iframe に sandbox 属性を指定します。 |
| [`vue/no-unused-components`](./reference/vue-no-unused-components.md) | [悪い例](./reference/vue-no-unused-components.md#悪い) · [良い例](./reference/vue-no-unused-components.md#良い) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |
| [`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | [悪い例](./reference/vue-no-unused-properties.md#悪い) · [良い例](./reference/vue-no-unused-properties.md#良い) | defineProps に宣言しているのに使わない prop を検出します。 |
| [`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | [悪い例](./reference/vue-no-unused-refs.md#悪い) · [良い例](./reference/vue-no-unused-refs.md#良い) | テンプレートに宣言しているのに参照しない ref を検出します。 |
| [`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | [悪い例](./reference/vue-no-unused-setup-bindings.md#悪い) · [良い例](./reference/vue-no-unused-setup-bindings.md#良い) | script setup に宣言しているのに読み取らない変数を検出します。 |
| [`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | [悪い例](./reference/vue-no-unused-vars.md#悪い) · [良い例](./reference/vue-no-unused-vars.md#良い) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |
| [`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | [悪い例](./reference/vue-no-use-v-else-with-v-for.md#悪い) · [良い例](./reference/vue-no-use-v-else-with-v-for.md#良い) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 |
| [`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | [悪い例](./reference/vue-no-use-v-if-with-v-for.md#悪い) · [良い例](./reference/vue-no-use-v-if-with-v-for.md#良い) | 同じ要素での v-if と v-for の併用を検出します。 |
| [`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | [悪い例](./reference/vue-no-useless-mustaches.md#悪い) · [良い例](./reference/vue-no-useless-mustaches.md#良い) | 文字列リテラルだけの不要な mustache を検出します。 |
| [`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | [悪い例](./reference/vue-no-useless-template-attributes.md#悪い) · [良い例](./reference/vue-no-useless-template-attributes.md#良い) | template 要素の効果がない属性を検出します。 |
| [`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | [悪い例](./reference/vue-no-useless-v-bind.md#悪い) · [良い例](./reference/vue-no-useless-v-bind.md#良い) | 文字列リテラルだけの不要な v-bind を検出します。 |
| [`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | [悪い例](./reference/vue-no-v-for-template-key-on-child.md#悪い) · [良い例](./reference/vue-no-v-for-template-key-on-child.md#良い) | template v-for の key を子ではなく template に指定します。 |
| [`vue/no-v-html`](./reference/vue-no-v-html.md) | [悪い例](./reference/vue-no-v-html.md#悪い) · [良い例](./reference/vue-no-v-html.md#良い) | 未処理の HTML を表示する v-html の XSS リスクを検出します。 |
| [`vue/no-v-text`](./reference/vue-no-v-text.md) | [悪い例](./reference/vue-no-v-text.md#悪い) · [良い例](./reference/vue-no-v-text.md#良い) | v-text の代わりに mustache を使う方針を適用します。 |
| [`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | [悪い例](./reference/vue-no-v-text-v-html-on-component.md#悪い) · [良い例](./reference/vue-no-v-text-v-html-on-component.md#良い) | コンポーネントでの v-text / v-html を検出します。 |
| [`vue/permitted-contents`](./reference/vue-permitted-contents.md) | [悪い例](./reference/vue-permitted-contents.md#悪い) · [良い例](./reference/vue-permitted-contents.md#良い) | HTML の要素ごとのコンテンツモデルを検査します。 |
| [`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | [悪い例](./reference/vue-prefer-props-shorthand.md#悪い) · [良い例](./reference/vue-prefer-props-shorthand.md#良い) | Vue 3.4 の同名 prop バインディングの省略形を使います。 |
| [`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | [悪い例](./reference/vue-prefer-true-attribute-shorthand.md#悪い) · [良い例](./reference/vue-prefer-true-attribute-shorthand.md#良い) | true を指定するバインディングを boolean 属性の省略形にします。 |
| [`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | [悪い例](./reference/vue-prop-name-casing.md#悪い) · [良い例](./reference/vue-prop-name-casing.md#良い) | 宣言する prop 名の形式を揃えます。 |
| [`vue/require-component-is`](./reference/vue-require-component-is.md) | [悪い例](./reference/vue-require-component-is.md#悪い) · [良い例](./reference/vue-require-component-is.md#良い) | 動的 component 要素に :is を指定します。 |
| [`vue/require-component-registration`](./reference/vue-require-component-registration.md) | [悪い例](./reference/vue-require-component-registration.md#悪い) · [良い例](./reference/vue-require-component-registration.md#良い) | 使用するコンポーネントを import または登録します。 |
| [`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | [悪い例](./reference/vue-require-scoped-style.md#悪い) · [良い例](./reference/vue-require-scoped-style.md#良い) | style に scoped を指定する方針を適用します。 |
| [`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | [悪い例](./reference/vue-require-toggle-inside-transition.md#悪い) · [良い例](./reference/vue-require-toggle-inside-transition.md#良い) | transition の子要素に表示を切り替える条件を指定します。 |
| [`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | [悪い例](./reference/vue-require-v-for-key.md#悪い) · [良い例](./reference/vue-require-v-for-key.md#良い) | v-for に安定した :key を指定します。 |
| [`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | [悪い例](./reference/vue-scoped-event-names.md#悪い) · [良い例](./reference/vue-scoped-event-names.md#良い) | イベント名を context:event の形式に揃えます。 |
| [`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | [悪い例](./reference/vue-sfc-element-order.md#悪い) · [良い例](./reference/vue-sfc-element-order.md#良い) | SFC のトップレベルブロックを設定した順に並べます。 |
| [`vue/single-style-block`](./reference/vue-single-style-block.md) | [悪い例](./reference/vue-single-style-block.md#悪い) · [良い例](./reference/vue-single-style-block.md#良い) | SFC の style を一つのブロックにまとめます。 |
| [`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | [悪い例](./reference/vue-slot-name-casing.md#悪い) · [良い例](./reference/vue-slot-name-casing.md#良い) | 名前付き slot を kebab-case に揃えます。 |
| [`vue/this-in-template`](./reference/vue-this-in-template.md) | [悪い例](./reference/vue-this-in-template.md#悪い) · [良い例](./reference/vue-this-in-template.md#良い) | テンプレートで不要な this. 参照を検出します。 |
| [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | [悪い例](./reference/vue-use-unique-element-ids.md#悪い) · [良い例](./reference/vue-use-unique-element-ids.md#良い) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |
| [`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | [悪い例](./reference/vue-use-v-on-exact.md#悪い) · [良い例](./reference/vue-use-v-on-exact.md#良い) | modifier 付きのイベント操作と競合する handler に .exact を指定します。 |
| [`vue/v-bind-style`](./reference/vue-v-bind-style.md) | [悪い例](./reference/vue-v-bind-style.md#悪い) · [良い例](./reference/vue-v-bind-style.md#良い) | v-bind の表記形式を揃えます。 |
| [`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | [悪い例](./reference/vue-v-on-event-hyphenation.md#悪い) · [良い例](./reference/vue-v-on-event-hyphenation.md#良い) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 |
| [`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | [悪い例](./reference/vue-v-on-handler-style.md#悪い) · [良い例](./reference/vue-v-on-handler-style.md#良い) | イベント handler の参照・関数形式を揃えます。 |
| [`vue/v-on-style`](./reference/vue-v-on-style.md) | [悪い例](./reference/vue-v-on-style.md#悪い) · [良い例](./reference/vue-v-on-style.md#良い) | v-on の表記形式を揃えます。 |
| [`vue/v-slot-style`](./reference/vue-v-slot-style.md) | [悪い例](./reference/vue-v-slot-style.md#悪い) · [良い例](./reference/vue-v-slot-style.md#良い) | v-slot の表記形式を揃えます。 |
| [`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | [悪い例](./reference/vue-valid-attribute-name.md#悪い) · [良い例](./reference/vue-valid-attribute-name.md#良い) | 有効な属性名を指定します。 |
| [`vue/valid-template-root`](./reference/vue-valid-template-root.md) | [悪い例](./reference/vue-valid-template-root.md#悪い) · [良い例](./reference/vue-valid-template-root.md#良い) | Vue 3 の fragment に対応する有効なテンプレートルートを検査します。 |
| [`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | [悪い例](./reference/vue-valid-v-bind.md#悪い) · [良い例](./reference/vue-valid-v-bind.md#良い) | v-bind の引数・値・modifier を検査します。 |
| [`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | [悪い例](./reference/vue-valid-v-cloak.md#悪い) · [良い例](./reference/vue-valid-v-cloak.md#良い) | v-cloak の引数・値・modifier を検査します。 |
| [`vue/valid-v-else`](./reference/vue-valid-v-else.md) | [悪い例](./reference/vue-valid-v-else.md#悪い) · [良い例](./reference/vue-valid-v-else.md#良い) | v-else の位置・引数・値を検査します。 |
| [`vue/valid-v-for`](./reference/vue-valid-v-for.md) | [悪い例](./reference/vue-valid-v-for.md#悪い) · [良い例](./reference/vue-valid-v-for.md#良い) | v-for の式と変数宣言を検査します。 |
| [`vue/valid-v-html`](./reference/vue-valid-v-html.md) | [悪い例](./reference/vue-valid-v-html.md#悪い) · [良い例](./reference/vue-valid-v-html.md#良い) | v-html の値・引数・modifier を検査します。 |
| [`vue/valid-v-if`](./reference/vue-valid-v-if.md) | [悪い例](./reference/vue-valid-v-if.md#悪い) · [良い例](./reference/vue-valid-v-if.md#良い) | v-if に有効な条件式を指定します。 |
| [`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | [悪い例](./reference/vue-valid-v-memo.md#悪い) · [良い例](./reference/vue-valid-v-memo.md#良い) | v-memo の値を配列の式にします。 |
| [`vue/valid-v-model`](./reference/vue-valid-v-model.md) | [悪い例](./reference/vue-valid-v-model.md#悪い) · [良い例](./reference/vue-valid-v-model.md#良い) | v-model の値・引数・modifier を検査します。 |
| [`vue/valid-v-on`](./reference/vue-valid-v-on.md) | [悪い例](./reference/vue-valid-v-on.md#悪い) · [良い例](./reference/vue-valid-v-on.md#良い) | v-on のイベント名・式・modifier を検査します。 |
| [`vue/valid-v-once`](./reference/vue-valid-v-once.md) | [悪い例](./reference/vue-valid-v-once.md#悪い) · [良い例](./reference/vue-valid-v-once.md#良い) | v-once の引数・値・modifier を検査します。 |
| [`vue/valid-v-show`](./reference/vue-valid-v-show.md) | [悪い例](./reference/vue-valid-v-show.md#悪い) · [良い例](./reference/vue-valid-v-show.md#良い) | v-show に有効な条件式を指定します。 |
| [`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | [悪い例](./reference/vue-valid-v-slot.md#悪い) · [良い例](./reference/vue-valid-v-slot.md#良い) | v-slot の適用先・宣言・modifier を検査します。 |
| [`vue/valid-v-text`](./reference/vue-valid-v-text.md) | [悪い例](./reference/vue-valid-v-text.md#悪い) · [良い例](./reference/vue-valid-v-text.md#良い) | v-text の値・引数・modifier を検査します。 |
| [`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | [悪い例](./reference/vue-warn-custom-block.md#悪い) · [良い例](./reference/vue-warn-custom-block.md#良い) | SFC のカスタムブロックを検出します。 |
| [`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | [悪い例](./reference/vue-warn-custom-directive.md#悪い) · [良い例](./reference/vue-warn-custom-directive.md#良い) | 登録が必要なカスタムディレクティブを検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

子への属性の継承は [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md) の対象です。
