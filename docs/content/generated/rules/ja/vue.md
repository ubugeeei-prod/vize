---
title: Vue ルール
---

# Vue ルール

すべての Vue ルールの目的・設定・悪い例・良い例をこのページにまとめています。色付きの行で変更を示し、コピーしたコードには完全なソースを保持します。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [悪い](#vue-a11y-img-alt-bad) · [良い](#vue-a11y-img-alt-good) | 画像に代替テキストの alt 属性を指定します。 |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [悪い](#vue-attribute-hyphenation-bad) · [良い](#vue-attribute-hyphenation-good) | コンポーネントの prop 属性名を設定した形式に揃えます。 |
| [`vue/attribute-order`](#vue-attribute-order) | [悪い](#vue-attribute-order-bad) · [良い](#vue-attribute-order-good) | テンプレートの属性を一定の順に並べます。 |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [悪い](#vue-component-definition-name-casing-bad) · [良い](#vue-component-definition-name-casing-good) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [悪い](#vue-component-name-in-template-casing-bad) · [良い](#vue-component-name-in-template-casing-good) | テンプレート内のコンポーネント名を指定した形式に揃えます。 |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [悪い](#vue-html-button-has-type-bad) · [良い](#vue-html-button-has-type-good) | button に有効な type を明示します。 |
| [`vue/html-quotes`](#vue-html-quotes) | [悪い](#vue-html-quotes-bad) · [良い](#vue-html-quotes-good) | HTML 属性値の引用符を揃えます。 |
| [`vue/html-self-closing`](#vue-html-self-closing) | [悪い](#vue-html-self-closing-bad) · [良い](#vue-html-self-closing-good) | 要素の種類ごとに自己終了タグの形式を揃えます。 |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [悪い](#vue-max-template-complexity-bad) · [良い](#vue-max-template-complexity-good) | コンポーネント自身のテンプレートの複雑度を制限します。 |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [悪い](#vue-multi-word-component-names-bad) · [良い](#vue-multi-word-component-names-good) | コンポーネント名を複数の単語で構成します。 |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [悪い](#vue-mustache-interpolation-spacing-bad) · [良い](#vue-mustache-interpolation-spacing-good) | mustache 内の空白を揃えます。 |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [悪い](#vue-no-array-index-key-bad) · [良い](#vue-no-array-index-key-good) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [悪い](#vue-no-bare-strings-in-template-bad) · [良い](#vue-no-bare-strings-in-template-good) | 国際化すべきテンプレートの直接指定テキストを検出します。 |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [悪い](#vue-no-boolean-attr-value-bad) · [良い](#vue-no-boolean-attr-value-good) | HTML の boolean 属性に不要な値を指定した箇所を検出します。 |
| [`vue/no-child-content`](#vue-no-child-content) | [悪い](#vue-no-child-content-bad) · [良い](#vue-no-child-content-good) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [悪い](#vue-no-deprecated-filter-bad) · [良い](#vue-no-deprecated-filter-good) | Vue 2 の pipe による filter 構文を検出します。 |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [悪い](#vue-no-deprecated-functional-template-bad) · [良い](#vue-no-deprecated-functional-template-good) | SFC の template で削除済みの functional 属性を検出します。 |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [悪い](#vue-no-deprecated-html-element-is-bad) · [良い](#vue-no-deprecated-html-element-is-good) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [悪い](#vue-no-deprecated-inline-template-bad) · [良い](#vue-no-deprecated-inline-template-good) | 削除済みの inline-template 属性を検出します。 |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [悪い](#vue-no-deprecated-router-link-tag-prop-bad) · [良い](#vue-no-deprecated-router-link-tag-prop-good) | router-link の削除済み tag prop を検出します。 |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [悪い](#vue-no-deprecated-scope-attribute-bad) · [良い](#vue-no-deprecated-scope-attribute-good) | template の削除済み scope 属性を検出します。 |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [悪い](#vue-no-deprecated-slot-attribute-bad) · [良い](#vue-no-deprecated-slot-attribute-good) | 削除済みの slot 属性を検出します。 |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [悪い](#vue-no-deprecated-slot-scope-attribute-bad) · [良い](#vue-no-deprecated-slot-scope-attribute-good) | 削除済みの slot-scope 属性を検出します。 |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [悪い](#vue-no-deprecated-v-bind-sync-bad) · [良い](#vue-no-deprecated-v-bind-sync-good) | 削除済みの v-bind の .sync modifier を検出します。 |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [悪い](#vue-no-deprecated-v-on-native-modifier-bad) · [良い](#vue-no-deprecated-v-on-native-modifier-good) | 削除済みの v-on の .native modifier を検出します。 |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [悪い](#vue-no-deprecated-v-on-number-modifiers-bad) · [良い](#vue-no-deprecated-v-on-number-modifiers-good) | v-on の削除済み数値 keyCode modifier を検出します。 |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [悪い](#vue-no-dupe-v-else-if-bad) · [良い](#vue-no-dupe-v-else-if-good) | v-if / v-else-if の条件重複を検出します。 |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [悪い](#vue-no-duplicate-attributes-bad) · [良い](#vue-no-duplicate-attributes-good) | 同じ要素の属性重複を検出します。 |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [悪い](#vue-no-empty-component-block-bad) · [良い](#vue-no-empty-component-block-good) | 空の SFC ブロックを検出します。 |
| [`vue/no-inline-style`](#vue-no-inline-style) | [悪い](#vue-no-inline-style-bad) · [良い](#vue-no-inline-style-good) | インラインの style 属性を検出します。 |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [悪い](#vue-no-invalid-html-attribute-bad) · [良い](#vue-no-invalid-html-attribute-good) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 |
| [`vue/no-lone-template`](#vue-no-lone-template) | [悪い](#vue-no-lone-template-bad) · [良い](#vue-no-lone-template-good) | 不要な template 要素を検出します。 |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [悪い](#vue-no-multi-spaces-bad) · [良い](#vue-no-multi-spaces-good) | 連続する不要な空白を検出します。 |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [悪い](#vue-no-multiple-objects-in-class-bad) · [良い](#vue-no-multiple-objects-in-class-good) | :class 配列内の複数のオブジェクト指定をまとめます。 |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [悪い](#vue-no-multiple-template-root-bad) · [良い](#vue-no-multiple-template-root-good) | 単一ルートを要求するテンプレートで複数ルートを検出します。 |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [悪い](#vue-no-mutating-props-bad) · [良い](#vue-no-mutating-props-good) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [悪い](#vue-no-negated-v-if-condition-bad) · [良い](#vue-no-negated-v-if-condition-good) | v-else がある条件分岐の否定条件を反転して読みやすくします。 |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [悪い](#vue-no-non-component-keep-alive-child-bad) · [良い](#vue-no-non-component-keep-alive-child-good) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [悪い](#vue-no-preprocessor-lang-bad) · [良い](#vue-no-preprocessor-lang-good) | CSS preprocessor より標準の CSS を使う方針を適用します。 |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [悪い](#vue-no-reserved-component-names-bad) · [良い](#vue-no-reserved-component-names-good) | 予約済みのコンポーネント名を検出します。 |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [悪い](#vue-no-root-v-if-bad) · [良い](#vue-no-root-v-if-good) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [悪い](#vue-no-script-non-standard-lang-bad) · [良い](#vue-no-script-non-standard-lang-good) | script の非標準 lang 指定を検出します。 |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [悪い](#vue-no-src-attribute-bad) · [良い](#vue-no-src-attribute-good) | SFC ブロックの外部 src 指定を検出します。 |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [悪い](#vue-no-static-inline-styles-bad) · [良い](#vue-no-static-inline-styles-good) | 静的なインライン style 属性を検出します。 |
| [`vue/no-template-key`](#vue-no-template-key) | [悪い](#vue-no-template-key-bad) · [良い](#vue-no-template-key-good) | v-for 用ではない template の key 指定を検出します。 |
| [`vue/no-template-lang`](#vue-no-template-lang) | [悪い](#vue-no-template-lang-bad) · [良い](#vue-no-template-lang-good) | template の lang 指定を検出します。 |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [悪い](#vue-no-template-shadow-bad) · [良い](#vue-no-template-shadow-good) | テンプレート変数が外側の名前を隠す箇所を検出します。 |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [悪い](#vue-no-template-target-blank-bad) · [良い](#vue-no-template-target-blank-good) | target=_blank の外部リンクに適切な rel を指定します。 |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [悪い](#vue-no-textarea-mustache-bad) · [良い](#vue-no-textarea-mustache-good) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [悪い](#vue-no-undefined-refs-bad) · [良い](#vue-no-undefined-refs-good) | テンプレート内の未定義変数参照を検出します。 |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [悪い](#vue-no-unsafe-url-bad) · [良い](#vue-no-unsafe-url-good) | 危険なスキームになり得る URL 属性やバインディングを検出します。 |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [悪い](#vue-no-unsandboxed-iframe-bad) · [良い](#vue-no-unsandboxed-iframe-good) | iframe に sandbox 属性を指定します。 |
| [`vue/no-unused-components`](#vue-no-unused-components) | [悪い](#vue-no-unused-components-bad) · [良い](#vue-no-unused-components-good) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [悪い](#vue-no-unused-properties-bad) · [良い](#vue-no-unused-properties-good) | defineProps に宣言しているのに使わない prop を検出します。 |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [悪い](#vue-no-unused-refs-bad) · [良い](#vue-no-unused-refs-good) | テンプレートに宣言しているのに参照しない ref を検出します。 |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [悪い](#vue-no-unused-setup-bindings-bad) · [良い](#vue-no-unused-setup-bindings-good) | script setup に宣言しているのに読み取らない変数を検出します。 |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [悪い](#vue-no-unused-vars-bad) · [良い](#vue-no-unused-vars-good) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [悪い](#vue-no-use-v-else-with-v-for-bad) · [良い](#vue-no-use-v-else-with-v-for-good) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [悪い](#vue-no-use-v-if-with-v-for-bad) · [良い](#vue-no-use-v-if-with-v-for-good) | 同じ要素での v-if と v-for の併用を検出します。 |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [悪い](#vue-no-useless-mustaches-bad) · [良い](#vue-no-useless-mustaches-good) | 文字列リテラルだけの不要な mustache を検出します。 |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [悪い](#vue-no-useless-template-attributes-bad) · [良い](#vue-no-useless-template-attributes-good) | template 要素の効果がない属性を検出します。 |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [悪い](#vue-no-useless-v-bind-bad) · [良い](#vue-no-useless-v-bind-good) | 文字列リテラルだけの不要な v-bind を検出します。 |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [悪い](#vue-no-v-for-template-key-on-child-bad) · [良い](#vue-no-v-for-template-key-on-child-good) | template v-for の key を子ではなく template に指定します。 |
| [`vue/no-v-html`](#vue-no-v-html) | [悪い](#vue-no-v-html-bad) · [良い](#vue-no-v-html-good) | 未処理の HTML を表示する v-html の XSS リスクを検出します。 |
| [`vue/no-v-text`](#vue-no-v-text) | [悪い](#vue-no-v-text-bad) · [良い](#vue-no-v-text-good) | v-text の代わりに mustache を使う方針を適用します。 |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [悪い](#vue-no-v-text-v-html-on-component-bad) · [良い](#vue-no-v-text-v-html-on-component-good) | コンポーネントでの v-text / v-html を検出します。 |
| [`vue/permitted-contents`](#vue-permitted-contents) | [悪い](#vue-permitted-contents-bad) · [良い](#vue-permitted-contents-good) | HTML の要素ごとのコンテンツモデルを検査します。 |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [悪い](#vue-prefer-props-shorthand-bad) · [良い](#vue-prefer-props-shorthand-good) | Vue 3.4 の同名 prop バインディングの省略形を使います。 |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [悪い](#vue-prefer-true-attribute-shorthand-bad) · [良い](#vue-prefer-true-attribute-shorthand-good) | true を指定するバインディングを boolean 属性の省略形にします。 |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [悪い](#vue-prop-name-casing-bad) · [良い](#vue-prop-name-casing-good) | 宣言する prop 名の形式を揃えます。 |
| [`vue/require-component-is`](#vue-require-component-is) | [悪い](#vue-require-component-is-bad) · [良い](#vue-require-component-is-good) | 動的 component 要素に :is を指定します。 |
| [`vue/require-component-registration`](#vue-require-component-registration) | [悪い](#vue-require-component-registration-bad) · [良い](#vue-require-component-registration-good) | 使用するコンポーネントを import または登録します。 |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [悪い](#vue-require-scoped-style-bad) · [良い](#vue-require-scoped-style-good) | style に scoped を指定する方針を適用します。 |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [悪い](#vue-require-toggle-inside-transition-bad) · [良い](#vue-require-toggle-inside-transition-good) | transition の子要素に表示を切り替える条件を指定します。 |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [悪い](#vue-require-v-for-key-bad) · [良い](#vue-require-v-for-key-good) | v-for に安定した :key を指定します。 |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [悪い](#vue-scoped-event-names-bad) · [良い](#vue-scoped-event-names-good) | イベント名を context:event の形式に揃えます。 |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [悪い](#vue-sfc-element-order-bad) · [良い](#vue-sfc-element-order-good) | SFC のトップレベルブロックを設定した順に並べます。 |
| [`vue/single-style-block`](#vue-single-style-block) | [悪い](#vue-single-style-block-bad) · [良い](#vue-single-style-block-good) | SFC の style を一つのブロックにまとめます。 |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [悪い](#vue-slot-name-casing-bad) · [良い](#vue-slot-name-casing-good) | 名前付き slot を kebab-case に揃えます。 |
| [`vue/this-in-template`](#vue-this-in-template) | [悪い](#vue-this-in-template-bad) · [良い](#vue-this-in-template-good) | テンプレートで不要な this. 参照を検出します。 |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い](#vue-use-unique-element-ids-bad) · [良い](#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [悪い](#vue-use-v-on-exact-bad) · [良い](#vue-use-v-on-exact-good) | modifier 付きのイベント操作と競合する handler に .exact を指定します。 |
| [`vue/v-bind-style`](#vue-v-bind-style) | [悪い](#vue-v-bind-style-bad) · [良い](#vue-v-bind-style-good) | v-bind の表記形式を揃えます。 |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [悪い](#vue-v-on-event-hyphenation-bad) · [良い](#vue-v-on-event-hyphenation-good) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [悪い](#vue-v-on-handler-style-bad) · [良い](#vue-v-on-handler-style-good) | イベント handler の参照・関数形式を揃えます。 |
| [`vue/v-on-style`](#vue-v-on-style) | [悪い](#vue-v-on-style-bad) · [良い](#vue-v-on-style-good) | v-on の表記形式を揃えます。 |
| [`vue/v-slot-style`](#vue-v-slot-style) | [悪い](#vue-v-slot-style-bad) · [良い](#vue-v-slot-style-good) | v-slot の表記形式を揃えます。 |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [悪い](#vue-valid-attribute-name-bad) · [良い](#vue-valid-attribute-name-good) | 有効な属性名を指定します。 |
| [`vue/valid-template-root`](#vue-valid-template-root) | [悪い](#vue-valid-template-root-bad) · [良い](#vue-valid-template-root-good) | Vue 3 の fragment に対応する有効なテンプレートルートを検査します。 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [悪い](#vue-valid-v-bind-bad) · [良い](#vue-valid-v-bind-good) | v-bind の引数・値・modifier を検査します。 |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [悪い](#vue-valid-v-cloak-bad) · [良い](#vue-valid-v-cloak-good) | v-cloak の引数・値・modifier を検査します。 |
| [`vue/valid-v-else`](#vue-valid-v-else) | [悪い](#vue-valid-v-else-bad) · [良い](#vue-valid-v-else-good) | v-else の位置・引数・値を検査します。 |
| [`vue/valid-v-for`](#vue-valid-v-for) | [悪い](#vue-valid-v-for-bad) · [良い](#vue-valid-v-for-good) | v-for の式と変数宣言を検査します。 |
| [`vue/valid-v-html`](#vue-valid-v-html) | [悪い](#vue-valid-v-html-bad) · [良い](#vue-valid-v-html-good) | v-html の値・引数・modifier を検査します。 |
| [`vue/valid-v-if`](#vue-valid-v-if) | [悪い](#vue-valid-v-if-bad) · [良い](#vue-valid-v-if-good) | v-if に有効な条件式を指定します。 |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [悪い](#vue-valid-v-memo-bad) · [良い](#vue-valid-v-memo-good) | v-memo の値を配列の式にします。 |
| [`vue/valid-v-model`](#vue-valid-v-model) | [悪い](#vue-valid-v-model-bad) · [良い](#vue-valid-v-model-good) | v-model の値・引数・modifier を検査します。 |
| [`vue/valid-v-on`](#vue-valid-v-on) | [悪い](#vue-valid-v-on-bad) · [良い](#vue-valid-v-on-good) | v-on のイベント名・式・modifier を検査します。 |
| [`vue/valid-v-once`](#vue-valid-v-once) | [悪い](#vue-valid-v-once-bad) · [良い](#vue-valid-v-once-good) | v-once の引数・値・modifier を検査します。 |
| [`vue/valid-v-show`](#vue-valid-v-show) | [悪い](#vue-valid-v-show-bad) · [良い](#vue-valid-v-show-good) | v-show に有効な条件式を指定します。 |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [悪い](#vue-valid-v-slot-bad) · [良い](#vue-valid-v-slot-good) | v-slot の適用先・宣言・modifier を検査します。 |
| [`vue/valid-v-text`](#vue-valid-v-text) | [悪い](#vue-valid-v-text-bad) · [良い](#vue-valid-v-text-good) | v-text の値・引数・modifier を検査します。 |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [悪い](#vue-warn-custom-block-bad) · [良い](#vue-warn-custom-block-good) | SFC のカスタムブロックを検出します。 |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [悪い](#vue-warn-custom-directive-bad) · [良い](#vue-warn-custom-directive-good) | 登録が必要なカスタムディレクティブを検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/a11y-img-alt`

画像に代替テキストの alt 属性を指定します。

[悪い例](#vue-a11y-img-alt-bad) · [良い例](#vue-a11y-img-alt-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-a11y-img-alt-bad"></span>

**悪い**

静的な画像にも src をバインドする画像にも alt がありません。

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**良い**

情報のある画像には説明、装飾には空の alt、動的な画像には説明のバインディングを指定します。

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [全ルール](all.md)

### `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

[悪い例](#vue-attribute-hyphenation-bad) · [良い例](#vue-attribute-hyphenation-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**悪い**

コンポーネントの属性名を camelCase の firstName にしています。

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**良い**

first-name に変更し、ハイフンで区切る既定の属性名の方針に合わせます。

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [全ルール](all.md)

### `vue/attribute-order`

テンプレートの属性を一定の順に並べます。

[悪い例](#vue-attribute-order-bad) · [良い例](#vue-attribute-order-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**悪い**

構造を決める v-if と通常の id 属性より前に、イベントを指定しています。

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**良い**

v-if、id、イベントの順に並べ、ルールの順序に合わせます。

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [全ルール](all.md)

### `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

[悪い例](#vue-component-definition-name-casing-bad) · [良い例](#vue-component-definition-name-casing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

対象はファイル名です。PascalCase と kebab-case は許可され、混在する形式は検出されます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**悪い**

ファイル名 myComponent.vue が先頭の小文字と途中の大文字を混在させ、PascalCase / kebab-case のどちらにもなっていません。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**良い**

ファイル名を PascalCase の MyComponent.vue に変更します。テンプレートの内容は同じです。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [全ルール](all.md)

### `vue/component-name-in-template-casing`

テンプレート内のコンポーネント名を指定した形式に揃えます。

[悪い例](#vue-component-name-in-template-casing-bad) · [良い例](#vue-component-name-in-template-casing-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**悪い**

PascalCase の方針に対し、コンポーネントを kebab-case と camelCase で記述しています。

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**良い**

MyComponent を PascalCase で記述します。標準の slot は小文字のままです。

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [全ルール](all.md)

### `vue/html-button-has-type`

button に有効な type を明示します。

[悪い例](#vue-html-button-has-type-bad) · [良い例](#vue-html-button-has-type-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-button-has-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-button-has-type-bad"></span>

**悪い**

一つは type を省略し、もう一つは対応しない foo を指定しています。

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**良い**

button、submit、reset を明示します。バインドする type は動的な値として扱います。

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [全ルール](all.md)

### `vue/html-quotes`

HTML 属性値の引用符を揃えます。

[悪い例](#vue-html-quotes-bad) · [良い例](#vue-html-quotes-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**悪い**

属性の値にシングルクォートを使うか、クォートを省略しています。

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**良い**

通常の属性とディレクティブの式をダブルクォートで囲みます。

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [全ルール](all.md)

### `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

[悪い例](#vue-html-self-closing-bad) · [良い例](#vue-html-self-closing-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**悪い**

空のコンポーネントを閉じタグの組で書き、img と br に既定の自己終了の表記を使っていません。

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**良い**

コンポーネントと void 要素を自己終了にします。内容がある div は閉じタグを残します。

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [全ルール](all.md)

### `vue/max-template-complexity`

コンポーネント自身のテンプレートの複雑度を制限します。

[悪い例](#vue-max-template-complexity-bad) · [良い例](#vue-max-template-complexity-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

悪い例は cyclomatic complexity 13、cognitive complexity 25 で、閾値 11 と 16 を超えます。コンポーネント単位で計測し、対象はインライン HTML テンプレートです。

例の二つの値の計算内訳は[複雑度の計算とコンポーネントの境界](../guide/cross-file-complexity.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-max-template-complexity-bad"></span>

**悪い**

親が記述する分岐、ループ、slot の内容、式の条件で 13 と 25 になり、既定の閾値 11 と 16 を超えています。

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

<span id="vue-max-template-complexity-good"></span>

**良い**

描画を RowList に分け、親には v-if を一つ残します。親自身の値は 2 と 1 になります。

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [全ルール](all.md)

### `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

[悪い例](#vue-multi-word-component-names-bad) · [良い例](#vue-multi-word-component-names-good)

既定の重大度: `error`  
プリセット: `essential`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

検出対象はファイル名です。同じコンポーネントのファイル名を変更します。子要素のタグ名は対象ではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**悪い**

Item.vue は一つの語だけのコンポーネント名です。

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**良い**

同じテンプレートの名前を、複数の語を持つ TodoItem.vue にします。

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [全ルール](all.md)

### `vue/mustache-interpolation-spacing`

mustache 内の空白を揃えます。

[悪い例](#vue-mustache-interpolation-spacing-bad) · [良い例](#vue-mustache-interpolation-spacing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**悪い**

補間の式と区切りの間で、片側または両側の空白がありません。

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**良い**

式と mustache の開始・終了の両方の区切りに空白を入れます。

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [全ルール](all.md)

### `vue/no-array-index-key`

v-for の配列インデックスをそのまま key に使う箇所を検出します。

[悪い例](#vue-no-array-index-key-bad) · [良い例](#vue-no-array-index-key-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-array-index-key-bad"></span>

**悪い**

現在の配列の位置を key に使い、並び替えで項目の識別子が変わります。

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**良い**

item.id を key に使い、位置が変わっても項目の識別子を維持します。

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [全ルール](all.md)

### `vue/no-bare-strings-in-template`

国際化すべきテンプレートの直接指定テキストを検出します。

[悪い例](#vue-no-bare-strings-in-template-bad) · [良い例](#vue-no-bare-strings-in-template-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-bare-strings-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-bare-strings-in-template-bad"></span>

**悪い**

表示する文字や名前を指定する属性に、翻訳を通さない文字列を直接書いています。

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**良い**

翻訳する内容を $t で取得します。記号だけ、数値だけの内容は例外です。

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [全ルール](all.md)

### `vue/no-boolean-attr-value`

HTML の boolean 属性に不要な値を指定した箇所を検出します。

[悪い例](#vue-no-boolean-attr-value-bad) · [良い例](#vue-no-boolean-attr-value-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**悪い**

boolean の disabled と checked に、不要な文字列の値を指定しています。

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**良い**

値を付けず、boolean 属性があることだけで同じ状態を表します。

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [全ルール](all.md)

### `vue/no-child-content`

v-html / v-text と子コンテンツを同時に指定する箇所を検出します。

[悪い例](#vue-no-child-content-bad) · [良い例](#vue-no-child-content-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**悪い**

v-text が p の内容を置き換えるため、中に書いた fallback の文字を表示できません。

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**良い**

子の文字を取り除き、p の内容を v-text だけで指定します。

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [全ルール](all.md)

### `vue/no-deprecated-filter`

Vue 2 の pipe による filter 構文を検出します。

[悪い例](#vue-no-deprecated-filter-bad) · [良い例](#vue-no-deprecated-filter-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-filter-bad"></span>

**悪い**

capitalize を適用するために、廃止された filter のパイプ構文を使っています。

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**良い**

通常の式で capitalize(message) を呼び出します。

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [全ルール](all.md)

### `vue/no-deprecated-functional-template`

SFC の template で削除済みの functional 属性を検出します。

[悪い例](#vue-no-deprecated-functional-template-bad) · [良い例](#vue-no-deprecated-functional-template-good)

既定の重大度: `error`  
プリセット: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-functional-template-bad"></span>

**悪い**

SFC の template に廃止された functional を指定し、旧来の props の参照を使っています。

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**良い**

functional を取り除き、コンポーネントの msg を直接参照します。

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [全ルール](all.md)

### `vue/no-deprecated-html-element-is`

通常の HTML 要素で旧形式の is を使う箇所を検出します。

[悪い例](#vue-no-deprecated-html-element-is-bad) · [良い例](#vue-no-deprecated-html-element-is-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-html-element-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-html-element-is-bad"></span>

**悪い**

標準の div で、接頭辞がない旧来の is 属性から Vue コンポーネントを指定しています。

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**良い**

動的な component では :is を使い、標準要素では vue: の接頭辞を明示します。

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [全ルール](all.md)

### `vue/no-deprecated-inline-template`

削除済みの inline-template 属性を検出します。

[悪い例](#vue-no-deprecated-inline-template-bad) · [良い例](#vue-no-deprecated-inline-template-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-inline-template-bad"></span>

**悪い**

Card に渡す内容に、廃止された inline-template を指定しています。

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**良い**

inline-template を取り除き、同じ内容を通常の形で渡します。

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [全ルール](all.md)

### `vue/no-deprecated-router-link-tag-prop`

router-link の削除済み tag prop を検出します。

[悪い例](#vue-no-deprecated-router-link-tag-prop-bad) · [良い例](#vue-no-deprecated-router-link-tag-prop-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

**悪い**

RouterLink の廃止された tag で button を指定しています。

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**良い**

slot から navigate を受け取り、明示的に記述した button で実行します。

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [全ルール](all.md)

### `vue/no-deprecated-scope-attribute`

template の削除済み scope 属性を検出します。

[悪い例](#vue-no-deprecated-scope-attribute-bad) · [良い例](#vue-no-deprecated-scope-attribute-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-scope-attribute-bad"></span>

**悪い**

slot の template に、廃止された scope で props を宣言しています。

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**良い**

現在の default slot のディレクティブで、同じ props を受け取ります。

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [全ルール](all.md)

### `vue/no-deprecated-slot-attribute`

削除済みの slot 属性を検出します。

[悪い例](#vue-no-deprecated-slot-attribute-bad) · [良い例](#vue-no-deprecated-slot-attribute-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-attribute-bad"></span>

**悪い**

旧来の slot 属性で header の slot を選んでいます。

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**良い**

現在の v-slot:header で header の slot を指定します。

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [全ルール](all.md)

### `vue/no-deprecated-slot-scope-attribute`

削除済みの slot-scope 属性を検出します。

[悪い例](#vue-no-deprecated-slot-scope-attribute-bad) · [良い例](#vue-no-deprecated-slot-scope-attribute-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

**悪い**

廃止された slot-scope で slot の props を受け取っています。

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**良い**

#default で同じ props を受け取り、slot-scope を取り除きます。

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [全ルール](all.md)

### `vue/no-deprecated-v-bind-sync`

削除済みの v-bind の .sync modifier を検出します。

[悪い例](#vue-no-deprecated-v-bind-sync-bad) · [良い例](#vue-no-deprecated-v-bind-sync-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

**悪い**

.camel と組み合わせたものも含め、廃止された .sync を使っています。

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**良い**

一方向なら通常の title のバインディング、更新の受け取りが必要なら v-model:title を使います。

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [全ルール](all.md)

### `vue/no-deprecated-v-on-native-modifier`

削除済みの v-on の .native modifier を検出します。

[悪い例](#vue-no-deprecated-v-on-native-modifier-bad) · [良い例](#vue-no-deprecated-v-on-native-modifier-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

**悪い**

コンポーネントのイベントに、廃止された .native を指定しています。

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**良い**

.native を取り除き、.stop などの他の modifier は残します。

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [全ルール](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済み数値 keyCode modifier を検出します。

[悪い例](#vue-no-deprecated-v-on-number-modifiers-bad) · [良い例](#vue-no-deprecated-v-on-number-modifiers-good)

既定の重大度: `error`  
プリセット: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

**悪い**

キーを、廃止された数値コード 13 と 27 で指定しています。

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**良い**

キーの名前を使い、enter と esc の modifier に変更します。

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [全ルール](all.md)

### `vue/no-dupe-v-else-if`

v-if / v-else-if の条件重複を検出します。

[悪い例](#vue-no-dupe-v-else-if-bad) · [良い例](#vue-no-dupe-v-else-if-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**悪い**

最初と同じ ready の条件を else-if に書き、後の分岐に到達できません。

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**良い**

二つ目で別の loading の状態を検査し、else-if に到達できる条件にします。

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [全ルール](all.md)

### `vue/no-duplicate-attributes`

同じ要素の属性重複を検出します。

[悪い例](#vue-no-duplicate-attributes-bad) · [良い例](#vue-no-duplicate-attributes-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**悪い**

一つの button に class を二回指定しています。

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**良い**

二つのクラスを一つの class 属性にまとめます。

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [全ルール](all.md)

### `vue/no-empty-component-block`

空の SFC ブロックを検出します。

[悪い例](#vue-no-empty-component-block-bad) · [良い例](#vue-no-empty-component-block-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-empty-component-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-empty-component-block-bad"></span>

**悪い**

template、script、style のブロックが空か、空白だけです。

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**良い**

残すブロックには、マークアップ、script の宣言、style の宣言を入れます。

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [全ルール](all.md)

### `vue/no-inline-style`

インラインの style 属性を検出します。

[悪い例](#vue-no-inline-style-bad) · [良い例](#vue-no-inline-style-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**悪い**

静的な style 属性に色の宣言を直接書いています。

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**良い**

固定した色にはクラスを使います。ratio に依存する幅の動的な style は、静的属性の検査の対象外です。

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [全ルール](all.md)

### `vue/no-invalid-html-attribute`

静的 HTML 属性の無効な値を検出します。現在は rel が対象です。

[悪い例](#vue-no-invalid-html-attribute-bad) · [良い例](#vue-no-invalid-html-attribute-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-invalid-html-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-invalid-html-attribute-bad"></span>

**悪い**

a の rel に、stylesheet の link 要素向けの stylesheet を指定しています。

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**良い**

a の rel に、ヘルプへの参照を表す help を指定します。

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [全ルール](all.md)

### `vue/no-lone-template`

不要な template 要素を検出します。

[悪い例](#vue-no-lone-template-bad) · [良い例](#vue-no-lone-template-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**悪い**

内側の template に、構造や slot を指定する役割がありません。

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**良い**

不要な template を取り除き、div に p を直接入れます。

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [全ルール](all.md)

### `vue/no-multi-spaces`

連続する不要な空白を検出します。

[悪い例](#vue-no-multi-spaces-bad) · [良い例](#vue-no-multi-spaces-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**悪い**

属性同士や、要素名と最初の属性の間に空白が二つあります。

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**良い**

同じ class と id 属性を保ち、属性間の連続した空白を一つの空白に揃えます。

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [全ルール](all.md)

### `vue/no-multiple-objects-in-class`

:class 配列内の複数のオブジェクト指定をまとめます。

[悪い例](#vue-no-multiple-objects-in-class-bad) · [良い例](#vue-no-multiple-objects-in-class-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-objects-in-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-objects-in-class-bad"></span>

**悪い**

class の配列に、まとめられる object literal を二つ直接指定しています。

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**良い**

条件を一つの object にまとめます。object 一つと文字列の組、literal ではない要素の配列は許可されます。

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [全ルール](all.md)

### `vue/no-multiple-template-root`

単一ルートを要求するテンプレートで複数ルートを検出します。

[悪い例](#vue-no-multiple-template-root-bad) · [良い例](#vue-no-multiple-template-root-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

単一ルートを要求する場合に有効にします。通常の Vue 3 は複数ルートを許可します。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-template-root-bad"></span>

**悪い**

単一ルートを要求する設定で、テンプレートのルートに p が二つ並んでいます。

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**良い**

section で囲み、ルートを一つにします。単一ルートの契約が必要な場合にだけ有効にする規約です。

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [全ルール](all.md)

### `vue/no-mutating-props`

親から受け取った prop を子コンポーネントで変更する箇所を検出します。

[悪い例](#vue-no-mutating-props-bad) · [良い例](#vue-no-mutating-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**悪い**

props.count の加算で、親から受け取った値を直接変更しています。

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**良い**

次の値を update:count で通知し、prop の変更は親が行う形にします。

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [全ルール](all.md)

### `vue/no-negated-v-if-condition`

v-else がある条件分岐の否定条件を反転して読みやすくします。

[悪い例](#vue-no-negated-v-if-condition-bad) · [良い例](#vue-no-negated-v-if-condition-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-negated-v-if-condition": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-negated-v-if-condition-bad"></span>

**悪い**

v-else と対になる v-if を、否定した条件で始めています。

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**良い**

先に正の ok の条件を使います。条件を反転する際は分岐の内容も入れ替えます。単独の否定の v-if や !== の比較は許可されます。

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [全ルール](all.md)

### `vue/no-non-component-keep-alive-child`

KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。

[悪い例](#vue-no-non-component-keep-alive-child-bad) · [良い例](#vue-no-non-component-keep-alive-child-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**悪い**

KeepAlive の条件付きの子が UserCard ではなく、標準の div になっています。

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**良い**

最初の例は UserCard を条件付きの子にします。v-show の wrapper はこの条件付きの子の検査の対象外を示す例で、標準要素がキャッシュされるという意味ではありません。

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [全ルール](all.md)

### `vue/no-preprocessor-lang`

CSS preprocessor より標準の CSS を使う方針を適用します。

[悪い例](#vue-no-preprocessor-lang-bad) · [良い例](#vue-no-preprocessor-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の検出を生成しません。悪い例・良い例は意図した規約の説明で、実行すると検出される例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

**設定できる ID（現在の SFC 検出なし）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**悪い**

style の lang に SCSS を指定しています。preprocessor を使わない規約の例で、現在の SFC 検査はこのルールを生成しません。

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**良い**

同じ CSS から preprocessor の lang を取り除きます。規約の修正例で、現在の実行結果の診断の違いを示すものではありません。

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [全ルール](all.md)

### `vue/no-reserved-component-names`

予約済みのコンポーネント名を検出します。

[悪い例](#vue-no-reserved-component-names-bad) · [良い例](#vue-no-reserved-component-names-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**悪い**

コンポーネント名の button が、標準の HTML 要素名と競合しています。

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**良い**

標準の button と重複しない、アプリの AppButton の名前を指定します。

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [全ルール](all.md)

### `vue/no-root-v-if`

テンプレートの単一ルートに v-if を指定する箇所を検出します。

[悪い例](#vue-no-root-v-if-bad) · [良い例](#vue-no-root-v-if-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-root-v-if-bad"></span>

**悪い**

コンポーネントのルート自体を v-if で表示・非表示にしています。

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**良い**

外側の div をルートとして残し、内側の p に表示条件を指定します。

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [全ルール](all.md)

### `vue/no-script-non-standard-lang`

script の非標準 lang 指定を検出します。

[悪い例](#vue-no-script-non-standard-lang-bad) · [良い例](#vue-no-script-non-standard-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の検出を生成しません。悪い例・良い例は意図した規約の説明で、実行すると検出される例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

**設定できる ID（現在の SFC 検出なし）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**悪い**

lang=coffee で CoffeeScript の構文を使っています。現在の SFC 検査はこの言語に対して、このカタログのルールを生成しません。

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**良い**

lang=ts と通常の TypeScript の宣言を使い、意図した言語の方針を示します。

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [全ルール](all.md)

### `vue/no-src-attribute`

SFC ブロックの外部 src 指定を検出します。

[悪い例](#vue-no-src-attribute-bad) · [良い例](#vue-no-src-attribute-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**悪い**

SFC の template、script、style を src の外部ファイルに分けています。

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**良い**

各ブロックに内容を記述し、外部の src 属性を使いません。

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [全ルール](all.md)

### `vue/no-static-inline-styles`

静的なインライン style 属性を検出します。

[悪い例](#vue-no-static-inline-styles-bad) · [良い例](#vue-no-static-inline-styles-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-static-inline-styles-bad"></span>

**悪い**

p の style 属性に、変化しない色を直接指定しています。

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**良い**

notice のクラスと scoped の CSS に、変化しない色を移します。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [全ルール](all.md)

### `vue/no-template-key`

v-for 用ではない template の key 指定を検出します。

[悪い例](#vue-no-template-key-bad) · [良い例](#vue-no-template-key-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**悪い**

繰り返しではない template に key を指定しています。

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**良い**

template の v-for に key を付け、繰り返す各 fragment を識別します。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [全ルール](all.md)

### `vue/no-template-lang`

template の lang 指定を検出します。

[悪い例](#vue-no-template-lang-bad) · [良い例](#vue-no-template-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の検出を生成しません。悪い例・良い例は意図した規約の説明で、実行すると検出される例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

**設定できる ID（現在の SFC 検出なし）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**悪い**

template の lang に Pug を指定しています。HTML のみを使う規約の例で、現在の SFC 検査はこの ID を検出しません。

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**良い**

lang を取り除き、通常の HTML の p を直接記述します。規約の例で、現在の SFC の診断を約束するものではありません。

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [全ルール](all.md)

### `vue/no-template-shadow`

テンプレート変数が外側の名前を隠す箇所を検出します。

[悪い例](#vue-no-template-shadow-bad) · [良い例](#vue-no-template-shadow-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の検査は入れ子になった v-for の変数を比較します。script 内の名前と同じという理由だけで単独の v-for を検出するものではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**悪い**

内側の v-for でも item を宣言し、外側の item を隠しています。

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**良い**

内側は child に変更し、外側の item と内側の child を分けます。

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全ルール](all.md)

### `vue/no-template-target-blank`

target=_blank の外部リンクに適切な rel を指定します。

[悪い例](#vue-no-template-target-blank-bad) · [良い例](#vue-no-template-target-blank-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-target-blank-bad"></span>

**悪い**

外部リンクを target=_blank で開きますが、必要な rel の保護がありません。

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**良い**

同じリンクに noopener noreferrer を指定します。

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [全ルール](all.md)

### `vue/no-textarea-mustache`

textarea 内の mustache を検出し、v-model の使用を勧めます。

[悪い例](#vue-no-textarea-mustache-bad) · [良い例](#vue-no-textarea-mustache-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**悪い**

textarea の値をバインドせず、子の補間に message を記述しています。

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**良い**

v-model で、編集する textarea の値と message を関連付けます。

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [全ルール](all.md)

### `vue/no-undefined-refs`

テンプレート内の未定義変数参照を検出します。

[悪い例](#vue-no-undefined-refs-bad) · [良い例](#vue-no-undefined-refs-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-undefined-refs-bad"></span>

**悪い**

script は message しか宣言していませんが、テンプレートが missing を参照しています。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**良い**

script setup で宣言済みの message を補間で参照し、未定義の名前を取り除きます。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [全ルール](all.md)

### `vue/no-unsafe-url`

危険なスキームになり得る URL 属性やバインディングを検出します。

[悪い例](#vue-no-unsafe-url-bad) · [良い例](#vue-no-unsafe-url-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**悪い**

a の移動先に、実行可能な javascript: のスキームを使っています。

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**良い**

実行可能な URL を除き、通常のローカルの移動先 /next に変更します。

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [全ルール](all.md)

### `vue/no-unsandboxed-iframe`

iframe に sandbox 属性を指定します。

[悪い例](#vue-no-unsandboxed-iframe-bad) · [良い例](#vue-no-unsandboxed-iframe-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsandboxed-iframe-bad"></span>

**悪い**

iframe に sandbox がなく、機能の制限を指定していません。

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**良い**

sandbox で制限し、script が必要な場合にだけ allow-scripts を明示します。

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [全ルール](all.md)

### `vue/no-unused-components`

登録しているのにテンプレートで使わないコンポーネントを検出します。

[悪い例](#vue-no-unused-components-bad) · [良い例](#vue-no-unused-components-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**悪い**

UserAvatar をコンポーネントとして import していますが、テンプレートで使っていません。

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**良い**

import した UserAvatar をテンプレートで表示し、user を渡します。

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [全ルール](all.md)

### `vue/no-unused-properties`

defineProps に宣言しているのに使わない prop を検出します。

[悪い例](#vue-no-unused-properties-bad) · [良い例](#vue-no-unused-properties-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**悪い**

description の prop を宣言していますが、表示には title しか使っていません。

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**良い**

宣言した両方の prop をテンプレートで参照します。

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [全ルール](all.md)

### `vue/no-unused-refs`

テンプレートに宣言しているのに参照しない ref を検出します。

[悪い例](#vue-no-unused-refs-bad) · [良い例](#vue-no-unused-refs-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-refs-bad"></span>

**悪い**

テンプレートの unused の ref に対応する script の参照がありません。

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**良い**

inputEl のテンプレート ref に、script setup の同じ名前の ref を対応させます。

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [全ルール](all.md)

### `vue/no-unused-setup-bindings`

script setup に宣言しているのに読み取らない変数を検出します。

[悪い例](#vue-no-unused-setup-bindings-bad) · [良い例](#vue-no-unused-setup-bindings-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-setup-bindings-bad"></span>

**悪い**

script setup の message をテンプレートで使っていません。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**良い**

script setup で宣言した message を p の補間で参照し、未使用の宣言を残しません。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [全ルール](all.md)

### `vue/no-unused-vars`

v-for / v-slot に宣言しているのに使わない変数を検出します。

[悪い例](#vue-no-unused-vars-bad) · [良い例](#vue-no-unused-vars-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**悪い**

ループで使わない index と、slot で使わない foo を宣言しています。

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**良い**

index を参照するか、意図して使わない _index にし、slot は data を表示します。index の key は使用の例であり、項目の安定した識別子として推奨するものではありません。

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [全ルール](all.md)

### `vue/no-use-v-else-with-v-for`

同じ要素での v-else / v-else-if と v-for の併用を検出します。

[悪い例](#vue-no-use-v-else-with-v-for-bad) · [良い例](#vue-no-use-v-else-with-v-for-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-else-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-else-with-v-for-bad"></span>

**悪い**

同じ p に v-else と v-for を指定しています。

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**良い**

template に v-else を分け、その子の p に v-for を指定します。

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [全ルール](all.md)

### `vue/no-use-v-if-with-v-for`

同じ要素での v-if と v-for の併用を検出します。

[悪い例](#vue-no-use-v-if-with-v-for-bad) · [良い例](#vue-no-use-v-if-with-v-for-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**悪い**

同じ li に v-if と v-for を指定し、ループの変数で表示条件を検査しています。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**良い**

computed で表示する項目を先に絞り込み、テンプレートはその配列を繰り返します。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [全ルール](all.md)

### `vue/no-useless-mustaches`

文字列リテラルだけの不要な mustache を検出します。

[悪い例](#vue-no-useless-mustaches-bad) · [良い例](#vue-no-useless-mustaches-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-mustaches": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-mustaches-bad"></span>

**悪い**

補間の内容が固定の文字列だけで、式の評価が不要です。

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**良い**

固定の文字は直接書きます。変数、値を埋め込む template string、区切りの空白の補間は残します。

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [全ルール](all.md)

### `vue/no-useless-template-attributes`

template 要素の効果がない属性を検出します。

[悪い例](#vue-no-useless-template-attributes-bad) · [良い例](#vue-no-useless-template-attributes-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**悪い**

条件付きの template に class を指定していますが、この構造用の wrapper は DOM 要素を表示しません。

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**良い**

実際に表示する p に class を移し、構造を指定する template の v-if は残します。

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [全ルール](all.md)

### `vue/no-useless-v-bind`

文字列リテラルだけの不要な v-bind を検出します。

[悪い例](#vue-no-useless-v-bind-bad) · [良い例](#vue-no-useless-v-bind-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-v-bind": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-v-bind-bad"></span>

**悪い**

foo に固定の文字列か、補間がない template string をバインドしています。

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**良い**

固定の値は静的な属性にし、変数や補間がある値はバインディングを残します。

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [全ルール](all.md)

### `vue/no-v-for-template-key-on-child`

template v-for の key を子ではなく template に指定します。

[悪い例](#vue-no-v-for-template-key-on-child-bad) · [良い例](#vue-no-v-for-template-key-on-child-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-for-template-key-on-child-bad"></span>

**悪い**

繰り返す template に key がなく、子の p に指定しています。

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**良い**

template の v-for に key を移し、繰り返す fragment 全体を識別します。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [全ルール](all.md)

### `vue/no-v-html`

未処理の HTML を表示する v-html の XSS リスクを検出します。

[悪い例](#vue-no-v-html-bad) · [良い例](#vue-no-v-html-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**悪い**

v-html で content を通常の文字ではなく HTML として扱っています。

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**良い**

mustache の補間で、HTML を挿入せず content をエスケープした文字として表示します。

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [全ルール](all.md)

### `vue/no-v-text`

v-text の代わりに mustache を使う方針を適用します。

[悪い例](#vue-no-v-text-bad) · [良い例](#vue-no-v-text-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-bad"></span>

**悪い**

要素の内容を v-text のディレクティブで指定しています。

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**良い**

同じ文字のバインディングを、要素の内容の mustache で指定します。

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [全ルール](all.md)

### `vue/no-v-text-v-html-on-component`

コンポーネントでの v-text / v-html を検出します。

[悪い例](#vue-no-v-text-v-html-on-component-bad) · [良い例](#vue-no-v-text-v-html-on-component-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**悪い**

コンポーネントに、要素の内容を置き換える v-html や v-text を指定しています。

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**良い**

標準の HTML 要素にはディレクティブを使えます。MyComponent には default slot から内容を渡します。

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [全ルール](all.md)

### `vue/permitted-contents`

HTML の要素ごとのコンテンツモデルを検査します。

[悪い例](#vue-permitted-contents-bad) · [良い例](#vue-permitted-contents-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**悪い**

p にブロックを入れ、tbody を省略し、操作要素を入れ子にするか、ul に div を直接入れています。

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**良い**

p はインラインの内容、table は tbody、ul は li を使います。独自の MyItem は既知の標準の ul の子として検査されません。

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [全ルール](all.md)

### `vue/prefer-props-shorthand`

Vue 3.4 の同名 prop バインディングの省略形を使います。

[悪い例](#vue-prefer-props-shorthand-bad) · [良い例](#vue-prefer-props-shorthand-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**悪い**

ハイフン区切りに対応する camelCase も含め、属性と対応する変数名を式に重複して書いています。

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**良い**

Vue 3.4 以降の同名の省略形で式を省きます。bar のように別の変数を渡す式は明示します。

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [全ルール](all.md)

### `vue/prefer-true-attribute-shorthand`

true を指定するバインディングを boolean 属性の省略形にします。

[悪い例](#vue-prefer-true-attribute-shorthand-bad) · [良い例](#vue-prefer-true-attribute-shorthand-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-true-attribute-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

**悪い**

標準の boolean 属性 disabled に、固定の true をバインドしています。

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**良い**

標準の boolean 属性は省略形にします。false のバインディングやコンポーネントの prop は値を残します。

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [全ルール](all.md)

### `vue/prop-name-casing`

宣言する prop 名の形式を揃えます。

[悪い例](#vue-prop-name-casing-bad) · [良い例](#vue-prop-name-casing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

宣言した props の名前を検査します。子に渡す属性名の表記を検査するルールではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**悪い**

宣言した prop の user_name をアンダースコア区切りにしています。

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**良い**

宣言とテンプレートの参照を、camelCase の userName に揃えます。

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [全ルール](all.md)

### `vue/require-component-is`

動的 component 要素に :is を指定します。

[悪い例](#vue-require-component-is-bad) · [良い例](#vue-require-component-is-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**悪い**

動的な `<component>` に `is` がなく、描画するコンポーネントを選べません。

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**良い**

`:is="currentComponent"` で描画対象を指定します。対象は実行時に変更できます。

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [全ルール](all.md)

### `vue/require-component-registration`

使用するコンポーネントを import または登録します。

[悪い例](#vue-require-component-registration-bad) · [良い例](#vue-require-component-registration-good)

既定の重大度: `warning`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

application plugin や Musea previewSetup が登録する component 名を明示します。PascalCase と kebab-case を許可し、正規表現は解釈しません。option だけではルールは有効になりません。後の設定は list 全体を置き換え、空 list は継承した名前を消します。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-registration-bad"></span>

**悪い**

`MissingWidget` は登録されておらず、設定したグローバル コンポーネント一覧にもありません。

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**良い**

`MyButton` は例の `globals` に含まれます。既知のグローバル登録を検査対象から除く設定で、import や登録そのものは行いません。

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [全ルール](all.md)

### `vue/require-scoped-style`

style に scoped を指定する方針を適用します。

[悪い例](#vue-require-scoped-style-bad) · [良い例](#vue-require-scoped-style-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**悪い**

`.button` の style にスコープがなく、他のコンポーネントの一致する要素にも作用します。

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**良い**

同じセレクタと宣言に `scoped` を付け、Vue のコンポーネント スコープを適用します。

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [全ルール](all.md)

### `vue/require-toggle-inside-transition`

transition の子要素に表示を切り替える条件を指定します。

[悪い例](#vue-require-toggle-inside-transition-bad) · [良い例](#vue-require-toggle-inside-transition-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-toggle-inside-transition-bad"></span>

**悪い**

`<Transition>` 内の静的な子に表示の切り替えや動的な選択がなく、enter / leave が発生する条件がありません。

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**良い**

`v-if="show"` で子の有無を切り替え、enter / leave の対象にします。

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [全ルール](all.md)

### `vue/require-v-for-key`

v-for に安定した :key を指定します。

[悪い例](#vue-require-v-for-key-bad) · [良い例](#vue-require-v-for-key-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**悪い**

繰り返す `<li>` に key がなく、一覧更新時に対応する項目を識別できません。

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**良い**

`:key="item.id"` で、現在の位置ではなく項目の識別子を各ノードに付けます。

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [全ルール](all.md)

### `vue/scoped-event-names`

イベント名を context:event の形式に揃えます。

[悪い例](#vue-scoped-event-names-bad) · [良い例](#vue-scoped-event-names-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**悪い**

`playAudio`、`pauseAudio`、`reloadAudio` は camelCase の末尾に対象を付けており、このルールのコロン区切りの規約に合いません。

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**良い**

`audio:play`、`audio:pause`、`audio:reload` に `audio:` のスコープを明示します。emit 側も同じ名前に合わせます。

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [全ルール](all.md)

### `vue/sfc-element-order`

SFC のトップレベルブロックを設定した順に並べます。

[悪い例](#vue-sfc-element-order-bad) · [良い例](#vue-sfc-element-order-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**悪い**

style が script より前にあり、設定した SFC ブロック順に合いません。

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**良い**

script → template → style の順に並べます。型付きオプションで別の順序を選べます。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [全ルール](all.md)

### `vue/single-style-block`

SFC の style を一つのブロックにまとめます。

[悪い例](#vue-single-style-block-bad) · [良い例](#vue-single-style-block-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**悪い**

panel と title の scoped スタイルを二つの style ブロックに分けています。

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**良い**

両方のセレクタを一つの scoped style にまとめ、どちらのスタイルも残します。

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [全ルール](all.md)

### `vue/slot-name-casing`

名前付き slot を kebab-case に揃えます。

[悪い例](#vue-slot-name-casing-bad) · [良い例](#vue-slot-name-casing-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-slot-name-casing-bad"></span>

**悪い**

名前付きスロット `mySlot` が camelCase で、ハイフン区切りの規約に合いません。

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**良い**

`#my-slot` を kebab-case にします。受け取る slot の名前も合わせます。

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [全ルール](all.md)

### `vue/this-in-template`

テンプレートで不要な this. 参照を検出します。

[悪い例](#vue-this-in-template-bad) · [良い例](#vue-this-in-template-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-this-in-template-bad"></span>

**悪い**

テンプレートで直接使える binding を `this.message`、`this.className`、`this.handleClick` と明示的に参照しています。

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**良い**

`message`、`className`、`handleClick` を直接使います。文字列 `'this.is.a.string'` はメンバー参照ではないため残します。

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [全ルール](all.md)

### `vue/use-unique-element-ids`

静的 ID の代わりに useId() で再利用可能な ID を生成します。

[悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**悪い**

固定の `email` ID がコンポーネントの各インスタンスで重複し、複数表示時に label の参照先が曖昧になります。

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**良い**

`useId()` の `emailId` を label の `for` と input の `id` の両方に binding します。

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全ルール](all.md)

### `vue/use-v-on-exact`

modifier 付きのイベント操作と競合する handler に .exact を指定します。

[悪い例](#vue-use-v-on-exact-bad) · [良い例](#vue-use-v-on-exact-good)

既定の重大度: `warning`  
プリセット: `essential`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**悪い**

通常の click handler も Ctrl-click で動くため、別の `.ctrl` handler と重複します。

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**良い**

`.exact` で通常の handler を修飾キーのない click に限定し、Ctrl 専用の handler と分けます。

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [全ルール](all.md)

### `vue/v-bind-style`

v-bind の表記形式を揃えます。

[悪い例](#vue-v-bind-style-bad) · [良い例](#vue-v-bind-style-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**悪い**

`v-bind:class` が長い形式で、設定したコロン省略形の規約に合いません。

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**良い**

同じ式を `:class` にします。値の型ではなく directive の書き方を検査するルールです。

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [全ルール](all.md)

### `vue/v-on-event-hyphenation`

コンポーネントのカスタムイベント名を設定した形式に揃えます。

[悪い例](#vue-v-on-event-hyphenation-bad) · [良い例](#vue-v-on-event-hyphenation-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-event-hyphenation-bad"></span>

**悪い**

カスタム コンポーネントの listener がハイフン区切りではなく `@myEvent` です。

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**良い**

`@my-event` に変更します。例のネイティブ要素の listener と動的なイベント引数はこの検査の対象外です。

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [全ルール](all.md)

### `vue/v-on-handler-style`

イベント handler の参照・関数形式を揃えます。

[悪い例](#vue-v-on-handler-style-bad) · [良い例](#vue-v-on-handler-style-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-handler-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-handler-style-bad"></span>

**悪い**

イベント属性の中に変更処理や複数の文を直接記述しています。

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**良い**

handler の参照を使うか、インライン処理が必要なら arrow / function 式で関数の境界を明示します。

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [全ルール](all.md)

### `vue/v-on-style`

v-on の表記形式を揃えます。

[悪い例](#vue-v-on-style-bad) · [良い例](#vue-v-on-style-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**悪い**

`v-on:click` が長い形式で、このルールの省略形に合いません。

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**良い**

handler を変えずに `@click` の省略形を使います。

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [全ルール](all.md)

### `vue/v-slot-style`

v-slot の表記形式を揃えます。

[悪い例](#vue-v-slot-style-bad) · [良い例](#vue-v-slot-style-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**悪い**

コンポーネントで `#default`、template で `v-slot:header` を使い、場所ごとの規約と逆になっています。

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**良い**

コンポーネントの default slot は `v-slot`、template の名前付き slot は `#header` にします。

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全ルール](all.md)

### `vue/valid-attribute-name`

有効な属性名を指定します。

[悪い例](#vue-valid-attribute-name-bad) · [良い例](#vue-valid-attribute-name-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

悪い例での診断: `parser/template`

不正な属性名は、この防御的なルールに届く前に parser/template で検出されます。悪い例で確認するのは parser/template の検出で、vue/valid-attribute-name が別に出るとは限りません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**悪い**

`my"attr` の引用符で属性名が壊れています。この例の診断は `parser/template` で、別のルール診断の生成を約束するものではありません。

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**良い**

`my-attr` は正しい属性名で、テンプレート parser が属性と値を読み取れます。

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [全ルール](all.md)

### `vue/valid-template-root`

Vue 3 の fragment に対応する有効なテンプレートルートを検査します。

[悪い例](#vue-valid-template-root-bad) · [良い例](#vue-valid-template-root-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-template-root-bad"></span>

**悪い**

描画上の役割を与える directive がない通常の `<template>` を、テンプレートのルートに置いています。

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**良い**

描画される `<div>` をルートにします。Vue 3 の fragment 全般を一つのルートに制限する例ではありません。

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [全ルール](all.md)

### `vue/valid-v-bind`

v-bind の引数・値・modifier を検査します。

[悪い例](#vue-valid-v-bind-bad) · [良い例](#vue-valid-v-bind-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-bind": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-bind-bad"></span>

**悪い**

引数のない `v-bind` に object 式がなく、空の引数形式には属性名がありません。

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**良い**

属性と式を指定するか object を binding します。Vue 3.4 以降では `:loading` の同名省略形も使えます。

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [全ルール](all.md)

### `vue/valid-v-cloak`

v-cloak の引数・値・modifier を検査します。

[悪い例](#vue-valid-v-cloak-bad) · [良い例](#vue-valid-v-cloak-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-cloak": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-cloak-bad"></span>

**悪い**

値・引数・修飾子を受け取らない `v-cloak` に、それらを指定しています。

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**良い**

値のない `v-cloak` を使います。mount 後に Vue が属性を除くまで CSS で非表示にできます。

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [全ルール](all.md)

### `vue/valid-v-else`

v-else の位置・引数・値を検査します。

[悪い例](#vue-valid-v-else-bad) · [良い例](#vue-valid-v-else-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-else-bad"></span>

**悪い**

`v-else` に式を渡す、`v-if` と併用する、直前の条件分岐がない、といった不正な組み合わせです。

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**良い**

対応する `v-if` の直後に、値のない `v-else` を置きます。

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [全ルール](all.md)

### `vue/valid-v-for`

v-for の式と変数宣言を検査します。

[悪い例](#vue-valid-v-for-bad) · [良い例](#vue-valid-v-for-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-for-bad"></span>

**悪い**

繰り返し式がないか、対応していない `.stop` 修飾子を付けています。

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**良い**

`item in items` や `(item, index) of items` の完全な式と、例の key を使います。

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [全ルール](all.md)

### `vue/valid-v-html`

v-html の値・引数・modifier を検査します。

[悪い例](#vue-valid-v-html-bad) · [良い例](#vue-valid-v-html-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-html-bad"></span>

**悪い**

`v-html` の式がないか、対応していない引数・修飾子を指定しています。

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**良い**

`v-html="html"` で正しい式を渡します。構文が正しくても HTML の無害化や未信頼の内容の安全性は保証されません。

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [全ルール](all.md)

### `vue/valid-v-if`

v-if に有効な条件式を指定します。

[悪い例](#vue-valid-v-if-bad) · [良い例](#vue-valid-v-if-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-if-bad"></span>

**悪い**

条件式がないか、同じノードに `v-if` と else directive を併記しています。

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**良い**

`ready` や `count > 0` の式を各 `v-if` に指定し、競合する else directive を除きます。

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [全ルール](all.md)

### `vue/valid-v-memo`

v-memo の値を配列の式にします。

[悪い例](#vue-valid-v-memo-bad) · [良い例](#vue-valid-v-memo-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-memo-bad"></span>

**悪い**

値のない `v-memo` では、サブツリーを再利用する判断に必要な依存式を渡せません。

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**良い**

`v-memo="[valueA, valueB]"` でメモ化に使う依存配列を渡します。

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [全ルール](all.md)

### `vue/valid-v-model`

v-model の値・引数・modifier を検査します。

[悪い例](#vue-valid-v-model-bad) · [良い例](#vue-valid-v-model-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-model-bad"></span>

**悪い**

ネイティブの `<div>` は `v-model` のフォーム要素ではなく、値のない input の directive には書き込み先の式がありません。

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**良い**

input、select、textarea、カスタム コンポーネントを例の書き込み可能な変数に binding します。

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [全ルール](all.md)

### `vue/valid-v-on`

v-on のイベント名・式・modifier を検査します。

[悪い例](#vue-valid-v-on-bad) · [良い例](#vue-valid-v-on-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-on-bad"></span>

**悪い**

イベント引数または必要な handler / object 式がありません。

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**良い**

イベントと handler を指定するか、引数なしの `v-on` に listener object を渡します。

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全ルール](all.md)

### `vue/valid-v-once`

v-once の引数・値・modifier を検査します。

[悪い例](#vue-valid-v-once-bad) · [良い例](#vue-valid-v-once-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-once": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-once-bad"></span>

**悪い**

一度だけ描画する印である `v-once` に、値・引数・修飾子を渡しています。

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**良い**

値のない `v-once` で、サブツリーを一度だけ描画する対象にします。

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [全ルール](all.md)

### `vue/valid-v-show`

v-show に有効な条件式を指定します。

[悪い例](#vue-valid-v-show-bad) · [良い例](#vue-valid-v-show-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-show-bad"></span>

**悪い**

`v-show` に表示条件の式がないか、display を変更する DOM 要素のない `<template>` に付けています。

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**良い**

`<div>` のような描画される要素に表示条件を指定します。

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [全ルール](all.md)

### `vue/valid-v-slot`

v-slot の適用先・宣言・modifier を検査します。

[悪い例](#vue-valid-v-slot-bad) · [良い例](#vue-valid-v-slot-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-slot-bad"></span>

**悪い**

ネイティブの `<div>` に slot directive を付けるか、他の default / 名前付き slot 宣言と競合させています。

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**良い**

default slot はコンポーネント上、名前付き slot は子の `<template #header>` に宣言します。

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [全ルール](all.md)

### `vue/valid-v-text`

v-text の値・引数・modifier を検査します。

[悪い例](#vue-valid-v-text-bad) · [良い例](#vue-valid-v-text-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-text-bad"></span>

**悪い**

`v-text` に文字列の式がないか、対応していない引数・修飾子を指定しています。

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**良い**

`v-text="msg"` は構文として有効です。別の `vue/no-v-text` は mustache の使用を推奨する場合があります。

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [全ルール](all.md)

### `vue/warn-custom-block`

SFC のカスタムブロックを検出します。

[悪い例](#vue-warn-custom-block-bad) · [良い例](#vue-warn-custom-block-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**悪い**

SFC に `<i18n>` のカスタム ブロックがあり、通常の template / script / style 処理とは別の連携が必要です。

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**良い**

標準の template と script setup を使います。この任意の移植性警告は、カスタム ブロック全般が Vue で無効という意味ではありません。

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [全ルール](all.md)

### `vue/warn-custom-directive`

登録が必要なカスタムディレクティブを検出します。

[悪い例](#vue-warn-custom-directive-bad) · [良い例](#vue-warn-custom-directive-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**悪い**

`v-focus`、`v-mask`、`v-click-outside` はプロジェクト固有の directive 実装を必要とし、この任意の規約で警告されます。

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**良い**

組み込みの `v-if`、`v-model`、`v-on` を使います。規約を無効にすれば、正しく登録した custom directive は有効な Vue として使えます。

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [全ルール](all.md)
