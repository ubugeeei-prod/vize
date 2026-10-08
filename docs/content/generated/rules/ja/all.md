---
title: 全 lint ルール
---

# 全 lint ルール

現在のソース カタログにある 251 項目の一覧です。このページに、目的・適用範囲・設定・悪い例・良い例と理由をまとめています。未対応の範囲も各例の前に明記しています。

Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`lint.vize.rules` に指定します。`vp run lint` で Vize と Oxlint の lint を実行します。

既定の重大度は実装の値です。プロジェクトでは `off` / `warn` / `error` に変更できます。Options は [ルール オプション](./options.md)、複数ファイルの検査は [Cross-file ルール](./cross-file.md) を参照してください。

プリセットが `_none_` のルールは明示的な有効化または追加の設定が必要です。

[ESLint からのルール移行対応表](./migration.md)で対応名と未実装の範囲を確認できます。

## 単一ファイルのルール (251)

| ルール | 例 | 重大度 | プリセット | 自動修正 | オプション | 実装 | 目的 | カテゴリ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [悪い例](#petite-vue-no-unsupported-directive-bad) · [良い例](#petite-vue-no-unsupported-directive-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) | petite-vue が対応しないディレクティブを検出します。 | Essential |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [悪い例](#petite-vue-valid-v-effect-bad) · [良い例](#petite-vue-valid-v-effect-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) | v-effect に空でない式を指定します。 | Essential |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [悪い例](#petite-vue-valid-v-scope-bad) · [良い例](#petite-vue-valid-v-scope-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) | v-scope の値にオブジェクトリテラルを指定します。 | Essential |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [悪い例](#vue-multi-word-component-names-bad) · [良い例](#vue-multi-word-component-names-good) | `error` | `essential`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) | コンポーネント名を複数の単語で構成します。 | Essential |
| [`vue/no-child-content`](#vue-no-child-content) | [悪い例](#vue-no-child-content-bad) · [良い例](#vue-no-child-content-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 | Essential |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [悪い例](#vue-no-deprecated-filter-bad) · [良い例](#vue-no-deprecated-filter-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) | Vue 2 の pipe による filter 構文を検出します。 | Essential |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [悪い例](#vue-no-deprecated-functional-template-bad) · [良い例](#vue-no-deprecated-functional-template-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) | SFC の template で削除済みの functional 属性を検出します。 | Essential |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [悪い例](#vue-no-deprecated-html-element-is-bad) · [良い例](#vue-no-deprecated-html-element-is-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 | Essential |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [悪い例](#vue-no-deprecated-inline-template-bad) · [良い例](#vue-no-deprecated-inline-template-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) | 削除済みの inline-template 属性を検出します。 | Essential |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [悪い例](#vue-no-deprecated-router-link-tag-prop-bad) · [良い例](#vue-no-deprecated-router-link-tag-prop-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) | router-link の削除済み tag prop を検出します。 | Essential |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [悪い例](#vue-no-deprecated-scope-attribute-bad) · [良い例](#vue-no-deprecated-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) | template の削除済み scope 属性を検出します。 | Essential |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [悪い例](#vue-no-deprecated-slot-attribute-bad) · [良い例](#vue-no-deprecated-slot-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) | 削除済みの slot 属性を検出します。 | Essential |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [悪い例](#vue-no-deprecated-slot-scope-attribute-bad) · [良い例](#vue-no-deprecated-slot-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) | 削除済みの slot-scope 属性を検出します。 | Essential |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [悪い例](#vue-no-deprecated-v-bind-sync-bad) · [良い例](#vue-no-deprecated-v-bind-sync-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) | 削除済みの v-bind の .sync modifier を検出します。 | Essential |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [悪い例](#vue-no-deprecated-v-on-native-modifier-bad) · [良い例](#vue-no-deprecated-v-on-native-modifier-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) | 削除済みの v-on の .native modifier を検出します。 | Essential |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [悪い例](#vue-no-deprecated-v-on-number-modifiers-bad) · [良い例](#vue-no-deprecated-v-on-number-modifiers-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) | v-on の削除済み数値 keyCode modifier を検出します。 | Essential |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [悪い例](#vue-no-dupe-v-else-if-bad) · [良い例](#vue-no-dupe-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) | v-if / v-else-if の条件重複を検出します。 | Essential |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [悪い例](#vue-no-duplicate-attributes-bad) · [良い例](#vue-no-duplicate-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) | 同じ要素の属性重複を検出します。 | Essential |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [悪い例](#vue-no-multiple-template-root-bad) · [良い例](#vue-no-multiple-template-root-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) | 単一ルートを要求するテンプレートで複数ルートを検出します。 | Essential |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [悪い例](#vue-no-mutating-props-bad) · [良い例](#vue-no-mutating-props-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 | Essential |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [悪い例](#vue-no-reserved-component-names-bad) · [良い例](#vue-no-reserved-component-names-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) | 予約済みのコンポーネント名を検出します。 | Essential |
| [`vue/no-template-key`](#vue-no-template-key) | [悪い例](#vue-no-template-key-bad) · [良い例](#vue-no-template-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) | v-for 用ではない template の key 指定を検出します。 | Essential |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [悪い例](#vue-no-textarea-mustache-bad) · [良い例](#vue-no-textarea-mustache-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) | textarea 内の mustache を検出し、v-model の使用を勧めます。 | Essential |
| [`vue/no-unused-components`](#vue-no-unused-components) | [悪い例](#vue-no-unused-components-bad) · [良い例](#vue-no-unused-components-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 | Essential |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [悪い例](#vue-no-unused-vars-bad) · [良い例](#vue-no-unused-vars-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) | v-for / v-slot に宣言しているのに使わない変数を検出します。 | Essential |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [悪い例](#vue-no-use-v-if-with-v-for-bad) · [良い例](#vue-no-use-v-if-with-v-for-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) | 同じ要素での v-if と v-for の併用を検出します。 | Essential |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [悪い例](#vue-no-useless-template-attributes-bad) · [良い例](#vue-no-useless-template-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) | template 要素の効果がない属性を検出します。 | Essential |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [悪い例](#vue-no-v-for-template-key-on-child-bad) · [良い例](#vue-no-v-for-template-key-on-child-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) | template v-for の key を子ではなく template に指定します。 | Essential |
| [`vue/no-v-html`](#vue-no-v-html) | [悪い例](#vue-no-v-html-bad) · [良い例](#vue-no-v-html-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) | 未処理の HTML を表示する v-html の XSS リスクを検出します。 | Essential |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [悪い例](#vue-no-v-text-v-html-on-component-bad) · [良い例](#vue-no-v-text-v-html-on-component-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) | コンポーネントでの v-text / v-html を検出します。 | Essential |
| [`vue/permitted-contents`](#vue-permitted-contents) | [悪い例](#vue-permitted-contents-bad) · [良い例](#vue-permitted-contents-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) | HTML の要素ごとのコンテンツモデルを検査します。 | Essential |
| [`vue/require-component-is`](#vue-require-component-is) | [悪い例](#vue-require-component-is-bad) · [良い例](#vue-require-component-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) | 動的 component 要素に :is を指定します。 | Essential |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [悪い例](#vue-require-toggle-inside-transition-bad) · [良い例](#vue-require-toggle-inside-transition-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) | transition の子要素に表示を切り替える条件を指定します。 | Essential |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [悪い例](#vue-require-v-for-key-bad) · [良い例](#vue-require-v-for-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) | v-for に安定した :key を指定します。 | Essential |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [悪い例](#vue-use-v-on-exact-bad) · [良い例](#vue-use-v-on-exact-good) | `warning` | `essential`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) | modifier 付きのイベント操作と競合する handler に .exact を指定します。 | Essential |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [悪い例](#vue-valid-attribute-name-bad) · [良い例](#vue-valid-attribute-name-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) | 有効な属性名を指定します。 | Essential |
| [`vue/valid-template-root`](#vue-valid-template-root) | [悪い例](#vue-valid-template-root-bad) · [良い例](#vue-valid-template-root-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) | Vue 3 の fragment に対応する有効なテンプレートルートを検査します。 | Essential |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [悪い例](#vue-valid-v-bind-bad) · [良い例](#vue-valid-v-bind-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) | v-bind の引数・値・modifier を検査します。 | Essential |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [悪い例](#vue-valid-v-cloak-bad) · [良い例](#vue-valid-v-cloak-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) | v-cloak の引数・値・modifier を検査します。 | Essential |
| [`vue/valid-v-else`](#vue-valid-v-else) | [悪い例](#vue-valid-v-else-bad) · [良い例](#vue-valid-v-else-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) | v-else の位置・引数・値を検査します。 | Essential |
| [`vue/valid-v-for`](#vue-valid-v-for) | [悪い例](#vue-valid-v-for-bad) · [良い例](#vue-valid-v-for-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) | v-for の式と変数宣言を検査します。 | Essential |
| [`vue/valid-v-html`](#vue-valid-v-html) | [悪い例](#vue-valid-v-html-bad) · [良い例](#vue-valid-v-html-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) | v-html の値・引数・modifier を検査します。 | Essential |
| [`vue/valid-v-if`](#vue-valid-v-if) | [悪い例](#vue-valid-v-if-bad) · [良い例](#vue-valid-v-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) | v-if に有効な条件式を指定します。 | Essential |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [悪い例](#vue-valid-v-memo-bad) · [良い例](#vue-valid-v-memo-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) | v-memo の値を配列の式にします。 | Essential |
| [`vue/valid-v-model`](#vue-valid-v-model) | [悪い例](#vue-valid-v-model-bad) · [良い例](#vue-valid-v-model-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) | v-model の値・引数・modifier を検査します。 | Essential |
| [`vue/valid-v-on`](#vue-valid-v-on) | [悪い例](#vue-valid-v-on-bad) · [良い例](#vue-valid-v-on-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) | v-on のイベント名・式・modifier を検査します。 | Essential |
| [`vue/valid-v-once`](#vue-valid-v-once) | [悪い例](#vue-valid-v-once-bad) · [良い例](#vue-valid-v-once-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) | v-once の引数・値・modifier を検査します。 | Essential |
| [`vue/valid-v-show`](#vue-valid-v-show) | [悪い例](#vue-valid-v-show-bad) · [良い例](#vue-valid-v-show-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) | v-show に有効な条件式を指定します。 | Essential |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [悪い例](#vue-valid-v-slot-bad) · [良い例](#vue-valid-v-slot-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) | v-slot の適用先・宣言・modifier を検査します。 | Essential |
| [`vue/valid-v-text`](#vue-valid-v-text) | [悪い例](#vue-valid-v-text-bad) · [良い例](#vue-valid-v-text-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) | v-text の値・引数・modifier を検査します。 | Essential |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [悪い例](#vue-attribute-hyphenation-bad) · [良い例](#vue-attribute-hyphenation-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) | コンポーネントの prop 属性名を設定した形式に揃えます。 | Strongly Recommended |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [悪い例](#vue-component-definition-name-casing-bad) · [良い例](#vue-component-definition-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 | Strongly Recommended |
| [`vue/html-quotes`](#vue-html-quotes) | [悪い例](#vue-html-quotes-bad) · [良い例](#vue-html-quotes-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) | HTML 属性値の引用符を揃えます。 | Strongly Recommended |
| [`vue/html-self-closing`](#vue-html-self-closing) | [悪い例](#vue-html-self-closing-bad) · [良い例](#vue-html-self-closing-good) | `warning` | `nuxt`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) | 要素の種類ごとに自己終了タグの形式を揃えます。 | Strongly Recommended |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [悪い例](#vue-mustache-interpolation-spacing-bad) · [良い例](#vue-mustache-interpolation-spacing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) | mustache 内の空白を揃えます。 | Strongly Recommended |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [悪い例](#vue-no-multi-spaces-bad) · [良い例](#vue-no-multi-spaces-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) | 連続する不要な空白を検出します。 | Strongly Recommended |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [悪い例](#vue-no-template-shadow-bad) · [良い例](#vue-no-template-shadow-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) | テンプレート変数が外側の名前を隠す箇所を検出します。 | Strongly Recommended |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [悪い例](#vue-no-unused-properties-bad) · [良い例](#vue-no-unused-properties-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) | defineProps に宣言しているのに使わない prop を検出します。 | Strongly Recommended |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [悪い例](#vue-prop-name-casing-bad) · [良い例](#vue-prop-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) | 宣言する prop 名の形式を揃えます。 | Strongly Recommended |
| [`vue/v-bind-style`](#vue-v-bind-style) | [悪い例](#vue-v-bind-style-bad) · [良い例](#vue-v-bind-style-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) | v-bind の表記形式を揃えます。 | Strongly Recommended |
| [`vue/v-on-style`](#vue-v-on-style) | [悪い例](#vue-v-on-style-bad) · [良い例](#vue-v-on-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) | v-on の表記形式を揃えます。 | Strongly Recommended |
| [`vue/v-slot-style`](#vue-v-slot-style) | [悪い例](#vue-v-slot-style-bad) · [良い例](#vue-v-slot-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) | v-slot の表記形式を揃えます。 | Strongly Recommended |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [悪い例](#ssr-no-browser-globals-in-ssr-bad) · [良い例](#ssr-no-browser-globals-in-ssr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 | Recommended |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [悪い例](#ssr-no-hydration-mismatch-bad) · [良い例](#ssr-no-hydration-mismatch-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) | サーバーとクライアントで一致しないテンプレート値を検出します。 | Recommended |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [悪い例](#vue-a11y-img-alt-bad) · [良い例](#vue-a11y-img-alt-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) | 画像に代替テキストの alt 属性を指定します。 | Recommended |
| [`vue/attribute-order`](#vue-attribute-order) | [悪い例](#vue-attribute-order-bad) · [良い例](#vue-attribute-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) | テンプレートの属性を一定の順に並べます。 | Recommended |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [悪い例](#vue-component-name-in-template-casing-bad) · [良い例](#vue-component-name-in-template-casing-good) | `warning` | `nuxt`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) | テンプレート内のコンポーネント名を指定した形式に揃えます。 | Recommended |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [悪い例](#vue-html-button-has-type-bad) · [良い例](#vue-html-button-has-type-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) | button に有効な type を明示します。 | Recommended |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [悪い例](#vue-max-template-complexity-bad) · [良い例](#vue-max-template-complexity-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) | コンポーネント自身のテンプレートの複雑度を制限します。 | Recommended |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [悪い例](#vue-no-array-index-key-bad) · [良い例](#vue-no-array-index-key-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 | Recommended |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [悪い例](#vue-no-bare-strings-in-template-bad) · [良い例](#vue-no-bare-strings-in-template-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) | 国際化すべきテンプレートの直接指定テキストを検出します。 | Recommended |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [悪い例](#vue-no-boolean-attr-value-bad) · [良い例](#vue-no-boolean-attr-value-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) | HTML の boolean 属性に不要な値を指定した箇所を検出します。 | Recommended |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [悪い例](#vue-no-empty-component-block-bad) · [良い例](#vue-no-empty-component-block-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) | 空の SFC ブロックを検出します。 | Recommended |
| [`vue/no-inline-style`](#vue-no-inline-style) | [悪い例](#vue-no-inline-style-bad) · [良い例](#vue-no-inline-style-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) | インラインの style 属性を検出します。 | Recommended |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [悪い例](#vue-no-invalid-html-attribute-bad) · [良い例](#vue-no-invalid-html-attribute-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 | Recommended |
| [`vue/no-lone-template`](#vue-no-lone-template) | [悪い例](#vue-no-lone-template-bad) · [良い例](#vue-no-lone-template-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) | 不要な template 要素を検出します。 | Recommended |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [悪い例](#vue-no-multiple-objects-in-class-bad) · [良い例](#vue-no-multiple-objects-in-class-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) | :class 配列内の複数のオブジェクト指定をまとめます。 | Recommended |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [悪い例](#vue-no-negated-v-if-condition-bad) · [良い例](#vue-no-negated-v-if-condition-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) | v-else がある条件分岐の否定条件を反転して読みやすくします。 | Recommended |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [悪い例](#vue-no-non-component-keep-alive-child-bad) · [良い例](#vue-no-non-component-keep-alive-child-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 | Recommended |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [悪い例](#vue-no-preprocessor-lang-bad) · [良い例](#vue-no-preprocessor-lang-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) | CSS preprocessor より標準の CSS を使う方針を適用します。 | Recommended |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [悪い例](#vue-no-root-v-if-bad) · [良い例](#vue-no-root-v-if-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 | Recommended |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [悪い例](#vue-no-script-non-standard-lang-bad) · [良い例](#vue-no-script-non-standard-lang-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) | script の非標準 lang 指定を検出します。 | Recommended |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [悪い例](#vue-no-src-attribute-bad) · [良い例](#vue-no-src-attribute-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) | SFC ブロックの外部 src 指定を検出します。 | Recommended |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [悪い例](#vue-no-static-inline-styles-bad) · [良い例](#vue-no-static-inline-styles-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) | 静的なインライン style 属性を検出します。 | Recommended |
| [`vue/no-template-lang`](#vue-no-template-lang) | [悪い例](#vue-no-template-lang-bad) · [良い例](#vue-no-template-lang-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) | template の lang 指定を検出します。 | Recommended |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [悪い例](#vue-no-template-target-blank-bad) · [良い例](#vue-no-template-target-blank-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) | target=_blank の外部リンクに適切な rel を指定します。 | Recommended |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [悪い例](#vue-no-undefined-refs-bad) · [良い例](#vue-no-undefined-refs-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) | テンプレート内の未定義変数参照を検出します。 | Recommended |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [悪い例](#vue-no-unsafe-url-bad) · [良い例](#vue-no-unsafe-url-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) | 危険なスキームになり得る URL 属性やバインディングを検出します。 | Recommended |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [悪い例](#vue-no-unsandboxed-iframe-bad) · [良い例](#vue-no-unsandboxed-iframe-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) | iframe に sandbox 属性を指定します。 | Recommended |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [悪い例](#vue-no-unused-refs-bad) · [良い例](#vue-no-unused-refs-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) | テンプレートに宣言しているのに参照しない ref を検出します。 | Recommended |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [悪い例](#vue-no-unused-setup-bindings-bad) · [良い例](#vue-no-unused-setup-bindings-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) | script setup に宣言しているのに読み取らない変数を検出します。 | Recommended |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [悪い例](#vue-no-use-v-else-with-v-for-bad) · [良い例](#vue-no-use-v-else-with-v-for-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 | Recommended |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [悪い例](#vue-no-useless-mustaches-bad) · [良い例](#vue-no-useless-mustaches-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) | 文字列リテラルだけの不要な mustache を検出します。 | Recommended |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [悪い例](#vue-no-useless-v-bind-bad) · [良い例](#vue-no-useless-v-bind-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) | 文字列リテラルだけの不要な v-bind を検出します。 | Recommended |
| [`vue/no-v-text`](#vue-no-v-text) | [悪い例](#vue-no-v-text-bad) · [良い例](#vue-no-v-text-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) | v-text の代わりに mustache を使う方針を適用します。 | Recommended |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [悪い例](#vue-prefer-props-shorthand-bad) · [良い例](#vue-prefer-props-shorthand-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) | Vue 3.4 の同名 prop バインディングの省略形を使います。 | Recommended |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [悪い例](#vue-prefer-true-attribute-shorthand-bad) · [良い例](#vue-prefer-true-attribute-shorthand-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) | true を指定するバインディングを boolean 属性の省略形にします。 | Recommended |
| [`vue/require-component-registration`](#vue-require-component-registration) | [悪い例](#vue-require-component-registration-bad) · [良い例](#vue-require-component-registration-good) | `warning` | `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) | 使用するコンポーネントを import または登録します。 | Recommended |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [悪い例](#vue-require-scoped-style-bad) · [良い例](#vue-require-scoped-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) | style に scoped を指定する方針を適用します。 | Recommended |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [悪い例](#vue-scoped-event-names-bad) · [良い例](#vue-scoped-event-names-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) | イベント名を context:event の形式に揃えます。 | Recommended |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [悪い例](#vue-sfc-element-order-bad) · [良い例](#vue-sfc-element-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) | SFC のトップレベルブロックを設定した順に並べます。 | Recommended |
| [`vue/single-style-block`](#vue-single-style-block) | [悪い例](#vue-single-style-block-bad) · [良い例](#vue-single-style-block-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) | SFC の style を一つのブロックにまとめます。 | Recommended |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [悪い例](#vue-slot-name-casing-bad) · [良い例](#vue-slot-name-casing-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) | 名前付き slot を kebab-case に揃えます。 | Recommended |
| [`vue/this-in-template`](#vue-this-in-template) | [悪い例](#vue-this-in-template-bad) · [良い例](#vue-this-in-template-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) | テンプレートで不要な this. 参照を検出します。 | Recommended |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [悪い例](#vue-v-on-event-hyphenation-bad) · [良い例](#vue-v-on-event-hyphenation-good) | `warning` | `nuxt`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 | Recommended |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [悪い例](#vue-v-on-handler-style-bad) · [良い例](#vue-v-on-handler-style-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) | イベント handler の参照・関数形式を揃えます。 | Recommended |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [悪い例](#vue-warn-custom-block-bad) · [良い例](#vue-warn-custom-block-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) | SFC のカスタムブロックを検出します。 | Recommended |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [悪い例](#vue-warn-custom-directive-bad) · [良い例](#vue-warn-custom-directive-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) | 登録が必要なカスタムディレクティブを検出します。 | Recommended |
| [`a11y/alt-text`](#a11y-alt-text) | [悪い例](#a11y-alt-text-bad) · [良い例](#a11y-alt-text-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) | 画像などのメディアに代替テキストを用意します。 | Accessibility |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [悪い例](#a11y-anchor-has-content-bad) · [良い例](#a11y-anchor-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) | リンクに支援技術で読める内容を用意します。 | Accessibility |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [悪い例](#a11y-anchor-is-valid-bad) · [良い例](#a11y-anchor-is-valid-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) | リンクの href に有効な移動先を指定します。 | Accessibility |
| [`a11y/aria-props`](#a11y-aria-props) | [悪い例](#a11y-aria-props-bad) · [良い例](#a11y-aria-props-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) | 存在しない ARIA 属性を検出します。 | Accessibility |
| [`a11y/aria-role`](#a11y-aria-role) | [悪い例](#a11y-aria-role-bad) · [良い例](#a11y-aria-role-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) | 有効で抽象的ではない ARIA role を指定します。 | Accessibility |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [悪い例](#a11y-aria-unsupported-elements-bad) · [良い例](#a11y-aria-unsupported-elements-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) | ARIA 属性を使用できない要素への指定を検出します。 | Accessibility |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [悪い例](#a11y-click-events-have-key-events-bad) · [良い例](#a11y-click-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) | クリックで操作する要素にキーボード操作も用意します。 | Accessibility |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [悪い例](#a11y-form-control-has-label-bad) · [良い例](#a11y-form-control-has-label-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) | フォーム部品に関連付けられたラベルを用意します。 | Accessibility |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [悪い例](#a11y-heading-has-content-bad) · [良い例](#a11y-heading-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) | 見出しに支援技術で読める内容を用意します。 | Accessibility |
| [`a11y/heading-levels`](#a11y-heading-levels) | [悪い例](#a11y-heading-levels-bad) · [良い例](#a11y-heading-levels-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) | 見出しの階層を飛ばした指定を検出します。 | Accessibility |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [悪い例](#a11y-iframe-has-title-bad) · [良い例](#a11y-iframe-has-title-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) | iframe に内容を説明する title を指定します。 | Accessibility |
| [`a11y/img-alt`](#a11y-img-alt) | [悪い例](#a11y-img-alt-bad) · [良い例](#a11y-img-alt-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 | Accessibility |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [悪い例](#a11y-interactive-supports-focus-bad) · [良い例](#a11y-interactive-supports-focus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) | 操作可能な role の要素をフォーカス可能にします。 | Accessibility |
| [`a11y/label-has-for`](#a11y-label-has-for) | [悪い例](#a11y-label-has-for-bad) · [良い例](#a11y-label-has-for-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) | label を対象のフォーム部品と関連付けます。 | Accessibility |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [悪い例](#a11y-landmark-roles-bad) · [良い例](#a11y-landmark-roles-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) | ランドマーク role の配置と重複を検査します。 | Accessibility |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [悪い例](#a11y-media-has-caption-bad) · [良い例](#a11y-media-has-caption-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) | 音声・動画に字幕を用意します。 | Accessibility |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [悪い例](#a11y-mouse-events-have-key-events-bad) · [良い例](#a11y-mouse-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) | マウス操作と対応する focus / blur 操作を用意します。 | Accessibility |
| [`a11y/no-access-key`](#a11y-no-access-key) | [悪い例](#a11y-no-access-key-bad) · [良い例](#a11y-no-access-key-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) | 環境のショートカットと衝突し得る accesskey を検出します。 | Accessibility |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [悪い例](#a11y-no-aria-hidden-on-focusable-bad) · [良い例](#a11y-no-aria-hidden-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) | フォーカス可能な要素を aria-hidden で隠した指定を検出します。 | Accessibility |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [悪い例](#a11y-no-autofocus-bad) · [良い例](#a11y-no-autofocus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) | 意図せずフォーカスを移動させる autofocus を検出します。 | Accessibility |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [悪い例](#a11y-no-distracting-elements-bad) · [良い例](#a11y-no-distracting-elements-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) | marquee や blink などの注意をそらす要素を検出します。 | Accessibility |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [悪い例](#a11y-no-i-for-icon-bad) · [良い例](#a11y-no-i-for-icon-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 | Accessibility |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [悪い例](#a11y-no-redundant-roles-bad) · [良い例](#a11y-no-redundant-roles-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) | 要素本来の意味と重複する ARIA role を検出します。 | Accessibility |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [悪い例](#a11y-no-refer-to-non-existent-id-bad) · [良い例](#a11y-no-refer-to-non-existent-id-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) | 文書内に存在しない ID への参照を検出します。 | Accessibility |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [悪い例](#a11y-no-role-presentation-on-focusable-bad) · [良い例](#a11y-no-role-presentation-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) | フォーカス可能な要素の意味を presentation で消した指定を検出します。 | Accessibility |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [悪い例](#a11y-no-static-element-interactions-bad) · [良い例](#a11y-no-static-element-interactions-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) | 操作部品ではない要素へのイベント指定を検出します。 | Accessibility |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [悪い例](#a11y-placeholder-label-option-bad) · [良い例](#a11y-placeholder-label-option-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) | select のプレースホルダー option に disabled または hidden を指定します。 | Accessibility |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [悪い例](#a11y-role-has-required-aria-props-bad) · [良い例](#a11y-role-has-required-aria-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) | ARIA role が必要とする属性を指定します。 | Accessibility |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [悪い例](#a11y-tabindex-no-positive-bad) · [良い例](#a11y-tabindex-no-positive-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) | 通常のフォーカス順序を変える正の tabindex を検出します。 | Accessibility |
| [`a11y/use-list`](#a11y-use-list) | [悪い例](#a11y-use-list-bad) · [良い例](#a11y-use-list-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) | 箇条書きに見えるテキストをリスト要素で表現します。 | Accessibility |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) | 静的 ID の代わりに useId() で再利用可能な ID を生成します。 | Accessibility |
| [`html/deprecated-attr`](#html-deprecated-attr) | [悪い例](#html-deprecated-attr-bad) · [良い例](#html-deprecated-attr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) | 非推奨の HTML 属性を検出します。 | HTML Conformance |
| [`html/deprecated-element`](#html-deprecated-element) | [悪い例](#html-deprecated-element-bad) · [良い例](#html-deprecated-element-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) | 非推奨の HTML 要素を検出します。 | HTML Conformance |
| [`html/id-duplication`](#html-id-duplication) | [悪い例](#html-id-duplication-bad) · [良い例](#html-id-duplication-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) | 同じテンプレート内の ID 重複を検出します。 | HTML Conformance |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [悪い例](#html-no-consecutive-br-bad) · [良い例](#html-no-consecutive-br-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) | 連続する br 要素による余白指定を検出します。 | HTML Conformance |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [悪い例](#html-no-dupe-style-properties-bad) · [良い例](#html-no-dupe-style-properties-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) | 静的 style 属性内のプロパティ重複を検出します。 | HTML Conformance |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [悪い例](#html-no-duplicate-class-bad) · [良い例](#html-no-duplicate-class-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) | 静的 class 属性内のクラス名重複を検出します。 | HTML Conformance |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [悪い例](#html-no-duplicate-dt-bad) · [良い例](#html-no-duplicate-dt-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) | dl 内の dt の名前重複を検出します。 | HTML Conformance |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [悪い例](#html-no-empty-palpable-content-bad) · [良い例](#html-no-empty-palpable-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) | 可視コンテンツを期待する要素が空の場合に検出します。 | HTML Conformance |
| [`html/require-datetime`](#html-require-datetime) | [悪い例](#html-require-datetime-bad) · [良い例](#html-require-datetime-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) | time 要素に機械可読の datetime を指定します。 | HTML Conformance |
| [`type/no-floating-promises`](#type-no-floating-promises) | [悪い例](#type-no-floating-promises-bad) · [良い例](#type-no-floating-promises-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) | 処理しないまま放置された Promise を検出します。 | Type Aware |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [悪い例](#type-no-reactivity-loss-bad) · [良い例](#type-no-reactivity-loss-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) | 代入や呼び出しによって反応性を失うスナップショットを検出します。 | Type Aware |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [悪い例](#type-no-unsafe-template-binding-bad) · [良い例](#type-no-unsafe-template-binding-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) | テンプレートで安全でない型の値を使用する箇所を検出します。 | Type Aware |
| [`type/require-typed-emits`](#type-require-typed-emits) | [悪い例](#type-require-typed-emits-bad) · [良い例](#type-require-typed-emits-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) | defineEmits に型定義を指定します。 | Type Aware |
| [`type/require-typed-props`](#type-require-typed-props) | [悪い例](#type-require-typed-props-bad) · [良い例](#type-require-typed-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) | defineProps に型定義を指定します。 | Type Aware |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [悪い例](#type-strict-boolean-expressions-bad) · [良い例](#type-strict-boolean-expressions-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) | script とテンプレートの条件式で安全な真偽判定を使います。 | Type Aware |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [悪い例](#script-no-get-current-instance-bad) · [良い例](#script-no-get-current-instance-good) | `error` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) | Vapor で null を返す getCurrentInstance() を検出します。 | Vapor |
| [`script/no-next-tick`](#script-no-next-tick) | [悪い例](#script-no-next-tick-bad) · [良い例](#script-no-next-tick-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 | Vapor |
| [`script/no-options-api`](#script-no-options-api) | [悪い例](#script-no-options-api-bad) · [良い例](#script-no-options-api-good) | `error` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) | Vapor で Options API を使用する箇所を検出します。 | Vapor |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [悪い例](#vapor-no-inline-template-bad) · [良い例](#vapor-no-inline-template-good) | `error` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) | Vapor で削除済みの inline-template 属性を検出します。 | Vapor |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [悪い例](#vapor-no-vue-lifecycle-events-bad) · [良い例](#vapor-no-vue-lifecycle-events-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) | Vapor が対応しない要素の @vue:* lifecycle event を検出します。 | Vapor |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [悪い例](#vapor-prefer-static-class-bad) · [良い例](#vapor-prefer-static-class-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) | 文字列リテラルの :class を静的 class に置き換えます。 | Vapor |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [悪い例](#vapor-require-vapor-attribute-bad) · [良い例](#vapor-require-vapor-attribute-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) | Vapor 向けの script setup に vapor 属性を付ける方針を適用します。 | Vapor |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [悪い例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い例](#ecosystem-nuxt-prefer-nuxt-link-good) | `warning` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) | Nuxt の内部リンクに NuxtLink を使います。 | Ecosystem |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [悪い例](#ecosystem-pinia-prefer-store-to-refs-bad) · [良い例](#ecosystem-pinia-prefer-store-to-refs-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) | Pinia store の分割代入に storeToRefs() を使います。 | Ecosystem |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [悪い例](#ecosystem-router-link-require-to-bad) · [良い例](#ecosystem-router-link-require-to-good) | `error` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) | RouterLink / NuxtLink に to を指定します。 | Ecosystem |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [悪い例](#ecosystem-void-link-require-href-bad) · [良い例](#ecosystem-void-link-require-href-good) | `error` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) | Void Vue の Link に href を指定します。 | Ecosystem |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [悪い例](#ecosystem-void-link-valid-method-bad) · [良い例](#ecosystem-void-link-valid-method-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) | Void Vue の Link に有効な静的 method を指定します。 | Ecosystem |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [悪い例](#ecosystem-vue-i18n-no-missing-key-bad) · [良い例](#ecosystem-vue-i18n-no-missing-key-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) | SFC 内の翻訳データに存在しない静的キーを検出します。 | Ecosystem |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [悪い例](#ecosystem-vue-router-prefer-named-link-bad) · [良い例](#ecosystem-vue-router-prefer-named-link-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 | Ecosystem |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [悪い例](#ecosystem-vue-router-prefer-named-push-bad) · [良い例](#ecosystem-vue-router-prefer-named-push-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) | Vue Router のプログラムによる移動に名前付きルートを使います。 | Ecosystem |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [悪い例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い例](#ecosystem-vue-test-utils-no-html-snapshot-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) | wrapper.html() 全体の snapshot に依存するテストを検出します。 | Ecosystem |
| [`css/no-display-none`](#css-no-display-none) | [悪い例](#css-no-display-none-bad) · [良い例](#css-no-display-none-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) | 表示切り替えに display: none を使う箇所で v-show を検討します。 | CSS |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [悪い例](#css-no-hardcoded-values-bad) · [良い例](#css-no-hardcoded-values-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) | CSS の直接指定値を CSS 変数にまとめます。 | CSS |
| [`css/no-id-selectors`](#css-no-id-selectors) | [悪い例](#css-no-id-selectors-bad) · [良い例](#css-no-id-selectors-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) | 詳細度が高い CSS の ID セレクターを検出します。 | CSS |
| [`css/no-important`](#css-no-important) | [悪い例](#css-no-important-bad) · [良い例](#css-no-important-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) | 通常のカスケードを上書きする !important を検出します。 | CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [悪い例](#css-no-utility-classes-bad) · [良い例](#css-no-utility-classes-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) | コンポーネント内で utility class を定義する箇所を検出します。 | CSS |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [悪い例](#css-no-v-bind-performance-bad) · [良い例](#css-no-v-bind-performance-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) | CSS v-bind() の実行時コストを検討するための警告です。 | CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [悪い例](#css-prefer-logical-properties-bad) · [良い例](#css-prefer-logical-properties-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) | 書字方向に対応する CSS の論理プロパティを使います。 | CSS |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [悪い例](#css-prefer-nested-selectors-bad) · [良い例](#css-prefer-nested-selectors-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) | 子孫セレクターを CSS nesting でまとめます。 | CSS |
| [`css/prefer-slotted`](#css-prefer-slotted) | [悪い例](#css-prefer-slotted-bad) · [良い例](#css-prefer-slotted-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 | CSS |
| [`css/require-font-display`](#css-require-font-display) | [悪い例](#css-require-font-display-bad) · [良い例](#css-require-font-display-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) | @font-face に font-display を指定します。 | CSS |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [悪い例](#musea-no-empty-variant-bad) · [良い例](#musea-no-empty-variant-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) | 内容のない variant ブロックを検出します。 | Musea |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [悪い例](#musea-prefer-design-tokens-bad) · [良い例](#musea-prefer-design-tokens-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) | 登録した design token に一致する直接指定値を CSS 変数で表現します。 | Musea |
| [`musea/require-component`](#musea-require-component) | [悪い例](#musea-require-component-bad) · [良い例](#musea-require-component-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) | art ブロックに対象の component を指定します。 | Musea |
| [`musea/require-title`](#musea-require-title) | [悪い例](#musea-require-title-bad) · [良い例](#musea-require-title-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) | art ブロックに title を指定します。 | Musea |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [悪い例](#musea-unique-variant-names-bad) · [良い例](#musea-unique-variant-names-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) | 同じ Art ファイル内の variant 名を一意にします。 | Musea |
| [`musea/valid-variant`](#musea-valid-variant) | [悪い例](#musea-valid-variant-bad) · [良い例](#musea-valid-variant-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) | variant ブロックに name を指定します。 | Musea |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [悪い例](#script-component-options-name-casing-bad) · [良い例](#script-component-options-name-casing-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) | コンポーネントの name オプションを PascalCase に揃えます。 | Script |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [悪い例](#script-custom-event-name-casing-bad) · [良い例](#script-custom-event-name-casing-good) | `error` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) | emit するカスタムイベント名を指定した形式に揃えます。 | Script |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [悪い例](#script-define-emits-declaration-bad) · [良い例](#script-define-emits-declaration-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) | defineEmits を型による宣言形式に揃えます。 | Script |
| [`script/define-macros-order`](#script-define-macros-order) | [悪い例](#script-define-macros-order-bad) · [良い例](#script-define-macros-order-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) | script setup のコンパイラーマクロを一定の順に宣言します。 | Script |
| [`script/define-props-declaration`](#script-define-props-declaration) | [悪い例](#script-define-props-declaration-bad) · [良い例](#script-define-props-declaration-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) | defineProps を型による宣言形式に揃えます。 | Script |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [悪い例](#script-define-props-destructuring-bad) · [良い例](#script-define-props-destructuring-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) | defineProps の分割代入スタイルを指定した方針に揃えます。 | Script |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [悪い例](#script-no-arrow-functions-in-watch-bad) · [良い例](#script-no-arrow-functions-in-watch-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) | Options API の watch に this を持たないアロー関数を使う箇所を検出します。 | Script |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [悪い例](#script-no-async-in-computed-bad) · [良い例](#script-no-async-in-computed-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) | computed の getter で非同期関数を使う箇所を検出します。 | Script |
| [`script/no-boolean-default`](#script-no-boolean-default) | [悪い例](#script-no-boolean-default-bad) · [良い例](#script-no-boolean-default-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) | Boolean prop の冗長な default を検出します。 | Script |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [悪い例](#script-no-deep-destructure-in-props-bad) · [良い例](#script-no-deep-destructure-in-props-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) | defineProps の深い分割代入を検出します。 | Script |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [悪い例](#script-no-deprecated-data-object-declaration-bad) · [良い例](#script-no-deprecated-data-object-declaration-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 | Script |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [悪い例](#script-no-deprecated-destroyed-lifecycle-bad) · [良い例](#script-no-deprecated-destroyed-lifecycle-good) | `error` | _none_ | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。 | Script |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [悪い例](#script-no-deprecated-dollar-listeners-api-bad) · [良い例](#script-no-deprecated-dollar-listeners-api-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) | Vue 3 で $attrs に統合された $listeners を検出します。 | Script |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [悪い例](#script-no-deprecated-dollar-scopedslots-api-bad) · [良い例](#script-no-deprecated-dollar-scopedslots-api-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) | Vue 3 で $slots に統合された $scopedSlots を検出します。 | Script |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [悪い例](#script-no-deprecated-events-api-bad) · [良い例](#script-no-deprecated-events-api-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) | Vue 3 で削除された $on / $off / $once を検出します。 | Script |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [悪い例](#script-no-deprecated-props-default-this-bad) · [良い例](#script-no-deprecated-props-default-this-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) | prop の default / validator 内で使えなくなった this を検出します。 | Script |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [悪い例](#script-no-dupe-keys-bad) · [良い例](#script-no-dupe-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) | Options API の props / data / computed などのキー重複を検出します。 | Script |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [悪い例](#script-no-duplicate-attr-inheritance-bad) · [良い例](#script-no-duplicate-attr-inheritance-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 | Script |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [悪い例](#script-no-export-in-script-setup-bad) · [良い例](#script-no-export-in-script-setup-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) | script setup 内の export 文を検出します。 | Script |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [悪い例](#script-no-import-compiler-macros-bad) · [良い例](#script-no-import-compiler-macros-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) | 自動的に使える Vue コンパイラーマクロの import を検出します。 | Script |
| [`script/no-internal-imports`](#script-no-internal-imports) | [悪い例](#script-no-internal-imports-bad) · [良い例](#script-no-internal-imports-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) | Vue 内部モジュールからの import を検出します。 | Script |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [悪い例](#script-no-multiple-slot-args-bad) · [良い例](#script-no-multiple-slot-args-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) | scoped slot 関数に複数の引数を渡す箇所を検出します。 | Script |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [悪い例](#script-no-potential-component-option-typo-bad) · [良い例](#script-no-potential-component-option-typo-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) | Options API のオプション名の入力ミスを検出します。 | Script |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [悪い例](#script-no-reactive-destructure-bad) · [良い例](#script-no-reactive-destructure-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) | reactive オブジェクトの反応性を失う分割代入を検出します。 | Script |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [悪い例](#script-no-ref-as-operand-bad) · [良い例](#script-no-ref-as-operand-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) | ref を演算の値として使う際に .value を参照します。 | Script |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [悪い例](#script-no-required-prop-with-default-bad) · [良い例](#script-no-required-prop-with-default-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) | required: true と default を同時に持つ prop を検出します。 | Script |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [悪い例](#script-no-reserved-identifiers-bad) · [良い例](#script-no-reserved-identifiers-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) | Vue コンパイラーが予約した識別子の宣言を検出します。 | Script |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [悪い例](#script-no-reserved-keys-bad) · [良い例](#script-no-reserved-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) | Options API のキーに Vue の予約名を使う箇所を検出します。 | Script |
| [`script/no-reserved-props`](#script-no-reserved-props) | [悪い例](#script-no-reserved-props-bad) · [良い例](#script-no-reserved-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) | prop 宣言に Vue の予約名を使う箇所を検出します。 | Script |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [悪い例](#script-no-restricted-globals-bad) · [良い例](#script-no-restricted-globals-good) | `error` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) | 設定で禁止した実行環境のグローバル参照を検出します。 | Script |
| [`script/no-restricted-members`](#script-no-restricted-members) | [悪い例](#script-no-restricted-members-bad) · [良い例](#script-no-restricted-members-good) | `error` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) | 設定で禁止した object.property へのアクセスを検出します。 | Script |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [悪い例](#script-no-side-effects-in-computed-properties-bad) · [良い例](#script-no-side-effects-in-computed-properties-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) | Options API の computed getter 内の副作用を検出します。 | Script |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [悪い例](#script-no-top-level-ref-in-script-bad) · [良い例](#script-no-top-level-ref-in-script-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 | Script |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [悪い例](#script-no-unstable-nested-components-bad) · [良い例](#script-no-unstable-nested-components-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 | Script |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [悪い例](#script-no-unused-emit-declarations-bad) · [良い例](#script-no-unused-emit-declarations-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) | 宣言したまま emit していないイベントを検出します。 | Script |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [悪い例](#script-no-use-computed-property-like-method-bad) · [良い例](#script-no-use-computed-property-like-method-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 | Script |
| [`script/no-with-defaults`](#script-no-with-defaults) | [悪い例](#script-no-with-defaults-bad) · [良い例](#script-no-with-defaults-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 | Script |
| [`script/prefer-computed`](#script-prefer-computed) | [悪い例](#script-prefer-computed-bad) · [良い例](#script-prefer-computed-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 | Script |
| [`script/prefer-define-options`](#script-prefer-define-options) | [悪い例](#script-prefer-define-options-bad) · [良い例](#script-prefer-define-options-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 | Script |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [悪い例](#script-prefer-import-from-vue-bad) · [良い例](#script-prefer-import-from-vue-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) | 内部パッケージではなく vue から import します。 | Script |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [悪い例](#script-prefer-ref-over-reactive-bad) · [良い例](#script-prefer-ref-over-reactive-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) | 状態管理に reactive() より ref() を使う方針を適用します。 | Script |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [悪い例](#script-prefer-use-attrs-bad) · [良い例](#script-prefer-use-attrs-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) | setup の context.attrs を useAttrs() に置き換えます。 | Script |
| [`script/prefer-use-id`](#script-prefer-use-id) | [悪い例](#script-prefer-use-id-bad) · [良い例](#script-prefer-use-id-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) | 一意な ID の生成に Vue 3.5 の useId() を使います。 | Script |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [悪い例](#script-prefer-use-slots-bad) · [良い例](#script-prefer-use-slots-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) | setup の context.slots を useSlots() に置き換えます。 | Script |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [悪い例](#script-prefer-use-template-ref-bad) · [良い例](#script-prefer-use-template-ref-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 | Script |
| [`script/require-default-prop`](#script-require-default-prop) | [悪い例](#script-require-default-prop-bad) · [良い例](#script-require-default-prop-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) | 任意指定で Boolean ではない prop に default を用意します。 | Script |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [悪い例](#script-require-explicit-emits-bad) · [良い例](#script-require-explicit-emits-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) | emit するイベントを defineEmits または emits に宣言します。 | Script |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [悪い例](#script-require-explicit-slots-bad) · [良い例](#script-require-explicit-slots-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) | useSlots() で使う slot を defineSlots の型で宣言します。 | Script |
| [`script/require-function-return-type`](#script-require-function-return-type) | [悪い例](#script-require-function-return-type-bad) · [良い例](#script-require-function-return-type-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) | 関数に戻り値の型注釈を指定します。 | Script |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [悪い例](#script-require-prop-type-constructor-bad) · [良い例](#script-require-prop-type-constructor-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) | prop の type に文字列ではなくコンストラクターを指定します。 | Script |
| [`script/require-prop-types`](#script-require-prop-types) | [悪い例](#script-require-prop-types-bad) · [良い例](#script-require-prop-types-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) | 各 prop の型を宣言します。 | Script |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [悪い例](#script-require-symbol-provide-bad) · [良い例](#script-require-symbol-provide-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) | provide / inject のキーに衝突しにくい Symbol を使います。 | Script |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [悪い例](#script-require-typed-object-prop-bad) · [良い例](#script-require-typed-object-prop-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) | Object / Array の prop に具体的な型を指定します。 | Script |
| [`script/require-typed-ref`](#script-require-typed-ref) | [悪い例](#script-require-typed-ref-bad) · [良い例](#script-require-typed-ref-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) | 空・null・undefined で初期化する ref() に型引数を指定します。 | Script |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [悪い例](#script-require-valid-default-prop-bad) · [良い例](#script-require-valid-default-prop-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) | prop の default を宣言した型に合う値にします。 | Script |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [悪い例](#script-return-in-computed-property-bad) · [良い例](#script-return-in-computed-property-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) | computed の getter に値を返す return を用意します。 | Script |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [悪い例](#script-return-in-emits-validator-bad) · [良い例](#script-return-in-emits-validator-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) | Options API の emits validator に戻り値を用意します。 | Script |
| [`script/valid-define-emits`](#script-valid-define-emits) | [悪い例](#script-valid-define-emits-bad) · [良い例](#script-valid-define-emits-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) | defineEmits の重複や型と実行時引数の併用を検出します。 | Script |
| [`script/valid-define-options`](#script-valid-define-options) | [悪い例](#script-valid-define-options-bad) · [良い例](#script-valid-define-options-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) | defineOptions の引数と使用回数を検査します。 | Script |
| [`script/valid-define-props`](#script-valid-define-props) | [悪い例](#script-valid-define-props-bad) · [良い例](#script-valid-define-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) | defineProps の重複や型と実行時引数の併用を検出します。 | Script |
| [`script/valid-next-tick`](#script-valid-next-tick) | [悪い例](#script-valid-next-tick-bad) · [良い例](#script-valid-next-tick-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) | nextTick() の完了を await、then、または callback で扱います。 | Script |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [悪い例](#nuxt-no-nuxt-config-test-key-bad) · [良い例](#nuxt-no-nuxt-config-test-key-good) | `error` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) | Nuxt が自動判定する test 環境の手動設定を検出します。 | Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [悪い例](#nuxt-no-page-meta-runtime-values-bad) · [良い例](#nuxt-no-page-meta-runtime-values-good) | `error` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 | Nuxt |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [悪い例](#nuxt-nuxt-config-keys-order-bad) · [良い例](#nuxt-nuxt-config-keys-order-good) | `error` | `nuxt` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) | Nuxt 設定のプロパティを推奨順に並べます。 | Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [悪い例](#nuxt-prefer-import-meta-bad) · [良い例](#nuxt-prefer-import-meta-good) | `error` | `nuxt` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 | Nuxt |

## プロジェクト ルールと analyzer 契約 (66)

以下は上の 251 件の単一ファイル カタログに加わる項目です。コンポーネントやプロジェクトの文脈、または追跡する graph の例を示します。CLI、実験的な library の生成元、生成元のない契約では対応範囲が異なります。[ファイル間検査の概要](./cross-file.md)を確認してください。

| ルール / コード | 例 | 現在の対応 |
| --- | --- | --- |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [悪い例](#ecosystem-vue-router-unknown-route-bad) · [良い例](#ecosystem-vue-router-unknown-route-good) | CLI のプロジェクト検査 |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [悪い例](#ecosystem-vue-router-extra-param-bad) · [良い例](#ecosystem-vue-router-extra-param-good) | CLI のプロジェクト検査 |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [悪い例](#ecosystem-vue-router-param-type-bad) · [良い例](#ecosystem-vue-router-param-type-good) | CLI のプロジェクト検査 |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [悪い例](#ecosystem-vue-router-missing-param-bad) · [良い例](#ecosystem-vue-router-missing-param-good) | CLI のプロジェクト検査 |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [悪い例](#html-cross-component-nesting-bad) · [良い例](#html-cross-component-nesting-good) | CLI のプロジェクト検査 |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [悪い例](#vue-cross-file-attrs-fallthrough-bad) · [良い例](#vue-cross-file-attrs-fallthrough-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [悪い例](#vize-croquis-cf-array-mutation-bad) · [良い例](#vize-croquis-cf-array-mutation-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [悪い例](#vize-croquis-cf-async-boundary-bad) · [良い例](#vize-croquis-cf-async-boundary-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [悪い例](#vize-croquis-cf-async-no-suspense-bad) · [良い例](#vize-croquis-cf-async-no-suspense-good) | library の生成元あり。必要な source 情報が未対応 |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [悪い例](#vize-croquis-cf-browser-api-ssr-bad) · [良い例](#vize-croquis-cf-browser-api-ssr-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [悪い例](#vize-croquis-cf-circular-dep-bad) · [良い例](#vize-croquis-cf-circular-dep-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [悪い例](#vize-croquis-cf-circular-reactive-dependency-bad) · [良い例](#vize-croquis-cf-circular-reactive-dependency-good) | CLI: 追跡 graph の例 |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [悪い例](#vize-croquis-cf-closure-captures-reactive-bad) · [良い例](#vize-croquis-cf-closure-captures-reactive-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [悪い例](#vize-croquis-cf-composable-outside-setup-bad) · [良い例](#vize-croquis-cf-composable-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [悪い例](#vize-croquis-cf-computed-side-effects-bad) · [良い例](#vize-croquis-cf-computed-side-effects-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [悪い例](#vize-croquis-cf-deep-import-bad) · [良い例](#vize-croquis-cf-deep-import-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [悪い例](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-destructuring-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [悪い例](#vize-croquis-cf-di-outside-setup-bad) · [良い例](#vize-croquis-cf-di-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [悪い例](#vize-croquis-cf-dom-access-without-next-tick-bad) · [良い例](#vize-croquis-cf-dom-access-without-next-tick-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [悪い例](#vize-croquis-cf-duplicate-id-bad) · [良い例](#vize-croquis-cf-duplicate-id-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [悪い例](#vize-croquis-cf-event-listener-leak-bad) · [良い例](#vize-croquis-cf-event-listener-leak-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [悪い例](#vize-croquis-cf-event-modifier-bad) · [良い例](#vize-croquis-cf-event-modifier-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [悪い例](#vize-croquis-cf-hydration-risk-bad) · [良い例](#vize-croquis-cf-hydration-risk-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [悪い例](#vize-croquis-cf-inherit-attrs-unused-bad) · [良い例](#vize-croquis-cf-inherit-attrs-unused-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [悪い例](#vize-croquis-cf-inject-without-symbol-bad) · [良い例](#vize-croquis-cf-inject-without-symbol-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [悪い例](#vize-croquis-cf-injected-async-mutation-race-bad) · [良い例](#vize-croquis-cf-injected-async-mutation-race-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [悪い例](#vize-croquis-cf-lifecycle-outside-setup-bad) · [良い例](#vize-croquis-cf-lifecycle-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [悪い例](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [良い例](#vize-croquis-cf-lifecycle-without-cleanup-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [悪い例](#vize-croquis-cf-missing-required-prop-bad) · [良い例](#vize-croquis-cf-missing-required-prop-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [悪い例](#vize-croquis-cf-missing-suspense-bad) · [良い例](#vize-croquis-cf-missing-suspense-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [悪い例](#vize-croquis-cf-module-scope-reactive-bad) · [良い例](#vize-croquis-cf-module-scope-reactive-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [悪い例](#vize-croquis-cf-multi-root-attrs-bad) · [良い例](#vize-croquis-cf-multi-root-attrs-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [悪い例](#vize-croquis-cf-mutated-after-escape-bad) · [良い例](#vize-croquis-cf-mutated-after-escape-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [悪い例](#vize-croquis-cf-non-reactive-provide-bad) · [良い例](#vize-croquis-cf-non-reactive-provide-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [悪い例](#vize-croquis-cf-non-unique-id-bad) · [良い例](#vize-croquis-cf-non-unique-id-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [悪い例](#vize-croquis-cf-object-identity-comparison-bad) · [良い例](#vize-croquis-cf-object-identity-comparison-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [悪い例](#vize-croquis-cf-pinia-getter-bad) · [良い例](#vize-croquis-cf-pinia-getter-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [悪い例](#vize-croquis-cf-prop-type-mismatch-bad) · [良い例](#vize-croquis-cf-prop-type-mismatch-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [悪い例](#vize-croquis-cf-provide-inject-type-bad) · [良い例](#vize-croquis-cf-provide-inject-type-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [悪い例](#vize-croquis-cf-provide-without-symbol-bad) · [良い例](#vize-croquis-cf-provide-without-symbol-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [悪い例](#vize-croquis-cf-reactive-export-bad) · [良い例](#vize-croquis-cf-reactive-export-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [悪い例](#vize-croquis-cf-reactivity-outside-setup-bad) · [良い例](#vize-croquis-cf-reactivity-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [悪い例](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-reassignment-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [悪い例](#vize-croquis-cf-reference-escapes-scope-bad) · [良い例](#vize-croquis-cf-reference-escapes-scope-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [悪い例](#vize-croquis-cf-setup-context-violation-bad) · [良い例](#vize-croquis-cf-setup-context-violation-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [悪い例](#vize-croquis-cf-shallow-deep-access-bad) · [良い例](#vize-croquis-cf-shallow-deep-access-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [悪い例](#vize-croquis-cf-spread-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-spread-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [悪い例](#vize-croquis-cf-suspense-no-fallback-bad) · [良い例](#vize-croquis-cf-suspense-no-fallback-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [悪い例](#vize-croquis-cf-template-ref-timing-bad) · [良い例](#vize-croquis-cf-template-ref-timing-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [悪い例](#vize-croquis-cf-toraw-mutation-bad) · [良い例](#vize-croquis-cf-toraw-mutation-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [悪い例](#vize-croquis-cf-uncaught-error-bad) · [良い例](#vize-croquis-cf-uncaught-error-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [悪い例](#vize-croquis-cf-undeclared-emit-bad) · [良い例](#vize-croquis-cf-undeclared-emit-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [悪い例](#vize-croquis-cf-undeclared-prop-bad) · [良い例](#vize-croquis-cf-undeclared-prop-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [悪い例](#vize-croquis-cf-undefined-slot-bad) · [良い例](#vize-croquis-cf-undefined-slot-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [悪い例](#vize-croquis-cf-unhandled-event-bad) · [良い例](#vize-croquis-cf-unhandled-event-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [悪い例](#vize-croquis-cf-unmatched-inject-bad) · [良い例](#vize-croquis-cf-unmatched-inject-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [悪い例](#vize-croquis-cf-unmatched-listener-bad) · [良い例](#vize-croquis-cf-unmatched-listener-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [悪い例](#vize-croquis-cf-unregistered-component-bad) · [良い例](#vize-croquis-cf-unregistered-component-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [悪い例](#vize-croquis-cf-unresolved-import-bad) · [良い例](#vize-croquis-cf-unresolved-import-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [悪い例](#vize-croquis-cf-unused-attrs-bad) · [良い例](#vize-croquis-cf-unused-attrs-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [悪い例](#vize-croquis-cf-unused-emit-bad) · [良い例](#vize-croquis-cf-unused-emit-good) | 実験的な library の生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [悪い例](#vize-croquis-cf-unused-provide-bad) · [良い例](#vize-croquis-cf-unused-provide-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [悪い例](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [悪い例](#vize-croquis-cf-watch-can-be-computed-bad) · [良い例](#vize-croquis-cf-watch-can-be-computed-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [悪い例](#vize-croquis-cf-watcheffect-async-bad) · [良い例](#vize-croquis-cf-watcheffect-async-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [悪い例](#vize-croquis-cf-watcher-outside-setup-bad) · [良い例](#vize-croquis-cf-watcher-outside-setup-good) | 契約のみ。生成元なし |

## 単一ファイルの例

<span id="petite-vue-no-unsupported-directive"></span>

### `petite-vue/no-unsupported-directive`

petite-vue が対応しないディレクティブを検出します。

[悪い例](#petite-vue-no-unsupported-directive-bad) · [良い例](#petite-vue-no-unsupported-directive-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**悪い**

`v-memo`、`v-slot:header`、カスタムの `v-my-directive` は petite-vue の対応ディレクティブ一覧にありません。petite-vue の script によって、この HTML が対象の方言として判定されます。

```html
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**良い**

対応している `v-scope`、`v-effect`、`v-if`、`v-bind`、`v-on` を使い、未対応のディレクティブへの依存を取り除きます。

```html
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [全ルール](all.md)

<span id="petite-vue-valid-v-effect"></span>

### `petite-vue/valid-v-effect`

v-effect に空でない式を指定します。

[悪い例](#petite-vue-valid-v-effect-bad) · [良い例](#petite-vue-valid-v-effect-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**悪い**

各 `v-effect` の値が未指定、空文字、空白のみであり、実行する式がありません。

```html
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**良い**

一方は `el.textContent` を更新し、もう一方は `count` を増やす式を指定しています。このルールが確認するのは式が空でないことであり、処理内容の妥当性ではありません。

```html
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [全ルール](all.md)

<span id="petite-vue-valid-v-scope"></span>

### `petite-vue/valid-v-scope`

v-scope の値にオブジェクトリテラルを指定します。

[悪い例](#petite-vue-valid-v-scope-bad) · [良い例](#petite-vue-valid-v-scope-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**悪い**

値を持つ四つの `v-scope` は識別子、関数呼び出し、算術式、数値であり、いずれもオブジェクトリテラルではありません。

```html
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**良い**

値を省略した `v-scope` はルートスコープを使います。ほかは括弧付きのものも含めてオブジェクトリテラルであり、このルールで許可されます。

```html
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [全ルール](all.md)

<span id="vue-multi-word-component-names"></span>

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

<span id="vue-no-child-content"></span>

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

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**良い**

子の文字を取り除き、p の内容を v-text だけで指定します。

```vue
<template>
  <p v-text="message" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [全ルール](all.md)

<span id="vue-no-deprecated-filter"></span>

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

```vue
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**良い**

通常の式で capitalize(message) を呼び出します。

```vue
<template>
{{ capitalize(message) }}
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [全ルール](all.md)

<span id="vue-no-deprecated-functional-template"></span>

### `vue/no-deprecated-functional-template`

SFC の template で削除済みの functional 属性を検出します。

[悪い例](#vue-no-deprecated-functional-template-bad) · [良い例](#vue-no-deprecated-functional-template-good)

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

```vue
<template functional>
<div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**良い**

functional を取り除き、コンポーネントの msg を直接参照します。

```vue
<template>
<div>{{ msg }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [全ルール](all.md)

<span id="vue-no-deprecated-html-element-is"></span>

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

```vue
<template>
<div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**良い**

動的な component では :is を使い、標準要素では vue: の接頭辞を明示します。

```vue
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [全ルール](all.md)

<span id="vue-no-deprecated-inline-template"></span>

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

```vue
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**良い**

inline-template を取り除き、同じ内容を通常の形で渡します。

```vue
<template>
<Card><p>Details</p></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [全ルール](all.md)

<span id="vue-no-deprecated-router-link-tag-prop"></span>

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

```vue
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**良い**

slot から navigate を受け取り、明示的に記述した button で実行します。

```vue
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [全ルール](all.md)

<span id="vue-no-deprecated-scope-attribute"></span>

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

```vue
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**良い**

現在の default slot のディレクティブで、同じ props を受け取ります。

```vue
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [全ルール](all.md)

<span id="vue-no-deprecated-slot-attribute"></span>

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

```vue
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

```vue
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [全ルール](all.md)

<span id="vue-no-deprecated-slot-scope-attribute"></span>

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

```vue
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**良い**

#default で同じ props を受け取り、slot-scope を取り除きます。

```vue
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [全ルール](all.md)

<span id="vue-no-deprecated-v-bind-sync"></span>

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

```vue
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**良い**

一方向なら通常の title のバインディング、更新の受け取りが必要なら v-model:title を使います。

```vue
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [全ルール](all.md)

<span id="vue-no-deprecated-v-on-native-modifier"></span>

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

```vue
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**良い**

.native を取り除き、.stop などの他の modifier は残します。

```vue
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [全ルール](all.md)

<span id="vue-no-deprecated-v-on-number-modifiers"></span>

### `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済み数値 keyCode modifier を検出します。

[悪い例](#vue-no-deprecated-v-on-number-modifiers-bad) · [良い例](#vue-no-deprecated-v-on-number-modifiers-good)

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

```vue
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**良い**

キーの名前を使い、enter と esc の modifier に変更します。

```vue
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [全ルール](all.md)

<span id="vue-no-dupe-v-else-if"></span>

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

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**良い**

二つ目で別の loading の状態を検査し、else-if に到達できる条件にします。

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [全ルール](all.md)

<span id="vue-no-duplicate-attributes"></span>

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

```vue
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**良い**

二つのクラスを一つの class 属性にまとめます。

```vue
<template>
  <button class="primary large">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [全ルール](all.md)

<span id="vue-no-multiple-template-root"></span>

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

```vue
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**良い**

section で囲み、ルートを一つにします。単一ルートの契約が必要な場合にだけ有効にする規約です。

```vue
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [全ルール](all.md)

<span id="vue-no-mutating-props"></span>

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

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**良い**

次の値を update:count で通知し、prop の変更は親が行う形にします。

```vue
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

<span id="vue-no-reserved-component-names"></span>

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

```vue
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**良い**

標準の button と重複しない、アプリの AppButton の名前を指定します。

```vue
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

<span id="vue-no-template-key"></span>

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

```vue
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**良い**

template の v-for に key を付け、繰り返す各 fragment を識別します。

```vue
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [全ルール](all.md)

<span id="vue-no-textarea-mustache"></span>

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

```vue
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**良い**

v-model で、編集する textarea の値と message を関連付けます。

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [全ルール](all.md)

<span id="vue-no-unused-components"></span>

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

```vue
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

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [全ルール](all.md)

<span id="vue-no-unused-vars"></span>

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

```vue
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

```vue
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

<span id="vue-no-use-v-if-with-v-for"></span>

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

```vue
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**良い**

computed で表示する項目を先に絞り込み、テンプレートはその配列を繰り返します。

```vue
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

<span id="vue-no-useless-template-attributes"></span>

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

```vue
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**良い**

実際に表示する p に class を移し、構造を指定する template の v-if は残します。

```vue
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [全ルール](all.md)

<span id="vue-no-v-for-template-key-on-child"></span>

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

```vue
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**良い**

template の v-for に key を移し、繰り返す fragment 全体を識別します。

```vue
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [全ルール](all.md)

<span id="vue-no-v-html"></span>

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

```vue
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**良い**

mustache の補間で、HTML を挿入せず content をエスケープした文字として表示します。

```vue
<template>
  <article>{{ content }}</article>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [全ルール](all.md)

<span id="vue-no-v-text-v-html-on-component"></span>

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

```vue
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**良い**

標準の HTML 要素にはディレクティブを使えます。MyComponent には default slot から内容を渡します。

```vue
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [全ルール](all.md)

<span id="vue-permitted-contents"></span>

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

```vue
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

```vue
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [全ルール](all.md)

<span id="vue-require-component-is"></span>

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

```vue
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**良い**

`:is="currentComponent"` で描画対象を指定します。対象は実行時に変更できます。

```vue
<template>
  <component :is="currentComponent" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [全ルール](all.md)

<span id="vue-require-toggle-inside-transition"></span>

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

```vue
<template>
<transition>
<div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**良い**

`v-if="show"` で子の有無を切り替え、enter / leave の対象にします。

```vue
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [全ルール](all.md)

<span id="vue-require-v-for-key"></span>

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

```vue
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**良い**

`:key="item.id"` で、現在の位置ではなく項目の識別子を各ノードに付けます。

```vue
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [全ルール](all.md)

<span id="vue-use-v-on-exact"></span>

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

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**良い**

`.exact` で通常の handler を修飾キーのない click に限定し、Ctrl 専用の handler と分けます。

```vue
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

<span id="vue-valid-attribute-name"></span>

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

```vue
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**良い**

`my-attr` は正しい属性名で、テンプレート parser が属性と値を読み取れます。

```vue
<template>
<div my-attr="value"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [全ルール](all.md)

<span id="vue-valid-template-root"></span>

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

```vue
<template>
<template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**良い**

描画される `<div>` をルートにします。Vue 3 の fragment 全般を一つのルートに制限する例ではありません。

```vue
<template>
<div>content</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [全ルール](all.md)

<span id="vue-valid-v-bind"></span>

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

```vue
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**良い**

属性と式を指定するか object を binding します。Vue 3.4 以降では `:loading` の同名省略形も使えます。

```vue
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [全ルール](all.md)

<span id="vue-valid-v-cloak"></span>

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

```vue
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**良い**

値のない `v-cloak` を使います。mount 後に Vue が属性を除くまで CSS で非表示にできます。

```vue
<template>
<div v-cloak></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [全ルール](all.md)

<span id="vue-valid-v-else"></span>

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

```vue
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**良い**

対応する `v-if` の直後に、値のない `v-else` を置きます。

```vue
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [全ルール](all.md)

<span id="vue-valid-v-for"></span>

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

```vue
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**良い**

`item in items` や `(item, index) of items` の完全な式と、例の key を使います。

```vue
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [全ルール](all.md)

<span id="vue-valid-v-html"></span>

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

```vue
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**良い**

`v-html="html"` で正しい式を渡します。構文が正しくても HTML の無害化や未信頼の内容の安全性は保証されません。

```vue
<template>
<div v-html="html"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [全ルール](all.md)

<span id="vue-valid-v-if"></span>

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

```vue
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**良い**

`ready` や `count > 0` の式を各 `v-if` に指定し、競合する else directive を除きます。

```vue
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [全ルール](all.md)

<span id="vue-valid-v-memo"></span>

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

```vue
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**良い**

`v-memo="[valueA, valueB]"` でメモ化に使う依存配列を渡します。

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [全ルール](all.md)

<span id="vue-valid-v-model"></span>

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

```vue
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**良い**

input、select、textarea、カスタム コンポーネントを例の書き込み可能な変数に binding します。

```vue
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [全ルール](all.md)

<span id="vue-valid-v-on"></span>

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

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**良い**

イベントと handler を指定するか、引数なしの `v-on` に listener object を渡します。

```vue
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全ルール](all.md)

<span id="vue-valid-v-once"></span>

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

```vue
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**良い**

値のない `v-once` で、サブツリーを一度だけ描画する対象にします。

```vue
<template>
<div v-once></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [全ルール](all.md)

<span id="vue-valid-v-show"></span>

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

```vue
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**良い**

`<div>` のような描画される要素に表示条件を指定します。

```vue
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [全ルール](all.md)

<span id="vue-valid-v-slot"></span>

### `vue/valid-v-slot`

v-slot の適用先・宣言・modifier を検査します。

[悪い例](#vue-valid-v-slot-bad) · [良い例](#vue-valid-v-slot-good)

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

```vue
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**良い**

default slot はコンポーネント上、名前付き slot は子の `<template #header>` に宣言します。

```vue
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [全ルール](all.md)

<span id="vue-valid-v-text"></span>

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

```vue
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**良い**

`v-text="msg"` は構文として有効です。別の `vue/no-v-text` は mustache の使用を推奨する場合があります。

```vue
<template>
<div v-text="msg"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [全ルール](all.md)

<span id="vue-attribute-hyphenation"></span>

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

```vue
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**良い**

first-name に変更し、ハイフンで区切る既定の属性名の方針に合わせます。

```vue
<template>
<UserCard first-name="Ada" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [全ルール](all.md)

<span id="vue-component-definition-name-casing"></span>

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

<span id="vue-html-quotes"></span>

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

```vue
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**良い**

通常の属性とディレクティブの式をダブルクォートで囲みます。

```vue
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [全ルール](all.md)

<span id="vue-html-self-closing"></span>

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

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**良い**

コンポーネントと void 要素を自己終了にします。内容がある div は閉じタグを残します。

```vue
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

<span id="vue-mustache-interpolation-spacing"></span>

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

```vue
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**良い**

式と mustache の開始・終了の両方の区切りに空白を入れます。

```vue
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [全ルール](all.md)

<span id="vue-no-multi-spaces"></span>

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

```vue
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**良い**

同じ class と id 属性を保ち、属性間の連続した空白を一つの空白に揃えます。

```vue
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [全ルール](all.md)

<span id="vue-no-template-shadow"></span>

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

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**良い**

内側は child に変更し、外側の item と内側の child を分けます。

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全ルール](all.md)

<span id="vue-no-unused-properties"></span>

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

```vue
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

<span id="vue-prop-name-casing"></span>

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

```vue
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**良い**

宣言とテンプレートの参照を、camelCase の userName に揃えます。

```vue
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [全ルール](all.md)

<span id="vue-v-bind-style"></span>

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

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**良い**

同じ式を `:class` にします。値の型ではなく directive の書き方を検査するルールです。

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [全ルール](all.md)

<span id="vue-v-on-style"></span>

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

```vue
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**良い**

handler を変えずに `@click` の省略形を使います。

```vue
<template>
  <div @click="handleClick"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [全ルール](all.md)

<span id="vue-v-slot-style"></span>

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

```vue
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

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全ルール](all.md)

<span id="ssr-no-browser-globals-in-ssr"></span>

### `ssr/no-browser-globals-in-ssr`

SSR で実行されるコードのブラウザー専用グローバル参照を検出します。

[悪い例](#ssr-no-browser-globals-in-ssr-bad) · [良い例](#ssr-no-browser-globals-in-ssr-good)

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
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**悪い**

setup の実行時に `window.innerWidth` を直接読みますが、サーバー上のコンポーネント実行時には `window` が存在しません。

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**良い**

初期の幅をサーバーでも扱える ref の値にし、ブラウザー API の参照を SSR の setup ではなくクライアントで実行する `onMounted` に移します。

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [全ルール](all.md)

<span id="ssr-no-hydration-mismatch"></span>

### `ssr/no-hydration-mismatch`

サーバーとクライアントで一致しないテンプレート値を検出します。

[悪い例](#ssr-no-hydration-mismatch-bad) · [良い例](#ssr-no-hydration-mismatch-good)

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
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**悪い**

テンプレートが描画時に `Math.random()` を評価し、同じ段落でもサーバーとクライアントで異なるテキストになり得ます。

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**良い**

新たな乱数ではなく、安定した `seed` の状態を段落に描画します。この Nuxt 形式の例では `useState` が状態を共有し、初期値も定数 `"stable"` です。

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [全ルール](all.md)

<span id="vue-a11y-img-alt"></span>

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

```vue
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**良い**

情報のある画像には説明、装飾には空の alt、動的な画像には説明のバインディングを指定します。

```vue
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

<span id="vue-attribute-order"></span>

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

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**良い**

v-if、id、イベントの順に並べ、ルールの順序に合わせます。

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [全ルール](all.md)

<span id="vue-component-name-in-template-casing"></span>

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

```vue
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

```vue
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

<span id="vue-html-button-has-type"></span>

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

```vue
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**良い**

button、submit、reset を明示します。バインドする type は動的な値として扱います。

```vue
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [全ルール](all.md)

<span id="vue-max-template-complexity"></span>

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

```vue
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

```vue
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [全ルール](all.md)

<span id="vue-no-array-index-key"></span>

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

```vue
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**良い**

item.id を key に使い、位置が変わっても項目の識別子を維持します。

```vue
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [全ルール](all.md)

<span id="vue-no-bare-strings-in-template"></span>

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

```vue
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

```vue
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

<span id="vue-no-boolean-attr-value"></span>

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

```vue
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**良い**

値を付けず、boolean 属性があることだけで同じ状態を表します。

```vue
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [全ルール](all.md)

<span id="vue-no-empty-component-block"></span>

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

```vue
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**良い**

残すブロックには、マークアップ、script の宣言、style の宣言を入れます。

```vue
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

<span id="vue-no-inline-style"></span>

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

```vue
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**良い**

固定した色にはクラスを使います。ratio に依存する幅の動的な style は、静的属性の検査の対象外です。

```vue
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [全ルール](all.md)

<span id="vue-no-invalid-html-attribute"></span>

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

```vue
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**良い**

a の rel に、ヘルプへの参照を表す help を指定します。

```vue
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [全ルール](all.md)

<span id="vue-no-lone-template"></span>

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

```vue
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**良い**

不要な template を取り除き、div に p を直接入れます。

```vue
<template>
<div><p>Details</p></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [全ルール](all.md)

<span id="vue-no-multiple-objects-in-class"></span>

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

```vue
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**良い**

条件を一つの object にまとめます。object 一つと文字列の組、literal ではない要素の配列は許可されます。

```vue
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [全ルール](all.md)

<span id="vue-no-negated-v-if-condition"></span>

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

```vue
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

<span id="vue-no-non-component-keep-alive-child"></span>

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

```vue
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

```vue
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

<span id="vue-no-preprocessor-lang"></span>

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

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**良い**

同じ CSS から preprocessor の lang を取り除きます。規約の修正例で、現在の実行結果の診断の違いを示すものではありません。

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [全ルール](all.md)

<span id="vue-no-root-v-if"></span>

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

```vue
<template>
<div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**良い**

外側の div をルートとして残し、内側の p に表示条件を指定します。

```vue
<template>
<div>
<p v-if="show">content</p>
</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [全ルール](all.md)

<span id="vue-no-script-non-standard-lang"></span>

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

```vue
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**良い**

lang=ts と通常の TypeScript の宣言を使い、意図した言語の方針を示します。

```vue
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [全ルール](all.md)

<span id="vue-no-src-attribute"></span>

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

```vue
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**良い**

各ブロックに内容を記述し、外部の src 属性を使いません。

```vue
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

<span id="vue-no-static-inline-styles"></span>

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

```vue
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**良い**

notice のクラスと scoped の CSS に、変化しない色を移します。

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [全ルール](all.md)

<span id="vue-no-template-lang"></span>

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

```vue
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**良い**

lang を取り除き、通常の HTML の p を直接記述します。規約の例で、現在の SFC の診断を約束するものではありません。

```vue
<template>
<p>Notice</p>
</template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [全ルール](all.md)

<span id="vue-no-template-target-blank"></span>

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

```vue
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**良い**

同じリンクに noopener noreferrer を指定します。

```vue
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [全ルール](all.md)

<span id="vue-no-undefined-refs"></span>

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

```vue
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**良い**

script setup で宣言済みの message を補間で参照し、未定義の名前を取り除きます。

```vue
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [全ルール](all.md)

<span id="vue-no-unsafe-url"></span>

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

```vue
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**良い**

実行可能な URL を除き、通常のローカルの移動先 /next に変更します。

```vue
<template>
<a href="/next">Continue</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [全ルール](all.md)

<span id="vue-no-unsandboxed-iframe"></span>

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

```vue
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**良い**

sandbox で制限し、script が必要な場合にだけ allow-scripts を明示します。

```vue
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [全ルール](all.md)

<span id="vue-no-unused-refs"></span>

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

```vue
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**良い**

inputEl のテンプレート ref に、script setup の同じ名前の ref を対応させます。

```vue
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [全ルール](all.md)

<span id="vue-no-unused-setup-bindings"></span>

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

```vue
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**良い**

script setup で宣言した message を p の補間で参照し、未使用の宣言を残しません。

```vue
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [全ルール](all.md)

<span id="vue-no-use-v-else-with-v-for"></span>

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

```vue
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**良い**

template に v-else を分け、その子の p に v-for を指定します。

```vue
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [全ルール](all.md)

<span id="vue-no-useless-mustaches"></span>

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

```vue
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**良い**

固定の文字は直接書きます。変数、値を埋め込む template string、区切りの空白の補間は残します。

```vue
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [全ルール](all.md)

<span id="vue-no-useless-v-bind"></span>

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

```vue
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**良い**

固定の値は静的な属性にし、変数や補間がある値はバインディングを残します。

```vue
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [全ルール](all.md)

<span id="vue-no-v-text"></span>

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

```vue
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**良い**

同じ文字のバインディングを、要素の内容の mustache で指定します。

```vue
<template>
<div>{{ message }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [全ルール](all.md)

<span id="vue-prefer-props-shorthand"></span>

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

```vue
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

```vue
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

<span id="vue-prefer-true-attribute-shorthand"></span>

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

```vue
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**良い**

標準の boolean 属性は省略形にします。false のバインディングやコンポーネントの prop は値を残します。

```vue
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [全ルール](all.md)

<span id="vue-require-component-registration"></span>

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

```vue
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**良い**

`MyButton` は例の `globals` に含まれます。既知のグローバル登録を検査対象から除く設定で、import や登録そのものは行いません。

```vue
<template>
<MyButton />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [全ルール](all.md)

<span id="vue-require-scoped-style"></span>

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

```vue
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**良い**

同じセレクタと宣言に `scoped` を付け、Vue のコンポーネント スコープを適用します。

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [全ルール](all.md)

<span id="vue-scoped-event-names"></span>

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

```vue
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

```vue
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

<span id="vue-sfc-element-order"></span>

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

```vue
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

```vue
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

<span id="vue-single-style-block"></span>

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

```vue
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

<span id="vue-slot-name-casing"></span>

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

```vue
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**良い**

`#my-slot` を kebab-case にします。受け取る slot の名前も合わせます。

```vue
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [全ルール](all.md)

<span id="vue-this-in-template"></span>

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

```vue
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**良い**

`message`、`className`、`handleClick` を直接使います。文字列 `'this.is.a.string'` はメンバー参照ではないため残します。

```vue
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [全ルール](all.md)

<span id="vue-v-on-event-hyphenation"></span>

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

```vue
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**良い**

`@my-event` に変更します。例のネイティブ要素の listener と動的なイベント引数はこの検査の対象外です。

```vue
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [全ルール](all.md)

<span id="vue-v-on-handler-style"></span>

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

```vue
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**良い**

handler の参照を使うか、インライン処理が必要なら arrow / function 式で関数の境界を明示します。

```vue
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [全ルール](all.md)

<span id="vue-warn-custom-block"></span>

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

```vue
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

```vue
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [全ルール](all.md)

<span id="vue-warn-custom-directive"></span>

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

```vue
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**良い**

組み込みの `v-if`、`v-model`、`v-on` を使います。規約を無効にすれば、正しく登録した custom directive は有効な Vue として使えます。

```vue
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [全ルール](all.md)

<span id="a11y-alt-text"></span>

### `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

[悪い例](#a11y-alt-text-bad) · [良い例](#a11y-alt-text-good)

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**悪い**

画像の送信ボタンには URL しかなく、操作を説明する `alt` がありません。

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**良い**

`alt="Submit search"` を追加し、検索を送信する操作の名前を指定します。

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全ルール](all.md)

<span id="a11y-anchor-has-content"></span>

### `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

[悪い例](#a11y-anchor-has-content-bad) · [良い例](#a11y-anchor-has-content-good)

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**悪い**

`/settings` へのリンクが空で、移動先を説明する内容がありません。

```vue
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**良い**

同じリンクに `Settings` の文字を入れ、移動先を示します。

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全ルール](all.md)

<span id="a11y-anchor-is-valid"></span>

### `a11y/anchor-is-valid`

リンクの href に有効な移動先を指定します。

[悪い例](#a11y-anchor-is-valid-bad) · [良い例](#a11y-anchor-is-valid-good)

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
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**悪い**

一つ目は操作に `#` を使い、二つ目は JavaScript URL を使っています。どちらも通常の移動先を持つリンクではありません。

```vue
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**良い**

`openPanel` は button で実行し、リンクには実際の移動先 `/docs/javascript-urls` を指定します。

```vue
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [全ルール](all.md)

<span id="a11y-aria-props"></span>

### `a11y/aria-props`

存在しない ARIA 属性を検出します。

[悪い例](#a11y-aria-props-bad) · [良い例](#a11y-aria-props-good)

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**悪い**

`aria-lable` は綴りが誤っており、対応する ARIA 属性ではありません。

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**良い**

正しい `aria-label` に変更してボタンの名前を指定します。

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全ルール](all.md)

<span id="a11y-aria-role"></span>

### `a11y/aria-role`

有効で抽象的ではない ARIA role を指定します。

[悪い例](#a11y-aria-role-bad) · [良い例](#a11y-aria-role-good)

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**悪い**

`datepicker` は認識される ARIA role ではありません。

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**良い**

認識される `dialog` を使い、日付を選択する領域の名前も指定します。

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全ルール](all.md)

<span id="a11y-aria-unsupported-elements"></span>

### `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

[悪い例](#a11y-aria-unsupported-elements-bad) · [良い例](#a11y-aria-unsupported-elements-good)

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
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**悪い**

ARIA 属性を使えない `meta` に `aria-hidden` を指定しています。

```vue
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**良い**

ARIA 属性だけを取り除き、文字コードの宣言は維持します。

```vue
<template>
  <meta charset="utf-8" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全ルール](all.md)

<span id="a11y-click-events-have-key-events"></span>

### `a11y/click-events-have-key-events`

クリックで操作する要素にキーボード操作も用意します。

[悪い例](#a11y-click-events-have-key-events-bad) · [良い例](#a11y-click-events-have-key-events-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

対話的な役割を持たない通常要素が対象です。button や対話的な ARIA role を持つ要素はこの検出の対象外です。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**悪い**

通常の `div` に click handler だけを指定し、キーボード操作に対応していません。

```vue
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**良い**

同じ `activate` を button に指定し、標準のキーボード操作を使います。

```vue
<template>
<button @click="activate">Activate</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [全ルール](all.md)

<span id="a11y-form-control-has-label"></span>

### `a11y/form-control-has-label`

フォーム部品に関連付けられたラベルを用意します。

[悪い例](#a11y-form-control-has-label-bad) · [良い例](#a11y-form-control-has-label-good)

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**悪い**

検索欄に入力内容を説明する label がありません。

```vue
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**良い**

入力欄を label で囲み、`Search` の文字と関連付けます。

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全ルール](all.md)

<span id="a11y-heading-has-content"></span>

### `a11y/heading-has-content`

見出しに支援技術で読める内容を用意します。

[悪い例](#a11y-heading-has-content-bad) · [良い例](#a11y-heading-has-content-good)

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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**悪い**

`h2` の見出しレベルはありますが、見出しの内容が空です。

```vue
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**良い**

同じ `h2` に `Billing settings` の内容を入れます。

```vue
<template>
  <h2>Billing settings</h2>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [全ルール](all.md)

<span id="a11y-heading-levels"></span>

### `a11y/heading-levels`

見出しの階層を飛ばした指定を検出します。

[悪い例](#a11y-heading-levels-bad) · [良い例](#a11y-heading-levels-good)

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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**悪い**

見出しが `h1` から `h3` に飛び、レベル 2 を省略しています。

```vue
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**良い**

請求設定の見出しを `h2` にし、階層を順に並べます。

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [全ルール](all.md)

<span id="a11y-iframe-has-title"></span>

### `a11y/iframe-has-title`

iframe に内容を説明する title を指定します。

[悪い例](#a11y-iframe-has-title-bad) · [良い例](#a11y-iframe-has-title-good)

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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**悪い**

決済画面の iframe に URL はありますが、内容を説明する title がありません。

```vue
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**良い**

`title="Checkout preview"` で iframe の内容を説明します。

```vue
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [全ルール](all.md)

<span id="a11y-img-alt"></span>

### `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

[悪い例](#a11y-img-alt-bad) · [良い例](#a11y-img-alt-good)

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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**悪い**

アバター画像に `alt` 属性がなく、画像の代替テキストを確認できません。

```vue
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**良い**

`alt="User avatar"` で画像の代わりとなる文字を指定します。

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全ルール](all.md)

<span id="a11y-interactive-supports-focus"></span>

### `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

[悪い例](#a11y-interactive-supports-focus-bad) · [良い例](#a11y-interactive-supports-focus-good)

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**悪い**

span に button の role と click handler を付けても、キーボードでフォーカスできる要素にはなりません。

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**良い**

フォーカスできる標準の button に変更し、同じ `open` を実行します。

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全ルール](all.md)

<span id="a11y-label-has-for"></span>

### `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

[悪い例](#a11y-label-has-for-bad) · [良い例](#a11y-label-has-for-good)

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**悪い**

離れた label に for がなく、入力欄を囲んでもいないため関連付けがありません。

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**良い**

`for="email"` を入力欄の ID と一致させて関連付けます。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全ルール](all.md)

<span id="a11y-landmark-roles"></span>

### `a11y/landmark-roles`

ランドマーク role の配置と重複を検査します。

[悪い例](#a11y-landmark-roles-bad) · [良い例](#a11y-landmark-roles-good)

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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**悪い**

同じテンプレートに main が二つあり、主要な領域が重複しています。

```vue
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**良い**

Dashboard を main として残し、Settings を名前付きの nav に変更します。

```vue
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [全ルール](all.md)

<span id="a11y-media-has-caption"></span>

### `a11y/media-has-caption`

音声・動画に字幕を用意します。

[悪い例](#a11y-media-has-caption-bad) · [良い例](#a11y-media-has-caption-good)

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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**悪い**

動画に再生操作はありますが、字幕の track がありません。

```vue
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**良い**

同じ動画に `kind="captions"` の track を追加して英語の字幕を指定します。

```vue
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [全ルール](all.md)

<span id="a11y-mouse-events-have-key-events"></span>

### `a11y/mouse-events-have-key-events`

マウス操作と対応する focus / blur 操作を用意します。

[悪い例](#a11y-mouse-events-have-key-events-bad) · [良い例](#a11y-mouse-events-have-key-events-good)

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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**悪い**

プレビューの表示切り替えを mouseenter と mouseleave だけに指定しています。

```vue
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**良い**

フォーカスできる button で、同じ操作を focus と blur からも実行します。

```vue
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [全ルール](all.md)

<span id="a11y-no-access-key"></span>

### `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

[悪い例](#a11y-no-access-key-bad) · [良い例](#a11y-no-access-key-good)

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**悪い**

`accesskey="s"` がブラウザーや支援技術のショートカットと競合する可能性があります。

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**良い**

accesskey を取り除き、通常の Save ボタンは残します。

```vue
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全ルール](all.md)

<span id="a11y-no-aria-hidden-on-focusable"></span>

### `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で隠した指定を検出します。

[悪い例](#a11y-no-aria-hidden-on-focusable-bad) · [良い例](#a11y-no-aria-hidden-on-focusable-good)

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**悪い**

フォーカスできる Close ボタンを `aria-hidden="true"` でアクセシビリティ ツリーから隠しています。

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**良い**

ボタンを隠さず、Close の aria-label を指定します。

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全ルール](all.md)

<span id="a11y-no-autofocus"></span>

### `a11y/no-autofocus`

意図せずフォーカスを移動させる autofocus を検出します。

[悪い例](#a11y-no-autofocus-bad) · [良い例](#a11y-no-autofocus-good)

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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**悪い**

検索の入力欄に autofocus があり、表示時に利用者の操作なしでフォーカスを要求します。

```vue
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**良い**

autofocus を取り除き、検索欄はそのまま残します。

```vue
<template>
  <input name="query" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [全ルール](all.md)

<span id="a11y-no-distracting-elements"></span>

### `a11y/no-distracting-elements`

marquee や blink などの注意をそらす要素を検出します。

[悪い例](#a11y-no-distracting-elements-bad) · [良い例](#a11y-no-distracting-elements-good)

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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**悪い**

marquee を使って文字を自動的に動かしています。

```vue
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**良い**

同じ案内を p に入れ、自動的に動く要素を使いません。

```vue
<template>
  <p>Limited offer</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [全ルール](all.md)

<span id="a11y-no-i-for-icon"></span>

### `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

[悪い例](#a11y-no-i-for-icon-bad) · [良い例](#a11y-no-i-for-icon-good)

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**悪い**

アイコンを i で表示しており、アイコンだけの操作を説明する文字がありません。

```vue
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**良い**

装飾の span でアイコンを隠し、別の `Delete item` の文字で操作を説明します。

```vue
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全ルール](all.md)

<span id="a11y-no-redundant-roles"></span>

### `a11y/no-redundant-roles`

要素本来の意味と重複する ARIA role を検出します。

[悪い例](#a11y-no-redundant-roles-bad) · [良い例](#a11y-no-redundant-roles-good)

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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**悪い**

button は元から button の役割を持つため、同じ role を重複して指定しています。

```vue
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**良い**

重複する role を取り除き、HTML の標準の役割を使います。

```vue
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [全ルール](all.md)

<span id="a11y-no-refer-to-non-existent-id"></span>

### `a11y/no-refer-to-non-existent-id`

文書内に存在しない ID への参照を検出します。

[悪い例](#a11y-no-refer-to-non-existent-id-bad) · [良い例](#a11y-no-refer-to-non-existent-id-good)

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
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**悪い**

aria-labelledby が save-label を参照していますが、その ID の要素がありません。

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**良い**

一致する ID の span を追加し、ボタンの名前を参照できるようにします。

```vue
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [全ルール](all.md)

<span id="a11y-no-role-presentation-on-focusable"></span>

### `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素の意味を presentation で消した指定を検出します。

[悪い例](#a11y-no-role-presentation-on-focusable-bad) · [良い例](#a11y-no-role-presentation-on-focusable-good)

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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**悪い**

focus できる Billing のリンクに role=presentation を指定し、操作可能なリンクの役割と矛盾させています。ブラウザはこの presentation の指定を無視する必要があります。

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**良い**

矛盾する presentation の指定を除き、Billing への標準のリンクの役割を使います。

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全ルール](all.md)

<span id="a11y-no-static-element-interactions"></span>

### `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

[悪い例](#a11y-no-static-element-interactions-bad) · [良い例](#a11y-no-static-element-interactions-good)

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**悪い**

操作の役割がない section に Enter キーの操作を指定しています。

```vue
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**良い**

同じ Enter キーの操作を標準の button に指定し、要素自体に操作の意味を持たせます。

```vue
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全ルール](all.md)

<span id="a11y-placeholder-label-option"></span>

### `a11y/placeholder-label-option`

select のプレースホルダー option に disabled または hidden を指定します。

[悪い例](#a11y-placeholder-label-option-bad) · [良い例](#a11y-placeholder-label-option-good)

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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**悪い**

空の値を持つ案内の option が、国の選択肢と同じように選択できる状態です。

```vue
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**良い**

disabled を追加し、案内を Japan の選択肢と区別します。

```vue
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [全ルール](all.md)

<span id="a11y-role-has-required-aria-props"></span>

### `a11y/role-has-required-aria-props`

ARIA role が必要とする属性を指定します。

[悪い例](#a11y-role-has-required-aria-props-bad) · [良い例](#a11y-role-has-required-aria-props-good)

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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**悪い**

checkbox の role に、選択状態を示す aria-checked がありません。

```vue
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**良い**

`aria-checked="false"` を追加し、checkbox に必要な状態を指定します。

```vue
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [全ルール](all.md)

<span id="a11y-tabindex-no-positive"></span>

### `a11y/tabindex-no-positive`

通常のフォーカス順序を変える正の tabindex を検出します。

[悪い例](#a11y-tabindex-no-positive-bad) · [良い例](#a11y-tabindex-no-positive-good)

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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**悪い**

正の tabindex 3 で、通常の操作要素より先に独自のフォーカス順を作っています。

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**良い**

正の tabindex を取り除き、button の標準のフォーカス順を使います。

```vue
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [全ルール](all.md)

<span id="a11y-use-list"></span>

### `a11y/use-list`

箇条書きに見えるテキストをリスト要素で表現します。

[悪い例](#a11y-use-list-bad) · [良い例](#a11y-use-list-good)

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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**悪い**

タスクを p と文字のハイフンで並べており、リストの要素を使っていません。

```vue
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**良い**

同じタスクを ul と li に入れ、リストとして表します。

```vue
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [全ルール](all.md)

<span id="vue-use-unique-element-ids"></span>

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

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**良い**

`useId()` の `emailId` を label の `for` と input の `id` の両方に binding します。

```vue
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

<span id="html-deprecated-attr"></span>

### `html/deprecated-attr`

非推奨の HTML 属性を検出します。

[悪い例](#html-deprecated-attr-bad) · [良い例](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**悪い**

p に見た目を指定する旧来の align 属性を使っています。

```vue
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**良い**

クラスと text-align: center で、配置を CSS に移します。

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [全ルール](all.md)

<span id="html-deprecated-element"></span>

### `html/deprecated-element`

非推奨の HTML 要素を検出します。

[悪い例](#html-deprecated-element-bad) · [良い例](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**悪い**

旧来の見た目のための center 要素を使っています。

```vue
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**良い**

内容は残し、section とスタイル用のクラスに置き換えます。

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [全ルール](all.md)

<span id="html-id-duplication"></span>

### `html/id-duplication`

同じテンプレート内の ID 重複を検出します。

[悪い例](#html-id-duplication-bad) · [良い例](#html-id-duplication-good)

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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**悪い**

入力欄と説明の p が同じ email の ID を使い、label の参照先が重複しています。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**良い**

入力欄は email、説明は email-help に分け、aria-describedby で説明を参照します。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [全ルール](all.md)

<span id="html-no-consecutive-br"></span>

### `html/no-consecutive-br`

連続する br 要素による余白指定を検出します。

[悪い例](#html-no-consecutive-br-bad) · [良い例](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**悪い**

一つの p の中に br を二つ続けて入れ、ブロック間の余白を作っています。

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**良い**

内容を別の p に分け、連続した br を使いません。

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [全ルール](all.md)

<span id="html-no-dupe-style-properties"></span>

### `html/no-dupe-style-properties`

静的 style 属性内のプロパティ重複を検出します。

[悪い例](#html-no-dupe-style-properties-bad) · [良い例](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**悪い**

静的な style の中で同じプロパティを重複させています。margin と MARGIN も同じプロパティとして扱われます。

```vue
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**良い**

静的な style では color と background を分けます。動的な style バインディングはこの静的属性の検査の対象外です。

```vue
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [全ルール](all.md)

<span id="html-no-duplicate-class"></span>

### `html/no-duplicate-class`

静的 class 属性内のクラス名重複を検出します。

[悪い例](#html-no-duplicate-class-bad) · [良い例](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**悪い**

静的な class の中に btn が二回あります。

```vue
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**良い**

btn は一回だけ残し、別の primary と合わせて指定します。

```vue
<template>
<div class="btn primary">click</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [全ルール](all.md)

<span id="html-no-duplicate-dt"></span>

### `html/no-duplicate-dt`

dl 内の dt の名前重複を検出します。

[悪い例](#html-no-duplicate-dt-bad) · [良い例](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**悪い**

同じ dl に API の用語を二回指定しています。

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**良い**

API の dt を一つにし、その後に二つの dd を並べます。

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [全ルール](all.md)

<span id="html-no-empty-palpable-content"></span>

### `html/no-empty-palpable-content`

可視コンテンツを期待する要素が空の場合に検出します。

[悪い例](#html-no-empty-palpable-content-bad) · [良い例](#html-no-empty-palpable-content-good)

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
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**悪い**

p、li、td がいずれも空で、意味のある内容がありません。

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**良い**

p は文字、li は補間で内容を入れ、空の td には aria-label で名前を指定します。

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [全ルール](all.md)

<span id="html-require-datetime"></span>

### `html/require-datetime`

time 要素に機械可読の datetime を指定します。

[悪い例](#html-require-datetime-bad) · [良い例](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**悪い**

time に人が読む日付だけがあり、機械が読む datetime がありません。

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**良い**

`datetime="2026-05-13"` に対応する日付を指定します。

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全ルール](all.md)

<span id="type-no-floating-promises"></span>

### `type/no-floating-promises`

処理しないまま放置された Promise を検出します。

[悪い例](#type-no-floating-promises-bad) · [良い例](#type-no-floating-promises-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**悪い**

async の `save` は Promise を返しますが、単独の `save()` 呼び出しが await も return もせず、意図的に捨てる指定もありません。

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**良い**

`void save()` で実行結果を意図的に捨てる指定をし、このルールの条件を満たします。結果を破棄する明示的な印であり、rejection を処理するものではありません。

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [全ルール](all.md)

<span id="type-no-reactivity-loss"></span>

### `type/no-reactivity-loss`

代入や呼び出しによって反応性を失うスナップショットを検出します。

[悪い例](#type-no-reactivity-loss-bad) · [良い例](#type-no-reactivity-loss-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**悪い**

`const count = state.count` がリアクティブなプロパティから数値のスナップショットを取り、その後の `state.count` の更新が変数へ反映されなくなります。

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**良い**

`toRef(state, "count")` で現在のプリミティブ値をコピーせず、`count` を元のリアクティブなプロパティにつなげます。

```vue
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [全ルール](all.md)

<span id="type-no-unsafe-template-binding"></span>

### `type/no-unsafe-template-binding`

テンプレートで安全でない型の値を使用する箇所を検出します。

[悪い例](#type-no-unsafe-template-binding-bad) · [良い例](#type-no-unsafe-template-binding-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**悪い**

補間する `value` の型を明示的に `any` とし、チェッカーがテンプレートの binding を安全な具体的な型として確認できません。

```vue
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**良い**

型注釈を `string` に変え、描画する値を変えずに、同じ補間へ検査できる具体的な型を与えます。

```vue
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [全ルール](all.md)

<span id="type-require-typed-emits"></span>

### `type/require-typed-emits`

defineEmits に型定義を指定します。

[悪い例](#type-require-typed-emits-bad) · [良い例](#type-require-typed-emits-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**悪い**

配列だけの `defineEmits(["save"])` はイベント名を宣言するだけで、型付きの payload 契約がありません。

```vue
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**良い**

`defineEmits<{ save: [] }>()` で空の payload タプルを持つ型付きの `save` イベントを宣言し、payload の引数を受け取らないことを明示します。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [全ルール](all.md)

<span id="type-require-typed-props"></span>

### `type/require-typed-props`

defineProps に型定義を指定します。

[悪い例](#type-require-typed-props-bad) · [良い例](#type-require-typed-props-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**悪い**

配列だけの `defineProps(["title"])` は `title` の名前だけを宣言し、型を指定していません。

```vue
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**良い**

`defineProps<{ title: string }>()` で、名前だけの実行時宣言に代えて `title` の string 型を明示します。

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [全ルール](all.md)

<span id="type-strict-boolean-expressions"></span>

### `type/strict-boolean-expressions`

script とテンプレートの条件式で安全な真偽判定を使います。

[悪い例](#type-strict-boolean-expressions-bad) · [良い例](#type-strict-boolean-expressions-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

typeAware とルールを明示的に有効にします。既定では null を含み得る数値は許可されず、通常の数値は許可されます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**悪い**

`if (count)` が nullable な数値の truthiness に依存し、明示的な真偽値の検査を使っていません。ゼロと未指定も同じ偽として扱います。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**良い**

`count !== undefined && count > 0` で存在と正の値を別々に検査し、省略可能な値を絞り込んだうえで明示的な真偽値の条件を作ります。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全ルール](all.md)

<span id="script-no-get-current-instance"></span>

### `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

[悪い例](#script-no-get-current-instance-bad) · [良い例](#script-no-get-current-instance-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**悪い**

Vapor を指定した setup が `getCurrentInstance` をインポートして呼び出し、Vapor 向けコンポーネントでこのルールが禁止するインスタンス API に依存しています。

```vue
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**良い**

`inject("app-config")` で明示的に提供された設定を受け取り、`getCurrentInstance` のインポートも呼び出しも使いません。

```vue
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全ルール](all.md)

<span id="script-no-next-tick"></span>

### `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

[悪い例](#script-no-next-tick-bad) · [良い例](#script-no-next-tick-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**悪い**

Vapor 向けコンポーネントが `nextTick` をインポートして await し、この移行ルールが拒否する DOM 更新待ちの依存を作っています。

```vue
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**良い**

`useTemplateRef` で input を取得し、`onMounted` でフォーカスします。例の `nextTick` への依存を、明示的なマウント時の処理に置き換えます。

```vue
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全ルール](all.md)

<span id="script-no-options-api"></span>

### `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

[悪い例](#script-no-options-api-bad) · [良い例](#script-no-options-api-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**悪い**

default export のオブジェクトに Options API の `data()` を宣言しており、このルールが禁止するコンポーネントオプションの形式です。

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**良い**

状態を Vapor の `<script setup>` 内の Composition API `ref` に移し、Options API のオブジェクトと `data` オプションを取り除きます。

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全ルール](all.md)

<span id="vapor-no-inline-template"></span>

### `vapor/no-inline-template`

Vapor で削除済みの inline-template 属性を検出します。

[悪い例](#vapor-no-inline-template-bad) · [良い例](#vapor-no-inline-template-good)

既定の重大度: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**悪い**

LegacyCard の内部のマークアップに inline-template を使っています。

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**良い**

同じマークアップを default slot として渡します。

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [全ルール](all.md)

<span id="vapor-no-vue-lifecycle-events"></span>

### `vapor/no-vue-lifecycle-events`

Vapor が対応しない要素の @vue:* lifecycle event を検出します。

[悪い例](#vapor-no-vue-lifecycle-events-bad) · [良い例](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**悪い**

input にテンプレートの @vue:mounted を指定しています。

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**良い**

script の onMounted からテンプレートの ref を参照し、input にフォーカスします。

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [全ルール](all.md)

<span id="vapor-prefer-static-class"></span>

### `vapor/prefer-static-class`

文字列リテラルの :class を静的 class に置き換えます。

[悪い例](#vapor-prefer-static-class-bad) · [良い例](#vapor-prefer-static-class-good)

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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**悪い**

変化しないクラスの文字列をバインディングで評価しています。

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**良い**

同じ panel のクラスを静的な class 属性に指定します。

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [全ルール](all.md)

<span id="vapor-require-vapor-attribute"></span>

### `vapor/require-vapor-attribute`

Vapor 向けの script setup に vapor 属性を付ける方針を適用します。

[悪い例](#vapor-require-vapor-attribute-bad) · [良い例](#vapor-require-vapor-attribute-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このルールは callback が空の placeholder です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。

**設定できる ID（現在の SFC 検出なし）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**悪い**

script setup に Vapor の指定がありません。これは規約を示す例で、現在の空の callback は診断を生成しません。

```vue
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**良い**

vapor を追加して Vapor のコンパイルを選択します。修正方針を示す例であり、現在の linter がこのルールを検出するという意味ではありません。

```vue
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

良い例は意図する規約を示します。現在の SFC の処理は、どちらの例でもこのルール固有の診断を生成しません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [全ルール](all.md)

<span id="ecosystem-nuxt-prefer-nuxt-link"></span>

### `ecosystem/nuxt-prefer-nuxt-link`

Nuxt の内部リンクに NuxtLink を使います。

[悪い例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い例](#ecosystem-nuxt-prefer-nuxt-link-good)

既定の重大度: `warning`  
プリセット: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**悪い**

Nuxt の内部ページへの移動に通常の a を使っています。

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**良い**

同じ移動先を NuxtLink に指定し、Nuxt のルーターを使います。

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [全ルール](all.md)

<span id="ecosystem-pinia-prefer-store-to-refs"></span>

### `ecosystem/pinia-prefer-store-to-refs`

Pinia store の分割代入に storeToRefs() を使います。

[悪い例](#ecosystem-pinia-prefer-store-to-refs-bad) · [良い例](#ecosystem-pinia-prefer-store-to-refs-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**悪い**

store から name を直接分割代入し、リアクティブな store の読み取りから値を切り離しています。

```vue
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**良い**

store は保持し、storeToRefs で name のリアクティブな参照を取り出します。

```vue
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [全ルール](all.md)

<span id="ecosystem-router-link-require-to"></span>

### `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

[悪い例](#ecosystem-router-link-require-to-bad) · [良い例](#ecosystem-router-link-require-to-good)

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

SFC の単一ルートにあるリンクは、親から属性を継承できるため対象外になる場合があります。この例は明示的な遷移先が必要な内部のリンクです。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**悪い**

nav の内側の RouterLink に to がなく、ルートの属性継承にも頼れません。

```vue
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**良い**

内部のリンクに `to="/settings"` で移動先を明示します。

```vue
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全ルール](all.md)

<span id="ecosystem-void-link-require-href"></span>

### `ecosystem/void-link-require-href`

Void Vue の Link に href を指定します。

[悪い例](#ecosystem-void-link-require-href-bad) · [良い例](#ecosystem-void-link-require-href-good)

既定の重大度: `error`  
プリセット: `ecosystem`  
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
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**悪い**

@void/vue から import した Link に href がありません。

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**良い**

同じ Link の href に設定画面の移動先を指定します。

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [全ルール](all.md)

<span id="ecosystem-void-link-valid-method"></span>

### `ecosystem/void-link-valid-method`

Void Vue の Link に有効な静的 method を指定します。

[悪い例](#ecosystem-void-link-valid-method-bad) · [良い例](#ecosystem-void-link-valid-method-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
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
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**悪い**

DELETE の操作に、ページ移動のリクエスト向けの prefetch を指定しています。

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**良い**

prefetch を取り除き、DELETE のリクエストを事前取得しない形にします。

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [全ルール](all.md)

<span id="ecosystem-vue-i18n-no-missing-key"></span>

### `ecosystem/vue-i18n-no-missing-key`

SFC 内の翻訳データに存在しない静的キーを検出します。

[悪い例](#ecosystem-vue-i18n-no-missing-key-bad) · [良い例](#ecosystem-vue-i18n-no-missing-key-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
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
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**悪い**

テンプレートは auth.missing を参照しますが、ローカルの英語メッセージには auth.login しかありません。

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**良い**

ローカルのメッセージにある auth.login を参照します。

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [全ルール](all.md)

<span id="ecosystem-vue-router-prefer-named-link"></span>

### `ecosystem/vue-router-prefer-named-link`

RouterLink の文字列パスを名前付きルートの指定に置き換えます。

[悪い例](#ecosystem-vue-router-prefer-named-link-bad) · [良い例](#ecosystem-vue-router-prefer-named-link-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**悪い**

RouterLink の移動先をルート名ではなく、文字列のパスで指定しています。

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**良い**

バインドする route のオブジェクトに settings の name を指定します。

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [全ルール](all.md)

<span id="ecosystem-vue-router-prefer-named-push"></span>

### `ecosystem/vue-router-prefer-named-push`

Vue Router のプログラムによる移動に名前付きルートを使います。

[悪い例](#ecosystem-vue-router-prefer-named-push-bad) · [良い例](#ecosystem-vue-router-prefer-named-push-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**悪い**

router.push に、現在の URL 表記に結び付く文字列のパスを渡しています。

```vue
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**良い**

settings のルート名を持つオブジェクトを router.push に渡します。

```vue
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [全ルール](all.md)

<span id="ecosystem-vue-test-utils-no-html-snapshot"></span>

### `ecosystem/vue-test-utils-no-html-snapshot`

wrapper.html() 全体の snapshot に依存するテストを検出します。

[悪い例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い例](#ecosystem-vue-test-utils-no-html-snapshot-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**悪い**

期待する振る舞いを検査する代わりに、wrapper の HTML 全体を snapshot にしています。

```vue
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**良い**

表示された文字に Saved が含まれることを直接検査します。

```vue
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [全ルール](all.md)

<span id="css-no-display-none"></span>

### `css/no-display-none`

表示切り替えに display: none を使う箇所で v-show を検討します。

[悪い例](#css-no-display-none-bad) · [良い例](#css-no-display-none-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**悪い**

ローカルの p を `.message` の CSS で非表示にし、テンプレートに表示条件を指定していません。

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**良い**

同じ p に `v-show="isSaved"` で表示条件を指定し、display: none を取り除きます。

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [全ルール](all.md)

<span id="css-no-hardcoded-values"></span>

### `css/no-hardcoded-values`

CSS の直接指定値を CSS 変数にまとめます。

[悪い例](#css-no-hardcoded-values-bad) · [良い例](#css-no-hardcoded-values-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**悪い**

button の余白と色に数値や 16 進の色を直接指定しています。

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**良い**

余白と色を名前付きのカスタムプロパティで参照し、トークンとして管理できる形にします。

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [全ルール](all.md)

<span id="css-no-id-selectors"></span>

### `css/no-id-selectors`

詳細度が高い CSS の ID セレクターを検出します。

[悪い例](#css-no-id-selectors-bad) · [良い例](#css-no-id-selectors-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**悪い**

`#submit` で ID セレクターにスタイルを結び付けています。

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**良い**

再利用できる `.submit` のクラスを使い、ID セレクターを取り除きます。

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [全ルール](all.md)

<span id="css-no-important"></span>

### `css/no-important`

通常のカスケードを上書きする !important を検出します。

[悪い例](#css-no-important-bad) · [良い例](#css-no-important-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**悪い**

color に `!important` を付け、通常のカスケードの優先順位を上書きしています。

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**良い**

important を使わず、カスタムプロパティから色を参照します。

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [全ルール](all.md)

<span id="css-no-utility-classes"></span>

### `css/no-utility-classes`

コンポーネント内で utility class を定義する箇所を検出します。

[悪い例](#css-no-utility-classes-bad) · [良い例](#css-no-utility-classes-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**悪い**

`.flex`、`.mt-4`、`.text-center` のように、個々の見た目を名前にしたクラスを定義しています。

```vue
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**良い**

コンポーネント固有の `.my-component` にスタイルをまとめます。

```vue
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [全ルール](all.md)

<span id="css-no-v-bind-performance"></span>

### `css/no-v-bind-performance`

CSS v-bind() の実行時コストを検討するための警告です。

[悪い例](#css-no-v-bind-performance-bad) · [良い例](#css-no-v-bind-performance-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**悪い**

変化する offset を SFC の CSS の v-bind() で参照しています。

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**良い**

変化する transform を要素の style バインディングに直接指定します。

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [全ルール](all.md)

<span id="css-prefer-logical-properties"></span>

### `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

[悪い例](#css-prefer-logical-properties-bad) · [良い例](#css-prefer-logical-properties-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**悪い**

margin-left は文字の方向に関係なく物理的な左側を指定します。

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**良い**

margin-inline-start を使い、インライン方向の開始側に余白を指定します。

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全ルール](all.md)

<span id="css-prefer-nested-selectors"></span>

### `css/prefer-nested-selectors`

子孫セレクターを CSS nesting でまとめます。

[悪い例](#css-prefer-nested-selectors-bad) · [良い例](#css-prefer-nested-selectors-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**悪い**

平らなルールの `.card .title` に、親のセレクターを含めて指定しています。

```vue
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**良い**

`.card` の中に `.title` を入れ、親子のスタイルを一緒に管理します。

```vue
<style scoped>
.card { .title { color: red; } }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [全ルール](all.md)

<span id="css-prefer-slotted"></span>

### `css/prefer-slotted`

slot や子コンポーネントに対する scoped CSS のセレクターを検査します。

[悪い例](#css-prefer-slotted-bad) · [良い例](#css-prefer-slotted-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**悪い**

scoped の CSS で、渡された要素ではなく slot の出口を対象にしています。

```vue
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**良い**

:slotted(.label) を使い、slot から渡される label の要素を対象にします。

```vue
<style scoped>
:slotted(.label) { color: red; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [全ルール](all.md)

<span id="css-require-font-display"></span>

### `css/require-font-display`

@font-face に font-display を指定します。

[悪い例](#css-require-font-display-bad) · [良い例](#css-require-font-display-good)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**悪い**

font-face にフォントの参照先はありますが、font-display の方針を指定していません。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**良い**

font-display: swap を追加し、フォールバックからフォントを表示する方針を指定します。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [全ルール](all.md)

<span id="musea-no-empty-variant"></span>

### `musea/no-empty-variant`

内容のない variant ブロックを検出します。

[悪い例](#musea-no-empty-variant-bad) · [良い例](#musea-no-empty-variant-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**悪い**

primary の名前がある variant が空で、プレビューする内容がありません。

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**良い**

variant の中に primary の Button と Save の内容を入れます。

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [全ルール](all.md)

<span id="musea-prefer-design-tokens"></span>

### `musea/prefer-design-tokens`

登録した design token に一致する直接指定値を CSS 変数で表現します。

[悪い例](#musea-prefer-design-tokens-bad) · [良い例](#musea-prefer-design-tokens-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: [型付きオプションと既定値](options.md)を参照してください。

.art.vue ファイルと下記の token 一覧が必要です。任意の色から token を推測するルールではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
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

<span id="musea-prefer-design-tokens-bad"></span>

**悪い**

art の例で、設定した primary のデザイントークンではなく、青の色を直接指定しています。

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**良い**

この例で設定する --color-primary のトークンを参照します。

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [全ルール](all.md)

<span id="musea-require-component"></span>

### `musea/require-component`

art ブロックに対象の component を指定します。

[悪い例](#musea-require-component-bad) · [良い例](#musea-require-component-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**悪い**

art に title はありますが、プレビューする component の指定がありません。

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**良い**

defineArt で ./Button.vue を art の component として指定します。

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [全ルール](all.md)

<span id="musea-require-title"></span>

### `musea/require-title`

art ブロックに title を指定します。

[悪い例](#musea-require-title-bad) · [良い例](#musea-require-title-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**悪い**

art に Button.vue の指定はありますが、title がありません。

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**良い**

defineArt のオプションに Button の title を指定します。

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [全ルール](all.md)

<span id="musea-unique-variant-names"></span>

### `musea/unique-variant-names`

同じ Art ファイル内の variant 名を一意にします。

[悪い例](#musea-unique-variant-names-bad) · [良い例](#musea-unique-variant-names-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**悪い**

同じ art の二つの variant に primary の名前を使っています。

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**良い**

primary と secondary の別々の名前を指定します。

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [全ルール](all.md)

<span id="musea-valid-variant"></span>

### `musea/valid-variant`

variant ブロックに name を指定します。

[悪い例](#musea-valid-variant-bad) · [良い例](#musea-valid-variant-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**悪い**

プレビューを識別する variant の name がありません。

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**良い**

variant に primary の name を指定します。

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [全ルール](all.md)

<span id="script-component-options-name-casing"></span>

### `script/component-options-name-casing`

コンポーネントの name オプションを PascalCase に揃えます。

[悪い例](#script-component-options-name-casing-bad) · [良い例](#script-component-options-name-casing-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**悪い**

コンポーネントの `name: 'my-component'` が kebab-case です。このルールは文字列リテラルのコンポーネント名に PascalCase を求めます。

```vue
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**良い**

`MyComponent` は大文字で始まり、英数字だけで構成されるため、名前の検査条件を満たします。

```vue
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [全ルール](all.md)

<span id="script-custom-event-name-casing"></span>

### `script/custom-event-name-casing`

emit するカスタムイベント名を指定した形式に揃えます。

[悪い例](#script-custom-event-name-casing-bad) · [良い例](#script-custom-event-name-casing-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**悪い**

emit する文字列 `my-event` にハイフンが含まれ、既定の camelCase イベント命名規則に違反しています。

```vue
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**良い**

宣言と呼び出しの両方を `myEvent` にそろえ、イベント名の一致を保ったまま既定の命名規則を満たします。kebab-case に設定した場合の期待値は異なります。

```vue
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [全ルール](all.md)

<span id="script-define-emits-declaration"></span>

### `script/define-emits-declaration`

defineEmits を型による宣言形式に揃えます。

[悪い例](#script-define-emits-declaration-bad) · [良い例](#script-define-emits-declaration-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**悪い**

`defineEmits(["change"])` は実行時の配列による宣言です。このスタイルルールは型ベースの宣言を推奨します。

```vue
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**良い**

`defineEmits<{ change: [id: number] }>()` で宣言を型引数へ移し、`emit("change", 1)` が渡す数値のペイロードも明示します。

```vue
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全ルール](all.md)

<span id="script-define-macros-order"></span>

### `script/define-macros-order`

script setup のコンパイラーマクロを一定の順に宣言します。

[悪い例](#script-define-macros-order-bad) · [良い例](#script-define-macros-order-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**悪い**

`defineProps` が `defineModel` より先にありますが、定められたマクロ順序では `defineModel` が先です。

```vue
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**良い**

宣言を `defineOptions`、`defineModel`、`defineProps`、`defineEmits`、`defineSlots` の順に並べ、無関係な実行時処理より前に置きます。

```vue
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [全ルール](all.md)

<span id="script-define-props-declaration"></span>

### `script/define-props-declaration`

defineProps を型による宣言形式に揃えます。

[悪い例](#script-define-props-declaration-bad) · [良い例](#script-define-props-declaration-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**悪い**

`defineProps({ title: String })` は実行時オブジェクトを渡しており、このルールが推奨する型ベースの props 宣言と異なります。

```vue
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**良い**

`defineProps<{ title: string }>()` の型引数で `title` を宣言し、実行時の宣言引数を使わずに `props.title` の参照を保ちます。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [全ルール](all.md)

<span id="script-define-props-destructuring"></span>

### `script/define-props-destructuring`

defineProps の分割代入スタイルを指定した方針に揃えます。

[悪い例](#script-define-props-destructuring-bad) · [良い例](#script-define-props-destructuring-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**悪い**

`defineProps` の結果を分割代入せず、単一の `props` 変数に代入しており、既定の分割代入の推奨に従っていません。

```vue
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**良い**

オブジェクトパターンで `foo` と `bar` を直接取り出し、省略可能な `bar` に既定値を付けます。Vue 3.5 以降のリアクティブな props 分割代入を前提とし、設定を `never` にした場合は逆の形式を推奨します。

```vue
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [全ルール](all.md)

<span id="script-no-arrow-functions-in-watch"></span>

### `script/no-arrow-functions-in-watch`

Options API の watch に this を持たないアロー関数を使う箇所を検出します。

[悪い例](#script-no-arrow-functions-in-watch-bad) · [良い例](#script-no-arrow-functions-in-watch-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**悪い**

Options API の watcher `value` と `other.handler` がアロー関数です。アロー関数の `this` は外側から引き継がれ、コンポーネントインスタンスとして束縛されません。

```vue
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**良い**

両ハンドラーを通常のメソッドに変え、Vue が `this` をコンポーネントに束縛できるようにします。オブジェクト形式の `deep: true` オプションも維持できます。

```vue
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [全ルール](all.md)

<span id="script-no-async-in-computed"></span>

### `script/no-async-in-computed`

computed の getter で非同期関数を使う箇所を検出します。

[悪い例](#script-no-async-in-computed-bad) · [良い例](#script-no-async-in-computed-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**悪い**

`computed` の getter が `async` であり、fetch の結果を同期的な計算値ではなく Promise として返します。

```vue
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**良い**

非同期の fetch を `watch` に移し、結果を `data.value` に保存します。クリーンアップで古い要求を中止し、無効になったコールバックが古い結果を書き込むのを防ぎます。async な computed getter は残りません。

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [全ルール](all.md)

<span id="script-no-boolean-default"></span>

### `script/no-boolean-default`

Boolean prop の冗長な default を検出します。

[悪い例](#script-no-boolean-default-bad) · [良い例](#script-no-boolean-default-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**悪い**

`disabled` と `checked` は、単独の型が `Boolean` である prop に `default` を指定しています。明示的な `false` もこのルールの対象です。

```vue
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**良い**

Boolean のみの props では `default` を省き、Vue の暗黙の false を使います。`[Boolean, String]` の共用型と Number の prop は、この検査が単独の `Boolean` コンストラクターに限られることを示します。

```vue
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [全ルール](all.md)

<span id="script-no-deep-destructure-in-props"></span>

### `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

[悪い例](#script-no-deep-destructure-in-props-bad) · [良い例](#script-no-deep-destructure-in-props-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**悪い**

代入パターンが `user` の内部まで進んで `name` を取り出し、既定で許される浅い props 分割代入の深さを超えています。

```vue
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**良い**

props オブジェクトを保ち、computed の getter で `props.user.name` を参照します。深い代入パターンを使わず、入れ子の参照を明示します。

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [全ルール](all.md)

<span id="script-no-deprecated-data-object-declaration"></span>

### `script/no-deprecated-data-object-declaration`

Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。

[悪い例](#script-no-deprecated-data-object-declaration-bad) · [良い例](#script-no-deprecated-data-object-declaration-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**悪い**

Options API の `data` がオブジェクトリテラルであり、Vue 3 では受け付けない Vue 2 の形式です。

```vue
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**良い**

`data()` が新しい `{ count: 0 }` を返すようにし、Vue 3 が求める関数形式のデータ宣言にします。

```vue
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [全ルール](all.md)

<span id="script-no-deprecated-destroyed-lifecycle"></span>

### `script/no-deprecated-destroyed-lifecycle`

Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。

[悪い例](#script-no-deprecated-destroyed-lifecycle-bad) · [良い例](#script-no-deprecated-destroyed-lifecycle-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**悪い**

タイマーの後片付けに、Vue 3 で削除された Vue 2 のライフサイクルオプション `beforeDestroy` を使っています。

```vue
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**良い**

フック名を `beforeUnmount` に変え、後片付けの本体を Vue 3 のライフサイクル名で保ちます。

```vue
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [全ルール](all.md)

<span id="script-no-deprecated-dollar-listeners-api"></span>

### `script/no-deprecated-dollar-listeners-api`

Vue 3 で $attrs に統合された $listeners を検出します。

[悪い例](#script-no-deprecated-dollar-listeners-api-bad) · [良い例](#script-no-deprecated-dollar-listeners-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**悪い**

メンバー参照と引数の裸の参照がいずれも `$listeners` を使っています。Vue 3 ではリスナーが属性に統合され、この API は削除されました。

```vue
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**良い**

参照を `this.$attrs` と setup コンテキストの `ctx.attrs` に移し、削除されたリスナー API を置き換えます。例の参照元は、それぞれのコンポーネントコンテキストで用意されている必要があります。

```vue
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [全ルール](all.md)

<span id="script-no-deprecated-dollar-scopedslots-api"></span>

### `script/no-deprecated-dollar-scopedslots-api`

Vue 3 で $slots に統合された $scopedSlots を検出します。

[悪い例](#script-no-deprecated-dollar-scopedslots-api-bad) · [良い例](#script-no-deprecated-dollar-scopedslots-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**悪い**

`this.$scopedSlots`、`ctx.$scopedSlots`、裸の `$scopedSlots` 参照が、Vue 3 で削除された Vue 2 の scoped slot API を使っています。

```vue
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**良い**

`$scopedSlots` を `$slots` に置き換え、統合された slot API を使います。この例は削除された API 名の置換を示すもので、参照元の setup コンテキストを作る例ではありません。

```vue
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [全ルール](all.md)

<span id="script-no-deprecated-events-api"></span>

### `script/no-deprecated-events-api`

Vue 3 で削除された $on / $off / $once を検出します。

[悪い例](#script-no-deprecated-events-api-bad) · [良い例](#script-no-deprecated-events-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**悪い**

`$on`、`$once`、`$off` の呼び出しが、Vue 3 で削除されたインスタンスのイベントバスメソッドを使っています。

```vue
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**良い**

有効な `$emit` は維持し、イベントバスへの購読は外部 emitter の `on` に移します。親へのイベント送信と外部イベントバスを別々の API で表します。

```vue
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [全ルール](all.md)

<span id="script-no-deprecated-props-default-this"></span>

### `script/no-deprecated-props-default-this`

prop の default / validator 内で使えなくなった this を検出します。

[悪い例](#script-no-deprecated-props-default-this-bad) · [良い例](#script-no-deprecated-props-default-this-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**悪い**

prop の既定値関数と validator が `this` を参照していますが、Vue 3 ではこれらの関数からコンポーネントインスタンスに依存できません。

```vue
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**良い**

既定値関数は引数の `props.baseSize` を使い、validator は引数の `value` を検査します。どちらも利用できないインスタンスの `this` に依存しなくなります。

```vue
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [全ルール](all.md)

<span id="script-no-dupe-keys"></span>

### `script/no-dupe-keys`

Options API の props / data / computed などのキー重複を検出します。

[悪い例](#script-no-dupe-keys-bad) · [良い例](#script-no-dupe-keys-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**悪い**

`foo` が props と data の両方に、`bar` が computed と methods の両方に宣言され、コンポーネントインスタンス上の同じキーを取り合っています。

```vue
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**良い**

prop、data、computed に別々の名前 `foo`、`bar`、`baz` を使い、オプション間の重複を取り除きます。

```vue
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [全ルール](all.md)

<span id="script-no-duplicate-attr-inheritance"></span>

### `script/no-duplicate-attr-inheritance`

fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。

[悪い例](#script-no-duplicate-attr-inheritance-bad) · [良い例](#script-no-duplicate-attr-inheritance-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**悪い**

明示的な `inheritAttrs: true` が Vue の既定動作を繰り返しています。このルールは、ルートの `$attrs` 展開が例にない場合もこの冗長なリテラルを報告します。

```vue
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**良い**

`inheritAttrs: false` は継承を無効にする指定であり、空のオプションは既定の継承を暗黙に使います。どちらも冗長な `true` を指定しません。

```vue
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [全ルール](all.md)

<span id="script-no-export-in-script-setup"></span>

### `script/no-export-in-script-setup`

script setup 内の export 文を検出します。

[悪い例](#script-no-export-in-script-setup-bad) · [良い例](#script-no-export-in-script-setup-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**悪い**

`export const count` が `<script setup>` からモジュールの値をエクスポートしようとしています。このブロックでは実行時の export は禁止されています。

```vue
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**良い**

`export` を取り除き、`count` をモジュールの export ではなく setup の変数にします。

```vue
<script setup lang="ts">
const count = 1;
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [全ルール](all.md)

<span id="script-no-import-compiler-macros"></span>

### `script/no-import-compiler-macros`

自動的に使える Vue コンパイラーマクロの import を検出します。

[悪い例](#script-no-import-compiler-macros-bad) · [良い例](#script-no-import-compiler-macros-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**悪い**

`vue` から `defineProps` と `defineEmits` をインポートしていますが、これらは `<script setup>` で直接使えるコンパイラーマクロです。

```vue
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**良い**

マクロのインポートを除き、型付きのマクロ呼び出しは保ちます。これらの宣言に実行時のインポートは不要です。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [全ルール](all.md)

<span id="script-no-internal-imports"></span>

### `script/no-internal-imports`

Vue 内部モジュールからの import を検出します。

[悪い例](#script-no-internal-imports-bad) · [良い例](#script-no-internal-imports-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**悪い**

両方の import が Vue の公開エントリーポイントではなく内部の `dist` ファイルを参照し、コンポーネントをビルド成果物のパスに依存させています。

```vue
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**良い**

必要なヘルパーを `vue` からインポートし、内部の配布ファイルの配置への依存を取り除きます。

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [全ルール](all.md)

<span id="script-no-multiple-slot-args"></span>

### `script/no-multiple-slot-args`

scoped slot 関数に複数の引数を渡す箇所を検出します。

[悪い例](#script-no-multiple-slot-args-bad) · [良い例](#script-no-multiple-slot-args-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**悪い**

slot 呼び出しが複数の位置引数、または個数不明の引数展開を渡しています。Vue の slot は位置引数の列ではなく、一つの props オブジェクトを受け取ります。

```vue
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**良い**

`{ foo, bar }` でデータを一つの引数にまとめます。`slotProps` を渡す呼び出しと引数を省いた呼び出しも、対応する slot 呼び出し形式に収まります。

```vue
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [全ルール](all.md)

<span id="script-no-potential-component-option-typo"></span>

### `script/no-potential-component-option-typo`

Options API のオプション名の入力ミスを検出します。

[悪い例](#script-no-potential-component-option-typo-bad) · [良い例](#script-no-potential-component-option-typo-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**悪い**

オプション名が既知の `methods` から一文字欠けた `method` になり、意図したメソッド宣言として扱われません。

```vue
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**良い**

キーを `methods` に修正し、`save()` を既知のコンポーネントオプション内に置きます。

```vue
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [全ルール](all.md)

<span id="script-no-reactive-destructure"></span>

### `script/no-reactive-destructure`

reactive オブジェクトの反応性を失う分割代入を検出します。

[悪い例](#script-no-reactive-destructure-bad) · [良い例](#script-no-reactive-destructure-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**悪い**

`const { count, name } = state` が `reactive` オブジェクトからプリミティブ値をコピーし、その後のプロパティ更新とのつながりを失います。

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**良い**

`toRefs(state)` を分割代入して `count` と `name` の ref を作り、それぞれを元のリアクティブなプロパティにつなげます。

```vue
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [全ルール](all.md)

<span id="script-no-ref-as-operand"></span>

### `script/no-ref-as-operand`

ref を演算の値として使う際に .value を参照します。

[悪い例](#script-no-ref-as-operand-bad) · [良い例](#script-no-ref-as-operand-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**悪い**

`count + 1` は ref が包む数値ではなく、ref オブジェクトそのものを算術演算の対象にしています。

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**良い**

`count.value + 1` で内側の数値を取り出してから加算します。script の演算では ref の値を明示的に参照します。

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [全ルール](all.md)

<span id="script-no-required-prop-with-default"></span>

### `script/no-required-prop-with-default`

required: true と default を同時に持つ prop を検出します。

[悪い例](#script-no-required-prop-with-default-bad) · [良い例](#script-no-required-prop-with-default-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**悪い**

`title` に必須指定と `"Untitled"` の既定値を同時に付け、必須の入力という契約と未指定時の代替値を併記しています。

```vue
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**良い**

`required: true` を除いて `title` を任意入力にし、`"Untitled"` を未指定時の既定値として残します。

```vue
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [全ルール](all.md)

<span id="script-no-reserved-identifiers"></span>

### `script/no-reserved-identifiers`

Vue コンパイラーが予約した識別子の宣言を検出します。

[悪い例](#script-no-reserved-identifiers-bad) · [良い例](#script-no-reserved-identifiers-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**悪い**

変数名 `__props`、`__emit`、`__sfc__` が Vue コンパイラーの生成コード用に予約された識別子と重なっています。

```vue
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**良い**

通常の名前 `props`、`emit`、`componentData` を使い、props と emits の宣言を保ったまま生成用識別子との重複を避けます。

```vue
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [全ルール](all.md)

<span id="script-no-reserved-keys"></span>

### `script/no-reserved-keys`

Options API のキーに Vue の予約名を使う箇所を検出します。

[悪い例](#script-no-reserved-keys-bad) · [良い例](#script-no-reserved-keys-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**悪い**

data の `$el` が Vue 組み込みのインスタンスプロパティと重なり、予約された `$` 接頭辞も使っています。

```vue
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**良い**

アプリケーションのデータ名を `elementLabel` に変え、組み込みのインスタンス API と予約接頭辞を避けます。

```vue
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [全ルール](all.md)

<span id="script-no-reserved-props"></span>

### `script/no-reserved-props`

prop 宣言に Vue の予約名を使う箇所を検出します。

[悪い例](#script-no-reserved-props-bad) · [良い例](#script-no-reserved-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**悪い**

オブジェクト形式の `ref` と `$foo`、配列形式の `key` は予約された prop 名です。`ref` と `key` はフレームワーク用であり、`$` で始まる名前も拒否されます。

```vue
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**良い**

通常の prop 名 `name` と `refValue` に変え、予約された名前と接頭辞を避けます。

```vue
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [全ルール](all.md)

<span id="script-no-restricted-globals"></span>

### `script/no-restricted-globals`

設定で禁止した実行環境のグローバル参照を検出します。

[悪い例](#script-no-restricted-globals-bad) · [良い例](#script-no-restricted-globals-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**悪い**

既定で制限されるグローバル `process`、`localStorage`、`sessionStorage` を直接参照し、設定やストレージ用の明示的なヘルパーを通していません。

```vue
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**良い**

`useFeatureFlag`、`authStorage.read`、`viewStorage.write` に移し、制限対象のグローバルの直接参照を除きます。残る `window.scrollY` はこのルールの既定の制限対象ではなく、SSR の安全性は別途確認が必要です。

```vue
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [全ルール](all.md)

<span id="script-no-restricted-members"></span>

### `script/no-restricted-members`

設定で禁止した object.property へのアクセスを検出します。

[悪い例](#script-no-restricted-members-bad) · [良い例](#script-no-restricted-members-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

この例では window.localStorage を禁止しています。既定の禁止リストはなく、有効にするだけでは検出されません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
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

<span id="script-no-restricted-members-bad"></span>

**悪い**

`ruleOptions` に `{ object: "window", property: "localStorage" }` を設定した場合、`window.localStorage` は禁止されたオブジェクトとメンバーの組み合わせです。このルールに既定の禁止メンバーはありません。

```vue
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**良い**

`authStorage.read("token")` でアプリケーションのストレージヘルパーに処理を任せ、設定で禁止した `window.localStorage` を参照しなくなります。

```vue
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [全ルール](all.md)

<span id="script-no-side-effects-in-computed-properties"></span>

### `script/no-side-effects-in-computed-properties`

Options API の computed getter 内の副作用を検出します。

[悪い例](#script-no-side-effects-in-computed-properties-bad) · [良い例](#script-no-side-effects-in-computed-properties-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**悪い**

`doubled` が `this.count` に代入し、`reversed` が `reverse()` で `this.items` を変更しています。どちらも値を導出すべき getter が元の状態を変更しています。

```vue
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**良い**

`doubled` は代入せず乗算結果を返します。`reversed` は配列をコピーしてから反転し、getter が元のコンポーネント状態を変更しないようにします。

```vue
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [全ルール](all.md)

<span id="script-no-top-level-ref-in-script"></span>

### `script/no-top-level-ref-in-script`

通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。

[悪い例](#script-no-top-level-ref-in-script-bad) · [良い例](#script-no-top-level-ref-in-script-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**悪い**

通常の `<script>` がモジュール直下で `count` と `user` を初期化しています。SSR では、この状態がコンポーネントインスタンスやリクエスト間で共有され得ます。

```vue
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**良い**

script setup の ref はコンポーネントごとに初期化されます。通常の script には定数、状態を作る関数、`setup()` 内で作る ref を置き、モジュール直下でリアクティブな状態を作りません。

```vue
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [全ルール](all.md)

<span id="script-no-unstable-nested-components"></span>

### `script/no-unstable-nested-components`

setup / render 内で毎回コンポーネントを定義する箇所を検出します。

[悪い例](#script-no-unstable-nested-components-bad) · [良い例](#script-no-unstable-nested-components-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**悪い**

親の `setup()` 内で `defineComponent` を呼び、setup が実行されるたびに新しい `Child` のコンポーネント定義を作っています。

```vue
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**良い**

`Child` の定義をモジュール直下に移し、`setup()` は作り直さず既存の定義を返すようにします。

```vue
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [全ルール](all.md)

<span id="script-no-unused-emit-declarations"></span>

### `script/no-unused-emit-declarations`

宣言したまま emit していないイベントを検出します。

[悪い例](#script-no-unused-emit-declarations-bad) · [良い例](#script-no-unused-emit-declarations-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**悪い**

`defineEmits` は `change` と `unused` を宣言していますが、受け取った `emit` 関数が送信する文字列イベントは `change` だけです。

```vue
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**良い**

`unused` を除き、イベント宣言を実際の送信にそろえます。この例では emit の参照を外部へ渡していないため、ローカルの使用状況から判断できます。

```vue
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [全ルール](all.md)

<span id="script-no-use-computed-property-like-method"></span>

### `script/no-use-computed-property-like-method`

Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。

[悪い例](#script-no-use-computed-property-like-method-bad) · [良い例](#script-no-use-computed-property-like-method-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**悪い**

`this.total()` は computed getter の公開する値を関数として呼び出しています。この getter が返す `3` は呼び出せません。

```vue
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**良い**

呼び出しの括弧を除いて `this.total` とし、`log` から計算済みの数値を参照します。

```vue
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [全ルール](all.md)

<span id="script-no-with-defaults"></span>

### `script/no-with-defaults`

Vue 3.5 以降の props 分割代入の既定値を勧めます。

[悪い例](#script-no-with-defaults-bad) · [良い例](#script-no-with-defaults-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**悪い**

型付きの props 宣言を `withDefaults` で包んで `count` と `name` の既定値を指定しており、ここで推奨する Vue 3.5 以降の分割代入の既定値を使っていません。

```vue
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**良い**

分割代入の変数に `count = 0` と `name = "Ada"` を直接指定し、`withDefaults` のラッパーを取り除きます。

```vue
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [全ルール](all.md)

<span id="script-prefer-computed"></span>

### `script/prefer-computed`

他の状態から導ける値を watcher で同期する代わりに computed で表現します。

[悪い例](#script-prefer-computed-bad) · [良い例](#script-prefer-computed-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

派生値だけを代入する watcher が対象です。ユーザーが編集するコピーや別の副作用を持つ処理は対象外です。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**悪い**

watcher が `count` から求めた値を別の ref `doubled` に書き込むだけであり、派生状態を手作業で同期しています。

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**良い**

`computed(() => count.value * 2)` で導出を直接表し、書き込み可能な追加 ref と同期用 watcher を取り除きます。

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [全ルール](all.md)

<span id="script-prefer-define-options"></span>

### `script/prefer-define-options`

name / inheritAttrs だけの通常 script を defineOptions() にまとめます。

[悪い例](#script-prefer-define-options-bad) · [良い例](#script-prefer-define-options-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**悪い**

通常の script の処理が `name` と `inheritAttrs` だけを持つオブジェクトの export に限られ、`defineOptions` で表せるオプションだけを宣言しています。

```vue
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**良い**

例の `data()` が実際の Options API の処理を持つため、オプションだけの script を対象とする慎重な提案の範囲外になります。この Good は許可される例外を示します。直接移行する場合は `<script setup>` 内で `defineOptions({ name: 'MyComponent', inheritAttrs: false })` を使います。

```vue
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [全ルール](all.md)

<span id="script-prefer-import-from-vue"></span>

### `script/prefer-import-from-vue`

内部パッケージではなく vue から import します。

[悪い例](#script-prefer-import-from-vue-bad) · [良い例](#script-prefer-import-from-vue-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**悪い**

`ref` と `h` を公開パッケージ `vue` ではなく、内部の `@vue/runtime-core` と `@vue/runtime-dom` からインポートしています。

```vue
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**良い**

両ヘルパーを `vue` からまとめてインポートし、内部パッケージではなく公開エントリーポイントを使います。

```vue
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [全ルール](all.md)

<span id="script-prefer-ref-over-reactive"></span>

### `script/prefer-ref-over-reactive`

状態管理に reactive() より ref() を使う方針を適用します。

[悪い例](#script-prefer-ref-over-reactive-bad) · [良い例](#script-prefer-ref-over-reactive-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**悪い**

状態を `reactive` で作り、この意見を持つルールが推奨する ref を使っていません。これはスタイルの推奨を示す例であり、reactive オブジェクト自体が不正という意味ではありません。

```vue
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**良い**

スカラーとオブジェクトの状態をどちらも `ref` で作ります。関連するフィールドを個別の ref に分ける例も含め、推奨する状態の作成形式にそろえます。

```vue
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [全ルール](all.md)

<span id="script-prefer-use-attrs"></span>

### `script/prefer-use-attrs`

setup の context.attrs を useAttrs() に置き換えます。

[悪い例](#script-prefer-use-attrs-bad) · [良い例](#script-prefer-use-attrs-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**悪い**

`setup` がコンテキスト引数の分割代入で `attrs` を受け取っています。このルールは Composition API のヘルパーへの置換を求めます。

```vue
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**良い**

setup 内で `useAttrs()` から `attrs` を取得し、第二引数に依存せず `attrs.class` の参照を保ちます。

```vue
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [全ルール](all.md)

<span id="script-prefer-use-id"></span>

### `script/prefer-use-id`

一意な ID の生成に Vue 3.5 の useId() を使います。

[悪い例](#script-prefer-use-id-bad) · [良い例](#script-prefer-use-id-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**悪い**

`id` に `Math.random()` の値が含まれ、input と label の識別子がサーバーとクライアントの描画で変わり得ます。ID の名前を持つこの変数が、ルールの認識する生成箇所です。

```vue
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**良い**

Vue 3.5 以降の `useId()` で識別子を作り、`:for` と `:id` は同じ変数を参照し続けます。ランダムな値の生成を取り除きます。

```vue
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [全ルール](all.md)

<span id="script-prefer-use-slots"></span>

### `script/prefer-use-slots`

setup の context.slots を useSlots() に置き換えます。

[悪い例](#script-prefer-use-slots-bad) · [良い例](#script-prefer-use-slots-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**悪い**

`setup` がコンテキスト引数から `slots` を分割代入で取り出しており、このルールが置換を推奨する参照形式です。

```vue
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**良い**

setup 内で `useSlots()` から slot を取得し、コンテキスト引数を使わずに render 関数と default slot の任意の呼び出しを保ちます。

```vue
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [全ルール](all.md)

<span id="script-prefer-use-template-ref"></span>

### `script/prefer-use-template-ref`

テンプレート参照に Vue 3.5 の useTemplateRef() を使います。

[悪い例](#script-prefer-use-template-ref-bad) · [良い例](#script-prefer-use-template-ref-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**悪い**

null で初期化した `input` の ref がテンプレートのリテラル `ref="input"` と対応し、通常の nullable なデータではなく要素参照であることが分かります。

```vue
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**良い**

Vue 3.5 以降の `useTemplateRef<HTMLInputElement>('input')` でテンプレート参照を明示します。対応する要素参照のない `error = ref(null)` は通常のデータであり、このルールの対象外です。

```vue
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [全ルール](all.md)

<span id="script-require-default-prop"></span>

### `script/require-default-prop`

任意指定で Boolean ではない prop に default を用意します。

[悪い例](#script-require-default-prop-bad) · [良い例](#script-require-default-prop-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**悪い**

`name` と `age` は必須でない Boolean 以外の実行時 props で、未指定時の既定値がありません。

```vue
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**良い**

`name` に `default: ''` を付けます。`enabled` は Boolean の暗黙の false を使い、必須の `id` には代替値が不要であるため、二つの除外条件も示しています。

```vue
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [全ルール](all.md)

<span id="script-require-explicit-emits"></span>

### `script/require-explicit-emits`

emit するイベントを defineEmits または emits に宣言します。

[悪い例](#script-require-explicit-emits-bad) · [良い例](#script-require-explicit-emits-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**悪い**

受け取った emit 関数が `save` を送信しますが、`defineEmits([])` にはそのイベントが宣言されていません。

```vue
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**良い**

宣言に `"save"` を追加し、送信する文字列イベントをコンポーネントの明示的なイベント契約に含めます。

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [全ルール](all.md)

<span id="script-require-explicit-slots"></span>

### `script/require-explicit-slots`

useSlots() で使う slot を defineSlots の型で宣言します。

[悪い例](#script-require-explicit-slots-bad) · [良い例](#script-require-explicit-slots-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**悪い**

型付きの `defineProps<{ id: number }>()` により TypeScript の構文があることが分かりますが、setup が `defineSlots` の宣言なしに `useSlots()` を使い、参照する slot の明示的な契約がありません。

```vue
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**良い**

`defineSlots` で `msg: string` を props に持つ `default` slot を宣言し、`useSlots()` と明示的な型付き slot 契約を併記します。

```vue
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [全ルール](all.md)

<span id="script-require-function-return-type"></span>

### `script/require-function-return-type`

関数に戻り値の型注釈を指定します。

[悪い例](#script-require-function-return-type-bad) · [良い例](#script-require-function-return-type-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**悪い**

`add` と `greet` は引数の型を指定していますが、戻り値の型を省略しています。この明示的な型指定の規則では、戻り値の推論だけでは条件を満たしません。

```vue
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**良い**

`add` に `: number`、`greet` に `: string` を付け、本体を変えずに戻り値の契約を明示します。

```vue
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [全ルール](all.md)

<span id="script-require-prop-type-constructor"></span>

### `script/require-prop-type-constructor`

prop の type に文字列ではなくコンストラクターを指定します。

[悪い例](#script-require-prop-type-constructor-bad) · [良い例](#script-require-prop-type-constructor-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**悪い**

props の実行時の型に文字列 `"String"` と `"Number"` を使い、コンストラクターの配列にも文字列を入れています。これらの文字列はコンストラクター関数ではありません。

```vue
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**良い**

型を実際の `String` と `Number` の識別子にし、共用型の配列も `[String, Number]` に変えます。

```vue
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [全ルール](all.md)

<span id="script-require-prop-types"></span>

### `script/require-prop-types`

各 prop の型を宣言します。

[悪い例](#script-require-prop-types-bad) · [良い例](#script-require-prop-types-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**悪い**

配列の要素は `status` という名前だけを宣言し、`null` の値と空の descriptor にも実行時の prop 型がありません。

```vue
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**良い**

`status: String` で省略形式のコンストラクターを指定し、`other` の descriptor に `type: Number` を付けます。両方の props が型の宣言を持つようになります。

```vue
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [全ルール](all.md)

<span id="script-require-symbol-provide"></span>

### `script/require-symbol-provide`

provide / inject のキーに衝突しにくい Symbol を使います。

[悪い例](#script-require-symbol-provide-bad) · [良い例](#script-require-symbol-provide-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**悪い**

`provide` と `inject` が `'user'`、`'theme'` のような文字列キーを使い、同じ綴りを使うほかの provider と衝突し得ます。

```vue
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**良い**

共有する `UserKey` を `Symbol` で作り、`InjectionKey<User>` の型を付けます。両呼び出しに同じキーを渡し、文字列リテラルを使わないようにします。

```vue
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [全ルール](all.md)

<span id="script-require-typed-object-prop"></span>

### `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

[悪い例](#script-require-typed-object-prop-bad) · [良い例](#script-require-typed-object-prop-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**悪い**

裸の `Object` と `Array` は広い実行時の分類しか表さず、`user` や `items` の要素の形を明示する静的な型がありません。

```vue
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**良い**

`PropType<User>` と `PropType<User[]>` を付け、実行時のコンストラクターを保ったままオブジェクトと要素の型を指定します。

```vue
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全ルール](all.md)

<span id="script-require-typed-ref"></span>

### `script/require-typed-ref`

空・null・undefined で初期化する ref() に型引数を指定します。

[悪い例](#script-require-typed-ref-bad) · [良い例](#script-require-typed-ref-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**悪い**

インポートした `ref` に型引数がなく、引数なし、`null`、`undefined` からは将来代入する値の型を推論できません。

```vue
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**良い**

型引数で string と nullable な User の ref を指定します。`ref(0)` には具体的な数値の初期値があり、型推論を使えます。

```vue
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [全ルール](all.md)

<span id="script-require-valid-default-prop"></span>

### `script/require-valid-default-prop`

prop の default を宣言した型に合う値にします。

[悪い例](#script-require-valid-default-prop-bad) · [良い例](#script-require-valid-default-prop-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**悪い**

Number と Boolean の props に型の合わないスカラー既定値を付け、Array と Object には factory ではなく共有されるリテラル値を使っています。

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**良い**

スカラーの既定値を `0` と `false` にし、配列とオブジェクトの既定値を新しい値を返す関数にします。`[String, Number]` の文字列既定値は、宣言した型の一つに合うため許可されます。

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [全ルール](all.md)

<span id="script-return-in-computed-property"></span>

### `script/return-in-computed-property`

computed の getter に値を返す return を用意します。

[悪い例](#script-return-in-computed-property-bad) · [良い例](#script-return-in-computed-property-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**悪い**

ブロック形式の computed getter が `1 + 2` を計算するだけで返さず、computed の値が undefined になります。

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**良い**

`return 1 + 2` で式を getter の戻り値にします。このルールは式文だけでなく、getter 自身の値を返す return を確認します。

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [全ルール](all.md)

<span id="script-return-in-emits-validator"></span>

### `script/return-in-emits-validator`

Options API の emits validator に戻り値を用意します。

[悪い例](#script-return-in-emits-validator-bad) · [良い例](#script-return-in-emits-validator-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の SFC フィルターが対象とする block-body arrow を使います。validator 本体には method shorthand の処理もありますが、現在の SFC prefilter はその形を確実には実行しません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**悪い**

`submit` の validator が payload をログ出力するだけで検査結果を返さず、ブロックの結果が undefined になります。

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**良い**

`return payload != null` を追加し、値を返さず終了する代わりに payload の真偽値の検査結果を返します。

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [全ルール](all.md)

<span id="script-valid-define-emits"></span>

### `script/valid-define-emits`

defineEmits の重複や型と実行時引数の併用を検出します。

[悪い例](#script-valid-define-emits-bad) · [良い例](#script-valid-define-emits-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**悪い**

同じ `defineEmits` に型引数と実行時配列 `["save"]` の両方を渡し、併用できない二つの宣言形式を混ぜています。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**良い**

実行時の引数を除き、`save` の宣言を型ベースの一つの形式に統一します。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [全ルール](all.md)

<span id="script-valid-define-options"></span>

### `script/valid-define-options`

defineOptions の引数と使用回数を検査します。

[悪い例](#script-valid-define-options-bad) · [良い例](#script-valid-define-options-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**悪い**

最初の呼び出しが専用マクロで宣言すべき `props` を `defineOptions` に入れています。後続の呼び出しはマクロを繰り返し、オブジェクトでない引数も渡しています。禁止された形式と呼び出し回数の制約を示す例です。

```vue
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**良い**

`defineOptions` を一度だけ呼び、通常の対応オプション `name` と `inheritAttrs` を持つオブジェクトを渡します。

```vue
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [全ルール](all.md)

<span id="script-valid-define-props"></span>

### `script/valid-define-props`

defineProps の重複や型と実行時引数の併用を検出します。

[悪い例](#script-valid-define-props-bad) · [良い例](#script-valid-define-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**悪い**

同じ `defineProps` に型引数 `{ title: string }` と実行時引数 `{ title: String }` の両方を渡しています。コンパイラーはこの併用を許可しません。

```vue
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**良い**

実行時オブジェクトを除き、二つの形式を併用せず `title` の型ベースの宣言だけを残します。

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [全ルール](all.md)

<span id="script-valid-next-tick"></span>

### `script/valid-next-tick`

nextTick() の完了を await、then、または callback で扱います。

[悪い例](#script-valid-next-tick-bad) · [良い例](#script-valid-next-tick-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**悪い**

インポートした `nextTick()` をコールバックなしの式文で呼び、返された Promise を無視しています。DOM 更新後まで待つ処理がありません。

```vue
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**良い**

`await nextTick()` で Promise を使い、後続の setup 処理へ進む前に次の DOM 更新を明示的に待ちます。

```vue
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [全ルール](all.md)

<span id="nuxt-no-nuxt-config-test-key"></span>

### `nuxt/no-nuxt-config-test-key`

Nuxt が自動判定する test 環境の手動設定を検出します。

[悪い例](#nuxt-no-nuxt-config-test-key-bad) · [良い例](#nuxt-no-nuxt-config-test-key-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**悪い**

エクスポートした Nuxt 設定で `test` キーに真偽値 `true` を指定しており、このルールが拒否する旧形式の設定です。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**良い**

空の設定にすることで、真偽値の `test` プロパティを取り除きます。テスト設定のオブジェクトまで禁止する例ではありません。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({});
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [全ルール](all.md)

<span id="nuxt-no-page-meta-runtime-values"></span>

### `nuxt/no-page-meta-runtime-values`

definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。

[悪い例](#nuxt-no-page-meta-runtime-values-bad) · [良い例](#nuxt-no-page-meta-runtime-values-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**悪い**

`definePageMeta` のオブジェクトを作る際に `useRoute()` を即座に評価しています。メタデータはマクロによって setup の実行時コンテキストの外へ巻き上げられます。

```vue
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**良い**

`validate` にコールバックを渡し、`useRoute().params.id` の評価をその実行時まで遅らせます。このルールは関数本体内の遅延評価と、メタデータの即時評価を区別します。

```vue
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [全ルール](all.md)

<span id="nuxt-nuxt-config-keys-order"></span>

### `nuxt/nuxt-config-keys-order`

Nuxt 設定のプロパティを推奨順に並べます。

[悪い例](#nuxt-nuxt-config-keys-order-bad) · [良い例](#nuxt-nuxt-config-keys-order-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**悪い**

設定内で `ssr` が `modules` より先に置かれ、このルールが推奨する Nuxt 設定キーの順序が逆になっています。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**良い**

`modules` を `ssr` より前に移し、各値を保ったまま指定の順序にそろえます。変更するのは配置であり、設定値の意味ではありません。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ modules: [], ssr: true });
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [全ルール](all.md)

<span id="nuxt-prefer-import-meta"></span>

### `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

[悪い例](#nuxt-prefer-import-meta-bad) · [良い例](#nuxt-prefer-import-meta-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**悪い**

`process.client` は旧形式の Nuxt 環境フラグであり、このルールは `import.meta` への移行を求めます。

```vue
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**良い**

`import.meta.client` に置き換え、ブラウザー側だけで実行する分岐を新しい環境フラグで表します。

```vue
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全ルール](all.md)


## プロジェクトの例

<span id="ecosystem-vue-router-unknown-route"></span>

### `ecosystem/vue-router-unknown-route`

登録済み router に存在しない名前です。

既定の重大度: error  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-unknown-route-bad"></span>

**悪い**

到達できる登録済み router の名前は `user-post` ですが、navigation で `user-posts` と誤記しています。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-unknown-route-good"></span>

**良い**

二つの path parameter を保ち、登録名の `user-post` に合わせます。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="ecosystem-vue-router-extra-param"></span>

### `ecosystem/vue-router-extra-param`

パスにない tab は Vue Router に破棄されます。

既定の重大度: error  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-extra-param-bad"></span>

**悪い**

`user-post` の path は `userId` と `postId` ですが、宣言にない `tab` も path parameter として渡しています。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-extra-param-good"></span>

**良い**

params から `tab` を除き、path の key だけを渡します。タブ選択が必要なら別途 query を使います。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="ecosystem-vue-router-param-type"></span>

### `ecosystem/vue-router-param-type`

postId は繰り返しパラメーターではないため配列を渡せません。

既定の重大度: error  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-param-type-bad"></span>

**悪い**

`postId` は一つの値を取る path parameter ですが、配列 `["2"]` を渡しています。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-param-type-good"></span>

**良い**

繰り返し指定ではない `postId` の segment に、一つの値 `"2"` を渡します。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="ecosystem-vue-router-missing-param"></span>

### `ecosystem/vue-router-missing-param`

必須の postId がありません。現在のルートに依存する遷移になります。

既定の重大度: warning  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

アプリの createApp(...).use(router) から登録済み router に到達できる構成が必要です。未確定または動的な定義では未知の名前と断定しません。省略パラメーターは現在のルートの値を継承する場合があるため warning です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-missing-param-bad"></span>

**悪い**

`user-post` の必須 `postId` を省略しています。Vue Router は現在の route から値を継承する場合があるため warning です。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-missing-param-good"></span>

**良い**

`userId` と `postId` を両方明示し、現在の route の parameter に依存しない navigation にします。

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="html-cross-component-nesting"></span>

### `html/cross-component-nesting`

import した子コンポーネントの要素を合成して HTML の入れ子を検査します。

既定の重大度: warning  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "html/cross-component-nesting": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="html-cross-component-nesting-bad"></span>

**悪い**

親の `<p>` 内に、ルートが `<div>` の子を合成し、paragraph と block の不正な入れ子を作ります。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

<span id="html-cross-component-nesting-good"></span>

**良い**

子の block 要素を含められる `<section>` を使い、子は変更しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="vue-cross-file-attrs-fallthrough"></span>

### `vue/cross-file-attrs-fallthrough`

属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。

既定の重大度: warning  
適用範囲: 到達可能なプロジェクトの宣言と import したコンポーネント  
オプション: crossFile と重大度（off/warn/error）  
自動修正: なし

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

**悪い**

親が `class="notice"` を fragment の子に渡しますが、属性の自動適用先がなく、子も `$attrs` を使いません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vue-cross-file-attrs-fallthrough-good"></span>

**良い**

子が `<main>` に `$attrs` を binding して適用先を選び、隣の `<aside>` は別に保ちます。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-array-mutation"></span>

### `vize:croquis/cf/array-mutation`

配列を添字で変更していますが、reactive な配列はそれを追跡しません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

旧 Vue 2.7 に限る例です。対応する Vue 2.7 と SFC コンパイラーを使うことを前提とします。Vue 3 の Proxy は配列のインデックス代入を追跡するため、`items[0] = next` は Vue 3 ではリアクティブであり、不具合ではありません。この公開コードには現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

<span id="vize-croquis-cf-array-mutation-bad"></span>

**悪い**

この旧 Vue 2.7 プロジェクトでは、`items[0] = next` が Vue 2 の配列 observer に通知せず配列を変えるため、表示中の最初の要素が更新されない場合があります。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

<span id="vize-croquis-cf-array-mutation-good"></span>

**良い**

`splice(0, 1, next)` で Vue 2 が監視する配列の変更メソッドを使い、同じ置換を画面に反映します。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-async-boundary"></span>

### `vize:croquis/cf/async-boundary`

reactive な状態が async 境界を越え、古い値が見えることがあります。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/async-boundary": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-async-boundary-bad"></span>

**悪い**

watcher に無効化時の cleanup がなく、古い query の遅い結果が新しい結果を上書きできます。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

<span id="vize-croquis-cf-async-boundary-good"></span>

**良い**

await より前に cleanup を登録し、旧 request を abort して `active` を無効化します。有効な応答だけを代入します。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-async-no-suspense"></span>

### `vize:croquis/cf/async-no-suspense`

async コンポーネントが Suspense 境界なしで描画されています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

Current support: `no-source-async-fact`

生成元は macros.is_async() を読みますが、現在のソース解析は top-level await を script-setup の scope に記録します。そのため以下の完全な悪い例・良い例では、現在の CLI は async-no-suspense を検出しません。Suspense の使い方を説明する例で、必要な macro 情報を渡す処理は今後の実装課題です。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

**悪い**

子に top-level await があり、親に `<Suspense>` がありません。現在のソース解析には、このコードの生成に必要な macro 情報が渡されません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

<span id="vize-croquis-cf-async-no-suspense-good"></span>

**良い**

同じ async な子を `<Suspense>` と loading fallback で包みます。規約の例であり、現在の検査は両方のソースでこのコードを生成しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-browser-api-ssr"></span>

### `vize:croquis/cf/browser-api-ssr`

サーバーでも描画され得る場所で、ブラウザ専用 API を使っています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/browser-api-ssr": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

**悪い**

setup 中に `window.innerWidth` を読み、ブラウザの `window` がない SSR 環境でも実行されます。

`App.vue`

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

**良い**

server で安全な値の ref を作り、client mount 後の `onMounted` 内で `window` を読みます。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-circular-dep"></span>

### `vize:croquis/cf/circular-dep`

コンポーネントの import が循環しています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

即時初期化による具体的な循環の例です。再帰する Vue コンポーネントや、あらゆる循環 import が必ず誤りになるわけではありません。現在この契約コードを生成する検査はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

<span id="vize-croquis-cf-circular-dep-bad"></span>

**悪い**

`a.ts` が `b.ts` を読み、`b.ts` が `a.ts` を読み返します。両方が相手の未初期化の定数から即座に定数を作るため、初期化前のアクセスが発生します。

`a.ts`

```ts
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

<span id="vize-croquis-cf-circular-dep-good"></span>

**良い**

両モジュールが独立した `labels.ts` の初期化済みの接頭辞を読み、循環と初期化前の相互参照を取り除きます。

`a.ts`

```ts
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-circular-reactive-dependency"></span>

### `vize:croquis/cf/circular-reactive-dependency`

reactive な計算が互いに循環して依存しています。

既定の重大度: context-dependent  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Example qualification: `illustrative-source-pair`

以下の完全な Vue プロジェクトは、更新の循環とその修正を具体的に示します。このソースによる CLI の検出は検証済みではありません。診断の生成元には、併記したグラフのように保持された参照の ID と流れが必要です。このソースだけで現在の処理がこの診断コードを生成すると断定しません。参照 ID を保持したグラフに対する専用の検出検証と、ソースの構文検査は分けて扱います。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

**悪い**

App が count（A）を保持して provide します。CycleView は nextCount（B）を computed で求め、watch でその値を同じ inject 済みの count に書き戻します。immediate の実行後も計算の入力が更新され続けるため、A → B → A の更新の循環になります。下の参照グラフはこの二つの参照を表し、同名の無関係な変数をつなぐものではありません。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

**良い**

B の値を A に書き戻す watch を取り除きます。count は App が保持し、明示的な Increment 操作でだけ変更します。CycleView は nextCount を読み取るだけで計算結果を書き戻しません。同じ二つの参照には A → B の依存だけが残ります。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-closure-captures-reactive"></span>

### `vize:croquis/cf/closure-captures-reactive`

クロージャが reactive な値を捕捉しており、その後の更新が見えません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

**悪い**

`makeReader` がクロージャーを作る前に `count.value` をコピーします。computed の読取処理はリアクティブな依存を読まず、最初の数値を返し続けます。

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

**良い**

呼び出すたびにクロージャー内で `count.value` を読み、computed が ref を追跡して増加後の `shown` を更新できるようにします。

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-composable-outside-setup"></span>

### `vize:croquis/cf/composable-outside-setup`

composable が `setup` の外で呼ばれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

対象はライフサイクルに依存するこの composable です。通常のユーティリティ関数や、setup 外のあらゆる Composition API 呼び出しを禁止する例ではありません。この契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useTitle } from './use-title';
const title = useTitle();
</script>

<template>
<h1>{{ title }}</h1>
</template>

```

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

**悪い**

`use-title.ts` の import 時点で、コンポーネントの setup が有効になる前に `onMounted` を登録します。後から公開関数を呼んでもモジュール直下の ref を返すだけで、ライフサイクルの所属は修復されません。

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

**良い**

状態作成とフック登録を `useTitle` の中へ移し、App の setup 内から同期的に呼びます。mounted フックがその App インスタンスに所属するようになります。

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-computed-side-effects"></span>

### `vize:croquis/cf/computed-side-effects`

computed の getter が状態を書いたり、他の副作用を起こしています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled, lastCalculated } = useDouble();
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>
</template>

```

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

**悪い**

`doubled` の評価が `lastCalculated` を書き換え、computed の値を読む操作が別の状態を変更しています。副作用が遅延評価の getter を読むタイミングに依存します。

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => {
    const next = count.value * 2;
    lastCalculated.value = next;
    return next;
  });
  return { count, doubled, lastCalculated };
}

```

<span id="vize-croquis-cf-computed-side-effects-good"></span>

**良い**

getter は導出した数値だけを返します。`count` の変更時に `lastCalculated` を書く処理は、初期値も含めて別の watcher に任せます。

`use-double.ts`

```ts
import { computed, ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => count.value * 2);
  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });
  return { count, doubled, lastCalculated };
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-deep-import"></span>

### `vize:croquis/cf/deep-import`

import の連鎖がプロジェクトの許す深さより深くなっています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

明示的に選んだプロジェクトの配置方針を示す例です。対応する深さのしきい値やオプションがあると主張するものではありません。この契約には現在の診断生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

<span id="vize-croquis-cf-deep-import-bad"></span>

**悪い**

entry が単純な値を `level-one`、`level-two`、`level-three` 経由で読み、浅い公開境界を望むプロジェクトに不要な深い import の列を作っています。

`entry.ts`

```ts
export { label } from './level-one';

```

<span id="vize-croquis-cf-deep-import-good"></span>

**良い**

値を直接再公開する `public-api.ts` を entry から使います。使用側の import 名を保ったまま、参照の列を短くします。

`entry.ts`

```ts
export { label } from './public-api';

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>

### `vize:croquis/cf/destructuring-breaks-reactivity`

reactive オブジェクトの分割代入はフィールドをコピーし、追跡を失います。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

**悪い**

`props` object の通常の分割代入は現在の `item` を取り出します。Vue 3.5 の `defineProps()` から直接行う分割代入とは別です。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

**良い**

`toRef(props, "item")` で `props` の property への接続を維持します。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-di-outside-setup"></span>

### `vize:croquis/cf/di-outside-setup`

`provide` または `inject` が `setup` の外で呼ばれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

コンポーネントの provide/inject を使う例です。`app.provide` や対応する `app.runWithContext` 内の inject は別の有効な所属先であり、この例で禁止するものではありません。現在この契約コードの生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`theme.ts`

```ts
import { inject, provide } from 'vue';
import type { InjectionKey } from 'vue';
export const ThemeKey: InjectionKey<string> = Symbol('theme');
export function provideTheme() { provide(ThemeKey, 'dark'); }
export function useTheme() { return inject(ThemeKey, 'light'); }

```

`ThemedText.vue`

```vue
<script setup lang="ts">
import { useTheme } from './theme';
const theme = useTheme();
</script>

<template>
<p>{{ theme }}</p>
</template>

```

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

**悪い**

`main.ts` が有効なコンポーネントインスタンスなしで component の `provide` を呼びます。子の `inject` は意図した祖先の値を受け取れず、`light` を使います。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { provideTheme } from './theme';
provideTheme();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
</script>

<template>
<ThemedText />
</template>

```

<span id="vize-croquis-cf-di-outside-setup-good"></span>

**良い**

App の setup で子を描画する前に provider を呼び、子が祖先コンポーネントの `dark` を受け取れるようにします。

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
import { provideTheme } from './theme';
provideTheme();
</script>

<template>
<ThemedText />
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-dom-access-without-next-tick"></span>

### `vize:croquis/cf/dom-access-without-next-tick`

Vue が更新を反映する前に DOM を読んでいます。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`read-label.ts`

```ts
export function readLabel(node: HTMLElement | null): string {
  return node?.textContent ?? '';
}

```

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

**悪い**

クリック処理が `count` を増やしてすぐ段落の DOM を読み、Vue による更新の反映を待っていません。`sampled` に前の count が入る可能性があります。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
function increment() {
  count.value++;
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

**良い**

状態を書いた後に `nextTick()` を待ち、Vue が段落を更新してから `readLabel` でテキストを読みます。

`App.vue`

```vue
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
async function increment() {
  count.value++;
  await nextTick();
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-duplicate-id"></span>

### `vize:croquis/cf/duplicate-id`

同じ要素 id が複数のコンポーネントで使われています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/duplicate-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./CheckoutForm.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-duplicate-id-bad"></span>

**悪い**

描画する shipping と billing の両方が `id="postal-code"` を使い、label の文書内の参照先が重複します。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

`BillingAddress.vue`

```vue
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

<span id="vize-croquis-cf-duplicate-id-good"></span>

**良い**

各コンポーネントの `useId()` を label と input の両方に binding し、固定 ID の重複をなくします。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

`BillingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-event-listener-leak"></span>

### `vize:croquis/cf/event-listener-leak`

イベントリスナーが登録されたまま、解除されていません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

**悪い**

マウント時に width の ref を参照する resize リスナーを window に追加しますが、アンマウント時に削除しません。再マウントを繰り返すと不要なリスナーや状態を保持し得ます。

`use-width.ts`

```ts
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

<span id="vize-croquis-cf-event-listener-leak-good"></span>

**良い**

`onUnmounted` でマウント時に登録した同じ `resize` 関数を削除し、そのインスタンスの外部リスナーの寿命を終えます。

`use-width.ts`

```ts
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-event-modifier"></span>

### `vize:croquis/cf/event-modifier`

emit が対応しない修飾子を、イベントリスナーが使っています。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-event-modifier-bad"></span>

**悪い**

`.stop` を子のカスタム `save` イベントに付けていますが、その payload が DOM Event とは限りません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save.stop="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-event-modifier-good"></span>

**良い**

カスタム イベントの listener から `.stop` を除きます。必要なら実際の DOM listener で伝播を制御します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-hydration-risk"></span>

### `vize:croquis/cf/hydration-risk`

このコードはprop を ref にコピーする操作など、複数のリアクティビティ検出をまとめています。ファイル間検査がすべての Date.now() 式を検出するという意味ではありません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-hydration-risk-bad"></span>

**悪い**

子の `ref(props.count)` は初期値を一度コピーし、後の親の prop 変更には追従しません。現在の prop-to-ref 生成元の例で、非決定的な SSR 全般の例ではありません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ count: number }>();
const count = ref(props.count);
</script>
<template><p>{{ count }}</p></template>
```

<span id="vize-croquis-cf-hydration-risk-good"></span>

**良い**

初期値を独立した state にコピーせず、`toRef(props, "count")` で prop を参照します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";
const props = defineProps<{ count: number }>();
const count = toRef(props, "count");
</script>
<template><p>{{ count }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-inherit-attrs-unused"></span>

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` が設定されているのに、属性が読まれていません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

**悪い**

子が `inheritAttrs: false` にして、親の `class="notice"` をどこにも渡していません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main>Content</main></template>
```

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

**良い**

明示的な継承制御を保ち、適用先の `<main>` に `$attrs` を binding します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main v-bind="$attrs">Content</main></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-inject-without-symbol"></span>

### `vize:croquis/cf/inject-without-symbol`

`inject` が `InjectionKey` のシンボルではなく素のキーを使っています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/inject-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

**悪い**

使用側が型付き symbol ではなく文字列 key `"theme"` を inject しています。

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

**良い**

使用側と提供側で同じ `ThemeKey` を import し、文字列名の重複に頼らない接続にします。

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-injected-async-mutation-race"></span>

### `vize:croquis/cf/injected-async-mutation-race`

inject した値が、競合し得る async タスクから変更されています。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/injected-async-mutation-race": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

**悪い**

`CountLoader.vue` が await の結果を、`CountSummary.vue` と共有する inject 済み store に直接書き込み、古い処理が両方に作用できます。

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

**良い**

loader は無効化した処理を中止し、有効な結果だけ emit します。store の変更は提供側の `applyLoadedCount` が担当します。

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-lifecycle-outside-setup"></span>

### `vize:croquis/cf/lifecycle-outside-setup`

ライフサイクルフックが `setup` の外で登録されています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`install-title.ts`

```ts
import { onMounted } from 'vue';
export function installTitle() {
  onMounted(() => { document.title = 'Mounted application'; });
}

```

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

**悪い**

entry がアプリのマウント前に `installTitle()` を呼び、有効なコンポーネントの setup なしで `onMounted` を登録しています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { installTitle } from './install-title';
installTitle();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">

</script>

<template>
<p>Application</p>
</template>

```

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

**良い**

同じヘルパーを App の setup から同期的に呼び、ライフサイクルのコールバックをそのインスタンスのマウントに結び付けます。

`App.vue`

```vue
<script setup lang="ts">
import { installTitle } from './install-title';
installTitle();
</script>

<template>
<p>Application</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-lifecycle-without-cleanup"></span>

### `vize:croquis/cf/lifecycle-without-cleanup`

ライフサイクルフックが後片付けなしで処理を始めています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

**悪い**

mount 時に window の resize listener を登録しますが、unmount 時に同じ callback を解除しません。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

**良い**

`onUnmounted` で、登録時と同じイベント名・関数の参照を使って listener を解除します。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
onUnmounted(() => { window.removeEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-missing-required-prop"></span>

### `vize:croquis/cf/missing-required-prop`

必須 prop が渡されていません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

**悪い**

親が `<Child />` を描画し、子の必須 `title: string` prop を渡していません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-missing-required-prop-good"></span>

**良い**

`title="Hello"` で、解決した子が宣言する必須 prop を渡します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-missing-suspense"></span>

### `vize:croquis/cf/missing-suspense`

async な依存が Suspense 境界の外で使われています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-missing-suspense-bad"></span>

**悪い**

`AsyncCard` の top-level await が setup を非同期にしますが、App はその依存を調整する Suspense の境界なしで描画しています。

`App.vue`

```vue
<script setup lang="ts">
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<AsyncCard />
</template>

```

<span id="vize-croquis-cf-missing-suspense-good"></span>

**良い**

App が非同期の子を `Suspense` で包み、子の setup が完了するまでローディングの fallback を表示します。

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-module-scope-reactive"></span>

### `vize:croquis/cf/module-scope-reactive`

reactive な状態がモジュールスコープで作られ、すべての呼び出し元で共有されます。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

アプリケーション全体の store を意図する場合、モジュール直下のリアクティブ状態は有効です。この例はコンポーネントやリクエスト間の分離を前提とし、この公開契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

**悪い**

モジュールが `count` を一度だけ初期化し、二つの Counter が同じ ref を受け取ります。この例では独立した状態を意図していますが、一方をクリックすると両方が変わります。

`counter.ts`

```ts
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

**良い**

`createCounter` の中で ref を作り、setup からの同期的な呼び出しごとに別の状態を渡します。それぞれのボタンが個別のカウンターを持つようになります。

`counter.ts`

```ts
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-multi-root-attrs"></span>

### `vize:croquis/cf/multi-root-attrs`

複数ルートのコンポーネントが属性を受け取りますが、付ける場所がありません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

**悪い**

子に `<main>` と `<aside>` の二つのルートがあり、親の class を自動で受け取る一つのルートがありません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

**良い**

二つ目のルートを保ったまま、`<main>` に `$attrs` を明示的に渡します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-mutated-after-escape"></span>

### `vize:croquis/cf/mutated-after-escape`

reactive オブジェクトが、所有者から漏れた後に変更されています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

変更しない履歴という明示的な所有方針を示す例です。リアクティブなオブジェクトを渡したり、後で変更したりすることを一般に禁止するものではありません。現在この契約の生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`archive.ts`

```ts
export interface Profile { name: string }
const records: Readonly<Profile>[] = [];
export function publish(profile: Readonly<Profile>): void { records.push(profile); }
export function latestName(): string { return records.at(-1)?.name ?? ''; }

```

`App.vue`

```vue
<script setup lang="ts">
import { publishProfile } from './profile';
import { latestName } from './archive';
publishProfile();
const archivedName = latestName();
</script>

<template>
<p>Archived name: {{ archivedName }}</p>
</template>

```

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

**悪い**

archive は `publish` に渡した同じオブジェクトを保持します。その後で所有側が名前を変更し、過去の記録まで Grace に変わります。TypeScript の Readonly 引数はオブジェクトをコピーしません。

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish(profile);
  profile.name = 'Grace';
}

```

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

**良い**

通常のコピーを公開し、保存した Ada の記録と後のリアクティブな profile の変更を分離します。archive のスナップショット方針を保てるようになります。

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish({ ...profile });
  profile.name = 'Grace';
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-non-reactive-provide"></span>

### `vize:croquis/cf/non-reactive-provide`

provide した値が reactive ではないため、子孫は更新を観測できません。

既定の重大度: context-dependent  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-reactive-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

**悪い**

`ThemeProvider.vue` が通常の object を provide しており、field を変更しても inject 側の Vue の reactive な依存になりません。

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

**良い**

提供する theme を `ref` で包み、inject する同じ参照で後の変更を追跡します。

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-non-unique-id"></span>

### `vize:croquis/cf/non-unique-id`

ループ内の要素 id が項目ごとに一意ではありません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-unique-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ResultsList.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-unique-id-bad"></span>

**悪い**

`v-for` の各反復で固定の `result-title` ID が繰り返されます。key があっても DOM の ID は一意になりません。

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

<span id="vize-croquis-cf-non-unique-id-good"></span>

**良い**

見出しの ID に result の識別子を含め、各項目の文書 ID を区別します。

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-object-identity-comparison"></span>

### `vize:croquis/cf/object-identity-comparison`

reactive オブジェクトを同一性で比較していますが、unwrap のたびに同一性は変わります。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

ID がレコードを一意に識別することを前提とします。同じリアクティブ Proxy の参照同士を比較することは有効であり、この契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

**悪い**

`proxy === raw` はラッパーの同一性を比較するため、同じユーザーを表していても false になります。アプリが意図するレコードの同一性と、オブジェクトの同一性が異なっています。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

**良い**

安定したレコードの `id` を比較し、raw か Proxy かに依存せず意図した同一性を判定します。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-pinia-getter"></span>

### `vize:croquis/cf/pinia-getter`

Pinia の getter を `storeToRefs` なしで読んでおり、reactive のままになりません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

Pinia のインストールを前提とし、main.ts がマウント前に plugin を登録します。追跡される計算やテンプレートの中で `store.doubled` を直接読むことは有効です。ここでの問題は通常の値としてスナップショットを取ることであり、現在この契約の生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

<span id="vize-croquis-cf-pinia-getter-bad"></span>

**悪い**

`const doubled = store.doubled` が setup 時点の getter の数値をコピーし、後の `store.count` 更新に追従しなくなります。

`App.vue`

```vue
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-pinia-getter-good"></span>

**良い**

`storeToRefs(store)` からリアクティブな getter の ref を受け取り、分割代入とテンプレートの unwrap 後も store とのつながりを保ちます。

`App.vue`

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-prop-type-mismatch"></span>

### `vize:croquis/cf/prop-type-mismatch`

渡した prop の値が、宣言された型と一致しません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

**悪い**

親が数値式 `42` を、解決した子の `title: string` prop に渡しています。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :title="42" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

**良い**

文字列の `title="Hello"` を渡し、子の宣言と合わせます。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-provide-inject-type"></span>

### `vize:croquis/cf/provide-inject-type`

provide した値と inject の型が一致しません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-inject-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

この検査は提供元と使用側の明示的な型注釈を比較し、リテラルからの型推論は使いません。この例の提供元の as string 注釈を残してください。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

**悪い**

提供側の `title` は明示的な `string` ですが、子孫が同じ key を `inject<number>` で要求します。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-provide-inject-type-good"></span>

**良い**

使用側を `inject<string>` に合わせます。この生成元は推論されたリテラル型ではなく明示的な注釈を比較するため、`as string` を残します。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-provide-without-symbol"></span>

### `vize:croquis/cf/provide-without-symbol`

`provide` が `InjectionKey` のシンボルではなく素のキーを使っています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

**悪い**

両コンポーネントが文字列 `"theme"` を使い、無関係な機能と key が衝突し得ます。

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

**良い**

型付きの `ThemeKey` symbol を一つ export し、provide と inject の両方で同じ値を import します。同じ説明の symbol を別々に作っても接続できません。

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-reactive-export"></span>

### `vize:croquis/cf/reactive-export`

reactive な状態がモジュールから export されています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

共有するアプリケーションの store を意図する場合、リアクティブな状態の export は有効です。この例は状態の分離を必要とするもので、あらゆる export が不正という意味ではありません。現在この契約の生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

<span id="vize-croquis-cf-reactive-export-bad"></span>

**悪い**

モジュールが初期化済みのリアクティブなオブジェクトを一つだけ公開し、すべての import 元が同じ count を受け取ります。リクエスト間でモジュールを共有する SSR では、この例のインスタンスやリクエストごとの分離を保てません。

`state.ts`

```ts
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

<span id="vize-croquis-cf-reactive-export-good"></span>

**良い**

モジュールから factory を公開し、App が setup 内で呼びます。公開された singleton ではなく、インスタンスごとに新しいリアクティブな count を得ます。

`state.ts`

```ts
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-reactivity-outside-setup"></span>

### `vize:croquis/cf/reactivity-outside-setup`

reactive API が `setup` の外で呼ばれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

Vue では component setup の外でも ref・reactive・computed を使えます。ここでのリスクは API の禁止ではなく、インスタンスを分離する明示的な方針に反した共有や所属です。この契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

**悪い**

二つのリアクティブ API がモジュールの読み込み時に実行されます。独立したカウンターを意図していても、二つの Counter が一つの ref と computed を共有します。

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

**良い**

各コンポーネントの setup 呼び出しの中で、`useCounter` が ref と computed を同期的に作り、各 widget に個別の状態と追跡される導出を与えます。

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>

### `vize:croquis/cf/reassignment-breaks-reactivity`

reactive な束縛を再代入すると、ただの値に置き換わります。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

**悪い**

子が作った prop の ref を `props.user` で上書きし、ref としての接続を失います。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

**良い**

`toRef` を `const` に保持し、参照を置き換える再代入を除きます。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-reference-escapes-scope"></span>

### `vize:croquis/cf/reference-escapes-scope`

reactive な参照が、寿命を持つスコープの外へ漏れています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

ref を composable から返したり、スコープをまたいで共有したりすることは有効です。この例では cache にスナップショットを求めており、アンマウントで ref 自体が無効になると主張するものではありません。現在この契約の生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

**悪い**

プロセス全体の cache がコンポーネントの count ref をそのまま保持し、アンマウント後もインスタンスの状態を参照できるようにします。スナップショットを保存する意図にもかかわらず、後の変更も観測できます。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

**良い**

cache に現在の通常の数値を渡し、コンポーネントが持つ ref を保持せずスナップショットを保存します。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-setup-context-violation"></span>

### `vize:croquis/cf/setup-context-violation`

Vue が許さない使い方で setup コンテキストを使っています。

既定の重大度: context-dependent  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

**悪い**

`ref(0)` を通常の script の module scope に作り、この analyzer が扱うインスタンスごとの setup 文脈から外しています。

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

<span id="vize-croquis-cf-setup-context-violation-good"></span>

**良い**

script setup に移し、各インスタンスが count を持ち、テンプレートから読み取れる形にします。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-shallow-deep-access"></span>

### `vize:croquis/cf/shallow-deep-access`

`shallowReactive` または `shallowRef` の深いプロパティを、追跡されるものとして読んでいます。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.user.name }}</p><button @click="profile.user.name = 'Grace'">Rename</button>
</template>

```

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

**悪い**

`shallowReactive` が追跡するのはルートの `user` プロパティであり、内側のオブジェクトは raw のままです。`profile.user.name` の変更は、追跡された深い変更としてテンプレートへ通知されません。

`profile.ts`

```ts
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

**良い**

深い `reactive` で内側の user オブジェクトも包み、同じ名前の代入を表示の更新につなげます。

`profile.ts`

```ts
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-spread-breaks-reactivity"></span>

### `vize:croquis/cf/spread-breaks-reactivity`

reactive オブジェクトのスプレッドは値をコピーし、追跡を失います。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/spread-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

**悪い**

`UserSummary.vue` が `props.user` を新しい object に展開し、渡された reactive な値をその時点でコピーします。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

**良い**

`toRef(props, "user")` で field をコピーせず、受け取る prop の参照を維持します。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-suspense-no-fallback"></span>

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` に fallback がありません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

fallback のない Suspense は有効な Vue の構文です。ローディング UI を置くという方針の例であり、コンパイルエラーではありません。この公開契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

**悪い**

Suspense の境界に非同期の子がありますが fallback がなく、この例の待機中の状態に表示する内容がありません。

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

**良い**

`#fallback` slot にローディングの段落を指定し、非同期の子が完了するまで表示します。

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-template-ref-timing"></span>

### `vize:croquis/cf/template-ref-timing`

テンプレート ref が、コンポーネントのマウント前に読まれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`focus-input.ts`

```ts
export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }

```

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

**悪い**

マウント前の setup でテンプレート ref を読み、値が null のままです。任意の focus 呼び出しは何もフォーカスしません。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
focusInput(input.value);
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

<span id="vize-croquis-cf-template-ref-timing-good"></span>

**良い**

`onMounted` で Vue が input 要素をテンプレート ref に設定するまで参照を遅らせ、ヘルパーが実際の要素をフォーカスできるようにします。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
onMounted(() => { focusInput(input.value); });
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-toraw-mutation"></span>

### `vize:croquis/cf/toraw-mutation`

`toRaw` した生のオブジェクトを変更しています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile, rename } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.name }}</p><button @click="rename(profile)">Rename</button>
</template>

```

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

**悪い**

`rename` が raw の対象を取り出して `raw.name` に代入し、表示中の名前を通知する Proxy の setter を通していません。

`profile.ts`

```ts
import { reactive, toRaw } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  const raw = toRaw(profile);
  raw.name = 'Grace';
}

```

<span id="vize-croquis-cf-toraw-mutation-good"></span>

**良い**

渡されたリアクティブ Proxy の `profile.name` を書き、同じ名前の変更を依存先に通知します。

`profile.ts`

```ts
import { reactive } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  profile.name = 'Grace';
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-uncaught-error"></span>

### `vize:croquis/cf/uncaught-error`

コンポーネントが例外を投げても、受け止めるエラー境界がありません。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

現在の生成元は JSON.parse(input) などのテンプレート式を検査します。script ブロックだけにある throw 文は対象外です。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-uncaught-error-bad"></span>

**悪い**

子のテンプレートが不正な input を `JSON.parse` し、到達する親にエラーを受け止める境界がありません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

<span id="vize-croquis-cf-uncaught-error-good"></span>

**良い**

親で子のエラーを受ける `onErrorCaptured` を登録します。`false` は伝播を止めます。実際の画面では復旧用の UI も用意します。

`App.vue`

```vue
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-undeclared-emit"></span>

### `vize:croquis/cf/undeclared-emit`

宣言されていないイベントを emit しています。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

**悪い**

子が `emit("save")` を呼びますが、`defineEmits` の宣言は `cancel` だけです。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-undeclared-emit-good"></span>

**良い**

引数なしの `save` を宣言し、emit するイベントをコンポーネントの契約に合わせます。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-undeclared-prop"></span>

### `vize:croquis/cf/undeclared-prop`

親が、子が宣言していない prop を渡しています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

**悪い**

親が `typo` を渡しますが、解決した子の宣言は `title` だけです。この analyzer の規約は、Vue の属性自動継承の一般的な動作とは別です。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-undeclared-prop-good"></span>

**良い**

意図しない `typo` binding を除き、宣言済みの `title` を残します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-undefined-slot"></span>

### `vize:croquis/cf/undefined-slot`

親が、子が公開していないスロットを埋めています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

<span id="vize-croquis-cf-undefined-slot-bad"></span>

**悪い**

App が `footer` slot を渡しますが、Card が宣言して描画するのは `header` だけです。渡した Notice に対応する子の slot 出口がありません。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

<span id="vize-croquis-cf-undefined-slot-good"></span>

**良い**

子の型付き slot 宣言と描画する出口に合わせて `header` を渡し、Notice をそこへ表示できるようにします。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unhandled-event"></span>

### `vize:croquis/cf/unhandled-event`

子が emit したイベントを、どの親も処理していません。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unhandled-event-bad"></span>

**悪い**

`Child.vue` が `save` を emit しますが、直接の wrapper が受け取りません。コンポーネントのイベントは wrapper を自動的に通り抜けません。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unhandled-event-good"></span>

**良い**

`Wrapper.vue` が直接の子に `save` listener を付けます。空の callback はこの検査での受信を示すもので、保存処理全体ではありません。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unmatched-inject"></span>

### `vize:croquis/cf/unmatched-inject`

`inject` が、どの祖先も provide していないキーを指定しています。

既定の重大度: error / warning (with default)  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unmatched-inject": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

**悪い**

`ThemeLabel.vue` が `ThemeKey` を inject しますが、到達する祖先 `App.vue` に同じ key の provide がありません。

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-unmatched-inject-good"></span>

**良い**

`App.vue` で同じ export 済み `ThemeKey` を使って reactive な theme を provide し、使用する子孫を描画します。

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unmatched-listener"></span>

### `vize:croquis/cf/unmatched-listener`

親が、子が emit しないイベントを購読しています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

**悪い**

親は `save` を待ちますが、解決した子の宣言は `cancel` だけです。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unmatched-listener-good"></span>

**良い**

子が `save` を宣言して emit し、親の listener 名と合わせます。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unregistered-component"></span>

### `vize:croquis/cf/unregistered-component`

テンプレートが、登録も import もされていないコンポーネントを使っています。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unregistered-component-bad"></span>

**悪い**

`Child.vue` は存在しますが、親がテンプレートの `Child` を import / 登録していません。

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unregistered-component-good"></span>

**良い**

親の script setup で `Child` を import し、テンプレートから binding を解決できる形にします。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unresolved-import"></span>

### `vize:croquis/cf/unresolved-import`

import がモジュールへ解決できません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unresolved-import-bad"></span>

**悪い**

親の import は `./Missing.vue` ですが、プロジェクトにあるファイルは `Child.vue` です。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Missing.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unresolved-import-good"></span>

**良い**

テンプレートの binding を変えず、存在する `./Child.vue` を import します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unused-attrs"></span>

### `vize:croquis/cf/unused-attrs`

複数ルートのコンポーネントに渡した fallthrough 属性が使われていません。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-attrs-bad"></span>

**悪い**

親の `tracking-code` は prop として使われず、複数ルートの子から転送もされません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-unused-attrs-good"></span>

**良い**

`<main>` に `$attrs` を binding し、継承属性の適用先を明示します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unused-emit"></span>

### `vize:croquis/cf/unused-emit`

宣言した emit が一度も使われていません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-emit-bad"></span>

**悪い**

子が `save` を宣言していますが、その名前で emit 関数を呼んでいません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unused-emit-good"></span>

**良い**

例では `emit("save")` を呼び、宣言したイベントを使います。実際の操作では対応する action の発生時に通知します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-unused-provide"></span>

### `vize:croquis/cf/unused-provide`

provide したキーが一度も inject されていません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unused-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-unused-provide-bad"></span>

**悪い**

`App.vue` に `ThemeKey` の provide がありますが、描画する `Dashboard.vue` の下に使用側がありません。

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue
<template>
  <h1>Dashboard</h1>
</template>
```

<span id="vize-croquis-cf-unused-provide-good"></span>

**良い**

dashboard が `ThemeLabel.vue` を描画し、祖先と同一の `ThemeKey` を inject します。

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>

### `vize:croquis/cf/value-extraction-breaks-reactivity`

reactive な値をローカルへ取り出すと、その後の更新が失われます。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/value-extraction-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

**悪い**

Vue 3.5 の reactive な props 分割代入で得た `item` を、`itemSnapshot` に一度だけ読み取っています。後の prop の置き換えは snapshot に反映されません。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

**良い**

`computed` 内で `item` を読み、reactive props destructuring の変換で評価ごとに追跡できる形にします。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-watch-can-be-computed"></span>

### `vize:croquis/cf/watch-can-be-computed`

ウォッチャが値を状態へ写しているだけで、computed にできます。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

純粋な派生状態についての公開された推奨を示す例です。外部への副作用や独立して書き込む状態には watcher が適しており、現在この契約の生成元はありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled } = useDouble();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

**悪い**

watcher は外部への副作用を行わず、二つ目の書き込み可能な ref を `count` の二倍へ同期するだけです。この例に派生値への独立した書き込みはありません。

`use-double.ts`

```ts
import { ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = ref(0);
  watch(count, next => { doubled.value = next * 2; }, { immediate: true });
  return { count, doubled };
}

```

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

**良い**

computed の getter で同じ導出を直接表し、手作業の同期と追加の書き込み可能な状態を取り除きます。

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-watcheffect-async"></span>

### `vize:croquis/cf/watcheffect-async`

`watchEffect` が async タスクを始め、前回の実行を片付けられません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/watcheffect-async": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

**悪い**

async `watchEffect` に暗黙の依存収集と await する request が混在し、無効化後の応答を防ぐ処理がありません。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

<span id="vize-croquis-cf-watcheffect-async-good"></span>

**良い**

`watch(() => props.query, ...)` で source を明示し、request の cleanup と無効化後の応答を除く条件を付けます。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[ファイル間ルール一覧](cross-file.md)

<span id="vize-croquis-cf-watcher-outside-setup"></span>

### `vize:croquis/cf/watcher-outside-setup`

`watch` または `watchEffect` が `setup` の外で呼ばれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

所有側が stop 関数を保持して呼んだり、アプリケーション全体の寿命を意図したりする場合、モジュール直下の watcher は有効です。この例はコンポーネントが寿命を持つことを前提とし、この契約には現在の生成元がありません。

**共通のプロジェクト ファイル**

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

**悪い**

watcher をいずれの Observer の setup 内でもなくモジュールの読み込み時に作り、両インスタンスが ref を共有します。個々の Observer のアンマウントでは自動停止されません。

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

**良い**

setup からの同期的な呼び出しごとに、`useObserver` 内で個別の ref と watcher を作ります。Vue が watcher を呼び出し元のコンポーネントの寿命に結び付けます。

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](cross-file.md)
