---
title: Vue ルール
---

# Vue ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/a11y-img-alt`](./all.md#vue-a11y-img-alt) | [悪い例](./all.md#vue-a11y-img-alt-bad) · [良い例](./all.md#vue-a11y-img-alt-good) | 画像に代替テキストの alt 属性を指定します。 |
| [`vue/attribute-hyphenation`](./all.md#vue-attribute-hyphenation) | [悪い例](./all.md#vue-attribute-hyphenation-bad) · [良い例](./all.md#vue-attribute-hyphenation-good) | コンポーネントの prop 属性名を設定した形式に揃えます。 |
| [`vue/attribute-order`](./all.md#vue-attribute-order) | [悪い例](./all.md#vue-attribute-order-bad) · [良い例](./all.md#vue-attribute-order-good) | テンプレートの属性を一定の順に並べます。 |
| [`vue/component-definition-name-casing`](./all.md#vue-component-definition-name-casing) | [悪い例](./all.md#vue-component-definition-name-casing-bad) · [良い例](./all.md#vue-component-definition-name-casing-good) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |
| [`vue/component-name-in-template-casing`](./all.md#vue-component-name-in-template-casing) | [悪い例](./all.md#vue-component-name-in-template-casing-bad) · [良い例](./all.md#vue-component-name-in-template-casing-good) | テンプレート内のコンポーネント名を指定した形式に揃えます。 |
| [`vue/html-button-has-type`](./all.md#vue-html-button-has-type) | [悪い例](./all.md#vue-html-button-has-type-bad) · [良い例](./all.md#vue-html-button-has-type-good) | button に有効な type を明示します。 |
| [`vue/html-quotes`](./all.md#vue-html-quotes) | [悪い例](./all.md#vue-html-quotes-bad) · [良い例](./all.md#vue-html-quotes-good) | HTML 属性値の引用符を揃えます。 |
| [`vue/html-self-closing`](./all.md#vue-html-self-closing) | [悪い例](./all.md#vue-html-self-closing-bad) · [良い例](./all.md#vue-html-self-closing-good) | 要素の種類ごとに自己終了タグの形式を揃えます。 |
| [`vue/max-template-complexity`](./all.md#vue-max-template-complexity) | [悪い例](./all.md#vue-max-template-complexity-bad) · [良い例](./all.md#vue-max-template-complexity-good) | コンポーネント自身のテンプレートの複雑度を制限します。 |
| [`vue/multi-word-component-names`](./all.md#vue-multi-word-component-names) | [悪い例](./all.md#vue-multi-word-component-names-bad) · [良い例](./all.md#vue-multi-word-component-names-good) | コンポーネント名を複数の単語で構成します。 |
| [`vue/mustache-interpolation-spacing`](./all.md#vue-mustache-interpolation-spacing) | [悪い例](./all.md#vue-mustache-interpolation-spacing-bad) · [良い例](./all.md#vue-mustache-interpolation-spacing-good) | mustache 内の空白を揃えます。 |
| [`vue/no-array-index-key`](./all.md#vue-no-array-index-key) | [悪い例](./all.md#vue-no-array-index-key-bad) · [良い例](./all.md#vue-no-array-index-key-good) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 |
| [`vue/no-bare-strings-in-template`](./all.md#vue-no-bare-strings-in-template) | [悪い例](./all.md#vue-no-bare-strings-in-template-bad) · [良い例](./all.md#vue-no-bare-strings-in-template-good) | 国際化すべきテンプレートの直接指定テキストを検出します。 |
| [`vue/no-boolean-attr-value`](./all.md#vue-no-boolean-attr-value) | [悪い例](./all.md#vue-no-boolean-attr-value-bad) · [良い例](./all.md#vue-no-boolean-attr-value-good) | HTML の boolean 属性に不要な値を指定した箇所を検出します。 |
| [`vue/no-child-content`](./all.md#vue-no-child-content) | [悪い例](./all.md#vue-no-child-content-bad) · [良い例](./all.md#vue-no-child-content-good) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |
| [`vue/no-deprecated-filter`](./all.md#vue-no-deprecated-filter) | [悪い例](./all.md#vue-no-deprecated-filter-bad) · [良い例](./all.md#vue-no-deprecated-filter-good) | Vue 2 の pipe による filter 構文を検出します。 |
| [`vue/no-deprecated-functional-template`](./all.md#vue-no-deprecated-functional-template) | [悪い例](./all.md#vue-no-deprecated-functional-template-bad) · [良い例](./all.md#vue-no-deprecated-functional-template-good) | SFC の template で削除済みの functional 属性を検出します。 |
| [`vue/no-deprecated-html-element-is`](./all.md#vue-no-deprecated-html-element-is) | [悪い例](./all.md#vue-no-deprecated-html-element-is-bad) · [良い例](./all.md#vue-no-deprecated-html-element-is-good) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 |
| [`vue/no-deprecated-inline-template`](./all.md#vue-no-deprecated-inline-template) | [悪い例](./all.md#vue-no-deprecated-inline-template-bad) · [良い例](./all.md#vue-no-deprecated-inline-template-good) | 削除済みの inline-template 属性を検出します。 |
| [`vue/no-deprecated-router-link-tag-prop`](./all.md#vue-no-deprecated-router-link-tag-prop) | [悪い例](./all.md#vue-no-deprecated-router-link-tag-prop-bad) · [良い例](./all.md#vue-no-deprecated-router-link-tag-prop-good) | router-link の削除済み tag prop を検出します。 |
| [`vue/no-deprecated-scope-attribute`](./all.md#vue-no-deprecated-scope-attribute) | [悪い例](./all.md#vue-no-deprecated-scope-attribute-bad) · [良い例](./all.md#vue-no-deprecated-scope-attribute-good) | template の削除済み scope 属性を検出します。 |
| [`vue/no-deprecated-slot-attribute`](./all.md#vue-no-deprecated-slot-attribute) | [悪い例](./all.md#vue-no-deprecated-slot-attribute-bad) · [良い例](./all.md#vue-no-deprecated-slot-attribute-good) | 削除済みの slot 属性を検出します。 |
| [`vue/no-deprecated-slot-scope-attribute`](./all.md#vue-no-deprecated-slot-scope-attribute) | [悪い例](./all.md#vue-no-deprecated-slot-scope-attribute-bad) · [良い例](./all.md#vue-no-deprecated-slot-scope-attribute-good) | 削除済みの slot-scope 属性を検出します。 |
| [`vue/no-deprecated-v-bind-sync`](./all.md#vue-no-deprecated-v-bind-sync) | [悪い例](./all.md#vue-no-deprecated-v-bind-sync-bad) · [良い例](./all.md#vue-no-deprecated-v-bind-sync-good) | 削除済みの v-bind の .sync modifier を検出します。 |
| [`vue/no-deprecated-v-on-native-modifier`](./all.md#vue-no-deprecated-v-on-native-modifier) | [悪い例](./all.md#vue-no-deprecated-v-on-native-modifier-bad) · [良い例](./all.md#vue-no-deprecated-v-on-native-modifier-good) | 削除済みの v-on の .native modifier を検出します。 |
| [`vue/no-deprecated-v-on-number-modifiers`](./all.md#vue-no-deprecated-v-on-number-modifiers) | [悪い例](./all.md#vue-no-deprecated-v-on-number-modifiers-bad) · [良い例](./all.md#vue-no-deprecated-v-on-number-modifiers-good) | v-on の削除済み数値 keyCode modifier を検出します。 |
| [`vue/no-dupe-v-else-if`](./all.md#vue-no-dupe-v-else-if) | [悪い例](./all.md#vue-no-dupe-v-else-if-bad) · [良い例](./all.md#vue-no-dupe-v-else-if-good) | v-if / v-else-if の条件重複を検出します。 |
| [`vue/no-duplicate-attributes`](./all.md#vue-no-duplicate-attributes) | [悪い例](./all.md#vue-no-duplicate-attributes-bad) · [良い例](./all.md#vue-no-duplicate-attributes-good) | 同じ要素の属性重複を検出します。 |
| [`vue/no-empty-component-block`](./all.md#vue-no-empty-component-block) | [悪い例](./all.md#vue-no-empty-component-block-bad) · [良い例](./all.md#vue-no-empty-component-block-good) | 空の SFC ブロックを検出します。 |
| [`vue/no-inline-style`](./all.md#vue-no-inline-style) | [悪い例](./all.md#vue-no-inline-style-bad) · [良い例](./all.md#vue-no-inline-style-good) | インラインの style 属性を検出します。 |
| [`vue/no-invalid-html-attribute`](./all.md#vue-no-invalid-html-attribute) | [悪い例](./all.md#vue-no-invalid-html-attribute-bad) · [良い例](./all.md#vue-no-invalid-html-attribute-good) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 |
| [`vue/no-lone-template`](./all.md#vue-no-lone-template) | [悪い例](./all.md#vue-no-lone-template-bad) · [良い例](./all.md#vue-no-lone-template-good) | 不要な template 要素を検出します。 |
| [`vue/no-multi-spaces`](./all.md#vue-no-multi-spaces) | [悪い例](./all.md#vue-no-multi-spaces-bad) · [良い例](./all.md#vue-no-multi-spaces-good) | 連続する不要な空白を検出します。 |
| [`vue/no-multiple-objects-in-class`](./all.md#vue-no-multiple-objects-in-class) | [悪い例](./all.md#vue-no-multiple-objects-in-class-bad) · [良い例](./all.md#vue-no-multiple-objects-in-class-good) | :class 配列内の複数のオブジェクト指定をまとめます。 |
| [`vue/no-multiple-template-root`](./all.md#vue-no-multiple-template-root) | [悪い例](./all.md#vue-no-multiple-template-root-bad) · [良い例](./all.md#vue-no-multiple-template-root-good) | 単一ルートを要求するテンプレートで複数ルートを検出します。 |
| [`vue/no-mutating-props`](./all.md#vue-no-mutating-props) | [悪い例](./all.md#vue-no-mutating-props-bad) · [良い例](./all.md#vue-no-mutating-props-good) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |
| [`vue/no-negated-v-if-condition`](./all.md#vue-no-negated-v-if-condition) | [悪い例](./all.md#vue-no-negated-v-if-condition-bad) · [良い例](./all.md#vue-no-negated-v-if-condition-good) | v-else がある条件分岐の否定条件を反転して読みやすくします。 |
| [`vue/no-non-component-keep-alive-child`](./all.md#vue-no-non-component-keep-alive-child) | [悪い例](./all.md#vue-no-non-component-keep-alive-child-bad) · [良い例](./all.md#vue-no-non-component-keep-alive-child-good) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |
| [`vue/no-preprocessor-lang`](./all.md#vue-no-preprocessor-lang) | [悪い例](./all.md#vue-no-preprocessor-lang-bad) · [良い例](./all.md#vue-no-preprocessor-lang-good) | CSS preprocessor より標準の CSS を使う方針を適用します。 |
| [`vue/no-reserved-component-names`](./all.md#vue-no-reserved-component-names) | [悪い例](./all.md#vue-no-reserved-component-names-bad) · [良い例](./all.md#vue-no-reserved-component-names-good) | 予約済みのコンポーネント名を検出します。 |
| [`vue/no-root-v-if`](./all.md#vue-no-root-v-if) | [悪い例](./all.md#vue-no-root-v-if-bad) · [良い例](./all.md#vue-no-root-v-if-good) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 |
| [`vue/no-script-non-standard-lang`](./all.md#vue-no-script-non-standard-lang) | [悪い例](./all.md#vue-no-script-non-standard-lang-bad) · [良い例](./all.md#vue-no-script-non-standard-lang-good) | script の非標準 lang 指定を検出します。 |
| [`vue/no-src-attribute`](./all.md#vue-no-src-attribute) | [悪い例](./all.md#vue-no-src-attribute-bad) · [良い例](./all.md#vue-no-src-attribute-good) | SFC ブロックの外部 src 指定を検出します。 |
| [`vue/no-static-inline-styles`](./all.md#vue-no-static-inline-styles) | [悪い例](./all.md#vue-no-static-inline-styles-bad) · [良い例](./all.md#vue-no-static-inline-styles-good) | 静的なインライン style 属性を検出します。 |
| [`vue/no-template-key`](./all.md#vue-no-template-key) | [悪い例](./all.md#vue-no-template-key-bad) · [良い例](./all.md#vue-no-template-key-good) | v-for 用ではない template の key 指定を検出します。 |
| [`vue/no-template-lang`](./all.md#vue-no-template-lang) | [悪い例](./all.md#vue-no-template-lang-bad) · [良い例](./all.md#vue-no-template-lang-good) | template の lang 指定を検出します。 |
| [`vue/no-template-shadow`](./all.md#vue-no-template-shadow) | [悪い例](./all.md#vue-no-template-shadow-bad) · [良い例](./all.md#vue-no-template-shadow-good) | テンプレート変数が外側の名前を隠す箇所を検出します。 |
| [`vue/no-template-target-blank`](./all.md#vue-no-template-target-blank) | [悪い例](./all.md#vue-no-template-target-blank-bad) · [良い例](./all.md#vue-no-template-target-blank-good) | target=_blank の外部リンクに適切な rel を指定します。 |
| [`vue/no-textarea-mustache`](./all.md#vue-no-textarea-mustache) | [悪い例](./all.md#vue-no-textarea-mustache-bad) · [良い例](./all.md#vue-no-textarea-mustache-good) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |
| [`vue/no-undefined-refs`](./all.md#vue-no-undefined-refs) | [悪い例](./all.md#vue-no-undefined-refs-bad) · [良い例](./all.md#vue-no-undefined-refs-good) | テンプレート内の未定義変数参照を検出します。 |
| [`vue/no-unsafe-url`](./all.md#vue-no-unsafe-url) | [悪い例](./all.md#vue-no-unsafe-url-bad) · [良い例](./all.md#vue-no-unsafe-url-good) | 危険なスキームになり得る URL 属性やバインディングを検出します。 |
| [`vue/no-unsandboxed-iframe`](./all.md#vue-no-unsandboxed-iframe) | [悪い例](./all.md#vue-no-unsandboxed-iframe-bad) · [良い例](./all.md#vue-no-unsandboxed-iframe-good) | iframe に sandbox 属性を指定します。 |
| [`vue/no-unused-components`](./all.md#vue-no-unused-components) | [悪い例](./all.md#vue-no-unused-components-bad) · [良い例](./all.md#vue-no-unused-components-good) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |
| [`vue/no-unused-properties`](./all.md#vue-no-unused-properties) | [悪い例](./all.md#vue-no-unused-properties-bad) · [良い例](./all.md#vue-no-unused-properties-good) | defineProps に宣言しているのに使わない prop を検出します。 |
| [`vue/no-unused-refs`](./all.md#vue-no-unused-refs) | [悪い例](./all.md#vue-no-unused-refs-bad) · [良い例](./all.md#vue-no-unused-refs-good) | テンプレートに宣言しているのに参照しない ref を検出します。 |
| [`vue/no-unused-setup-bindings`](./all.md#vue-no-unused-setup-bindings) | [悪い例](./all.md#vue-no-unused-setup-bindings-bad) · [良い例](./all.md#vue-no-unused-setup-bindings-good) | script setup に宣言しているのに読み取らない変数を検出します。 |
| [`vue/no-unused-vars`](./all.md#vue-no-unused-vars) | [悪い例](./all.md#vue-no-unused-vars-bad) · [良い例](./all.md#vue-no-unused-vars-good) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |
| [`vue/no-use-v-else-with-v-for`](./all.md#vue-no-use-v-else-with-v-for) | [悪い例](./all.md#vue-no-use-v-else-with-v-for-bad) · [良い例](./all.md#vue-no-use-v-else-with-v-for-good) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 |
| [`vue/no-use-v-if-with-v-for`](./all.md#vue-no-use-v-if-with-v-for) | [悪い例](./all.md#vue-no-use-v-if-with-v-for-bad) · [良い例](./all.md#vue-no-use-v-if-with-v-for-good) | 同じ要素での v-if と v-for の併用を検出します。 |
| [`vue/no-useless-mustaches`](./all.md#vue-no-useless-mustaches) | [悪い例](./all.md#vue-no-useless-mustaches-bad) · [良い例](./all.md#vue-no-useless-mustaches-good) | 文字列リテラルだけの不要な mustache を検出します。 |
| [`vue/no-useless-template-attributes`](./all.md#vue-no-useless-template-attributes) | [悪い例](./all.md#vue-no-useless-template-attributes-bad) · [良い例](./all.md#vue-no-useless-template-attributes-good) | template 要素の効果がない属性を検出します。 |
| [`vue/no-useless-v-bind`](./all.md#vue-no-useless-v-bind) | [悪い例](./all.md#vue-no-useless-v-bind-bad) · [良い例](./all.md#vue-no-useless-v-bind-good) | 文字列リテラルだけの不要な v-bind を検出します。 |
| [`vue/no-v-for-template-key-on-child`](./all.md#vue-no-v-for-template-key-on-child) | [悪い例](./all.md#vue-no-v-for-template-key-on-child-bad) · [良い例](./all.md#vue-no-v-for-template-key-on-child-good) | template v-for の key を子ではなく template に指定します。 |
| [`vue/no-v-html`](./all.md#vue-no-v-html) | [悪い例](./all.md#vue-no-v-html-bad) · [良い例](./all.md#vue-no-v-html-good) | 未処理の HTML を表示する v-html の XSS リスクを検出します。 |
| [`vue/no-v-text`](./all.md#vue-no-v-text) | [悪い例](./all.md#vue-no-v-text-bad) · [良い例](./all.md#vue-no-v-text-good) | v-text の代わりに mustache を使う方針を適用します。 |
| [`vue/no-v-text-v-html-on-component`](./all.md#vue-no-v-text-v-html-on-component) | [悪い例](./all.md#vue-no-v-text-v-html-on-component-bad) · [良い例](./all.md#vue-no-v-text-v-html-on-component-good) | コンポーネントでの v-text / v-html を検出します。 |
| [`vue/permitted-contents`](./all.md#vue-permitted-contents) | [悪い例](./all.md#vue-permitted-contents-bad) · [良い例](./all.md#vue-permitted-contents-good) | HTML の要素ごとのコンテンツモデルを検査します。 |
| [`vue/prefer-props-shorthand`](./all.md#vue-prefer-props-shorthand) | [悪い例](./all.md#vue-prefer-props-shorthand-bad) · [良い例](./all.md#vue-prefer-props-shorthand-good) | Vue 3.4 の同名 prop バインディングの省略形を使います。 |
| [`vue/prefer-true-attribute-shorthand`](./all.md#vue-prefer-true-attribute-shorthand) | [悪い例](./all.md#vue-prefer-true-attribute-shorthand-bad) · [良い例](./all.md#vue-prefer-true-attribute-shorthand-good) | true を指定するバインディングを boolean 属性の省略形にします。 |
| [`vue/prop-name-casing`](./all.md#vue-prop-name-casing) | [悪い例](./all.md#vue-prop-name-casing-bad) · [良い例](./all.md#vue-prop-name-casing-good) | 宣言する prop 名の形式を揃えます。 |
| [`vue/require-component-is`](./all.md#vue-require-component-is) | [悪い例](./all.md#vue-require-component-is-bad) · [良い例](./all.md#vue-require-component-is-good) | 動的 component 要素に :is を指定します。 |
| [`vue/require-component-registration`](./all.md#vue-require-component-registration) | [悪い例](./all.md#vue-require-component-registration-bad) · [良い例](./all.md#vue-require-component-registration-good) | 使用するコンポーネントを import または登録します。 |
| [`vue/require-scoped-style`](./all.md#vue-require-scoped-style) | [悪い例](./all.md#vue-require-scoped-style-bad) · [良い例](./all.md#vue-require-scoped-style-good) | style に scoped を指定する方針を適用します。 |
| [`vue/require-toggle-inside-transition`](./all.md#vue-require-toggle-inside-transition) | [悪い例](./all.md#vue-require-toggle-inside-transition-bad) · [良い例](./all.md#vue-require-toggle-inside-transition-good) | transition の子要素に表示を切り替える条件を指定します。 |
| [`vue/require-v-for-key`](./all.md#vue-require-v-for-key) | [悪い例](./all.md#vue-require-v-for-key-bad) · [良い例](./all.md#vue-require-v-for-key-good) | v-for に安定した :key を指定します。 |
| [`vue/scoped-event-names`](./all.md#vue-scoped-event-names) | [悪い例](./all.md#vue-scoped-event-names-bad) · [良い例](./all.md#vue-scoped-event-names-good) | イベント名を context:event の形式に揃えます。 |
| [`vue/sfc-element-order`](./all.md#vue-sfc-element-order) | [悪い例](./all.md#vue-sfc-element-order-bad) · [良い例](./all.md#vue-sfc-element-order-good) | SFC のトップレベルブロックを設定した順に並べます。 |
| [`vue/single-style-block`](./all.md#vue-single-style-block) | [悪い例](./all.md#vue-single-style-block-bad) · [良い例](./all.md#vue-single-style-block-good) | SFC の style を一つのブロックにまとめます。 |
| [`vue/slot-name-casing`](./all.md#vue-slot-name-casing) | [悪い例](./all.md#vue-slot-name-casing-bad) · [良い例](./all.md#vue-slot-name-casing-good) | 名前付き slot を kebab-case に揃えます。 |
| [`vue/this-in-template`](./all.md#vue-this-in-template) | [悪い例](./all.md#vue-this-in-template-bad) · [良い例](./all.md#vue-this-in-template-good) | テンプレートで不要な this. 参照を検出します。 |
| [`vue/use-unique-element-ids`](./all.md#vue-use-unique-element-ids) | [悪い例](./all.md#vue-use-unique-element-ids-bad) · [良い例](./all.md#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |
| [`vue/use-v-on-exact`](./all.md#vue-use-v-on-exact) | [悪い例](./all.md#vue-use-v-on-exact-bad) · [良い例](./all.md#vue-use-v-on-exact-good) | modifier 付きのイベント操作と競合する handler に .exact を指定します。 |
| [`vue/v-bind-style`](./all.md#vue-v-bind-style) | [悪い例](./all.md#vue-v-bind-style-bad) · [良い例](./all.md#vue-v-bind-style-good) | v-bind の表記形式を揃えます。 |
| [`vue/v-on-event-hyphenation`](./all.md#vue-v-on-event-hyphenation) | [悪い例](./all.md#vue-v-on-event-hyphenation-bad) · [良い例](./all.md#vue-v-on-event-hyphenation-good) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 |
| [`vue/v-on-handler-style`](./all.md#vue-v-on-handler-style) | [悪い例](./all.md#vue-v-on-handler-style-bad) · [良い例](./all.md#vue-v-on-handler-style-good) | イベント handler の参照・関数形式を揃えます。 |
| [`vue/v-on-style`](./all.md#vue-v-on-style) | [悪い例](./all.md#vue-v-on-style-bad) · [良い例](./all.md#vue-v-on-style-good) | v-on の表記形式を揃えます。 |
| [`vue/v-slot-style`](./all.md#vue-v-slot-style) | [悪い例](./all.md#vue-v-slot-style-bad) · [良い例](./all.md#vue-v-slot-style-good) | v-slot の表記形式を揃えます。 |
| [`vue/valid-attribute-name`](./all.md#vue-valid-attribute-name) | [悪い例](./all.md#vue-valid-attribute-name-bad) · [良い例](./all.md#vue-valid-attribute-name-good) | 有効な属性名を指定します。 |
| [`vue/valid-template-root`](./all.md#vue-valid-template-root) | [悪い例](./all.md#vue-valid-template-root-bad) · [良い例](./all.md#vue-valid-template-root-good) | Vue 3 の fragment に対応する有効なテンプレートルートを検査します。 |
| [`vue/valid-v-bind`](./all.md#vue-valid-v-bind) | [悪い例](./all.md#vue-valid-v-bind-bad) · [良い例](./all.md#vue-valid-v-bind-good) | v-bind の引数・値・modifier を検査します。 |
| [`vue/valid-v-cloak`](./all.md#vue-valid-v-cloak) | [悪い例](./all.md#vue-valid-v-cloak-bad) · [良い例](./all.md#vue-valid-v-cloak-good) | v-cloak の引数・値・modifier を検査します。 |
| [`vue/valid-v-else`](./all.md#vue-valid-v-else) | [悪い例](./all.md#vue-valid-v-else-bad) · [良い例](./all.md#vue-valid-v-else-good) | v-else の位置・引数・値を検査します。 |
| [`vue/valid-v-for`](./all.md#vue-valid-v-for) | [悪い例](./all.md#vue-valid-v-for-bad) · [良い例](./all.md#vue-valid-v-for-good) | v-for の式と変数宣言を検査します。 |
| [`vue/valid-v-html`](./all.md#vue-valid-v-html) | [悪い例](./all.md#vue-valid-v-html-bad) · [良い例](./all.md#vue-valid-v-html-good) | v-html の値・引数・modifier を検査します。 |
| [`vue/valid-v-if`](./all.md#vue-valid-v-if) | [悪い例](./all.md#vue-valid-v-if-bad) · [良い例](./all.md#vue-valid-v-if-good) | v-if に有効な条件式を指定します。 |
| [`vue/valid-v-memo`](./all.md#vue-valid-v-memo) | [悪い例](./all.md#vue-valid-v-memo-bad) · [良い例](./all.md#vue-valid-v-memo-good) | v-memo の値を配列の式にします。 |
| [`vue/valid-v-model`](./all.md#vue-valid-v-model) | [悪い例](./all.md#vue-valid-v-model-bad) · [良い例](./all.md#vue-valid-v-model-good) | v-model の値・引数・modifier を検査します。 |
| [`vue/valid-v-on`](./all.md#vue-valid-v-on) | [悪い例](./all.md#vue-valid-v-on-bad) · [良い例](./all.md#vue-valid-v-on-good) | v-on のイベント名・式・modifier を検査します。 |
| [`vue/valid-v-once`](./all.md#vue-valid-v-once) | [悪い例](./all.md#vue-valid-v-once-bad) · [良い例](./all.md#vue-valid-v-once-good) | v-once の引数・値・modifier を検査します。 |
| [`vue/valid-v-show`](./all.md#vue-valid-v-show) | [悪い例](./all.md#vue-valid-v-show-bad) · [良い例](./all.md#vue-valid-v-show-good) | v-show に有効な条件式を指定します。 |
| [`vue/valid-v-slot`](./all.md#vue-valid-v-slot) | [悪い例](./all.md#vue-valid-v-slot-bad) · [良い例](./all.md#vue-valid-v-slot-good) | v-slot の適用先・宣言・modifier を検査します。 |
| [`vue/valid-v-text`](./all.md#vue-valid-v-text) | [悪い例](./all.md#vue-valid-v-text-bad) · [良い例](./all.md#vue-valid-v-text-good) | v-text の値・引数・modifier を検査します。 |
| [`vue/warn-custom-block`](./all.md#vue-warn-custom-block) | [悪い例](./all.md#vue-warn-custom-block-bad) · [良い例](./all.md#vue-warn-custom-block-good) | SFC のカスタムブロックを検出します。 |
| [`vue/warn-custom-directive`](./all.md#vue-warn-custom-directive) | [悪い例](./all.md#vue-warn-custom-directive-bad) · [良い例](./all.md#vue-warn-custom-directive-good) | 登録が必要なカスタムディレクティブを検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

子への属性の継承は [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md) の対象です。
