---
title: 全 lint ルール
---

# 全 lint ルール

ルールの目的、既定の設定、所属するカテゴリを探すための一覧です。説明・設定・完全な悪い例と良い例・現在の対応範囲は、各カテゴリのページで確認できます。この一覧の既存のルール・例へのリンクも引き続き使用できます。

Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`vue.linter.rules` に指定します。`vp run lint` で Vize と Oxlint の lint を実行します。

既定の重大度は実装で定められた値です。プロジェクトでは `off` / `warn` / `error` に変更できます。オプションは[ルールのオプション](./options.md)、複数ファイルの検査は[ファイル間ルール](./cross-file.md)を参照してください。

プリセット欄が「none」のルールは、明示的な有効化または追加の設定が必要です。

[ESLint からのルール移行対応表](./migration.md)で対応名と未実装の範囲を確認できます。

## 単一ファイルのルール (258)

| ルール | 例 | 重大度 | プリセット | 自動修正 | オプション | 実装 | 目的 | カテゴリ |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [悪い例](#petite-vue-no-unsupported-directive-bad) · [良い例](#petite-vue-no-unsupported-directive-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) | petite-vue が対応しないディレクティブを検出します。 | Essential |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [悪い例](#petite-vue-valid-v-effect-bad) · [良い例](#petite-vue-valid-v-effect-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) | v-effect に空でない式を指定します。 | Essential |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [悪い例](#petite-vue-valid-v-scope-bad) · [良い例](#petite-vue-valid-v-scope-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) | v-scope に値を指定する場合は、オブジェクトリテラルにします。 | Essential |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [悪い例](#vue-multi-word-component-names-bad) · [良い例](#vue-multi-word-component-names-good) | `error` | `essential`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) | コンポーネント名を複数の単語で構成します。 | Essential |
| [`vue/no-child-content`](#vue-no-child-content) | [悪い例](#vue-no-child-content-bad) · [良い例](#vue-no-child-content-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 | Essential |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [悪い例](#vue-no-deprecated-filter-bad) · [良い例](#vue-no-deprecated-filter-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56) | Vue 2 のパイプによるフィルター構文を検出します。 | Essential |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [悪い例](#vue-no-deprecated-functional-template-bad) · [良い例](#vue-no-deprecated-functional-template-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) | SFC の template で削除済みの functional 属性を検出します。 | Essential |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [悪い例](#vue-no-deprecated-html-element-is-bad) · [良い例](#vue-no-deprecated-html-element-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) | 通常の HTML 要素で旧形式の is を使う箇所を検出します。 | Essential |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [悪い例](#vue-no-deprecated-inline-template-bad) · [良い例](#vue-no-deprecated-inline-template-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) | 削除済みの inline-template 属性を検出します。 | Essential |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [悪い例](#vue-no-deprecated-router-link-tag-prop-bad) · [良い例](#vue-no-deprecated-router-link-tag-prop-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) | router-link の削除済み tag prop を検出します。 | Essential |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [悪い例](#vue-no-deprecated-scope-attribute-bad) · [良い例](#vue-no-deprecated-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) | template の削除済み scope 属性を検出します。 | Essential |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [悪い例](#vue-no-deprecated-slot-attribute-bad) · [良い例](#vue-no-deprecated-slot-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) | 削除済みの slot 属性を検出します。 | Essential |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [悪い例](#vue-no-deprecated-slot-scope-attribute-bad) · [良い例](#vue-no-deprecated-slot-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) | 削除済みの slot-scope 属性を検出します。 | Essential |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [悪い例](#vue-no-deprecated-v-bind-sync-bad) · [良い例](#vue-no-deprecated-v-bind-sync-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) | 削除済みの v-bind の .sync 修飾子を検出します。 | Essential |
| [`vue/no-deprecated-v-is`](#vue-no-deprecated-v-is) | [悪い例](#vue-no-deprecated-v-is-bad) · [良い例](#vue-no-deprecated-v-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27) | 非推奨の v-is ディレクティブを検出します。 | Essential |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [悪い例](#vue-no-deprecated-v-on-native-modifier-bad) · [良い例](#vue-no-deprecated-v-on-native-modifier-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) | 削除済みの v-on の .native 修飾子を検出します。 | Essential |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [悪い例](#vue-no-deprecated-v-on-number-modifiers-bad) · [良い例](#vue-no-deprecated-v-on-number-modifiers-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) | v-on の削除済みの数値 keyCode 修飾子を検出します。 | Essential |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [悪い例](#vue-no-dupe-v-else-if-bad) · [良い例](#vue-no-dupe-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) | v-if / v-else-if の条件重複を検出します。 | Essential |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [悪い例](#vue-no-duplicate-attributes-bad) · [良い例](#vue-no-duplicate-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) | 同じ要素の属性重複を検出します。 | Essential |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [悪い例](#vue-no-multiple-template-root-bad) · [良い例](#vue-no-multiple-template-root-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L23) | 単一ルートを要求するテンプレートで複数ルートを検出します。 | Essential |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [悪い例](#vue-no-mutating-props-bad) · [良い例](#vue-no-mutating-props-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L36) | 親から受け取った prop を子コンポーネントで変更する箇所を検出します。 | Essential |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [悪い例](#vue-no-reserved-component-names-bad) · [良い例](#vue-no-reserved-component-names-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) | 予約済みのコンポーネント名を検出します。 | Essential |
| [`vue/no-template-key`](#vue-no-template-key) | [悪い例](#vue-no-template-key-bad) · [良い例](#vue-no-template-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) | v-for 用ではない template の key 指定を検出します。 | Essential |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [悪い例](#vue-no-textarea-mustache-bad) · [良い例](#vue-no-textarea-mustache-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) | textarea 内の mustache を検出し、v-model の使用を勧めます。 | Essential |
| [`vue/no-unused-components`](#vue-no-unused-components) | [悪い例](#vue-no-unused-components-bad) · [良い例](#vue-no-unused-components-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L47) | 登録しているのにテンプレートで使わないコンポーネントを検出します。 | Essential |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [悪い例](#vue-no-unused-vars-bad) · [良い例](#vue-no-unused-vars-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) | v-for / v-slot に宣言しているのに使わない変数を検出します。 | Essential |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [悪い例](#vue-no-use-v-if-with-v-for-bad) · [良い例](#vue-no-use-v-if-with-v-for-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) | 同じ要素での v-if と v-for の併用を検出します。 | Essential |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [悪い例](#vue-no-useless-template-attributes-bad) · [良い例](#vue-no-useless-template-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) | template 要素の効果がない属性を検出します。 | Essential |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [悪い例](#vue-no-v-for-template-key-on-child-bad) · [良い例](#vue-no-v-for-template-key-on-child-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) | template v-for の key を子ではなく template に指定します。 | Essential |
| [`vue/no-v-html`](#vue-no-v-html) | [悪い例](#vue-no-v-html-bad) · [良い例](#vue-no-v-html-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) | XSS を防ぐため、v-html を使用した箇所に警告を出します。 | Essential |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [悪い例](#vue-no-v-text-v-html-on-component-bad) · [良い例](#vue-no-v-text-v-html-on-component-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) | コンポーネントでの v-text / v-html を検出します。 | Essential |
| [`vue/permitted-contents`](#vue-permitted-contents) | [悪い例](#vue-permitted-contents-bad) · [良い例](#vue-permitted-contents-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) | HTML の要素ごとのコンテンツモデルを検査します。 | Essential |
| [`vue/require-component-is`](#vue-require-component-is) | [悪い例](#vue-require-component-is-bad) · [良い例](#vue-require-component-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) | 動的 component 要素に :is を指定します。 | Essential |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [悪い例](#vue-require-toggle-inside-transition-bad) · [良い例](#vue-require-toggle-inside-transition-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49) | transition の子要素に表示を切り替える条件を指定します。 | Essential |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [悪い例](#vue-require-v-for-key-bad) · [良い例](#vue-require-v-for-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L46) | v-for に安定した :key を指定します。 | Essential |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [悪い例](#vue-use-v-on-exact-bad) · [良い例](#vue-use-v-on-exact-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31) | 修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。 | Essential |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [悪い例](#vue-valid-attribute-name-bad) · [良い例](#vue-valid-attribute-name-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) | 有効な属性名を指定します。 | Essential |
| [`vue/valid-template-root`](#vue-valid-template-root) | [悪い例](#vue-valid-template-root-bad) · [良い例](#vue-valid-template-root-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) | Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。 | Essential |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [悪い例](#vue-valid-v-bind-bad) · [良い例](#vue-valid-v-bind-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) | v-bind の引数・値・修飾子を検査します。 | Essential |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [悪い例](#vue-valid-v-cloak-bad) · [良い例](#vue-valid-v-cloak-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) | v-cloak の引数・値・修飾子を検査します。 | Essential |
| [`vue/valid-v-else`](#vue-valid-v-else) | [悪い例](#vue-valid-v-else-bad) · [良い例](#vue-valid-v-else-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L31) | v-else の位置・引数・値を検査します。 | Essential |
| [`vue/valid-v-else-if`](#vue-valid-v-else-if) | [悪い例](#vue-valid-v-else-if-bad) · [良い例](#vue-valid-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34) | v-else-if の位置・引数・値を検査します。 | Essential |
| [`vue/valid-v-for`](#vue-valid-v-for) | [悪い例](#vue-valid-v-for-bad) · [良い例](#vue-valid-v-for-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L39) | v-for の式と変数宣言を検査します。 | Essential |
| [`vue/valid-v-html`](#vue-valid-v-html) | [悪い例](#vue-valid-v-html-bad) · [良い例](#vue-valid-v-html-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) | v-html の値・引数・修飾子を検査します。 | Essential |
| [`vue/valid-v-if`](#vue-valid-v-if) | [悪い例](#vue-valid-v-if-bad) · [良い例](#vue-valid-v-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) | v-if に有効な条件式を指定します。 | Essential |
| [`vue/valid-v-is`](#vue-valid-v-is) | [悪い例](#vue-valid-v-is-bad) · [良い例](#vue-valid-v-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30) | v-is の適用先・引数・値を検査します。 | Essential |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [悪い例](#vue-valid-v-memo-bad) · [良い例](#vue-valid-v-memo-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) | v-memo の値を配列の式にします。 | Essential |
| [`vue/valid-v-model`](#vue-valid-v-model) | [悪い例](#vue-valid-v-model-bad) · [良い例](#vue-valid-v-model-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) | v-model の値・引数・修飾子を検査します。 | Essential |
| [`vue/valid-v-on`](#vue-valid-v-on) | [悪い例](#vue-valid-v-on-bad) · [良い例](#vue-valid-v-on-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) | v-on のイベント名・式・修飾子を検査します。 | Essential |
| [`vue/valid-v-once`](#vue-valid-v-once) | [悪い例](#vue-valid-v-once-bad) · [良い例](#vue-valid-v-once-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) | v-once の引数・値・修飾子を検査します。 | Essential |
| [`vue/valid-v-pre`](#vue-valid-v-pre) | [悪い例](#vue-valid-v-pre-bad) · [良い例](#vue-valid-v-pre-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30) | v-pre の引数・値・修飾子を検査します。 | Essential |
| [`vue/valid-v-show`](#vue-valid-v-show) | [悪い例](#vue-valid-v-show-bad) · [良い例](#vue-valid-v-show-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) | v-show に有効な条件式を指定します。 | Essential |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [悪い例](#vue-valid-v-slot-bad) · [良い例](#vue-valid-v-slot-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) | v-slot の適用先・宣言・修飾子を検査します。 | Essential |
| [`vue/valid-v-text`](#vue-valid-v-text) | [悪い例](#vue-valid-v-text-bad) · [良い例](#vue-valid-v-text-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) | v-text の値・引数・修飾子を検査します。 | Essential |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [悪い例](#vue-attribute-hyphenation-bad) · [良い例](#vue-attribute-hyphenation-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L32) | コンポーネントの prop 属性名を設定した形式に揃えます。 | Strongly Recommended |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [悪い例](#vue-component-definition-name-casing-bad) · [良い例](#vue-component-definition-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) | コンポーネント定義名を PascalCase または kebab-case に揃えます。 | Strongly Recommended |
| [`vue/html-quotes`](#vue-html-quotes) | [悪い例](#vue-html-quotes-bad) · [良い例](#vue-html-quotes-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) | HTML 属性値の引用符を揃えます。 | Strongly Recommended |
| [`vue/html-self-closing`](#vue-html-self-closing) | [悪い例](#vue-html-self-closing-bad) · [良い例](#vue-html-self-closing-good) | `warning` | `nuxt`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) | 要素の種類ごとに自己終了タグの形式を揃えます。 | Strongly Recommended |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [悪い例](#vue-mustache-interpolation-spacing-bad) · [良い例](#vue-mustache-interpolation-spacing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L34) | mustache 内の空白を揃えます。 | Strongly Recommended |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [悪い例](#vue-no-multi-spaces-bad) · [良い例](#vue-no-multi-spaces-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) | 連続する不要な空白を検出します。 | Strongly Recommended |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [悪い例](#vue-no-template-shadow-bad) · [良い例](#vue-no-template-shadow-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) | テンプレート変数が外側の名前を隠す箇所を検出します。 | Strongly Recommended |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [悪い例](#vue-no-unused-properties-bad) · [良い例](#vue-no-unused-properties-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) | defineProps に宣言しているのに使わない prop を検出します。 | Strongly Recommended |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [悪い例](#vue-prop-name-casing-bad) · [良い例](#vue-prop-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) | 宣言する prop 名の形式を揃えます。 | Strongly Recommended |
| [`vue/v-bind-style`](#vue-v-bind-style) | [悪い例](#vue-v-bind-style-bad) · [良い例](#vue-v-bind-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28) | v-bind の表記形式を揃えます。 | Strongly Recommended |
| [`vue/v-on-style`](#vue-v-on-style) | [悪い例](#vue-v-on-style-bad) · [良い例](#vue-v-on-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) | v-on の表記形式を揃えます。 | Strongly Recommended |
| [`vue/v-slot-style`](#vue-v-slot-style) | [悪い例](#vue-v-slot-style-bad) · [良い例](#vue-v-slot-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) | v-slot の表記形式を揃えます。 | Strongly Recommended |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [悪い例](#ssr-no-browser-globals-in-ssr-bad) · [良い例](#ssr-no-browser-globals-in-ssr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 | Recommended |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [悪い例](#ssr-no-hydration-mismatch-bad) · [良い例](#ssr-no-hydration-mismatch-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) | サーバーとクライアントで一致しないテンプレート値を検出します。 | Recommended |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [悪い例](#vue-a11y-img-alt-bad) · [良い例](#vue-a11y-img-alt-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) | 画像に代替テキストの alt 属性を指定します。 | Recommended |
| [`vue/attribute-order`](#vue-attribute-order) | [悪い例](#vue-attribute-order-bad) · [良い例](#vue-attribute-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) | テンプレートの属性を一定の順に並べます。 | Recommended |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [悪い例](#vue-component-name-in-template-casing-bad) · [良い例](#vue-component-name-in-template-casing-good) | `warning` | `nuxt`, `opinionated` | あり | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) | テンプレート内のコンポーネント名を指定した形式に揃えます。 | Recommended |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [悪い例](#vue-html-button-has-type-bad) · [良い例](#vue-html-button-has-type-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) | button に有効な type を明示します。 | Recommended |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [悪い例](#vue-max-template-complexity-bad) · [良い例](#vue-max-template-complexity-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) | コンポーネント自身のテンプレートの複雑度を制限します。 | Recommended |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [悪い例](#vue-no-array-index-key-bad) · [良い例](#vue-no-array-index-key-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) | v-for の配列インデックスをそのまま key に使う箇所を検出します。 | Recommended |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [悪い例](#vue-no-bare-strings-in-template-bad) · [良い例](#vue-no-bare-strings-in-template-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) | テンプレートに直接書かれた、国際化の対象となるテキストを検出します。 | Recommended |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [悪い例](#vue-no-boolean-attr-value-bad) · [良い例](#vue-no-boolean-attr-value-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) | HTML の真偽属性に不要な値を指定した箇所を検出します。 | Recommended |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [悪い例](#vue-no-empty-component-block-bad) · [良い例](#vue-no-empty-component-block-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) | 空の SFC ブロックを検出します。 | Recommended |
| [`vue/no-inline-style`](#vue-no-inline-style) | [悪い例](#vue-no-inline-style-bad) · [良い例](#vue-no-inline-style-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) | インラインの style 属性を検出します。 | Recommended |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [悪い例](#vue-no-invalid-html-attribute-bad) · [良い例](#vue-no-invalid-html-attribute-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) | 静的 HTML 属性の無効な値を検出します。現在は rel が対象です。 | Recommended |
| [`vue/no-lone-template`](#vue-no-lone-template) | [悪い例](#vue-no-lone-template-bad) · [良い例](#vue-no-lone-template-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) | 不要な template 要素を検出します。 | Recommended |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [悪い例](#vue-no-multiple-objects-in-class-bad) · [良い例](#vue-no-multiple-objects-in-class-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) | :class 配列内の複数のオブジェクト指定をまとめます。 | Recommended |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [悪い例](#vue-no-negated-v-if-condition-bad) · [良い例](#vue-no-negated-v-if-condition-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) | v-else がある条件分岐の否定条件を反転して読みやすくします。 | Recommended |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [悪い例](#vue-no-non-component-keep-alive-child-bad) · [良い例](#vue-no-non-component-keep-alive-child-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L26) | KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。 | Recommended |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [悪い例](#vue-no-preprocessor-lang-bad) · [良い例](#vue-no-preprocessor-lang-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11) | CSS プリプロセッサーより標準の CSS を使う方針を示します。 | Recommended |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [悪い例](#vue-no-root-v-if-bad) · [良い例](#vue-no-root-v-if-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) | テンプレートの単一ルートに v-if を指定する箇所を検出します。 | Recommended |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [悪い例](#vue-no-script-non-standard-lang-bad) · [良い例](#vue-no-script-non-standard-lang-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) | script の非標準の lang 指定を避ける方針を示します。 | Recommended |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [悪い例](#vue-no-src-attribute-bad) · [良い例](#vue-no-src-attribute-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) | SFC ブロックの外部 src 指定を検出します。 | Recommended |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [悪い例](#vue-no-static-inline-styles-bad) · [良い例](#vue-no-static-inline-styles-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) | 静的なインライン style 属性を検出します。 | Recommended |
| [`vue/no-template-lang`](#vue-no-template-lang) | [悪い例](#vue-no-template-lang-bad) · [良い例](#vue-no-template-lang-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11) | template の lang 指定を避ける方針を示します。 | Recommended |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [悪い例](#vue-no-template-target-blank-bad) · [良い例](#vue-no-template-target-blank-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) | target=_blank の外部リンクに適切な rel を指定します。 | Recommended |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [悪い例](#vue-no-undefined-refs-bad) · [良い例](#vue-no-undefined-refs-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) | テンプレート内の未定義変数参照を検出します。 | Recommended |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [悪い例](#vue-no-unsafe-url-bad) · [良い例](#vue-no-unsafe-url-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) | 危険なスキームを含む可能性がある URL 属性やバインディングを検出します。 | Recommended |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [悪い例](#vue-no-unsandboxed-iframe-bad) · [良い例](#vue-no-unsandboxed-iframe-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) | iframe に sandbox 属性を指定します。 | Recommended |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [悪い例](#vue-no-unused-refs-bad) · [良い例](#vue-no-unused-refs-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) | テンプレートに宣言しているのに参照しない ref を検出します。 | Recommended |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [悪い例](#vue-no-unused-setup-bindings-bad) · [良い例](#vue-no-unused-setup-bindings-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) | script setup に宣言しているのに読み取らない変数を検出します。 | Recommended |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [悪い例](#vue-no-use-v-else-with-v-for-bad) · [良い例](#vue-no-use-v-else-with-v-for-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) | 同じ要素での v-else / v-else-if と v-for の併用を検出します。 | Recommended |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [悪い例](#vue-no-useless-mustaches-bad) · [良い例](#vue-no-useless-mustaches-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) | 文字列リテラルだけの不要な mustache を検出します。 | Recommended |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [悪い例](#vue-no-useless-v-bind-bad) · [良い例](#vue-no-useless-v-bind-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) | 文字列リテラルだけの不要な v-bind を検出します。 | Recommended |
| [`vue/no-v-text`](#vue-no-v-text) | [悪い例](#vue-no-v-text-bad) · [良い例](#vue-no-v-text-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) | v-text の代わりに mustache を使う方針を適用します。 | Recommended |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [悪い例](#vue-prefer-props-shorthand-bad) · [良い例](#vue-prefer-props-shorthand-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) | Vue 3.4 の同名 prop バインディングの省略形を使います。 | Recommended |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [悪い例](#vue-prefer-true-attribute-shorthand-bad) · [良い例](#vue-prefer-true-attribute-shorthand-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) | true を指定するバインディングを真偽属性の省略形にします。 | Recommended |
| [`vue/require-component-registration`](#vue-require-component-registration) | [悪い例](#vue-require-component-registration-bad) · [良い例](#vue-require-component-registration-good) | `warning` | `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L57) | 使用するコンポーネントを import または登録します。 | Recommended |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [悪い例](#vue-require-scoped-style-bad) · [良い例](#vue-require-scoped-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) | style に scoped を指定する方針を適用します。 | Recommended |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [悪い例](#vue-scoped-event-names-bad) · [良い例](#vue-scoped-event-names-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) | イベント名を context:event の形式に揃えます。 | Recommended |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [悪い例](#vue-sfc-element-order-bad) · [良い例](#vue-sfc-element-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) | SFC のトップレベルブロックを設定した順に並べます。 | Recommended |
| [`vue/single-style-block`](#vue-single-style-block) | [悪い例](#vue-single-style-block-bad) · [良い例](#vue-single-style-block-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) | SFC の style を一つのブロックにまとめます。 | Recommended |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [悪い例](#vue-slot-name-casing-bad) · [良い例](#vue-slot-name-casing-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) | 名前付きスロットを kebab-case に揃えます。 | Recommended |
| [`vue/this-in-template`](#vue-this-in-template) | [悪い例](#vue-this-in-template-bad) · [良い例](#vue-this-in-template-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L37) | テンプレートで不要な this. 参照を検出します。 | Recommended |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [悪い例](#vue-v-on-event-hyphenation-bad) · [良い例](#vue-v-on-event-hyphenation-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) | コンポーネントのカスタムイベント名を設定した形式に揃えます。 | Recommended |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [悪い例](#vue-v-on-handler-style-bad) · [良い例](#vue-v-on-handler-style-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) | イベントハンドラーの書き方を参照または関数の形式に揃えます。 | Recommended |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [悪い例](#vue-warn-custom-block-bad) · [良い例](#vue-warn-custom-block-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) | SFC のカスタムブロックを検出します。 | Recommended |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [悪い例](#vue-warn-custom-directive-bad) · [良い例](#vue-warn-custom-directive-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) | 登録が必要なカスタムディレクティブを検出します。 | Recommended |
| [`a11y/alt-text`](#a11y-alt-text) | [悪い例](#a11y-alt-text-bad) · [良い例](#a11y-alt-text-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) | 画像などのメディアに代替テキストを用意します。 | Accessibility |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [悪い例](#a11y-anchor-has-content-bad) · [良い例](#a11y-anchor-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) | リンクに支援技術で読める内容を用意します。 | Accessibility |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [悪い例](#a11y-anchor-is-valid-bad) · [良い例](#a11y-anchor-is-valid-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) | リンクの href に有効な移動先を指定します。 | Accessibility |
| [`a11y/aria-props`](#a11y-aria-props) | [悪い例](#a11y-aria-props-bad) · [良い例](#a11y-aria-props-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) | 存在しない ARIA 属性を検出します。 | Accessibility |
| [`a11y/aria-role`](#a11y-aria-role) | [悪い例](#a11y-aria-role-bad) · [良い例](#a11y-aria-role-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) | 抽象ロールを除く、有効な ARIA role を指定します。 | Accessibility |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [悪い例](#a11y-aria-unsupported-elements-bad) · [良い例](#a11y-aria-unsupported-elements-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) | ARIA 属性を使用できない要素への指定を検出します。 | Accessibility |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [悪い例](#a11y-click-events-have-key-events-bad) · [良い例](#a11y-click-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) | クリックで操作する要素にキーボード操作も用意します。 | Accessibility |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [悪い例](#a11y-form-control-has-label-bad) · [良い例](#a11y-form-control-has-label-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L21) | フォーム部品に関連付けられたラベルを用意します。 | Accessibility |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [悪い例](#a11y-heading-has-content-bad) · [良い例](#a11y-heading-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) | 見出しに支援技術で読める内容を用意します。 | Accessibility |
| [`a11y/heading-levels`](#a11y-heading-levels) | [悪い例](#a11y-heading-levels-bad) · [良い例](#a11y-heading-levels-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) | 見出しの階層を飛ばした指定を検出します。 | Accessibility |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [悪い例](#a11y-iframe-has-title-bad) · [良い例](#a11y-iframe-has-title-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) | iframe に内容を説明する title を指定します。 | Accessibility |
| [`a11y/img-alt`](#a11y-img-alt) | [悪い例](#a11y-img-alt-bad) · [良い例](#a11y-img-alt-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) | 画像に alt 属性を指定します。装飾画像は空の alt を使います。 | Accessibility |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [悪い例](#a11y-interactive-supports-focus-bad) · [良い例](#a11y-interactive-supports-focus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L36) | 操作可能な role の要素をフォーカス可能にします。 | Accessibility |
| [`a11y/label-has-for`](#a11y-label-has-for) | [悪い例](#a11y-label-has-for-bad) · [良い例](#a11y-label-has-for-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) | label を対象のフォーム部品と関連付けます。 | Accessibility |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [悪い例](#a11y-landmark-roles-bad) · [良い例](#a11y-landmark-roles-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) | ランドマーク role の配置と重複を検査します。 | Accessibility |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [悪い例](#a11y-media-has-caption-bad) · [良い例](#a11y-media-has-caption-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) | 音声・動画に字幕を用意します。 | Accessibility |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [悪い例](#a11y-mouse-events-have-key-events-bad) · [良い例](#a11y-mouse-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) | マウス操作と対応する focus / blur 操作を用意します。 | Accessibility |
| [`a11y/no-access-key`](#a11y-no-access-key) | [悪い例](#a11y-no-access-key-bad) · [良い例](#a11y-no-access-key-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) | 環境のショートカットと衝突し得る accesskey を検出します。 | Accessibility |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [悪い例](#a11y-no-aria-hidden-on-focusable-bad) · [良い例](#a11y-no-aria-hidden-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) | フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。 | Accessibility |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [悪い例](#a11y-no-autofocus-bad) · [良い例](#a11y-no-autofocus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) | 意図せずフォーカスを移動させる autofocus を検出します。 | Accessibility |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [悪い例](#a11y-no-distracting-elements-bad) · [良い例](#a11y-no-distracting-elements-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) | marquee や blink などの注意をそらす要素を検出します。 | Accessibility |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [悪い例](#a11y-no-i-for-icon-bad) · [良い例](#a11y-no-i-for-icon-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) | アイコン用の i 要素を検出し、意味に合う要素を勧めます。 | Accessibility |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [悪い例](#a11y-no-redundant-roles-bad) · [良い例](#a11y-no-redundant-roles-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) | 要素本来の意味と重複する ARIA role を検出します。 | Accessibility |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [悪い例](#a11y-no-refer-to-non-existent-id-bad) · [良い例](#a11y-no-refer-to-non-existent-id-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) | 文書内に存在しない ID への参照を検出します。 | Accessibility |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [悪い例](#a11y-no-role-presentation-on-focusable-bad) · [良い例](#a11y-no-role-presentation-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) | フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。 | Accessibility |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [悪い例](#a11y-no-static-element-interactions-bad) · [良い例](#a11y-no-static-element-interactions-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) | 操作部品ではない要素へのイベント指定を検出します。 | Accessibility |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [悪い例](#a11y-placeholder-label-option-bad) · [良い例](#a11y-placeholder-label-option-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47) | required の select を値が空のプレースホルダー option から始めます。 | Accessibility |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [悪い例](#a11y-role-has-required-aria-props-bad) · [良い例](#a11y-role-has-required-aria-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) | ARIA role が必要とする属性を指定します。 | Accessibility |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [悪い例](#a11y-tabindex-no-positive-bad) · [良い例](#a11y-tabindex-no-positive-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) | 通常のフォーカス順序を変える正の tabindex を検出します。 | Accessibility |
| [`a11y/use-list`](#a11y-use-list) | [悪い例](#a11y-use-list-bad) · [良い例](#a11y-use-list-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) | 箇条書きに見えるテキストをリスト要素で表現します。 | Accessibility |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) | 静的 ID の代わりに useId() で一意な ID を生成します。 | Accessibility |
| [`html/deprecated-attr`](#html-deprecated-attr) | [悪い例](#html-deprecated-attr-bad) · [良い例](#html-deprecated-attr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) | 非推奨の HTML 属性を検出します。 | HTML Conformance |
| [`html/deprecated-element`](#html-deprecated-element) | [悪い例](#html-deprecated-element-bad) · [良い例](#html-deprecated-element-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) | 非推奨の HTML 要素を検出します。 | HTML Conformance |
| [`html/id-duplication`](#html-id-duplication) | [悪い例](#html-id-duplication-bad) · [良い例](#html-id-duplication-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) | 同じテンプレート内の ID 重複を検出します。 | HTML Conformance |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [悪い例](#html-no-consecutive-br-bad) · [良い例](#html-no-consecutive-br-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) | 連続する br 要素による余白指定を検出します。 | HTML Conformance |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [悪い例](#html-no-dupe-style-properties-bad) · [良い例](#html-no-dupe-style-properties-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) | 静的 style 属性内のプロパティ重複を検出します。 | HTML Conformance |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [悪い例](#html-no-duplicate-class-bad) · [良い例](#html-no-duplicate-class-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) | 静的 class 属性内のクラス名重複を検出します。 | HTML Conformance |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [悪い例](#html-no-duplicate-dt-bad) · [良い例](#html-no-duplicate-dt-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) | dl 内の dt の名前重複を検出します。 | HTML Conformance |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [悪い例](#html-no-empty-palpable-content-bad) · [良い例](#html-no-empty-palpable-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) | 可視コンテンツを期待する要素が空の場合に検出します。 | HTML Conformance |
| [`html/require-datetime`](#html-require-datetime) | [悪い例](#html-require-datetime-bad) · [良い例](#html-require-datetime-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) | time 要素に機械可読の datetime を指定します。 | HTML Conformance |
| [`type/no-floating-promises`](#type-no-floating-promises) | [悪い例](#type-no-floating-promises-bad) · [良い例](#type-no-floating-promises-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) | 処理しないまま放置された Promise を検出します。 | Type Aware |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [悪い例](#type-no-reactivity-loss-bad) · [良い例](#type-no-reactivity-loss-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) | リアクティブな値を代入や呼び出しで取り出し、元の更新に追従しないスナップショットになる箇所を検出します。 | Type Aware |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [悪い例](#type-no-unsafe-template-binding-bad) · [良い例](#type-no-unsafe-template-binding-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) | テンプレートで安全でない型の値を使用する箇所を検出します。 | Type Aware |
| [`type/require-typed-emits`](#type-require-typed-emits) | [悪い例](#type-require-typed-emits-bad) · [良い例](#type-require-typed-emits-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) | defineEmits に型定義を指定します。 | Type Aware |
| [`type/require-typed-props`](#type-require-typed-props) | [悪い例](#type-require-typed-props-bad) · [良い例](#type-require-typed-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) | defineProps に型定義を指定します。 | Type Aware |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [悪い例](#type-strict-boolean-expressions-bad) · [良い例](#type-strict-boolean-expressions-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) | script とテンプレートの条件式で安全な真偽判定を使います。 | Type Aware |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [悪い例](#script-no-get-current-instance-bad) · [良い例](#script-no-get-current-instance-good) | `error` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) | Vapor で null を返す getCurrentInstance() を検出します。 | Vapor |
| [`script/no-next-tick`](#script-no-next-tick) | [悪い例](#script-no-next-tick-bad) · [良い例](#script-no-next-tick-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 | Vapor |
| [`script/no-options-api`](#script-no-options-api) | [悪い例](#script-no-options-api-bad) · [良い例](#script-no-options-api-good) | `error` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) | Vapor で Options API を使用する箇所を検出します。 | Vapor |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [悪い例](#vapor-no-inline-template-bad) · [良い例](#vapor-no-inline-template-good) | `error` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) | Vapor で削除済みの inline-template 属性を検出します。 | Vapor |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [悪い例](#vapor-no-vue-lifecycle-events-bad) · [良い例](#vapor-no-vue-lifecycle-events-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) | Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。 | Vapor |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [悪い例](#vapor-prefer-static-class-bad) · [良い例](#vapor-prefer-static-class-good) | `warning` | `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) | 文字列リテラルの :class を静的 class に置き換えます。 | Vapor |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [悪い例](#vapor-require-vapor-attribute-bad) · [良い例](#vapor-require-vapor-attribute-good) | `warning` | `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) | Vapor 向けの script setup に vapor 属性を付ける規約を示します。 | Vapor |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [悪い例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い例](#ecosystem-nuxt-prefer-nuxt-link-good) | `warning` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) | Nuxt の内部リンクに NuxtLink を使います。 | Ecosystem |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [悪い例](#ecosystem-pinia-prefer-store-to-refs-bad) · [良い例](#ecosystem-pinia-prefer-store-to-refs-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) | Pinia store の分割代入に storeToRefs() を使います。 | Ecosystem |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [悪い例](#ecosystem-router-link-require-to-bad) · [良い例](#ecosystem-router-link-require-to-good) | `error` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) | RouterLink / NuxtLink に to を指定します。 | Ecosystem |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [悪い例](#ecosystem-void-link-require-href-bad) · [良い例](#ecosystem-void-link-require-href-good) | `error` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) | Void Vue の Link に href を指定します。 | Ecosystem |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [悪い例](#ecosystem-void-link-valid-method-bad) · [良い例](#ecosystem-void-link-valid-method-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) | Void Vue の Link に有効な静的 method を指定します。 | Ecosystem |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [悪い例](#ecosystem-vue-i18n-no-missing-key-bad) · [良い例](#ecosystem-vue-i18n-no-missing-key-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) | SFC 内の翻訳データに存在しない静的キーを検出します。 | Ecosystem |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [悪い例](#ecosystem-vue-router-prefer-named-link-bad) · [良い例](#ecosystem-vue-router-prefer-named-link-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 | Ecosystem |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [悪い例](#ecosystem-vue-router-prefer-named-push-bad) · [良い例](#ecosystem-vue-router-prefer-named-push-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) | Vue Router のプログラムによる移動に名前付きルートを使います。 | Ecosystem |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [悪い例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い例](#ecosystem-vue-test-utils-no-html-snapshot-good) | `warning` | `ecosystem` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) | wrapper.html() 全体のスナップショットに依存するテストを検出します。 | Ecosystem |
| [`css/no-display-none`](#css-no-display-none) | [悪い例](#css-no-display-none-bad) · [良い例](#css-no-display-none-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) | 表示切り替えに display: none を使う箇所で v-show を検討します。 | CSS |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [悪い例](#css-no-hardcoded-values-bad) · [良い例](#css-no-hardcoded-values-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) | CSS に直接記述した値を CSS 変数にまとめます。 | CSS |
| [`css/no-id-selectors`](#css-no-id-selectors) | [悪い例](#css-no-id-selectors-bad) · [良い例](#css-no-id-selectors-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) | 詳細度が高い CSS の ID セレクターを検出します。 | CSS |
| [`css/no-important`](#css-no-important) | [悪い例](#css-no-important-bad) · [良い例](#css-no-important-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) | 通常のカスケードを上書きする !important を検出します。 | CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [悪い例](#css-no-utility-classes-bad) · [良い例](#css-no-utility-classes-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) | コンポーネント内でユーティリティクラスを定義する箇所を検出します。 | CSS |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [悪い例](#css-no-v-bind-performance-bad) · [良い例](#css-no-v-bind-performance-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) | CSS v-bind() の実行時コストを検討するための警告です。 | CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [悪い例](#css-prefer-logical-properties-bad) · [良い例](#css-prefer-logical-properties-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) | 書字方向に対応する CSS の論理プロパティを使います。 | CSS |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [悪い例](#css-prefer-nested-selectors-bad) · [良い例](#css-prefer-nested-selectors-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) | 子孫セレクターを CSS のネスト構文でまとめます。 | CSS |
| [`css/prefer-slotted`](#css-prefer-slotted) | [悪い例](#css-prefer-slotted-bad) · [良い例](#css-prefer-slotted-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) | slot や子コンポーネントに対する scoped CSS のセレクターを検査します。 | CSS |
| [`css/require-font-display`](#css-require-font-display) | [悪い例](#css-require-font-display-bad) · [良い例](#css-require-font-display-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) | @font-face に font-display を指定します。 | CSS |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [悪い例](#musea-no-empty-variant-bad) · [良い例](#musea-no-empty-variant-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) | 内容のない variant ブロックを検出します。 | Musea |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [悪い例](#musea-prefer-design-tokens-bad) · [良い例](#musea-prefer-design-tokens-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) | CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。 | Musea |
| [`musea/require-component`](#musea-require-component) | [悪い例](#musea-require-component-bad) · [良い例](#musea-require-component-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) | art ブロックに対象の component を指定します。 | Musea |
| [`musea/require-title`](#musea-require-title) | [悪い例](#musea-require-title-bad) · [良い例](#musea-require-title-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) | art ブロックに title を指定します。 | Musea |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [悪い例](#musea-unique-variant-names-bad) · [良い例](#musea-unique-variant-names-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) | 同じ Art ファイル内の variant 名を一意にします。 | Musea |
| [`musea/valid-variant`](#musea-valid-variant) | [悪い例](#musea-valid-variant-bad) · [良い例](#musea-valid-variant-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) | variant ブロックに name を指定します。 | Musea |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [悪い例](#script-component-options-name-casing-bad) · [良い例](#script-component-options-name-casing-good) | `error` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) | コンポーネントの name オプションを PascalCase に揃えます。 | Script |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [悪い例](#script-custom-event-name-casing-bad) · [良い例](#script-custom-event-name-casing-good) | `error` | `opinionated`, `nuxt` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) | emit するカスタムイベント名を指定した形式に揃えます。 | Script |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [悪い例](#script-define-emits-declaration-bad) · [良い例](#script-define-emits-declaration-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) | defineEmits を型による宣言形式に揃えます。 | Script |
| [`script/define-macros-order`](#script-define-macros-order) | [悪い例](#script-define-macros-order-bad) · [良い例](#script-define-macros-order-good) | `warning` | `opinionated`, `nuxt` | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44) | script setup のコンパイラーマクロを一定の順に宣言します。 | Script |
| [`script/define-props-declaration`](#script-define-props-declaration) | [悪い例](#script-define-props-declaration-bad) · [良い例](#script-define-props-declaration-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) | defineProps を型による宣言形式に揃えます。 | Script |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [悪い例](#script-define-props-destructuring-bad) · [良い例](#script-define-props-destructuring-good) | `warning` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) | defineProps の分割代入スタイルを指定した方針に揃えます。 | Script |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [悪い例](#script-no-arrow-functions-in-watch-bad) · [良い例](#script-no-arrow-functions-in-watch-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) | Options API の watch ハンドラーにアロー関数を使う箇所を検出します。 | Script |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [悪い例](#script-no-async-in-computed-bad) · [良い例](#script-no-async-in-computed-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) | computed の getter で非同期関数を使う箇所を検出します。 | Script |
| [`script/no-boolean-default`](#script-no-boolean-default) | [悪い例](#script-no-boolean-default-bad) · [良い例](#script-no-boolean-default-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) | Boolean prop の冗長な default を検出します。 | Script |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [悪い例](#script-no-deep-destructure-in-props-bad) · [良い例](#script-no-deep-destructure-in-props-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39) | defineProps の深い分割代入を検出します。 | Script |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [悪い例](#script-no-deprecated-data-object-declaration-bad) · [良い例](#script-no-deprecated-data-object-declaration-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 | Script |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [悪い例](#script-no-deprecated-destroyed-lifecycle-bad) · [良い例](#script-no-deprecated-destroyed-lifecycle-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。 | Script |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [悪い例](#script-no-deprecated-dollar-listeners-api-bad) · [良い例](#script-no-deprecated-dollar-listeners-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54) | Vue 3 で $attrs に統合された $listeners を検出します。 | Script |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [悪い例](#script-no-deprecated-dollar-scopedslots-api-bad) · [良い例](#script-no-deprecated-dollar-scopedslots-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45) | Vue 3 で $slots に統合された $scopedSlots を検出します。 | Script |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [悪い例](#script-no-deprecated-events-api-bad) · [良い例](#script-no-deprecated-events-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51) | Vue 3 で削除された $on / $off / $once を検出します。 | Script |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [悪い例](#script-no-deprecated-props-default-this-bad) · [良い例](#script-no-deprecated-props-default-this-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) | prop の default / validator 内で使えなくなった this を検出します。 | Script |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [悪い例](#script-no-dupe-keys-bad) · [良い例](#script-no-dupe-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L55) | Options API の props / data / computed などのキー重複を検出します。 | Script |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [悪い例](#script-no-duplicate-attr-inheritance-bad) · [良い例](#script-no-duplicate-attr-inheritance-good) | `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 | Script |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [悪い例](#script-no-export-in-script-setup-bad) · [良い例](#script-no-export-in-script-setup-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) | script setup 内の export 文を検出します。 | Script |
| [`script/no-expose-after-await`](#script-no-expose-after-await) | [悪い例](#script-no-expose-after-await-bad) · [良い例](#script-no-expose-after-await-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34) | await の後の expose() と defineExpose() を禁止します。 | Script |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [悪い例](#script-no-import-compiler-macros-bad) · [良い例](#script-no-import-compiler-macros-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) | 自動的に使える Vue コンパイラーマクロの import を検出します。 | Script |
| [`script/no-internal-imports`](#script-no-internal-imports) | [悪い例](#script-no-internal-imports-bad) · [良い例](#script-no-internal-imports-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) | Vue 内部モジュールからの import を検出します。 | Script |
| [`script/no-lifecycle-after-await`](#script-no-lifecycle-after-await) | [悪い例](#script-no-lifecycle-after-await-bad) · [良い例](#script-no-lifecycle-after-await-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45) | setup() 内で await の後にライフサイクルフックを登録することを禁止します。 | Script |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [悪い例](#script-no-multiple-slot-args-bad) · [良い例](#script-no-multiple-slot-args-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) | scoped slot 関数に複数の引数を渡す箇所を検出します。 | Script |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [悪い例](#script-no-potential-component-option-typo-bad) · [良い例](#script-no-potential-component-option-typo-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) | Options API のオプション名の入力ミスを検出します。 | Script |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [悪い例](#script-no-reactive-destructure-bad) · [良い例](#script-no-reactive-destructure-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) | reactive オブジェクトを分割代入し、リアクティビティを失う箇所を検出します。 | Script |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [悪い例](#script-no-ref-as-operand-bad) · [良い例](#script-no-ref-as-operand-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) | ref を演算の値として使う際に .value を参照します。 | Script |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [悪い例](#script-no-required-prop-with-default-bad) · [良い例](#script-no-required-prop-with-default-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L34) | required: true と default を同時に持つ prop を検出します。 | Script |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [悪い例](#script-no-reserved-identifiers-bad) · [良い例](#script-no-reserved-identifiers-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) | Vue コンパイラーが予約した識別子の宣言を検出します。 | Script |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [悪い例](#script-no-reserved-keys-bad) · [良い例](#script-no-reserved-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30) | Options API のキーに Vue の予約名を使う箇所を検出します。 | Script |
| [`script/no-reserved-props`](#script-no-reserved-props) | [悪い例](#script-no-reserved-props-bad) · [良い例](#script-no-reserved-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) | prop 宣言に Vue の予約名を使う箇所を検出します。 | Script |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [悪い例](#script-no-restricted-globals-bad) · [良い例](#script-no-restricted-globals-good) | `error` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) | 設定で禁止した実行環境のグローバル参照を検出します。 | Script |
| [`script/no-restricted-members`](#script-no-restricted-members) | [悪い例](#script-no-restricted-members-bad) · [良い例](#script-no-restricted-members-good) | `error` | _none_ | なし | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) | 設定で禁止した object.property へのアクセスを検出します。 | Script |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [悪い例](#script-no-side-effects-in-computed-properties-bad) · [良い例](#script-no-side-effects-in-computed-properties-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L77) | Options API の computed getter 内の副作用を検出します。 | Script |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [悪い例](#script-no-top-level-ref-in-script-bad) · [良い例](#script-no-top-level-ref-in-script-good) | `error` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 | Script |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [悪い例](#script-no-unstable-nested-components-bad) · [良い例](#script-no-unstable-nested-components-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 | Script |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [悪い例](#script-no-unused-emit-declarations-bad) · [良い例](#script-no-unused-emit-declarations-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) | 宣言したまま emit していないイベントを検出します。 | Script |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [悪い例](#script-no-use-computed-property-like-method-bad) · [良い例](#script-no-use-computed-property-like-method-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 | Script |
| [`script/no-watch-after-await`](#script-no-watch-after-await) | [悪い例](#script-no-watch-after-await-bad) · [良い例](#script-no-watch-after-await-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44) | setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。 | Script |
| [`script/no-with-defaults`](#script-no-with-defaults) | [悪い例](#script-no-with-defaults-bad) · [良い例](#script-no-with-defaults-good) | `warning` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 | Script |
| [`script/prefer-computed`](#script-prefer-computed) | [悪い例](#script-prefer-computed-bad) · [良い例](#script-prefer-computed-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 | Script |
| [`script/prefer-define-options`](#script-prefer-define-options) | [悪い例](#script-prefer-define-options-bad) · [良い例](#script-prefer-define-options-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 | Script |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [悪い例](#script-prefer-import-from-vue-bad) · [良い例](#script-prefer-import-from-vue-good) | `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36) | 内部パッケージではなく vue から import します。 | Script |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [悪い例](#script-prefer-ref-over-reactive-bad) · [良い例](#script-prefer-ref-over-reactive-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) | 状態管理に reactive() より ref() を使う方針を適用します。 | Script |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [悪い例](#script-prefer-use-attrs-bad) · [良い例](#script-prefer-use-attrs-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) | setup の context.attrs を useAttrs() に置き換えます。 | Script |
| [`script/prefer-use-id`](#script-prefer-use-id) | [悪い例](#script-prefer-use-id-bad) · [良い例](#script-prefer-use-id-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45) | 一意な ID の生成に Vue 3.5 の useId() を使います。 | Script |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [悪い例](#script-prefer-use-slots-bad) · [良い例](#script-prefer-use-slots-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) | setup の context.slots を useSlots() に置き換えます。 | Script |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [悪い例](#script-prefer-use-template-ref-bad) · [良い例](#script-prefer-use-template-ref-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 | Script |
| [`script/require-default-prop`](#script-require-default-prop) | [悪い例](#script-require-default-prop-bad) · [良い例](#script-require-default-prop-good) | `error` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) | 省略可能な、Boolean 以外の prop に default を用意します。 | Script |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [悪い例](#script-require-explicit-emits-bad) · [良い例](#script-require-explicit-emits-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L65) | emit するイベントを defineEmits または emits に宣言します。 | Script |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [悪い例](#script-require-explicit-slots-bad) · [良い例](#script-require-explicit-slots-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) | useSlots() で使う slot を defineSlots の型で宣言します。 | Script |
| [`script/require-function-return-type`](#script-require-function-return-type) | [悪い例](#script-require-function-return-type-bad) · [良い例](#script-require-function-return-type-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) | 関数に戻り値の型注釈を指定します。 | Script |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [悪い例](#script-require-prop-type-constructor-bad) · [良い例](#script-require-prop-type-constructor-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L65) | prop の type に文字列ではなくコンストラクターを指定します。 | Script |
| [`script/require-prop-types`](#script-require-prop-types) | [悪い例](#script-require-prop-types-bad) · [良い例](#script-require-prop-types-good) | `error` | `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) | 各 prop の型を宣言します。 | Script |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [悪い例](#script-require-symbol-provide-bad) · [良い例](#script-require-symbol-provide-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) | provide / inject のキーに衝突しにくい Symbol を使います。 | Script |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [悪い例](#script-require-typed-object-prop-bad) · [良い例](#script-require-typed-object-prop-good) | `warning` | `opinionated`, `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) | Object / Array の prop に具体的な型を指定します。 | Script |
| [`script/require-typed-ref`](#script-require-typed-ref) | [悪い例](#script-require-typed-ref-bad) · [良い例](#script-require-typed-ref-good) | `warning` | _none_ | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) | 引数なし、null、undefined で呼び出す ref() に型引数を指定します。 | Script |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [悪い例](#script-require-valid-default-prop-bad) · [良い例](#script-require-valid-default-prop-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) | prop の default を宣言した型に合う値にします。 | Script |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [悪い例](#script-return-in-computed-property-bad) · [良い例](#script-return-in-computed-property-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) | computed の getter に値を返す return を用意します。 | Script |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [悪い例](#script-return-in-emits-validator-bad) · [良い例](#script-return-in-emits-validator-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61) | Options API の emits validator に戻り値を用意します。 | Script |
| [`script/valid-define-emits`](#script-valid-define-emits) | [悪い例](#script-valid-define-emits-bad) · [良い例](#script-valid-define-emits-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) | defineEmits の重複や型と実行時引数の併用を検出します。 | Script |
| [`script/valid-define-options`](#script-valid-define-options) | [悪い例](#script-valid-define-options-bad) · [良い例](#script-valid-define-options-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) | defineOptions の引数と使用回数を検査します。 | Script |
| [`script/valid-define-props`](#script-valid-define-props) | [悪い例](#script-valid-define-props-bad) · [良い例](#script-valid-define-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) | defineProps の重複や型と実行時引数の併用を検出します。 | Script |
| [`script/valid-next-tick`](#script-valid-next-tick) | [悪い例](#script-valid-next-tick-bad) · [良い例](#script-valid-next-tick-good) | `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) | nextTick() の完了を await、then、またはコールバックで扱います。 | Script |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [悪い例](#nuxt-no-nuxt-config-test-key-bad) · [良い例](#nuxt-no-nuxt-config-test-key-good) | `error` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) | Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。 | Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [悪い例](#nuxt-no-page-meta-runtime-values-bad) · [良い例](#nuxt-no-page-meta-runtime-values-good) | `error` | `nuxt` | なし | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 | Nuxt |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [悪い例](#nuxt-nuxt-config-keys-order-bad) · [良い例](#nuxt-nuxt-config-keys-order-good) | `error` | `nuxt` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) | Nuxt 設定のプロパティを推奨順に並べます。 | Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [悪い例](#nuxt-prefer-import-meta-bad) · [良い例](#nuxt-prefer-import-meta-good) | `error` | `nuxt` | あり | なし | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 | Nuxt |

<span id="プロジェクト-ルールと-analyzer-契約-66"></span>

## プロジェクトのルールと解析器の診断契約 (66)

以下の項目は、上の 258 件の単一ファイルのルールを補足します。それぞれの例には、コンポーネントやプロジェクトの構成、または参照を追跡するグラフの具体的な条件が含まれます。CLI の検査、実験的なライブラリーの生成元、生成元のない診断契約では対応範囲が異なります。[ファイル間検査の概要](./cross-file.md)を確認してください。

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
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [悪い例](#vize-croquis-cf-async-no-suspense-bad) · [良い例](#vize-croquis-cf-async-no-suspense-good) | ライブラリーの生成元あり。必要なソース情報が未対応 |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [悪い例](#vize-croquis-cf-browser-api-ssr-bad) · [良い例](#vize-croquis-cf-browser-api-ssr-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [悪い例](#vize-croquis-cf-circular-dep-bad) · [良い例](#vize-croquis-cf-circular-dep-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [悪い例](#vize-croquis-cf-circular-reactive-dependency-bad) · [良い例](#vize-croquis-cf-circular-reactive-dependency-good) | CLI: 参照を追跡するグラフの例 |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [悪い例](#vize-croquis-cf-closure-captures-reactive-bad) · [良い例](#vize-croquis-cf-closure-captures-reactive-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [悪い例](#vize-croquis-cf-composable-outside-setup-bad) · [良い例](#vize-croquis-cf-composable-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [悪い例](#vize-croquis-cf-computed-side-effects-bad) · [良い例](#vize-croquis-cf-computed-side-effects-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [悪い例](#vize-croquis-cf-deep-import-bad) · [良い例](#vize-croquis-cf-deep-import-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [悪い例](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-destructuring-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [悪い例](#vize-croquis-cf-di-outside-setup-bad) · [良い例](#vize-croquis-cf-di-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [悪い例](#vize-croquis-cf-dom-access-without-next-tick-bad) · [良い例](#vize-croquis-cf-dom-access-without-next-tick-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [悪い例](#vize-croquis-cf-duplicate-id-bad) · [良い例](#vize-croquis-cf-duplicate-id-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [悪い例](#vize-croquis-cf-event-listener-leak-bad) · [良い例](#vize-croquis-cf-event-listener-leak-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [悪い例](#vize-croquis-cf-event-modifier-bad) · [良い例](#vize-croquis-cf-event-modifier-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [悪い例](#vize-croquis-cf-hydration-risk-bad) · [良い例](#vize-croquis-cf-hydration-risk-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [悪い例](#vize-croquis-cf-inherit-attrs-unused-bad) · [良い例](#vize-croquis-cf-inherit-attrs-unused-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [悪い例](#vize-croquis-cf-inject-without-symbol-bad) · [良い例](#vize-croquis-cf-inject-without-symbol-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [悪い例](#vize-croquis-cf-injected-async-mutation-race-bad) · [良い例](#vize-croquis-cf-injected-async-mutation-race-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [悪い例](#vize-croquis-cf-lifecycle-outside-setup-bad) · [良い例](#vize-croquis-cf-lifecycle-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [悪い例](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [良い例](#vize-croquis-cf-lifecycle-without-cleanup-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [悪い例](#vize-croquis-cf-missing-required-prop-bad) · [良い例](#vize-croquis-cf-missing-required-prop-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [悪い例](#vize-croquis-cf-missing-suspense-bad) · [良い例](#vize-croquis-cf-missing-suspense-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [悪い例](#vize-croquis-cf-module-scope-reactive-bad) · [良い例](#vize-croquis-cf-module-scope-reactive-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [悪い例](#vize-croquis-cf-multi-root-attrs-bad) · [良い例](#vize-croquis-cf-multi-root-attrs-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [悪い例](#vize-croquis-cf-mutated-after-escape-bad) · [良い例](#vize-croquis-cf-mutated-after-escape-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [悪い例](#vize-croquis-cf-non-reactive-provide-bad) · [良い例](#vize-croquis-cf-non-reactive-provide-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [悪い例](#vize-croquis-cf-non-unique-id-bad) · [良い例](#vize-croquis-cf-non-unique-id-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [悪い例](#vize-croquis-cf-object-identity-comparison-bad) · [良い例](#vize-croquis-cf-object-identity-comparison-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [悪い例](#vize-croquis-cf-pinia-getter-bad) · [良い例](#vize-croquis-cf-pinia-getter-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [悪い例](#vize-croquis-cf-prop-type-mismatch-bad) · [良い例](#vize-croquis-cf-prop-type-mismatch-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [悪い例](#vize-croquis-cf-provide-inject-type-bad) · [良い例](#vize-croquis-cf-provide-inject-type-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [悪い例](#vize-croquis-cf-provide-without-symbol-bad) · [良い例](#vize-croquis-cf-provide-without-symbol-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [悪い例](#vize-croquis-cf-reactive-export-bad) · [良い例](#vize-croquis-cf-reactive-export-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [悪い例](#vize-croquis-cf-reactivity-outside-setup-bad) · [良い例](#vize-croquis-cf-reactivity-outside-setup-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [悪い例](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-reassignment-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [悪い例](#vize-croquis-cf-reference-escapes-scope-bad) · [良い例](#vize-croquis-cf-reference-escapes-scope-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [悪い例](#vize-croquis-cf-setup-context-violation-bad) · [良い例](#vize-croquis-cf-setup-context-violation-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [悪い例](#vize-croquis-cf-shallow-deep-access-bad) · [良い例](#vize-croquis-cf-shallow-deep-access-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [悪い例](#vize-croquis-cf-spread-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-spread-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [悪い例](#vize-croquis-cf-suspense-no-fallback-bad) · [良い例](#vize-croquis-cf-suspense-no-fallback-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [悪い例](#vize-croquis-cf-template-ref-timing-bad) · [良い例](#vize-croquis-cf-template-ref-timing-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [悪い例](#vize-croquis-cf-toraw-mutation-bad) · [良い例](#vize-croquis-cf-toraw-mutation-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [悪い例](#vize-croquis-cf-uncaught-error-bad) · [良い例](#vize-croquis-cf-uncaught-error-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [悪い例](#vize-croquis-cf-undeclared-emit-bad) · [良い例](#vize-croquis-cf-undeclared-emit-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [悪い例](#vize-croquis-cf-undeclared-prop-bad) · [良い例](#vize-croquis-cf-undeclared-prop-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [悪い例](#vize-croquis-cf-undefined-slot-bad) · [良い例](#vize-croquis-cf-undefined-slot-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [悪い例](#vize-croquis-cf-unhandled-event-bad) · [良い例](#vize-croquis-cf-unhandled-event-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [悪い例](#vize-croquis-cf-unmatched-inject-bad) · [良い例](#vize-croquis-cf-unmatched-inject-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [悪い例](#vize-croquis-cf-unmatched-listener-bad) · [良い例](#vize-croquis-cf-unmatched-listener-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [悪い例](#vize-croquis-cf-unregistered-component-bad) · [良い例](#vize-croquis-cf-unregistered-component-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [悪い例](#vize-croquis-cf-unresolved-import-bad) · [良い例](#vize-croquis-cf-unresolved-import-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [悪い例](#vize-croquis-cf-unused-attrs-bad) · [良い例](#vize-croquis-cf-unused-attrs-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [悪い例](#vize-croquis-cf-unused-emit-bad) · [良い例](#vize-croquis-cf-unused-emit-good) | 実験的なライブラリーの生成元。CLI の個別コードでは未生成 |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [悪い例](#vize-croquis-cf-unused-provide-bad) · [良い例](#vize-croquis-cf-unused-provide-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [悪い例](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [良い例](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [悪い例](#vize-croquis-cf-watch-can-be-computed-bad) · [良い例](#vize-croquis-cf-watch-can-be-computed-good) | 契約のみ。生成元なし |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [悪い例](#vize-croquis-cf-watcheffect-async-bad) · [良い例](#vize-croquis-cf-watcheffect-async-good) | CLI のプロジェクト検査 |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [悪い例](#vize-croquis-cf-watcher-outside-setup-bad) · [良い例](#vize-croquis-cf-watcher-outside-setup-good) | 契約のみ。生成元なし |

## 単一ファイルの例

### `petite-vue/no-unsupported-directive`

petite-vue が対応しないディレクティブを検出します。

<span id="petite-vue-no-unsupported-directive-bad"></span>

[悪い](./petite-vue.md#petite-vue-no-unsupported-directive-bad)

<span id="petite-vue-no-unsupported-directive-good"></span>

[良い](./petite-vue.md#petite-vue-no-unsupported-directive-good)

[カテゴリの例](./petite-vue.md#petite-vue-no-unsupported-directive) · [ルール参照](./reference/petite-vue-no-unsupported-directive.md)

### `petite-vue/valid-v-effect`

v-effect に空でない式を指定します。

<span id="petite-vue-valid-v-effect-bad"></span>

[悪い](./petite-vue.md#petite-vue-valid-v-effect-bad)

<span id="petite-vue-valid-v-effect-good"></span>

[良い](./petite-vue.md#petite-vue-valid-v-effect-good)

[カテゴリの例](./petite-vue.md#petite-vue-valid-v-effect) · [ルール参照](./reference/petite-vue-valid-v-effect.md)

### `petite-vue/valid-v-scope`

v-scope に値を指定する場合は、オブジェクトリテラルにします。

<span id="petite-vue-valid-v-scope-bad"></span>

[悪い](./petite-vue.md#petite-vue-valid-v-scope-bad)

<span id="petite-vue-valid-v-scope-good"></span>

[良い](./petite-vue.md#petite-vue-valid-v-scope-good)

[カテゴリの例](./petite-vue.md#petite-vue-valid-v-scope) · [ルール参照](./reference/petite-vue-valid-v-scope.md)

### `vue/multi-word-component-names`

コンポーネント名を複数の単語で構成します。

<span id="vue-multi-word-component-names-bad"></span>

[悪い](./vue.md#vue-multi-word-component-names-bad)

<span id="vue-multi-word-component-names-good"></span>

[良い](./vue.md#vue-multi-word-component-names-good)

[カテゴリの例](./vue.md#vue-multi-word-component-names) · [ルール参照](./reference/vue-multi-word-component-names.md)

### `vue/no-child-content`

v-html / v-text と子コンテンツを同時に指定する箇所を検出します。

<span id="vue-no-child-content-bad"></span>

[悪い](./vue.md#vue-no-child-content-bad)

<span id="vue-no-child-content-good"></span>

[良い](./vue.md#vue-no-child-content-good)

[カテゴリの例](./vue.md#vue-no-child-content) · [ルール参照](./reference/vue-no-child-content.md)

### `vue/no-deprecated-filter`

Vue 2 のパイプによるフィルター構文を検出します。

<span id="vue-no-deprecated-filter-bad"></span>

[悪い](./vue.md#vue-no-deprecated-filter-bad)

<span id="vue-no-deprecated-filter-good"></span>

[良い](./vue.md#vue-no-deprecated-filter-good)

[カテゴリの例](./vue.md#vue-no-deprecated-filter) · [ルール参照](./reference/vue-no-deprecated-filter.md)

### `vue/no-deprecated-functional-template`

SFC の template で削除済みの functional 属性を検出します。

<span id="vue-no-deprecated-functional-template-bad"></span>

[悪い](./vue.md#vue-no-deprecated-functional-template-bad)

<span id="vue-no-deprecated-functional-template-good"></span>

[良い](./vue.md#vue-no-deprecated-functional-template-good)

[カテゴリの例](./vue.md#vue-no-deprecated-functional-template) · [ルール参照](./reference/vue-no-deprecated-functional-template.md)

### `vue/no-deprecated-html-element-is`

通常の HTML 要素で旧形式の is を使う箇所を検出します。

<span id="vue-no-deprecated-html-element-is-bad"></span>

[悪い](./vue.md#vue-no-deprecated-html-element-is-bad)

<span id="vue-no-deprecated-html-element-is-good"></span>

[良い](./vue.md#vue-no-deprecated-html-element-is-good)

[カテゴリの例](./vue.md#vue-no-deprecated-html-element-is) · [ルール参照](./reference/vue-no-deprecated-html-element-is.md)

### `vue/no-deprecated-inline-template`

削除済みの inline-template 属性を検出します。

<span id="vue-no-deprecated-inline-template-bad"></span>

[悪い](./vue.md#vue-no-deprecated-inline-template-bad)

<span id="vue-no-deprecated-inline-template-good"></span>

[良い](./vue.md#vue-no-deprecated-inline-template-good)

[カテゴリの例](./vue.md#vue-no-deprecated-inline-template) · [ルール参照](./reference/vue-no-deprecated-inline-template.md)

### `vue/no-deprecated-router-link-tag-prop`

router-link の削除済み tag prop を検出します。

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

[悪い](./vue.md#vue-no-deprecated-router-link-tag-prop-bad)

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

[良い](./vue.md#vue-no-deprecated-router-link-tag-prop-good)

[カテゴリの例](./vue.md#vue-no-deprecated-router-link-tag-prop) · [ルール参照](./reference/vue-no-deprecated-router-link-tag-prop.md)

### `vue/no-deprecated-scope-attribute`

template の削除済み scope 属性を検出します。

<span id="vue-no-deprecated-scope-attribute-bad"></span>

[悪い](./vue.md#vue-no-deprecated-scope-attribute-bad)

<span id="vue-no-deprecated-scope-attribute-good"></span>

[良い](./vue.md#vue-no-deprecated-scope-attribute-good)

[カテゴリの例](./vue.md#vue-no-deprecated-scope-attribute) · [ルール参照](./reference/vue-no-deprecated-scope-attribute.md)

### `vue/no-deprecated-slot-attribute`

削除済みの slot 属性を検出します。

<span id="vue-no-deprecated-slot-attribute-bad"></span>

[悪い](./vue.md#vue-no-deprecated-slot-attribute-bad)

<span id="vue-no-deprecated-slot-attribute-good"></span>

[良い](./vue.md#vue-no-deprecated-slot-attribute-good)

[カテゴリの例](./vue.md#vue-no-deprecated-slot-attribute) · [ルール参照](./reference/vue-no-deprecated-slot-attribute.md)

### `vue/no-deprecated-slot-scope-attribute`

削除済みの slot-scope 属性を検出します。

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

[悪い](./vue.md#vue-no-deprecated-slot-scope-attribute-bad)

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

[良い](./vue.md#vue-no-deprecated-slot-scope-attribute-good)

[カテゴリの例](./vue.md#vue-no-deprecated-slot-scope-attribute) · [ルール参照](./reference/vue-no-deprecated-slot-scope-attribute.md)

### `vue/no-deprecated-v-bind-sync`

削除済みの v-bind の .sync 修飾子を検出します。

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

[悪い](./vue.md#vue-no-deprecated-v-bind-sync-bad)

<span id="vue-no-deprecated-v-bind-sync-good"></span>

[良い](./vue.md#vue-no-deprecated-v-bind-sync-good)

[カテゴリの例](./vue.md#vue-no-deprecated-v-bind-sync) · [ルール参照](./reference/vue-no-deprecated-v-bind-sync.md)

### `vue/no-deprecated-v-is`

非推奨の v-is ディレクティブを検出します。

<span id="vue-no-deprecated-v-is-bad"></span>

[悪い](./vue.md#vue-no-deprecated-v-is-bad)

<span id="vue-no-deprecated-v-is-good"></span>

[良い](./vue.md#vue-no-deprecated-v-is-good)

[カテゴリの例](./vue.md#vue-no-deprecated-v-is) · [ルール参照](./reference/vue-no-deprecated-v-is.md)

### `vue/no-deprecated-v-on-native-modifier`

削除済みの v-on の .native 修飾子を検出します。

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

[悪い](./vue.md#vue-no-deprecated-v-on-native-modifier-bad)

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

[良い](./vue.md#vue-no-deprecated-v-on-native-modifier-good)

[カテゴリの例](./vue.md#vue-no-deprecated-v-on-native-modifier) · [ルール参照](./reference/vue-no-deprecated-v-on-native-modifier.md)

### `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済みの数値 keyCode 修飾子を検出します。

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

[悪い](./vue.md#vue-no-deprecated-v-on-number-modifiers-bad)

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

[良い](./vue.md#vue-no-deprecated-v-on-number-modifiers-good)

[カテゴリの例](./vue.md#vue-no-deprecated-v-on-number-modifiers) · [ルール参照](./reference/vue-no-deprecated-v-on-number-modifiers.md)

### `vue/no-dupe-v-else-if`

v-if / v-else-if の条件重複を検出します。

<span id="vue-no-dupe-v-else-if-bad"></span>

[悪い](./vue.md#vue-no-dupe-v-else-if-bad)

<span id="vue-no-dupe-v-else-if-good"></span>

[良い](./vue.md#vue-no-dupe-v-else-if-good)

[カテゴリの例](./vue.md#vue-no-dupe-v-else-if) · [ルール参照](./reference/vue-no-dupe-v-else-if.md)

### `vue/no-duplicate-attributes`

同じ要素の属性重複を検出します。

<span id="vue-no-duplicate-attributes-bad"></span>

[悪い](./vue.md#vue-no-duplicate-attributes-bad)

<span id="vue-no-duplicate-attributes-good"></span>

[良い](./vue.md#vue-no-duplicate-attributes-good)

[カテゴリの例](./vue.md#vue-no-duplicate-attributes) · [ルール参照](./reference/vue-no-duplicate-attributes.md)

### `vue/no-multiple-template-root`

単一ルートを要求するテンプレートで複数ルートを検出します。

<span id="vue-no-multiple-template-root-bad"></span>

[悪い](./vue.md#vue-no-multiple-template-root-bad)

<span id="vue-no-multiple-template-root-good"></span>

[良い](./vue.md#vue-no-multiple-template-root-good)

[カテゴリの例](./vue.md#vue-no-multiple-template-root) · [ルール参照](./reference/vue-no-multiple-template-root.md)

### `vue/no-mutating-props`

親から受け取った prop を子コンポーネントで変更する箇所を検出します。

<span id="vue-no-mutating-props-bad"></span>

[悪い](./vue.md#vue-no-mutating-props-bad)

<span id="vue-no-mutating-props-good"></span>

[良い](./vue.md#vue-no-mutating-props-good)

[カテゴリの例](./vue.md#vue-no-mutating-props) · [ルール参照](./reference/vue-no-mutating-props.md)

### `vue/no-reserved-component-names`

予約済みのコンポーネント名を検出します。

<span id="vue-no-reserved-component-names-bad"></span>

[悪い](./vue.md#vue-no-reserved-component-names-bad)

<span id="vue-no-reserved-component-names-good"></span>

[良い](./vue.md#vue-no-reserved-component-names-good)

[カテゴリの例](./vue.md#vue-no-reserved-component-names) · [ルール参照](./reference/vue-no-reserved-component-names.md)

### `vue/no-template-key`

v-for 用ではない template の key 指定を検出します。

<span id="vue-no-template-key-bad"></span>

[悪い](./vue.md#vue-no-template-key-bad)

<span id="vue-no-template-key-good"></span>

[良い](./vue.md#vue-no-template-key-good)

[カテゴリの例](./vue.md#vue-no-template-key) · [ルール参照](./reference/vue-no-template-key.md)

### `vue/no-textarea-mustache`

textarea 内の mustache を検出し、v-model の使用を勧めます。

<span id="vue-no-textarea-mustache-bad"></span>

[悪い](./vue.md#vue-no-textarea-mustache-bad)

<span id="vue-no-textarea-mustache-good"></span>

[良い](./vue.md#vue-no-textarea-mustache-good)

[カテゴリの例](./vue.md#vue-no-textarea-mustache) · [ルール参照](./reference/vue-no-textarea-mustache.md)

### `vue/no-unused-components`

登録しているのにテンプレートで使わないコンポーネントを検出します。

<span id="vue-no-unused-components-bad"></span>

[悪い](./vue.md#vue-no-unused-components-bad)

<span id="vue-no-unused-components-good"></span>

[良い](./vue.md#vue-no-unused-components-good)

[カテゴリの例](./vue.md#vue-no-unused-components) · [ルール参照](./reference/vue-no-unused-components.md)

### `vue/no-unused-vars`

v-for / v-slot に宣言しているのに使わない変数を検出します。

<span id="vue-no-unused-vars-bad"></span>

[悪い](./vue.md#vue-no-unused-vars-bad)

<span id="vue-no-unused-vars-good"></span>

[良い](./vue.md#vue-no-unused-vars-good)

[カテゴリの例](./vue.md#vue-no-unused-vars) · [ルール参照](./reference/vue-no-unused-vars.md)

### `vue/no-use-v-if-with-v-for`

同じ要素での v-if と v-for の併用を検出します。

<span id="vue-no-use-v-if-with-v-for-bad"></span>

[悪い](./vue.md#vue-no-use-v-if-with-v-for-bad)

<span id="vue-no-use-v-if-with-v-for-good"></span>

[良い](./vue.md#vue-no-use-v-if-with-v-for-good)

[カテゴリの例](./vue.md#vue-no-use-v-if-with-v-for) · [ルール参照](./reference/vue-no-use-v-if-with-v-for.md)

### `vue/no-useless-template-attributes`

template 要素の効果がない属性を検出します。

<span id="vue-no-useless-template-attributes-bad"></span>

[悪い](./vue.md#vue-no-useless-template-attributes-bad)

<span id="vue-no-useless-template-attributes-good"></span>

[良い](./vue.md#vue-no-useless-template-attributes-good)

[カテゴリの例](./vue.md#vue-no-useless-template-attributes) · [ルール参照](./reference/vue-no-useless-template-attributes.md)

### `vue/no-v-for-template-key-on-child`

template v-for の key を子ではなく template に指定します。

<span id="vue-no-v-for-template-key-on-child-bad"></span>

[悪い](./vue.md#vue-no-v-for-template-key-on-child-bad)

<span id="vue-no-v-for-template-key-on-child-good"></span>

[良い](./vue.md#vue-no-v-for-template-key-on-child-good)

[カテゴリの例](./vue.md#vue-no-v-for-template-key-on-child) · [ルール参照](./reference/vue-no-v-for-template-key-on-child.md)

### `vue/no-v-html`

XSS を防ぐため、v-html を使用した箇所に警告を出します。

<span id="vue-no-v-html-bad"></span>

[悪い](./vue.md#vue-no-v-html-bad)

<span id="vue-no-v-html-good"></span>

[良い](./vue.md#vue-no-v-html-good)

[カテゴリの例](./vue.md#vue-no-v-html) · [ルール参照](./reference/vue-no-v-html.md)

### `vue/no-v-text-v-html-on-component`

コンポーネントでの v-text / v-html を検出します。

<span id="vue-no-v-text-v-html-on-component-bad"></span>

[悪い](./vue.md#vue-no-v-text-v-html-on-component-bad)

<span id="vue-no-v-text-v-html-on-component-good"></span>

[良い](./vue.md#vue-no-v-text-v-html-on-component-good)

[カテゴリの例](./vue.md#vue-no-v-text-v-html-on-component) · [ルール参照](./reference/vue-no-v-text-v-html-on-component.md)

### `vue/permitted-contents`

HTML の要素ごとのコンテンツモデルを検査します。

<span id="vue-permitted-contents-bad"></span>

[悪い](./vue.md#vue-permitted-contents-bad)

<span id="vue-permitted-contents-good"></span>

[良い](./vue.md#vue-permitted-contents-good)

[カテゴリの例](./vue.md#vue-permitted-contents) · [ルール参照](./reference/vue-permitted-contents.md)

### `vue/require-component-is`

動的 component 要素に :is を指定します。

<span id="vue-require-component-is-bad"></span>

[悪い](./vue.md#vue-require-component-is-bad)

<span id="vue-require-component-is-good"></span>

[良い](./vue.md#vue-require-component-is-good)

[カテゴリの例](./vue.md#vue-require-component-is) · [ルール参照](./reference/vue-require-component-is.md)

### `vue/require-toggle-inside-transition`

transition の子要素に表示を切り替える条件を指定します。

<span id="vue-require-toggle-inside-transition-bad"></span>

[悪い](./vue.md#vue-require-toggle-inside-transition-bad)

<span id="vue-require-toggle-inside-transition-good"></span>

[良い](./vue.md#vue-require-toggle-inside-transition-good)

[カテゴリの例](./vue.md#vue-require-toggle-inside-transition) · [ルール参照](./reference/vue-require-toggle-inside-transition.md)

### `vue/require-v-for-key`

v-for に安定した :key を指定します。

<span id="vue-require-v-for-key-bad"></span>

[悪い](./vue.md#vue-require-v-for-key-bad)

<span id="vue-require-v-for-key-good"></span>

[良い](./vue.md#vue-require-v-for-key-good)

[カテゴリの例](./vue.md#vue-require-v-for-key) · [ルール参照](./reference/vue-require-v-for-key.md)

### `vue/use-v-on-exact`

修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。

<span id="vue-use-v-on-exact-bad"></span>

[悪い](./vue.md#vue-use-v-on-exact-bad)

<span id="vue-use-v-on-exact-good"></span>

[良い](./vue.md#vue-use-v-on-exact-good)

[カテゴリの例](./vue.md#vue-use-v-on-exact) · [ルール参照](./reference/vue-use-v-on-exact.md)

### `vue/valid-attribute-name`

有効な属性名を指定します。

<span id="vue-valid-attribute-name-bad"></span>

[悪い](./vue.md#vue-valid-attribute-name-bad)

<span id="vue-valid-attribute-name-good"></span>

[良い](./vue.md#vue-valid-attribute-name-good)

[カテゴリの例](./vue.md#vue-valid-attribute-name) · [ルール参照](./reference/vue-valid-attribute-name.md)

### `vue/valid-template-root`

Vue 3 のフラグメントに対応する有効なテンプレートルートを検査します。

<span id="vue-valid-template-root-bad"></span>

[悪い](./vue.md#vue-valid-template-root-bad)

<span id="vue-valid-template-root-good"></span>

[良い](./vue.md#vue-valid-template-root-good)

[カテゴリの例](./vue.md#vue-valid-template-root) · [ルール参照](./reference/vue-valid-template-root.md)

### `vue/valid-v-bind`

v-bind の引数・値・修飾子を検査します。

<span id="vue-valid-v-bind-bad"></span>

[悪い](./vue.md#vue-valid-v-bind-bad)

<span id="vue-valid-v-bind-good"></span>

[良い](./vue.md#vue-valid-v-bind-good)

[カテゴリの例](./vue.md#vue-valid-v-bind) · [ルール参照](./reference/vue-valid-v-bind.md)

### `vue/valid-v-cloak`

v-cloak の引数・値・修飾子を検査します。

<span id="vue-valid-v-cloak-bad"></span>

[悪い](./vue.md#vue-valid-v-cloak-bad)

<span id="vue-valid-v-cloak-good"></span>

[良い](./vue.md#vue-valid-v-cloak-good)

[カテゴリの例](./vue.md#vue-valid-v-cloak) · [ルール参照](./reference/vue-valid-v-cloak.md)

### `vue/valid-v-else`

v-else の位置・引数・値を検査します。

<span id="vue-valid-v-else-bad"></span>

[悪い](./vue.md#vue-valid-v-else-bad)

<span id="vue-valid-v-else-good"></span>

[良い](./vue.md#vue-valid-v-else-good)

[カテゴリの例](./vue.md#vue-valid-v-else) · [ルール参照](./reference/vue-valid-v-else.md)

### `vue/valid-v-else-if`

v-else-if の位置・引数・値を検査します。

<span id="vue-valid-v-else-if-bad"></span>

[悪い](./vue.md#vue-valid-v-else-if-bad)

<span id="vue-valid-v-else-if-good"></span>

[良い](./vue.md#vue-valid-v-else-if-good)

[カテゴリの例](./vue.md#vue-valid-v-else-if) · [ルール参照](./reference/vue-valid-v-else-if.md)

### `vue/valid-v-for`

v-for の式と変数宣言を検査します。

<span id="vue-valid-v-for-bad"></span>

[悪い](./vue.md#vue-valid-v-for-bad)

<span id="vue-valid-v-for-good"></span>

[良い](./vue.md#vue-valid-v-for-good)

[カテゴリの例](./vue.md#vue-valid-v-for) · [ルール参照](./reference/vue-valid-v-for.md)

### `vue/valid-v-html`

v-html の値・引数・修飾子を検査します。

<span id="vue-valid-v-html-bad"></span>

[悪い](./vue.md#vue-valid-v-html-bad)

<span id="vue-valid-v-html-good"></span>

[良い](./vue.md#vue-valid-v-html-good)

[カテゴリの例](./vue.md#vue-valid-v-html) · [ルール参照](./reference/vue-valid-v-html.md)

### `vue/valid-v-if`

v-if に有効な条件式を指定します。

<span id="vue-valid-v-if-bad"></span>

[悪い](./vue.md#vue-valid-v-if-bad)

<span id="vue-valid-v-if-good"></span>

[良い](./vue.md#vue-valid-v-if-good)

[カテゴリの例](./vue.md#vue-valid-v-if) · [ルール参照](./reference/vue-valid-v-if.md)

### `vue/valid-v-is`

v-is の適用先・引数・値を検査します。

<span id="vue-valid-v-is-bad"></span>

[悪い](./vue.md#vue-valid-v-is-bad)

<span id="vue-valid-v-is-good"></span>

[良い](./vue.md#vue-valid-v-is-good)

[カテゴリの例](./vue.md#vue-valid-v-is) · [ルール参照](./reference/vue-valid-v-is.md)

### `vue/valid-v-memo`

v-memo の値を配列の式にします。

<span id="vue-valid-v-memo-bad"></span>

[悪い](./vue.md#vue-valid-v-memo-bad)

<span id="vue-valid-v-memo-good"></span>

[良い](./vue.md#vue-valid-v-memo-good)

[カテゴリの例](./vue.md#vue-valid-v-memo) · [ルール参照](./reference/vue-valid-v-memo.md)

### `vue/valid-v-model`

v-model の値・引数・修飾子を検査します。

<span id="vue-valid-v-model-bad"></span>

[悪い](./vue.md#vue-valid-v-model-bad)

<span id="vue-valid-v-model-good"></span>

[良い](./vue.md#vue-valid-v-model-good)

[カテゴリの例](./vue.md#vue-valid-v-model) · [ルール参照](./reference/vue-valid-v-model.md)

### `vue/valid-v-on`

v-on のイベント名・式・修飾子を検査します。

<span id="vue-valid-v-on-bad"></span>

[悪い](./vue.md#vue-valid-v-on-bad)

<span id="vue-valid-v-on-good"></span>

[良い](./vue.md#vue-valid-v-on-good)

[カテゴリの例](./vue.md#vue-valid-v-on) · [ルール参照](./reference/vue-valid-v-on.md)

### `vue/valid-v-once`

v-once の引数・値・修飾子を検査します。

<span id="vue-valid-v-once-bad"></span>

[悪い](./vue.md#vue-valid-v-once-bad)

<span id="vue-valid-v-once-good"></span>

[良い](./vue.md#vue-valid-v-once-good)

[カテゴリの例](./vue.md#vue-valid-v-once) · [ルール参照](./reference/vue-valid-v-once.md)

### `vue/valid-v-pre`

v-pre の引数・値・修飾子を検査します。

<span id="vue-valid-v-pre-bad"></span>

[悪い](./vue.md#vue-valid-v-pre-bad)

<span id="vue-valid-v-pre-good"></span>

[良い](./vue.md#vue-valid-v-pre-good)

[カテゴリの例](./vue.md#vue-valid-v-pre) · [ルール参照](./reference/vue-valid-v-pre.md)

### `vue/valid-v-show`

v-show に有効な条件式を指定します。

<span id="vue-valid-v-show-bad"></span>

[悪い](./vue.md#vue-valid-v-show-bad)

<span id="vue-valid-v-show-good"></span>

[良い](./vue.md#vue-valid-v-show-good)

[カテゴリの例](./vue.md#vue-valid-v-show) · [ルール参照](./reference/vue-valid-v-show.md)

### `vue/valid-v-slot`

v-slot の適用先・宣言・修飾子を検査します。

<span id="vue-valid-v-slot-bad"></span>

[悪い](./vue.md#vue-valid-v-slot-bad)

<span id="vue-valid-v-slot-good"></span>

[良い](./vue.md#vue-valid-v-slot-good)

[カテゴリの例](./vue.md#vue-valid-v-slot) · [ルール参照](./reference/vue-valid-v-slot.md)

### `vue/valid-v-text`

v-text の値・引数・修飾子を検査します。

<span id="vue-valid-v-text-bad"></span>

[悪い](./vue.md#vue-valid-v-text-bad)

<span id="vue-valid-v-text-good"></span>

[良い](./vue.md#vue-valid-v-text-good)

[カテゴリの例](./vue.md#vue-valid-v-text) · [ルール参照](./reference/vue-valid-v-text.md)

### `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

<span id="vue-attribute-hyphenation-bad"></span>

[悪い](./vue.md#vue-attribute-hyphenation-bad)

<span id="vue-attribute-hyphenation-good"></span>

[良い](./vue.md#vue-attribute-hyphenation-good)

[カテゴリの例](./vue.md#vue-attribute-hyphenation) · [ルール参照](./reference/vue-attribute-hyphenation.md)

### `vue/component-definition-name-casing`

コンポーネント定義名を PascalCase または kebab-case に揃えます。

<span id="vue-component-definition-name-casing-bad"></span>

[悪い](./vue.md#vue-component-definition-name-casing-bad)

<span id="vue-component-definition-name-casing-good"></span>

[良い](./vue.md#vue-component-definition-name-casing-good)

[カテゴリの例](./vue.md#vue-component-definition-name-casing) · [ルール参照](./reference/vue-component-definition-name-casing.md)

### `vue/html-quotes`

HTML 属性値の引用符を揃えます。

<span id="vue-html-quotes-bad"></span>

[悪い](./vue.md#vue-html-quotes-bad)

<span id="vue-html-quotes-good"></span>

[良い](./vue.md#vue-html-quotes-good)

[カテゴリの例](./vue.md#vue-html-quotes) · [ルール参照](./reference/vue-html-quotes.md)

### `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

<span id="vue-html-self-closing-bad"></span>

[悪い](./vue.md#vue-html-self-closing-bad)

<span id="vue-html-self-closing-good"></span>

[良い](./vue.md#vue-html-self-closing-good)

[カテゴリの例](./vue.md#vue-html-self-closing) · [ルール参照](./reference/vue-html-self-closing.md)

### `vue/mustache-interpolation-spacing`

mustache 内の空白を揃えます。

<span id="vue-mustache-interpolation-spacing-bad"></span>

[悪い](./vue.md#vue-mustache-interpolation-spacing-bad)

<span id="vue-mustache-interpolation-spacing-good"></span>

[良い](./vue.md#vue-mustache-interpolation-spacing-good)

[カテゴリの例](./vue.md#vue-mustache-interpolation-spacing) · [ルール参照](./reference/vue-mustache-interpolation-spacing.md)

### `vue/no-multi-spaces`

連続する不要な空白を検出します。

<span id="vue-no-multi-spaces-bad"></span>

[悪い](./vue.md#vue-no-multi-spaces-bad)

<span id="vue-no-multi-spaces-good"></span>

[良い](./vue.md#vue-no-multi-spaces-good)

[カテゴリの例](./vue.md#vue-no-multi-spaces) · [ルール参照](./reference/vue-no-multi-spaces.md)

### `vue/no-template-shadow`

テンプレート変数が外側の名前を隠す箇所を検出します。

<span id="vue-no-template-shadow-bad"></span>

[悪い](./vue.md#vue-no-template-shadow-bad)

<span id="vue-no-template-shadow-good"></span>

[良い](./vue.md#vue-no-template-shadow-good)

[カテゴリの例](./vue.md#vue-no-template-shadow) · [ルール参照](./reference/vue-no-template-shadow.md)

### `vue/no-unused-properties`

defineProps に宣言しているのに使わない prop を検出します。

<span id="vue-no-unused-properties-bad"></span>

[悪い](./vue.md#vue-no-unused-properties-bad)

<span id="vue-no-unused-properties-good"></span>

[良い](./vue.md#vue-no-unused-properties-good)

[カテゴリの例](./vue.md#vue-no-unused-properties) · [ルール参照](./reference/vue-no-unused-properties.md)

### `vue/prop-name-casing`

宣言する prop 名の形式を揃えます。

<span id="vue-prop-name-casing-bad"></span>

[悪い](./vue.md#vue-prop-name-casing-bad)

<span id="vue-prop-name-casing-good"></span>

[良い](./vue.md#vue-prop-name-casing-good)

[カテゴリの例](./vue.md#vue-prop-name-casing) · [ルール参照](./reference/vue-prop-name-casing.md)

### `vue/v-bind-style`

v-bind の表記形式を揃えます。

<span id="vue-v-bind-style-bad"></span>

[悪い](./vue.md#vue-v-bind-style-bad)

<span id="vue-v-bind-style-good"></span>

[良い](./vue.md#vue-v-bind-style-good)

[カテゴリの例](./vue.md#vue-v-bind-style) · [ルール参照](./reference/vue-v-bind-style.md)

### `vue/v-on-style`

v-on の表記形式を揃えます。

<span id="vue-v-on-style-bad"></span>

[悪い](./vue.md#vue-v-on-style-bad)

<span id="vue-v-on-style-good"></span>

[良い](./vue.md#vue-v-on-style-good)

[カテゴリの例](./vue.md#vue-v-on-style) · [ルール参照](./reference/vue-v-on-style.md)

### `vue/v-slot-style`

v-slot の表記形式を揃えます。

<span id="vue-v-slot-style-bad"></span>

[悪い](./vue.md#vue-v-slot-style-bad)

<span id="vue-v-slot-style-good"></span>

[良い](./vue.md#vue-v-slot-style-good)

[カテゴリの例](./vue.md#vue-v-slot-style) · [ルール参照](./reference/vue-v-slot-style.md)

### `ssr/no-browser-globals-in-ssr`

SSR で実行されるコードのブラウザー専用グローバル参照を検出します。

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

[悪い](./ssr.md#ssr-no-browser-globals-in-ssr-bad)

<span id="ssr-no-browser-globals-in-ssr-good"></span>

[良い](./ssr.md#ssr-no-browser-globals-in-ssr-good)

[カテゴリの例](./ssr.md#ssr-no-browser-globals-in-ssr) · [ルール参照](./reference/ssr-no-browser-globals-in-ssr.md)

### `ssr/no-hydration-mismatch`

サーバーとクライアントで一致しないテンプレート値を検出します。

<span id="ssr-no-hydration-mismatch-bad"></span>

[悪い](./ssr.md#ssr-no-hydration-mismatch-bad)

<span id="ssr-no-hydration-mismatch-good"></span>

[良い](./ssr.md#ssr-no-hydration-mismatch-good)

[カテゴリの例](./ssr.md#ssr-no-hydration-mismatch) · [ルール参照](./reference/ssr-no-hydration-mismatch.md)

### `vue/a11y-img-alt`

画像に代替テキストの alt 属性を指定します。

<span id="vue-a11y-img-alt-bad"></span>

[悪い](./vue.md#vue-a11y-img-alt-bad)

<span id="vue-a11y-img-alt-good"></span>

[良い](./vue.md#vue-a11y-img-alt-good)

[カテゴリの例](./vue.md#vue-a11y-img-alt) · [ルール参照](./reference/vue-a11y-img-alt.md)

### `vue/attribute-order`

テンプレートの属性を一定の順に並べます。

<span id="vue-attribute-order-bad"></span>

[悪い](./vue.md#vue-attribute-order-bad)

<span id="vue-attribute-order-good"></span>

[良い](./vue.md#vue-attribute-order-good)

[カテゴリの例](./vue.md#vue-attribute-order) · [ルール参照](./reference/vue-attribute-order.md)

### `vue/component-name-in-template-casing`

テンプレート内のコンポーネント名を指定した形式に揃えます。

<span id="vue-component-name-in-template-casing-bad"></span>

[悪い](./vue.md#vue-component-name-in-template-casing-bad)

<span id="vue-component-name-in-template-casing-good"></span>

[良い](./vue.md#vue-component-name-in-template-casing-good)

[カテゴリの例](./vue.md#vue-component-name-in-template-casing) · [ルール参照](./reference/vue-component-name-in-template-casing.md)

### `vue/html-button-has-type`

button に有効な type を明示します。

<span id="vue-html-button-has-type-bad"></span>

[悪い](./vue.md#vue-html-button-has-type-bad)

<span id="vue-html-button-has-type-good"></span>

[良い](./vue.md#vue-html-button-has-type-good)

[カテゴリの例](./vue.md#vue-html-button-has-type) · [ルール参照](./reference/vue-html-button-has-type.md)

### `vue/max-template-complexity`

コンポーネント自身のテンプレートの複雑度を制限します。

<span id="vue-max-template-complexity-bad"></span>

[悪い](./vue.md#vue-max-template-complexity-bad)

<span id="vue-max-template-complexity-good"></span>

[良い](./vue.md#vue-max-template-complexity-good)

[カテゴリの例](./vue.md#vue-max-template-complexity) · [ルール参照](./reference/vue-max-template-complexity.md)

### `vue/no-array-index-key`

v-for の配列インデックスをそのまま key に使う箇所を検出します。

<span id="vue-no-array-index-key-bad"></span>

[悪い](./vue.md#vue-no-array-index-key-bad)

<span id="vue-no-array-index-key-good"></span>

[良い](./vue.md#vue-no-array-index-key-good)

[カテゴリの例](./vue.md#vue-no-array-index-key) · [ルール参照](./reference/vue-no-array-index-key.md)

### `vue/no-bare-strings-in-template`

テンプレートに直接書かれた、国際化の対象となるテキストを検出します。

<span id="vue-no-bare-strings-in-template-bad"></span>

[悪い](./vue.md#vue-no-bare-strings-in-template-bad)

<span id="vue-no-bare-strings-in-template-good"></span>

[良い](./vue.md#vue-no-bare-strings-in-template-good)

[カテゴリの例](./vue.md#vue-no-bare-strings-in-template) · [ルール参照](./reference/vue-no-bare-strings-in-template.md)

### `vue/no-boolean-attr-value`

HTML の真偽属性に不要な値を指定した箇所を検出します。

<span id="vue-no-boolean-attr-value-bad"></span>

[悪い](./vue.md#vue-no-boolean-attr-value-bad)

<span id="vue-no-boolean-attr-value-good"></span>

[良い](./vue.md#vue-no-boolean-attr-value-good)

[カテゴリの例](./vue.md#vue-no-boolean-attr-value) · [ルール参照](./reference/vue-no-boolean-attr-value.md)

### `vue/no-empty-component-block`

空の SFC ブロックを検出します。

<span id="vue-no-empty-component-block-bad"></span>

[悪い](./vue.md#vue-no-empty-component-block-bad)

<span id="vue-no-empty-component-block-good"></span>

[良い](./vue.md#vue-no-empty-component-block-good)

[カテゴリの例](./vue.md#vue-no-empty-component-block) · [ルール参照](./reference/vue-no-empty-component-block.md)

### `vue/no-inline-style`

インラインの style 属性を検出します。

<span id="vue-no-inline-style-bad"></span>

[悪い](./vue.md#vue-no-inline-style-bad)

<span id="vue-no-inline-style-good"></span>

[良い](./vue.md#vue-no-inline-style-good)

[カテゴリの例](./vue.md#vue-no-inline-style) · [ルール参照](./reference/vue-no-inline-style.md)

### `vue/no-invalid-html-attribute`

静的 HTML 属性の無効な値を検出します。現在は rel が対象です。

<span id="vue-no-invalid-html-attribute-bad"></span>

[悪い](./vue.md#vue-no-invalid-html-attribute-bad)

<span id="vue-no-invalid-html-attribute-good"></span>

[良い](./vue.md#vue-no-invalid-html-attribute-good)

[カテゴリの例](./vue.md#vue-no-invalid-html-attribute) · [ルール参照](./reference/vue-no-invalid-html-attribute.md)

### `vue/no-lone-template`

不要な template 要素を検出します。

<span id="vue-no-lone-template-bad"></span>

[悪い](./vue.md#vue-no-lone-template-bad)

<span id="vue-no-lone-template-good"></span>

[良い](./vue.md#vue-no-lone-template-good)

[カテゴリの例](./vue.md#vue-no-lone-template) · [ルール参照](./reference/vue-no-lone-template.md)

### `vue/no-multiple-objects-in-class`

:class 配列内の複数のオブジェクト指定をまとめます。

<span id="vue-no-multiple-objects-in-class-bad"></span>

[悪い](./vue.md#vue-no-multiple-objects-in-class-bad)

<span id="vue-no-multiple-objects-in-class-good"></span>

[良い](./vue.md#vue-no-multiple-objects-in-class-good)

[カテゴリの例](./vue.md#vue-no-multiple-objects-in-class) · [ルール参照](./reference/vue-no-multiple-objects-in-class.md)

### `vue/no-negated-v-if-condition`

v-else がある条件分岐の否定条件を反転して読みやすくします。

<span id="vue-no-negated-v-if-condition-bad"></span>

[悪い](./vue.md#vue-no-negated-v-if-condition-bad)

<span id="vue-no-negated-v-if-condition-good"></span>

[良い](./vue.md#vue-no-negated-v-if-condition-good)

[カテゴリの例](./vue.md#vue-no-negated-v-if-condition) · [ルール参照](./reference/vue-no-negated-v-if-condition.md)

### `vue/no-non-component-keep-alive-child`

KeepAlive の直下に通常の HTML 要素を置く箇所を検出します。

<span id="vue-no-non-component-keep-alive-child-bad"></span>

[悪い](./vue.md#vue-no-non-component-keep-alive-child-bad)

<span id="vue-no-non-component-keep-alive-child-good"></span>

[良い](./vue.md#vue-no-non-component-keep-alive-child-good)

[カテゴリの例](./vue.md#vue-no-non-component-keep-alive-child) · [ルール参照](./reference/vue-no-non-component-keep-alive-child.md)

### `vue/no-preprocessor-lang`

CSS プリプロセッサーより標準の CSS を使う方針を示します。

<span id="vue-no-preprocessor-lang-bad"></span>

[悪い](./vue.md#vue-no-preprocessor-lang-bad)

<span id="vue-no-preprocessor-lang-good"></span>

[良い](./vue.md#vue-no-preprocessor-lang-good)

[カテゴリの例](./vue.md#vue-no-preprocessor-lang) · [ルール参照](./reference/vue-no-preprocessor-lang.md)

### `vue/no-root-v-if`

テンプレートの単一ルートに v-if を指定する箇所を検出します。

<span id="vue-no-root-v-if-bad"></span>

[悪い](./vue.md#vue-no-root-v-if-bad)

<span id="vue-no-root-v-if-good"></span>

[良い](./vue.md#vue-no-root-v-if-good)

[カテゴリの例](./vue.md#vue-no-root-v-if) · [ルール参照](./reference/vue-no-root-v-if.md)

### `vue/no-script-non-standard-lang`

script の非標準の lang 指定を避ける方針を示します。

<span id="vue-no-script-non-standard-lang-bad"></span>

[悪い](./vue.md#vue-no-script-non-standard-lang-bad)

<span id="vue-no-script-non-standard-lang-good"></span>

[良い](./vue.md#vue-no-script-non-standard-lang-good)

[カテゴリの例](./vue.md#vue-no-script-non-standard-lang) · [ルール参照](./reference/vue-no-script-non-standard-lang.md)

### `vue/no-src-attribute`

SFC ブロックの外部 src 指定を検出します。

<span id="vue-no-src-attribute-bad"></span>

[悪い](./vue.md#vue-no-src-attribute-bad)

<span id="vue-no-src-attribute-good"></span>

[良い](./vue.md#vue-no-src-attribute-good)

[カテゴリの例](./vue.md#vue-no-src-attribute) · [ルール参照](./reference/vue-no-src-attribute.md)

### `vue/no-static-inline-styles`

静的なインライン style 属性を検出します。

<span id="vue-no-static-inline-styles-bad"></span>

[悪い](./vue.md#vue-no-static-inline-styles-bad)

<span id="vue-no-static-inline-styles-good"></span>

[良い](./vue.md#vue-no-static-inline-styles-good)

[カテゴリの例](./vue.md#vue-no-static-inline-styles) · [ルール参照](./reference/vue-no-static-inline-styles.md)

### `vue/no-template-lang`

template の lang 指定を避ける方針を示します。

<span id="vue-no-template-lang-bad"></span>

[悪い](./vue.md#vue-no-template-lang-bad)

<span id="vue-no-template-lang-good"></span>

[良い](./vue.md#vue-no-template-lang-good)

[カテゴリの例](./vue.md#vue-no-template-lang) · [ルール参照](./reference/vue-no-template-lang.md)

### `vue/no-template-target-blank`

target=_blank の外部リンクに適切な rel を指定します。

<span id="vue-no-template-target-blank-bad"></span>

[悪い](./vue.md#vue-no-template-target-blank-bad)

<span id="vue-no-template-target-blank-good"></span>

[良い](./vue.md#vue-no-template-target-blank-good)

[カテゴリの例](./vue.md#vue-no-template-target-blank) · [ルール参照](./reference/vue-no-template-target-blank.md)

### `vue/no-undefined-refs`

テンプレート内の未定義変数参照を検出します。

<span id="vue-no-undefined-refs-bad"></span>

[悪い](./vue.md#vue-no-undefined-refs-bad)

<span id="vue-no-undefined-refs-good"></span>

[良い](./vue.md#vue-no-undefined-refs-good)

[カテゴリの例](./vue.md#vue-no-undefined-refs) · [ルール参照](./reference/vue-no-undefined-refs.md)

### `vue/no-unsafe-url`

危険なスキームを含む可能性がある URL 属性やバインディングを検出します。

<span id="vue-no-unsafe-url-bad"></span>

[悪い](./vue.md#vue-no-unsafe-url-bad)

<span id="vue-no-unsafe-url-good"></span>

[良い](./vue.md#vue-no-unsafe-url-good)

[カテゴリの例](./vue.md#vue-no-unsafe-url) · [ルール参照](./reference/vue-no-unsafe-url.md)

### `vue/no-unsandboxed-iframe`

iframe に sandbox 属性を指定します。

<span id="vue-no-unsandboxed-iframe-bad"></span>

[悪い](./vue.md#vue-no-unsandboxed-iframe-bad)

<span id="vue-no-unsandboxed-iframe-good"></span>

[良い](./vue.md#vue-no-unsandboxed-iframe-good)

[カテゴリの例](./vue.md#vue-no-unsandboxed-iframe) · [ルール参照](./reference/vue-no-unsandboxed-iframe.md)

### `vue/no-unused-refs`

テンプレートに宣言しているのに参照しない ref を検出します。

<span id="vue-no-unused-refs-bad"></span>

[悪い](./vue.md#vue-no-unused-refs-bad)

<span id="vue-no-unused-refs-good"></span>

[良い](./vue.md#vue-no-unused-refs-good)

[カテゴリの例](./vue.md#vue-no-unused-refs) · [ルール参照](./reference/vue-no-unused-refs.md)

### `vue/no-unused-setup-bindings`

script setup に宣言しているのに読み取らない変数を検出します。

<span id="vue-no-unused-setup-bindings-bad"></span>

[悪い](./vue.md#vue-no-unused-setup-bindings-bad)

<span id="vue-no-unused-setup-bindings-good"></span>

[良い](./vue.md#vue-no-unused-setup-bindings-good)

[カテゴリの例](./vue.md#vue-no-unused-setup-bindings) · [ルール参照](./reference/vue-no-unused-setup-bindings.md)

### `vue/no-use-v-else-with-v-for`

同じ要素での v-else / v-else-if と v-for の併用を検出します。

<span id="vue-no-use-v-else-with-v-for-bad"></span>

[悪い](./vue.md#vue-no-use-v-else-with-v-for-bad)

<span id="vue-no-use-v-else-with-v-for-good"></span>

[良い](./vue.md#vue-no-use-v-else-with-v-for-good)

[カテゴリの例](./vue.md#vue-no-use-v-else-with-v-for) · [ルール参照](./reference/vue-no-use-v-else-with-v-for.md)

### `vue/no-useless-mustaches`

文字列リテラルだけの不要な mustache を検出します。

<span id="vue-no-useless-mustaches-bad"></span>

[悪い](./vue.md#vue-no-useless-mustaches-bad)

<span id="vue-no-useless-mustaches-good"></span>

[良い](./vue.md#vue-no-useless-mustaches-good)

[カテゴリの例](./vue.md#vue-no-useless-mustaches) · [ルール参照](./reference/vue-no-useless-mustaches.md)

### `vue/no-useless-v-bind`

文字列リテラルだけの不要な v-bind を検出します。

<span id="vue-no-useless-v-bind-bad"></span>

[悪い](./vue.md#vue-no-useless-v-bind-bad)

<span id="vue-no-useless-v-bind-good"></span>

[良い](./vue.md#vue-no-useless-v-bind-good)

[カテゴリの例](./vue.md#vue-no-useless-v-bind) · [ルール参照](./reference/vue-no-useless-v-bind.md)

### `vue/no-v-text`

v-text の代わりに mustache を使う方針を適用します。

<span id="vue-no-v-text-bad"></span>

[悪い](./vue.md#vue-no-v-text-bad)

<span id="vue-no-v-text-good"></span>

[良い](./vue.md#vue-no-v-text-good)

[カテゴリの例](./vue.md#vue-no-v-text) · [ルール参照](./reference/vue-no-v-text.md)

### `vue/prefer-props-shorthand`

Vue 3.4 の同名 prop バインディングの省略形を使います。

<span id="vue-prefer-props-shorthand-bad"></span>

[悪い](./vue.md#vue-prefer-props-shorthand-bad)

<span id="vue-prefer-props-shorthand-good"></span>

[良い](./vue.md#vue-prefer-props-shorthand-good)

[カテゴリの例](./vue.md#vue-prefer-props-shorthand) · [ルール参照](./reference/vue-prefer-props-shorthand.md)

### `vue/prefer-true-attribute-shorthand`

true を指定するバインディングを真偽属性の省略形にします。

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

[悪い](./vue.md#vue-prefer-true-attribute-shorthand-bad)

<span id="vue-prefer-true-attribute-shorthand-good"></span>

[良い](./vue.md#vue-prefer-true-attribute-shorthand-good)

[カテゴリの例](./vue.md#vue-prefer-true-attribute-shorthand) · [ルール参照](./reference/vue-prefer-true-attribute-shorthand.md)

### `vue/require-component-registration`

使用するコンポーネントを import または登録します。

<span id="vue-require-component-registration-bad"></span>

[悪い](./vue.md#vue-require-component-registration-bad)

<span id="vue-require-component-registration-good"></span>

[良い](./vue.md#vue-require-component-registration-good)

[カテゴリの例](./vue.md#vue-require-component-registration) · [ルール参照](./reference/vue-require-component-registration.md)

### `vue/require-scoped-style`

style に scoped を指定する方針を適用します。

<span id="vue-require-scoped-style-bad"></span>

[悪い](./vue.md#vue-require-scoped-style-bad)

<span id="vue-require-scoped-style-good"></span>

[良い](./vue.md#vue-require-scoped-style-good)

[カテゴリの例](./vue.md#vue-require-scoped-style) · [ルール参照](./reference/vue-require-scoped-style.md)

### `vue/scoped-event-names`

イベント名を context:event の形式に揃えます。

<span id="vue-scoped-event-names-bad"></span>

[悪い](./vue.md#vue-scoped-event-names-bad)

<span id="vue-scoped-event-names-good"></span>

[良い](./vue.md#vue-scoped-event-names-good)

[カテゴリの例](./vue.md#vue-scoped-event-names) · [ルール参照](./reference/vue-scoped-event-names.md)

### `vue/sfc-element-order`

SFC のトップレベルブロックを設定した順に並べます。

<span id="vue-sfc-element-order-bad"></span>

[悪い](./vue.md#vue-sfc-element-order-bad)

<span id="vue-sfc-element-order-good"></span>

[良い](./vue.md#vue-sfc-element-order-good)

[カテゴリの例](./vue.md#vue-sfc-element-order) · [ルール参照](./reference/vue-sfc-element-order.md)

### `vue/single-style-block`

SFC の style を一つのブロックにまとめます。

<span id="vue-single-style-block-bad"></span>

[悪い](./vue.md#vue-single-style-block-bad)

<span id="vue-single-style-block-good"></span>

[良い](./vue.md#vue-single-style-block-good)

[カテゴリの例](./vue.md#vue-single-style-block) · [ルール参照](./reference/vue-single-style-block.md)

### `vue/slot-name-casing`

名前付きスロットを kebab-case に揃えます。

<span id="vue-slot-name-casing-bad"></span>

[悪い](./vue.md#vue-slot-name-casing-bad)

<span id="vue-slot-name-casing-good"></span>

[良い](./vue.md#vue-slot-name-casing-good)

[カテゴリの例](./vue.md#vue-slot-name-casing) · [ルール参照](./reference/vue-slot-name-casing.md)

### `vue/this-in-template`

テンプレートで不要な this. 参照を検出します。

<span id="vue-this-in-template-bad"></span>

[悪い](./vue.md#vue-this-in-template-bad)

<span id="vue-this-in-template-good"></span>

[良い](./vue.md#vue-this-in-template-good)

[カテゴリの例](./vue.md#vue-this-in-template) · [ルール参照](./reference/vue-this-in-template.md)

### `vue/v-on-event-hyphenation`

コンポーネントのカスタムイベント名を設定した形式に揃えます。

<span id="vue-v-on-event-hyphenation-bad"></span>

[悪い](./vue.md#vue-v-on-event-hyphenation-bad)

<span id="vue-v-on-event-hyphenation-good"></span>

[良い](./vue.md#vue-v-on-event-hyphenation-good)

[カテゴリの例](./vue.md#vue-v-on-event-hyphenation) · [ルール参照](./reference/vue-v-on-event-hyphenation.md)

### `vue/v-on-handler-style`

イベントハンドラーの書き方を参照または関数の形式に揃えます。

<span id="vue-v-on-handler-style-bad"></span>

[悪い](./vue.md#vue-v-on-handler-style-bad)

<span id="vue-v-on-handler-style-good"></span>

[良い](./vue.md#vue-v-on-handler-style-good)

[カテゴリの例](./vue.md#vue-v-on-handler-style) · [ルール参照](./reference/vue-v-on-handler-style.md)

### `vue/warn-custom-block`

SFC のカスタムブロックを検出します。

<span id="vue-warn-custom-block-bad"></span>

[悪い](./vue.md#vue-warn-custom-block-bad)

<span id="vue-warn-custom-block-good"></span>

[良い](./vue.md#vue-warn-custom-block-good)

[カテゴリの例](./vue.md#vue-warn-custom-block) · [ルール参照](./reference/vue-warn-custom-block.md)

### `vue/warn-custom-directive`

登録が必要なカスタムディレクティブを検出します。

<span id="vue-warn-custom-directive-bad"></span>

[悪い](./vue.md#vue-warn-custom-directive-bad)

<span id="vue-warn-custom-directive-good"></span>

[良い](./vue.md#vue-warn-custom-directive-good)

[カテゴリの例](./vue.md#vue-warn-custom-directive) · [ルール参照](./reference/vue-warn-custom-directive.md)

### `a11y/alt-text`

画像などのメディアに代替テキストを用意します。

<span id="a11y-alt-text-bad"></span>

[悪い](./accessibility.md#a11y-alt-text-bad)

<span id="a11y-alt-text-good"></span>

[良い](./accessibility.md#a11y-alt-text-good)

[カテゴリの例](./accessibility.md#a11y-alt-text) · [ルール参照](./reference/a11y-alt-text.md)

### `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

<span id="a11y-anchor-has-content-bad"></span>

[悪い](./accessibility.md#a11y-anchor-has-content-bad)

<span id="a11y-anchor-has-content-good"></span>

[良い](./accessibility.md#a11y-anchor-has-content-good)

[カテゴリの例](./accessibility.md#a11y-anchor-has-content) · [ルール参照](./reference/a11y-anchor-has-content.md)

### `a11y/anchor-is-valid`

リンクの href に有効な移動先を指定します。

<span id="a11y-anchor-is-valid-bad"></span>

[悪い](./accessibility.md#a11y-anchor-is-valid-bad)

<span id="a11y-anchor-is-valid-good"></span>

[良い](./accessibility.md#a11y-anchor-is-valid-good)

[カテゴリの例](./accessibility.md#a11y-anchor-is-valid) · [ルール参照](./reference/a11y-anchor-is-valid.md)

### `a11y/aria-props`

存在しない ARIA 属性を検出します。

<span id="a11y-aria-props-bad"></span>

[悪い](./accessibility.md#a11y-aria-props-bad)

<span id="a11y-aria-props-good"></span>

[良い](./accessibility.md#a11y-aria-props-good)

[カテゴリの例](./accessibility.md#a11y-aria-props) · [ルール参照](./reference/a11y-aria-props.md)

### `a11y/aria-role`

抽象ロールを除く、有効な ARIA role を指定します。

<span id="a11y-aria-role-bad"></span>

[悪い](./accessibility.md#a11y-aria-role-bad)

<span id="a11y-aria-role-good"></span>

[良い](./accessibility.md#a11y-aria-role-good)

[カテゴリの例](./accessibility.md#a11y-aria-role) · [ルール参照](./reference/a11y-aria-role.md)

### `a11y/aria-unsupported-elements`

ARIA 属性を使用できない要素への指定を検出します。

<span id="a11y-aria-unsupported-elements-bad"></span>

[悪い](./accessibility.md#a11y-aria-unsupported-elements-bad)

<span id="a11y-aria-unsupported-elements-good"></span>

[良い](./accessibility.md#a11y-aria-unsupported-elements-good)

[カテゴリの例](./accessibility.md#a11y-aria-unsupported-elements) · [ルール参照](./reference/a11y-aria-unsupported-elements.md)

### `a11y/click-events-have-key-events`

クリックで操作する要素にキーボード操作も用意します。

<span id="a11y-click-events-have-key-events-bad"></span>

[悪い](./accessibility.md#a11y-click-events-have-key-events-bad)

<span id="a11y-click-events-have-key-events-good"></span>

[良い](./accessibility.md#a11y-click-events-have-key-events-good)

[カテゴリの例](./accessibility.md#a11y-click-events-have-key-events) · [ルール参照](./reference/a11y-click-events-have-key-events.md)

### `a11y/form-control-has-label`

フォーム部品に関連付けられたラベルを用意します。

<span id="a11y-form-control-has-label-bad"></span>

[悪い](./accessibility.md#a11y-form-control-has-label-bad)

<span id="a11y-form-control-has-label-good"></span>

[良い](./accessibility.md#a11y-form-control-has-label-good)

[カテゴリの例](./accessibility.md#a11y-form-control-has-label) · [ルール参照](./reference/a11y-form-control-has-label.md)

### `a11y/heading-has-content`

見出しに支援技術で読める内容を用意します。

<span id="a11y-heading-has-content-bad"></span>

[悪い](./accessibility.md#a11y-heading-has-content-bad)

<span id="a11y-heading-has-content-good"></span>

[良い](./accessibility.md#a11y-heading-has-content-good)

[カテゴリの例](./accessibility.md#a11y-heading-has-content) · [ルール参照](./reference/a11y-heading-has-content.md)

### `a11y/heading-levels`

見出しの階層を飛ばした指定を検出します。

<span id="a11y-heading-levels-bad"></span>

[悪い](./accessibility.md#a11y-heading-levels-bad)

<span id="a11y-heading-levels-good"></span>

[良い](./accessibility.md#a11y-heading-levels-good)

[カテゴリの例](./accessibility.md#a11y-heading-levels) · [ルール参照](./reference/a11y-heading-levels.md)

### `a11y/iframe-has-title`

iframe に内容を説明する title を指定します。

<span id="a11y-iframe-has-title-bad"></span>

[悪い](./accessibility.md#a11y-iframe-has-title-bad)

<span id="a11y-iframe-has-title-good"></span>

[良い](./accessibility.md#a11y-iframe-has-title-good)

[カテゴリの例](./accessibility.md#a11y-iframe-has-title) · [ルール参照](./reference/a11y-iframe-has-title.md)

### `a11y/img-alt`

画像に alt 属性を指定します。装飾画像は空の alt を使います。

<span id="a11y-img-alt-bad"></span>

[悪い](./accessibility.md#a11y-img-alt-bad)

<span id="a11y-img-alt-good"></span>

[良い](./accessibility.md#a11y-img-alt-good)

[カテゴリの例](./accessibility.md#a11y-img-alt) · [ルール参照](./reference/a11y-img-alt.md)

### `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

<span id="a11y-interactive-supports-focus-bad"></span>

[悪い](./accessibility.md#a11y-interactive-supports-focus-bad)

<span id="a11y-interactive-supports-focus-good"></span>

[良い](./accessibility.md#a11y-interactive-supports-focus-good)

[カテゴリの例](./accessibility.md#a11y-interactive-supports-focus) · [ルール参照](./reference/a11y-interactive-supports-focus.md)

### `a11y/label-has-for`

label を対象のフォーム部品と関連付けます。

<span id="a11y-label-has-for-bad"></span>

[悪い](./accessibility.md#a11y-label-has-for-bad)

<span id="a11y-label-has-for-good"></span>

[良い](./accessibility.md#a11y-label-has-for-good)

[カテゴリの例](./accessibility.md#a11y-label-has-for) · [ルール参照](./reference/a11y-label-has-for.md)

### `a11y/landmark-roles`

ランドマーク role の配置と重複を検査します。

<span id="a11y-landmark-roles-bad"></span>

[悪い](./accessibility.md#a11y-landmark-roles-bad)

<span id="a11y-landmark-roles-good"></span>

[良い](./accessibility.md#a11y-landmark-roles-good)

[カテゴリの例](./accessibility.md#a11y-landmark-roles) · [ルール参照](./reference/a11y-landmark-roles.md)

### `a11y/media-has-caption`

音声・動画に字幕を用意します。

<span id="a11y-media-has-caption-bad"></span>

[悪い](./accessibility.md#a11y-media-has-caption-bad)

<span id="a11y-media-has-caption-good"></span>

[良い](./accessibility.md#a11y-media-has-caption-good)

[カテゴリの例](./accessibility.md#a11y-media-has-caption) · [ルール参照](./reference/a11y-media-has-caption.md)

### `a11y/mouse-events-have-key-events`

マウス操作と対応する focus / blur 操作を用意します。

<span id="a11y-mouse-events-have-key-events-bad"></span>

[悪い](./accessibility.md#a11y-mouse-events-have-key-events-bad)

<span id="a11y-mouse-events-have-key-events-good"></span>

[良い](./accessibility.md#a11y-mouse-events-have-key-events-good)

[カテゴリの例](./accessibility.md#a11y-mouse-events-have-key-events) · [ルール参照](./reference/a11y-mouse-events-have-key-events.md)

### `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

<span id="a11y-no-access-key-bad"></span>

[悪い](./accessibility.md#a11y-no-access-key-bad)

<span id="a11y-no-access-key-good"></span>

[良い](./accessibility.md#a11y-no-access-key-good)

[カテゴリの例](./accessibility.md#a11y-no-access-key) · [ルール参照](./reference/a11y-no-access-key.md)

### `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素を aria-hidden で支援技術から隠す指定を検出します。

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

[悪い](./accessibility.md#a11y-no-aria-hidden-on-focusable-bad)

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

[良い](./accessibility.md#a11y-no-aria-hidden-on-focusable-good)

[カテゴリの例](./accessibility.md#a11y-no-aria-hidden-on-focusable) · [ルール参照](./reference/a11y-no-aria-hidden-on-focusable.md)

### `a11y/no-autofocus`

意図せずフォーカスを移動させる autofocus を検出します。

<span id="a11y-no-autofocus-bad"></span>

[悪い](./accessibility.md#a11y-no-autofocus-bad)

<span id="a11y-no-autofocus-good"></span>

[良い](./accessibility.md#a11y-no-autofocus-good)

[カテゴリの例](./accessibility.md#a11y-no-autofocus) · [ルール参照](./reference/a11y-no-autofocus.md)

### `a11y/no-distracting-elements`

marquee や blink などの注意をそらす要素を検出します。

<span id="a11y-no-distracting-elements-bad"></span>

[悪い](./accessibility.md#a11y-no-distracting-elements-bad)

<span id="a11y-no-distracting-elements-good"></span>

[良い](./accessibility.md#a11y-no-distracting-elements-good)

[カテゴリの例](./accessibility.md#a11y-no-distracting-elements) · [ルール参照](./reference/a11y-no-distracting-elements.md)

### `a11y/no-i-for-icon`

アイコン用の i 要素を検出し、意味に合う要素を勧めます。

<span id="a11y-no-i-for-icon-bad"></span>

[悪い](./accessibility.md#a11y-no-i-for-icon-bad)

<span id="a11y-no-i-for-icon-good"></span>

[良い](./accessibility.md#a11y-no-i-for-icon-good)

[カテゴリの例](./accessibility.md#a11y-no-i-for-icon) · [ルール参照](./reference/a11y-no-i-for-icon.md)

### `a11y/no-redundant-roles`

要素本来の意味と重複する ARIA role を検出します。

<span id="a11y-no-redundant-roles-bad"></span>

[悪い](./accessibility.md#a11y-no-redundant-roles-bad)

<span id="a11y-no-redundant-roles-good"></span>

[良い](./accessibility.md#a11y-no-redundant-roles-good)

[カテゴリの例](./accessibility.md#a11y-no-redundant-roles) · [ルール参照](./reference/a11y-no-redundant-roles.md)

### `a11y/no-refer-to-non-existent-id`

文書内に存在しない ID への参照を検出します。

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

[悪い](./accessibility.md#a11y-no-refer-to-non-existent-id-bad)

<span id="a11y-no-refer-to-non-existent-id-good"></span>

[良い](./accessibility.md#a11y-no-refer-to-non-existent-id-good)

[カテゴリの例](./accessibility.md#a11y-no-refer-to-non-existent-id) · [ルール参照](./reference/a11y-no-refer-to-non-existent-id.md)

### `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

[悪い](./accessibility.md#a11y-no-role-presentation-on-focusable-bad)

<span id="a11y-no-role-presentation-on-focusable-good"></span>

[良い](./accessibility.md#a11y-no-role-presentation-on-focusable-good)

[カテゴリの例](./accessibility.md#a11y-no-role-presentation-on-focusable) · [ルール参照](./reference/a11y-no-role-presentation-on-focusable.md)

### `a11y/no-static-element-interactions`

操作部品ではない要素へのイベント指定を検出します。

<span id="a11y-no-static-element-interactions-bad"></span>

[悪い](./accessibility.md#a11y-no-static-element-interactions-bad)

<span id="a11y-no-static-element-interactions-good"></span>

[良い](./accessibility.md#a11y-no-static-element-interactions-good)

[カテゴリの例](./accessibility.md#a11y-no-static-element-interactions) · [ルール参照](./reference/a11y-no-static-element-interactions.md)

### `a11y/placeholder-label-option`

required の select を値が空のプレースホルダー option から始めます。

<span id="a11y-placeholder-label-option-bad"></span>

[悪い](./accessibility.md#a11y-placeholder-label-option-bad)

<span id="a11y-placeholder-label-option-good"></span>

[良い](./accessibility.md#a11y-placeholder-label-option-good)

[カテゴリの例](./accessibility.md#a11y-placeholder-label-option) · [ルール参照](./reference/a11y-placeholder-label-option.md)

### `a11y/role-has-required-aria-props`

ARIA role が必要とする属性を指定します。

<span id="a11y-role-has-required-aria-props-bad"></span>

[悪い](./accessibility.md#a11y-role-has-required-aria-props-bad)

<span id="a11y-role-has-required-aria-props-good"></span>

[良い](./accessibility.md#a11y-role-has-required-aria-props-good)

[カテゴリの例](./accessibility.md#a11y-role-has-required-aria-props) · [ルール参照](./reference/a11y-role-has-required-aria-props.md)

### `a11y/tabindex-no-positive`

通常のフォーカス順序を変える正の tabindex を検出します。

<span id="a11y-tabindex-no-positive-bad"></span>

[悪い](./accessibility.md#a11y-tabindex-no-positive-bad)

<span id="a11y-tabindex-no-positive-good"></span>

[良い](./accessibility.md#a11y-tabindex-no-positive-good)

[カテゴリの例](./accessibility.md#a11y-tabindex-no-positive) · [ルール参照](./reference/a11y-tabindex-no-positive.md)

### `a11y/use-list`

箇条書きに見えるテキストをリスト要素で表現します。

<span id="a11y-use-list-bad"></span>

[悪い](./accessibility.md#a11y-use-list-bad)

<span id="a11y-use-list-good"></span>

[良い](./accessibility.md#a11y-use-list-good)

[カテゴリの例](./accessibility.md#a11y-use-list) · [ルール参照](./reference/a11y-use-list.md)

### `vue/use-unique-element-ids`

静的 ID の代わりに useId() で一意な ID を生成します。

<span id="vue-use-unique-element-ids-bad"></span>

[悪い](./vue.md#vue-use-unique-element-ids-bad)

<span id="vue-use-unique-element-ids-good"></span>

[良い](./vue.md#vue-use-unique-element-ids-good)

[カテゴリの例](./vue.md#vue-use-unique-element-ids) · [ルール参照](./reference/vue-use-unique-element-ids.md)

### `html/deprecated-attr`

非推奨の HTML 属性を検出します。

<span id="html-deprecated-attr-bad"></span>

[悪い](./html.md#html-deprecated-attr-bad)

<span id="html-deprecated-attr-good"></span>

[良い](./html.md#html-deprecated-attr-good)

[カテゴリの例](./html.md#html-deprecated-attr) · [ルール参照](./reference/html-deprecated-attr.md)

### `html/deprecated-element`

非推奨の HTML 要素を検出します。

<span id="html-deprecated-element-bad"></span>

[悪い](./html.md#html-deprecated-element-bad)

<span id="html-deprecated-element-good"></span>

[良い](./html.md#html-deprecated-element-good)

[カテゴリの例](./html.md#html-deprecated-element) · [ルール参照](./reference/html-deprecated-element.md)

### `html/id-duplication`

同じテンプレート内の ID 重複を検出します。

<span id="html-id-duplication-bad"></span>

[悪い](./html.md#html-id-duplication-bad)

<span id="html-id-duplication-good"></span>

[良い](./html.md#html-id-duplication-good)

[カテゴリの例](./html.md#html-id-duplication) · [ルール参照](./reference/html-id-duplication.md)

### `html/no-consecutive-br`

連続する br 要素による余白指定を検出します。

<span id="html-no-consecutive-br-bad"></span>

[悪い](./html.md#html-no-consecutive-br-bad)

<span id="html-no-consecutive-br-good"></span>

[良い](./html.md#html-no-consecutive-br-good)

[カテゴリの例](./html.md#html-no-consecutive-br) · [ルール参照](./reference/html-no-consecutive-br.md)

### `html/no-dupe-style-properties`

静的 style 属性内のプロパティ重複を検出します。

<span id="html-no-dupe-style-properties-bad"></span>

[悪い](./html.md#html-no-dupe-style-properties-bad)

<span id="html-no-dupe-style-properties-good"></span>

[良い](./html.md#html-no-dupe-style-properties-good)

[カテゴリの例](./html.md#html-no-dupe-style-properties) · [ルール参照](./reference/html-no-dupe-style-properties.md)

### `html/no-duplicate-class`

静的 class 属性内のクラス名重複を検出します。

<span id="html-no-duplicate-class-bad"></span>

[悪い](./html.md#html-no-duplicate-class-bad)

<span id="html-no-duplicate-class-good"></span>

[良い](./html.md#html-no-duplicate-class-good)

[カテゴリの例](./html.md#html-no-duplicate-class) · [ルール参照](./reference/html-no-duplicate-class.md)

### `html/no-duplicate-dt`

dl 内の dt の名前重複を検出します。

<span id="html-no-duplicate-dt-bad"></span>

[悪い](./html.md#html-no-duplicate-dt-bad)

<span id="html-no-duplicate-dt-good"></span>

[良い](./html.md#html-no-duplicate-dt-good)

[カテゴリの例](./html.md#html-no-duplicate-dt) · [ルール参照](./reference/html-no-duplicate-dt.md)

### `html/no-empty-palpable-content`

可視コンテンツを期待する要素が空の場合に検出します。

<span id="html-no-empty-palpable-content-bad"></span>

[悪い](./html.md#html-no-empty-palpable-content-bad)

<span id="html-no-empty-palpable-content-good"></span>

[良い](./html.md#html-no-empty-palpable-content-good)

[カテゴリの例](./html.md#html-no-empty-palpable-content) · [ルール参照](./reference/html-no-empty-palpable-content.md)

### `html/require-datetime`

time 要素に機械可読の datetime を指定します。

<span id="html-require-datetime-bad"></span>

[悪い](./html.md#html-require-datetime-bad)

<span id="html-require-datetime-good"></span>

[良い](./html.md#html-require-datetime-good)

[カテゴリの例](./html.md#html-require-datetime) · [ルール参照](./reference/html-require-datetime.md)

### `type/no-floating-promises`

処理しないまま放置された Promise を検出します。

<span id="type-no-floating-promises-bad"></span>

[悪い](./type-and-script.md#type-no-floating-promises-bad)

<span id="type-no-floating-promises-good"></span>

[良い](./type-and-script.md#type-no-floating-promises-good)

[カテゴリの例](./type-and-script.md#type-no-floating-promises) · [ルール参照](./reference/type-no-floating-promises.md)

### `type/no-reactivity-loss`

リアクティブな値を代入や呼び出しで取り出し、元の更新に追従しないスナップショットになる箇所を検出します。

<span id="type-no-reactivity-loss-bad"></span>

[悪い](./type-and-script.md#type-no-reactivity-loss-bad)

<span id="type-no-reactivity-loss-good"></span>

[良い](./type-and-script.md#type-no-reactivity-loss-good)

[カテゴリの例](./type-and-script.md#type-no-reactivity-loss) · [ルール参照](./reference/type-no-reactivity-loss.md)

### `type/no-unsafe-template-binding`

テンプレートで安全でない型の値を使用する箇所を検出します。

<span id="type-no-unsafe-template-binding-bad"></span>

[悪い](./type-and-script.md#type-no-unsafe-template-binding-bad)

<span id="type-no-unsafe-template-binding-good"></span>

[良い](./type-and-script.md#type-no-unsafe-template-binding-good)

[カテゴリの例](./type-and-script.md#type-no-unsafe-template-binding) · [ルール参照](./reference/type-no-unsafe-template-binding.md)

### `type/require-typed-emits`

defineEmits に型定義を指定します。

<span id="type-require-typed-emits-bad"></span>

[悪い](./type-and-script.md#type-require-typed-emits-bad)

<span id="type-require-typed-emits-good"></span>

[良い](./type-and-script.md#type-require-typed-emits-good)

[カテゴリの例](./type-and-script.md#type-require-typed-emits) · [ルール参照](./reference/type-require-typed-emits.md)

### `type/require-typed-props`

defineProps に型定義を指定します。

<span id="type-require-typed-props-bad"></span>

[悪い](./type-and-script.md#type-require-typed-props-bad)

<span id="type-require-typed-props-good"></span>

[良い](./type-and-script.md#type-require-typed-props-good)

[カテゴリの例](./type-and-script.md#type-require-typed-props) · [ルール参照](./reference/type-require-typed-props.md)

### `type/strict-boolean-expressions`

script とテンプレートの条件式で安全な真偽判定を使います。

<span id="type-strict-boolean-expressions-bad"></span>

[悪い](./type-and-script.md#type-strict-boolean-expressions-bad)

<span id="type-strict-boolean-expressions-good"></span>

[良い](./type-and-script.md#type-strict-boolean-expressions-good)

[カテゴリの例](./type-and-script.md#type-strict-boolean-expressions) · [ルール参照](./reference/type-strict-boolean-expressions.md)

### `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

<span id="script-no-get-current-instance-bad"></span>

[悪い](./type-and-script.md#script-no-get-current-instance-bad)

<span id="script-no-get-current-instance-good"></span>

[良い](./type-and-script.md#script-no-get-current-instance-good)

[カテゴリの例](./type-and-script.md#script-no-get-current-instance) · [ルール参照](./reference/script-no-get-current-instance.md)

### `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

<span id="script-no-next-tick-bad"></span>

[悪い](./type-and-script.md#script-no-next-tick-bad)

<span id="script-no-next-tick-good"></span>

[良い](./type-and-script.md#script-no-next-tick-good)

[カテゴリの例](./type-and-script.md#script-no-next-tick) · [ルール参照](./reference/script-no-next-tick.md)

### `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

<span id="script-no-options-api-bad"></span>

[悪い](./type-and-script.md#script-no-options-api-bad)

<span id="script-no-options-api-good"></span>

[良い](./type-and-script.md#script-no-options-api-good)

[カテゴリの例](./type-and-script.md#script-no-options-api) · [ルール参照](./reference/script-no-options-api.md)

### `vapor/no-inline-template`

Vapor で削除済みの inline-template 属性を検出します。

<span id="vapor-no-inline-template-bad"></span>

[悪い](./vapor.md#vapor-no-inline-template-bad)

<span id="vapor-no-inline-template-good"></span>

[良い](./vapor.md#vapor-no-inline-template-good)

[カテゴリの例](./vapor.md#vapor-no-inline-template) · [ルール参照](./reference/vapor-no-inline-template.md)

### `vapor/no-vue-lifecycle-events`

Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。

<span id="vapor-no-vue-lifecycle-events-bad"></span>

[悪い](./vapor.md#vapor-no-vue-lifecycle-events-bad)

<span id="vapor-no-vue-lifecycle-events-good"></span>

[良い](./vapor.md#vapor-no-vue-lifecycle-events-good)

[カテゴリの例](./vapor.md#vapor-no-vue-lifecycle-events) · [ルール参照](./reference/vapor-no-vue-lifecycle-events.md)

### `vapor/prefer-static-class`

文字列リテラルの :class を静的 class に置き換えます。

<span id="vapor-prefer-static-class-bad"></span>

[悪い](./vapor.md#vapor-prefer-static-class-bad)

<span id="vapor-prefer-static-class-good"></span>

[良い](./vapor.md#vapor-prefer-static-class-good)

[カテゴリの例](./vapor.md#vapor-prefer-static-class) · [ルール参照](./reference/vapor-prefer-static-class.md)

### `vapor/require-vapor-attribute`

Vapor 向けの script setup に vapor 属性を付ける規約を示します。

<span id="vapor-require-vapor-attribute-bad"></span>

[悪い](./vapor.md#vapor-require-vapor-attribute-bad)

<span id="vapor-require-vapor-attribute-good"></span>

[良い](./vapor.md#vapor-require-vapor-attribute-good)

[カテゴリの例](./vapor.md#vapor-require-vapor-attribute) · [ルール参照](./reference/vapor-require-vapor-attribute.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Nuxt の内部リンクに NuxtLink を使います。

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

[悪い](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-bad)

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

[良い](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-good)

[カテゴリの例](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link) · [ルール参照](./reference/ecosystem-nuxt-prefer-nuxt-link.md)

### `ecosystem/pinia-prefer-store-to-refs`

Pinia store の分割代入に storeToRefs() を使います。

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

[悪い](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-bad)

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

[良い](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-good)

[カテゴリの例](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs) · [ルール参照](./reference/ecosystem-pinia-prefer-store-to-refs.md)

### `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

<span id="ecosystem-router-link-require-to-bad"></span>

[悪い](./ecosystem.md#ecosystem-router-link-require-to-bad)

<span id="ecosystem-router-link-require-to-good"></span>

[良い](./ecosystem.md#ecosystem-router-link-require-to-good)

[カテゴリの例](./ecosystem.md#ecosystem-router-link-require-to) · [ルール参照](./reference/ecosystem-router-link-require-to.md)

### `ecosystem/void-link-require-href`

Void Vue の Link に href を指定します。

<span id="ecosystem-void-link-require-href-bad"></span>

[悪い](./ecosystem.md#ecosystem-void-link-require-href-bad)

<span id="ecosystem-void-link-require-href-good"></span>

[良い](./ecosystem.md#ecosystem-void-link-require-href-good)

[カテゴリの例](./ecosystem.md#ecosystem-void-link-require-href) · [ルール参照](./reference/ecosystem-void-link-require-href.md)

### `ecosystem/void-link-valid-method`

Void Vue の Link に有効な静的 method を指定します。

<span id="ecosystem-void-link-valid-method-bad"></span>

[悪い](./ecosystem.md#ecosystem-void-link-valid-method-bad)

<span id="ecosystem-void-link-valid-method-good"></span>

[良い](./ecosystem.md#ecosystem-void-link-valid-method-good)

[カテゴリの例](./ecosystem.md#ecosystem-void-link-valid-method) · [ルール参照](./reference/ecosystem-void-link-valid-method.md)

### `ecosystem/vue-i18n-no-missing-key`

SFC 内の翻訳データに存在しない静的キーを検出します。

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

[悪い](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-bad)

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

[良い](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-good)

[カテゴリの例](./ecosystem.md#ecosystem-vue-i18n-no-missing-key) · [ルール参照](./reference/ecosystem-vue-i18n-no-missing-key.md)

### `ecosystem/vue-router-prefer-named-link`

RouterLink の文字列パスを名前付きルートの指定に置き換えます。

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

[悪い](./ecosystem.md#ecosystem-vue-router-prefer-named-link-bad)

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

[良い](./ecosystem.md#ecosystem-vue-router-prefer-named-link-good)

[カテゴリの例](./ecosystem.md#ecosystem-vue-router-prefer-named-link) · [ルール参照](./reference/ecosystem-vue-router-prefer-named-link.md)

### `ecosystem/vue-router-prefer-named-push`

Vue Router のプログラムによる移動に名前付きルートを使います。

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

[悪い](./ecosystem.md#ecosystem-vue-router-prefer-named-push-bad)

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

[良い](./ecosystem.md#ecosystem-vue-router-prefer-named-push-good)

[カテゴリの例](./ecosystem.md#ecosystem-vue-router-prefer-named-push) · [ルール参照](./reference/ecosystem-vue-router-prefer-named-push.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

wrapper.html() 全体のスナップショットに依存するテストを検出します。

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

[悪い](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-bad)

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

[良い](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-good)

[カテゴリの例](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot) · [ルール参照](./reference/ecosystem-vue-test-utils-no-html-snapshot.md)

### `css/no-display-none`

表示切り替えに display: none を使う箇所で v-show を検討します。

<span id="css-no-display-none-bad"></span>

[悪い](./musea-and-css.md#css-no-display-none-bad)

<span id="css-no-display-none-good"></span>

[良い](./musea-and-css.md#css-no-display-none-good)

[カテゴリの例](./musea-and-css.md#css-no-display-none) · [ルール参照](./reference/css-no-display-none.md)

### `css/no-hardcoded-values`

CSS に直接記述した値を CSS 変数にまとめます。

<span id="css-no-hardcoded-values-bad"></span>

[悪い](./musea-and-css.md#css-no-hardcoded-values-bad)

<span id="css-no-hardcoded-values-good"></span>

[良い](./musea-and-css.md#css-no-hardcoded-values-good)

[カテゴリの例](./musea-and-css.md#css-no-hardcoded-values) · [ルール参照](./reference/css-no-hardcoded-values.md)

### `css/no-id-selectors`

詳細度が高い CSS の ID セレクターを検出します。

<span id="css-no-id-selectors-bad"></span>

[悪い](./musea-and-css.md#css-no-id-selectors-bad)

<span id="css-no-id-selectors-good"></span>

[良い](./musea-and-css.md#css-no-id-selectors-good)

[カテゴリの例](./musea-and-css.md#css-no-id-selectors) · [ルール参照](./reference/css-no-id-selectors.md)

### `css/no-important`

通常のカスケードを上書きする !important を検出します。

<span id="css-no-important-bad"></span>

[悪い](./musea-and-css.md#css-no-important-bad)

<span id="css-no-important-good"></span>

[良い](./musea-and-css.md#css-no-important-good)

[カテゴリの例](./musea-and-css.md#css-no-important) · [ルール参照](./reference/css-no-important.md)

### `css/no-utility-classes`

コンポーネント内でユーティリティクラスを定義する箇所を検出します。

<span id="css-no-utility-classes-bad"></span>

[悪い](./musea-and-css.md#css-no-utility-classes-bad)

<span id="css-no-utility-classes-good"></span>

[良い](./musea-and-css.md#css-no-utility-classes-good)

[カテゴリの例](./musea-and-css.md#css-no-utility-classes) · [ルール参照](./reference/css-no-utility-classes.md)

### `css/no-v-bind-performance`

CSS v-bind() の実行時コストを検討するための警告です。

<span id="css-no-v-bind-performance-bad"></span>

[悪い](./musea-and-css.md#css-no-v-bind-performance-bad)

<span id="css-no-v-bind-performance-good"></span>

[良い](./musea-and-css.md#css-no-v-bind-performance-good)

[カテゴリの例](./musea-and-css.md#css-no-v-bind-performance) · [ルール参照](./reference/css-no-v-bind-performance.md)

### `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

<span id="css-prefer-logical-properties-bad"></span>

[悪い](./musea-and-css.md#css-prefer-logical-properties-bad)

<span id="css-prefer-logical-properties-good"></span>

[良い](./musea-and-css.md#css-prefer-logical-properties-good)

[カテゴリの例](./musea-and-css.md#css-prefer-logical-properties) · [ルール参照](./reference/css-prefer-logical-properties.md)

### `css/prefer-nested-selectors`

子孫セレクターを CSS のネスト構文でまとめます。

<span id="css-prefer-nested-selectors-bad"></span>

[悪い](./musea-and-css.md#css-prefer-nested-selectors-bad)

<span id="css-prefer-nested-selectors-good"></span>

[良い](./musea-and-css.md#css-prefer-nested-selectors-good)

[カテゴリの例](./musea-and-css.md#css-prefer-nested-selectors) · [ルール参照](./reference/css-prefer-nested-selectors.md)

### `css/prefer-slotted`

slot や子コンポーネントに対する scoped CSS のセレクターを検査します。

<span id="css-prefer-slotted-bad"></span>

[悪い](./musea-and-css.md#css-prefer-slotted-bad)

<span id="css-prefer-slotted-good"></span>

[良い](./musea-and-css.md#css-prefer-slotted-good)

[カテゴリの例](./musea-and-css.md#css-prefer-slotted) · [ルール参照](./reference/css-prefer-slotted.md)

### `css/require-font-display`

@font-face に font-display を指定します。

<span id="css-require-font-display-bad"></span>

[悪い](./musea-and-css.md#css-require-font-display-bad)

<span id="css-require-font-display-good"></span>

[良い](./musea-and-css.md#css-require-font-display-good)

[カテゴリの例](./musea-and-css.md#css-require-font-display) · [ルール参照](./reference/css-require-font-display.md)

### `musea/no-empty-variant`

内容のない variant ブロックを検出します。

<span id="musea-no-empty-variant-bad"></span>

[悪い](./musea-and-css.md#musea-no-empty-variant-bad)

<span id="musea-no-empty-variant-good"></span>

[良い](./musea-and-css.md#musea-no-empty-variant-good)

[カテゴリの例](./musea-and-css.md#musea-no-empty-variant) · [ルール参照](./reference/musea-no-empty-variant.md)

### `musea/prefer-design-tokens`

CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。

<span id="musea-prefer-design-tokens-bad"></span>

[悪い](./musea-and-css.md#musea-prefer-design-tokens-bad)

<span id="musea-prefer-design-tokens-good"></span>

[良い](./musea-and-css.md#musea-prefer-design-tokens-good)

[カテゴリの例](./musea-and-css.md#musea-prefer-design-tokens) · [ルール参照](./reference/musea-prefer-design-tokens.md)

### `musea/require-component`

art ブロックに対象の component を指定します。

<span id="musea-require-component-bad"></span>

[悪い](./musea-and-css.md#musea-require-component-bad)

<span id="musea-require-component-good"></span>

[良い](./musea-and-css.md#musea-require-component-good)

[カテゴリの例](./musea-and-css.md#musea-require-component) · [ルール参照](./reference/musea-require-component.md)

### `musea/require-title`

art ブロックに title を指定します。

<span id="musea-require-title-bad"></span>

[悪い](./musea-and-css.md#musea-require-title-bad)

<span id="musea-require-title-good"></span>

[良い](./musea-and-css.md#musea-require-title-good)

[カテゴリの例](./musea-and-css.md#musea-require-title) · [ルール参照](./reference/musea-require-title.md)

### `musea/unique-variant-names`

同じ Art ファイル内の variant 名を一意にします。

<span id="musea-unique-variant-names-bad"></span>

[悪い](./musea-and-css.md#musea-unique-variant-names-bad)

<span id="musea-unique-variant-names-good"></span>

[良い](./musea-and-css.md#musea-unique-variant-names-good)

[カテゴリの例](./musea-and-css.md#musea-unique-variant-names) · [ルール参照](./reference/musea-unique-variant-names.md)

### `musea/valid-variant`

variant ブロックに name を指定します。

<span id="musea-valid-variant-bad"></span>

[悪い](./musea-and-css.md#musea-valid-variant-bad)

<span id="musea-valid-variant-good"></span>

[良い](./musea-and-css.md#musea-valid-variant-good)

[カテゴリの例](./musea-and-css.md#musea-valid-variant) · [ルール参照](./reference/musea-valid-variant.md)

### `script/component-options-name-casing`

コンポーネントの name オプションを PascalCase に揃えます。

<span id="script-component-options-name-casing-bad"></span>

[悪い](./type-and-script.md#script-component-options-name-casing-bad)

<span id="script-component-options-name-casing-good"></span>

[良い](./type-and-script.md#script-component-options-name-casing-good)

[カテゴリの例](./type-and-script.md#script-component-options-name-casing) · [ルール参照](./reference/script-component-options-name-casing.md)

### `script/custom-event-name-casing`

emit するカスタムイベント名を指定した形式に揃えます。

<span id="script-custom-event-name-casing-bad"></span>

[悪い](./type-and-script.md#script-custom-event-name-casing-bad)

<span id="script-custom-event-name-casing-good"></span>

[良い](./type-and-script.md#script-custom-event-name-casing-good)

[カテゴリの例](./type-and-script.md#script-custom-event-name-casing) · [ルール参照](./reference/script-custom-event-name-casing.md)

### `script/define-emits-declaration`

defineEmits を型による宣言形式に揃えます。

<span id="script-define-emits-declaration-bad"></span>

[悪い](./type-and-script.md#script-define-emits-declaration-bad)

<span id="script-define-emits-declaration-good"></span>

[良い](./type-and-script.md#script-define-emits-declaration-good)

[カテゴリの例](./type-and-script.md#script-define-emits-declaration) · [ルール参照](./reference/script-define-emits-declaration.md)

### `script/define-macros-order`

script setup のコンパイラーマクロを一定の順に宣言します。

<span id="script-define-macros-order-bad"></span>

[悪い](./type-and-script.md#script-define-macros-order-bad)

<span id="script-define-macros-order-good"></span>

[良い](./type-and-script.md#script-define-macros-order-good)

[カテゴリの例](./type-and-script.md#script-define-macros-order) · [ルール参照](./reference/script-define-macros-order.md)

### `script/define-props-declaration`

defineProps を型による宣言形式に揃えます。

<span id="script-define-props-declaration-bad"></span>

[悪い](./type-and-script.md#script-define-props-declaration-bad)

<span id="script-define-props-declaration-good"></span>

[良い](./type-and-script.md#script-define-props-declaration-good)

[カテゴリの例](./type-and-script.md#script-define-props-declaration) · [ルール参照](./reference/script-define-props-declaration.md)

### `script/define-props-destructuring`

defineProps の分割代入スタイルを指定した方針に揃えます。

<span id="script-define-props-destructuring-bad"></span>

[悪い](./type-and-script.md#script-define-props-destructuring-bad)

<span id="script-define-props-destructuring-good"></span>

[良い](./type-and-script.md#script-define-props-destructuring-good)

[カテゴリの例](./type-and-script.md#script-define-props-destructuring) · [ルール参照](./reference/script-define-props-destructuring.md)

### `script/no-arrow-functions-in-watch`

Options API の watch ハンドラーにアロー関数を使う箇所を検出します。

<span id="script-no-arrow-functions-in-watch-bad"></span>

[悪い](./type-and-script.md#script-no-arrow-functions-in-watch-bad)

<span id="script-no-arrow-functions-in-watch-good"></span>

[良い](./type-and-script.md#script-no-arrow-functions-in-watch-good)

[カテゴリの例](./type-and-script.md#script-no-arrow-functions-in-watch) · [ルール参照](./reference/script-no-arrow-functions-in-watch.md)

### `script/no-async-in-computed`

computed の getter で非同期関数を使う箇所を検出します。

<span id="script-no-async-in-computed-bad"></span>

[悪い](./type-and-script.md#script-no-async-in-computed-bad)

<span id="script-no-async-in-computed-good"></span>

[良い](./type-and-script.md#script-no-async-in-computed-good)

[カテゴリの例](./type-and-script.md#script-no-async-in-computed) · [ルール参照](./reference/script-no-async-in-computed.md)

### `script/no-boolean-default`

Boolean prop の冗長な default を検出します。

<span id="script-no-boolean-default-bad"></span>

[悪い](./type-and-script.md#script-no-boolean-default-bad)

<span id="script-no-boolean-default-good"></span>

[良い](./type-and-script.md#script-no-boolean-default-good)

[カテゴリの例](./type-and-script.md#script-no-boolean-default) · [ルール参照](./reference/script-no-boolean-default.md)

### `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

<span id="script-no-deep-destructure-in-props-bad"></span>

[悪い](./type-and-script.md#script-no-deep-destructure-in-props-bad)

<span id="script-no-deep-destructure-in-props-good"></span>

[良い](./type-and-script.md#script-no-deep-destructure-in-props-good)

[カテゴリの例](./type-and-script.md#script-no-deep-destructure-in-props) · [ルール参照](./reference/script-no-deep-destructure-in-props.md)

### `script/no-deprecated-data-object-declaration`

Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。

<span id="script-no-deprecated-data-object-declaration-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-data-object-declaration-bad)

<span id="script-no-deprecated-data-object-declaration-good"></span>

[良い](./type-and-script.md#script-no-deprecated-data-object-declaration-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-data-object-declaration) · [ルール参照](./reference/script-no-deprecated-data-object-declaration.md)

### `script/no-deprecated-destroyed-lifecycle`

Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-bad)

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

[良い](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-destroyed-lifecycle) · [ルール参照](./reference/script-no-deprecated-destroyed-lifecycle.md)

### `script/no-deprecated-dollar-listeners-api`

Vue 3 で $attrs に統合された $listeners を検出します。

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-dollar-listeners-api-bad)

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

[良い](./type-and-script.md#script-no-deprecated-dollar-listeners-api-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-dollar-listeners-api) · [ルール参照](./reference/script-no-deprecated-dollar-listeners-api.md)

### `script/no-deprecated-dollar-scopedslots-api`

Vue 3 で $slots に統合された $scopedSlots を検出します。

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-bad)

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

[良い](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api) · [ルール参照](./reference/script-no-deprecated-dollar-scopedslots-api.md)

### `script/no-deprecated-events-api`

Vue 3 で削除された $on / $off / $once を検出します。

<span id="script-no-deprecated-events-api-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-events-api-bad)

<span id="script-no-deprecated-events-api-good"></span>

[良い](./type-and-script.md#script-no-deprecated-events-api-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-events-api) · [ルール参照](./reference/script-no-deprecated-events-api.md)

### `script/no-deprecated-props-default-this`

prop の default / validator 内で使えなくなった this を検出します。

<span id="script-no-deprecated-props-default-this-bad"></span>

[悪い](./type-and-script.md#script-no-deprecated-props-default-this-bad)

<span id="script-no-deprecated-props-default-this-good"></span>

[良い](./type-and-script.md#script-no-deprecated-props-default-this-good)

[カテゴリの例](./type-and-script.md#script-no-deprecated-props-default-this) · [ルール参照](./reference/script-no-deprecated-props-default-this.md)

### `script/no-dupe-keys`

Options API の props / data / computed などのキー重複を検出します。

<span id="script-no-dupe-keys-bad"></span>

[悪い](./type-and-script.md#script-no-dupe-keys-bad)

<span id="script-no-dupe-keys-good"></span>

[良い](./type-and-script.md#script-no-dupe-keys-good)

[カテゴリの例](./type-and-script.md#script-no-dupe-keys) · [ルール参照](./reference/script-no-dupe-keys.md)

### `script/no-duplicate-attr-inheritance`

fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。

<span id="script-no-duplicate-attr-inheritance-bad"></span>

[悪い](./type-and-script.md#script-no-duplicate-attr-inheritance-bad)

<span id="script-no-duplicate-attr-inheritance-good"></span>

[良い](./type-and-script.md#script-no-duplicate-attr-inheritance-good)

[カテゴリの例](./type-and-script.md#script-no-duplicate-attr-inheritance) · [ルール参照](./reference/script-no-duplicate-attr-inheritance.md)

### `script/no-export-in-script-setup`

script setup 内の export 文を検出します。

<span id="script-no-export-in-script-setup-bad"></span>

[悪い](./type-and-script.md#script-no-export-in-script-setup-bad)

<span id="script-no-export-in-script-setup-good"></span>

[良い](./type-and-script.md#script-no-export-in-script-setup-good)

[カテゴリの例](./type-and-script.md#script-no-export-in-script-setup) · [ルール参照](./reference/script-no-export-in-script-setup.md)

### `script/no-expose-after-await`

await の後の expose() と defineExpose() を禁止します。

<span id="script-no-expose-after-await-bad"></span>

[悪い](./type-and-script.md#script-no-expose-after-await-bad)

<span id="script-no-expose-after-await-good"></span>

[良い](./type-and-script.md#script-no-expose-after-await-good)

[カテゴリの例](./type-and-script.md#script-no-expose-after-await) · [ルール参照](./reference/script-no-expose-after-await.md)

### `script/no-import-compiler-macros`

自動的に使える Vue コンパイラーマクロの import を検出します。

<span id="script-no-import-compiler-macros-bad"></span>

[悪い](./type-and-script.md#script-no-import-compiler-macros-bad)

<span id="script-no-import-compiler-macros-good"></span>

[良い](./type-and-script.md#script-no-import-compiler-macros-good)

[カテゴリの例](./type-and-script.md#script-no-import-compiler-macros) · [ルール参照](./reference/script-no-import-compiler-macros.md)

### `script/no-internal-imports`

Vue 内部モジュールからの import を検出します。

<span id="script-no-internal-imports-bad"></span>

[悪い](./type-and-script.md#script-no-internal-imports-bad)

<span id="script-no-internal-imports-good"></span>

[良い](./type-and-script.md#script-no-internal-imports-good)

[カテゴリの例](./type-and-script.md#script-no-internal-imports) · [ルール参照](./reference/script-no-internal-imports.md)

### `script/no-lifecycle-after-await`

setup() 内で await の後にライフサイクルフックを登録することを禁止します。

<span id="script-no-lifecycle-after-await-bad"></span>

[悪い](./type-and-script.md#script-no-lifecycle-after-await-bad)

<span id="script-no-lifecycle-after-await-good"></span>

[良い](./type-and-script.md#script-no-lifecycle-after-await-good)

[カテゴリの例](./type-and-script.md#script-no-lifecycle-after-await) · [ルール参照](./reference/script-no-lifecycle-after-await.md)

### `script/no-multiple-slot-args`

scoped slot 関数に複数の引数を渡す箇所を検出します。

<span id="script-no-multiple-slot-args-bad"></span>

[悪い](./type-and-script.md#script-no-multiple-slot-args-bad)

<span id="script-no-multiple-slot-args-good"></span>

[良い](./type-and-script.md#script-no-multiple-slot-args-good)

[カテゴリの例](./type-and-script.md#script-no-multiple-slot-args) · [ルール参照](./reference/script-no-multiple-slot-args.md)

### `script/no-potential-component-option-typo`

Options API のオプション名の入力ミスを検出します。

<span id="script-no-potential-component-option-typo-bad"></span>

[悪い](./type-and-script.md#script-no-potential-component-option-typo-bad)

<span id="script-no-potential-component-option-typo-good"></span>

[良い](./type-and-script.md#script-no-potential-component-option-typo-good)

[カテゴリの例](./type-and-script.md#script-no-potential-component-option-typo) · [ルール参照](./reference/script-no-potential-component-option-typo.md)

### `script/no-reactive-destructure`

reactive オブジェクトを分割代入し、リアクティビティを失う箇所を検出します。

<span id="script-no-reactive-destructure-bad"></span>

[悪い](./type-and-script.md#script-no-reactive-destructure-bad)

<span id="script-no-reactive-destructure-good"></span>

[良い](./type-and-script.md#script-no-reactive-destructure-good)

[カテゴリの例](./type-and-script.md#script-no-reactive-destructure) · [ルール参照](./reference/script-no-reactive-destructure.md)

### `script/no-ref-as-operand`

ref を演算の値として使う際に .value を参照します。

<span id="script-no-ref-as-operand-bad"></span>

[悪い](./type-and-script.md#script-no-ref-as-operand-bad)

<span id="script-no-ref-as-operand-good"></span>

[良い](./type-and-script.md#script-no-ref-as-operand-good)

[カテゴリの例](./type-and-script.md#script-no-ref-as-operand) · [ルール参照](./reference/script-no-ref-as-operand.md)

### `script/no-required-prop-with-default`

required: true と default を同時に持つ prop を検出します。

<span id="script-no-required-prop-with-default-bad"></span>

[悪い](./type-and-script.md#script-no-required-prop-with-default-bad)

<span id="script-no-required-prop-with-default-good"></span>

[良い](./type-and-script.md#script-no-required-prop-with-default-good)

[カテゴリの例](./type-and-script.md#script-no-required-prop-with-default) · [ルール参照](./reference/script-no-required-prop-with-default.md)

### `script/no-reserved-identifiers`

Vue コンパイラーが予約した識別子の宣言を検出します。

<span id="script-no-reserved-identifiers-bad"></span>

[悪い](./type-and-script.md#script-no-reserved-identifiers-bad)

<span id="script-no-reserved-identifiers-good"></span>

[良い](./type-and-script.md#script-no-reserved-identifiers-good)

[カテゴリの例](./type-and-script.md#script-no-reserved-identifiers) · [ルール参照](./reference/script-no-reserved-identifiers.md)

### `script/no-reserved-keys`

Options API のキーに Vue の予約名を使う箇所を検出します。

<span id="script-no-reserved-keys-bad"></span>

[悪い](./type-and-script.md#script-no-reserved-keys-bad)

<span id="script-no-reserved-keys-good"></span>

[良い](./type-and-script.md#script-no-reserved-keys-good)

[カテゴリの例](./type-and-script.md#script-no-reserved-keys) · [ルール参照](./reference/script-no-reserved-keys.md)

### `script/no-reserved-props`

prop 宣言に Vue の予約名を使う箇所を検出します。

<span id="script-no-reserved-props-bad"></span>

[悪い](./type-and-script.md#script-no-reserved-props-bad)

<span id="script-no-reserved-props-good"></span>

[良い](./type-and-script.md#script-no-reserved-props-good)

[カテゴリの例](./type-and-script.md#script-no-reserved-props) · [ルール参照](./reference/script-no-reserved-props.md)

### `script/no-restricted-globals`

設定で禁止した実行環境のグローバル参照を検出します。

<span id="script-no-restricted-globals-bad"></span>

[悪い](./type-and-script.md#script-no-restricted-globals-bad)

<span id="script-no-restricted-globals-good"></span>

[良い](./type-and-script.md#script-no-restricted-globals-good)

[カテゴリの例](./type-and-script.md#script-no-restricted-globals) · [ルール参照](./reference/script-no-restricted-globals.md)

### `script/no-restricted-members`

設定で禁止した object.property へのアクセスを検出します。

<span id="script-no-restricted-members-bad"></span>

[悪い](./type-and-script.md#script-no-restricted-members-bad)

<span id="script-no-restricted-members-good"></span>

[良い](./type-and-script.md#script-no-restricted-members-good)

[カテゴリの例](./type-and-script.md#script-no-restricted-members) · [ルール参照](./reference/script-no-restricted-members.md)

### `script/no-side-effects-in-computed-properties`

Options API の computed getter 内の副作用を検出します。

<span id="script-no-side-effects-in-computed-properties-bad"></span>

[悪い](./type-and-script.md#script-no-side-effects-in-computed-properties-bad)

<span id="script-no-side-effects-in-computed-properties-good"></span>

[良い](./type-and-script.md#script-no-side-effects-in-computed-properties-good)

[カテゴリの例](./type-and-script.md#script-no-side-effects-in-computed-properties) · [ルール参照](./reference/script-no-side-effects-in-computed-properties.md)

### `script/no-top-level-ref-in-script`

通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。

<span id="script-no-top-level-ref-in-script-bad"></span>

[悪い](./type-and-script.md#script-no-top-level-ref-in-script-bad)

<span id="script-no-top-level-ref-in-script-good"></span>

[良い](./type-and-script.md#script-no-top-level-ref-in-script-good)

[カテゴリの例](./type-and-script.md#script-no-top-level-ref-in-script) · [ルール参照](./reference/script-no-top-level-ref-in-script.md)

### `script/no-unstable-nested-components`

setup / render 内で毎回コンポーネントを定義する箇所を検出します。

<span id="script-no-unstable-nested-components-bad"></span>

[悪い](./type-and-script.md#script-no-unstable-nested-components-bad)

<span id="script-no-unstable-nested-components-good"></span>

[良い](./type-and-script.md#script-no-unstable-nested-components-good)

[カテゴリの例](./type-and-script.md#script-no-unstable-nested-components) · [ルール参照](./reference/script-no-unstable-nested-components.md)

### `script/no-unused-emit-declarations`

宣言したまま emit していないイベントを検出します。

<span id="script-no-unused-emit-declarations-bad"></span>

[悪い](./type-and-script.md#script-no-unused-emit-declarations-bad)

<span id="script-no-unused-emit-declarations-good"></span>

[良い](./type-and-script.md#script-no-unused-emit-declarations-good)

[カテゴリの例](./type-and-script.md#script-no-unused-emit-declarations) · [ルール参照](./reference/script-no-unused-emit-declarations.md)

### `script/no-use-computed-property-like-method`

Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。

<span id="script-no-use-computed-property-like-method-bad"></span>

[悪い](./type-and-script.md#script-no-use-computed-property-like-method-bad)

<span id="script-no-use-computed-property-like-method-good"></span>

[良い](./type-and-script.md#script-no-use-computed-property-like-method-good)

[カテゴリの例](./type-and-script.md#script-no-use-computed-property-like-method) · [ルール参照](./reference/script-no-use-computed-property-like-method.md)

### `script/no-watch-after-await`

setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。

<span id="script-no-watch-after-await-bad"></span>

[悪い](./type-and-script.md#script-no-watch-after-await-bad)

<span id="script-no-watch-after-await-good"></span>

[良い](./type-and-script.md#script-no-watch-after-await-good)

[カテゴリの例](./type-and-script.md#script-no-watch-after-await) · [ルール参照](./reference/script-no-watch-after-await.md)

### `script/no-with-defaults`

Vue 3.5 以降の props 分割代入の既定値を勧めます。

<span id="script-no-with-defaults-bad"></span>

[悪い](./type-and-script.md#script-no-with-defaults-bad)

<span id="script-no-with-defaults-good"></span>

[良い](./type-and-script.md#script-no-with-defaults-good)

[カテゴリの例](./type-and-script.md#script-no-with-defaults) · [ルール参照](./reference/script-no-with-defaults.md)

### `script/prefer-computed`

他の状態から導ける値を watcher で同期する代わりに computed で表現します。

<span id="script-prefer-computed-bad"></span>

[悪い](./type-and-script.md#script-prefer-computed-bad)

<span id="script-prefer-computed-good"></span>

[良い](./type-and-script.md#script-prefer-computed-good)

[カテゴリの例](./type-and-script.md#script-prefer-computed) · [ルール参照](./reference/script-prefer-computed.md)

### `script/prefer-define-options`

name / inheritAttrs だけの通常 script を defineOptions() にまとめます。

<span id="script-prefer-define-options-bad"></span>

[悪い](./type-and-script.md#script-prefer-define-options-bad)

<span id="script-prefer-define-options-good"></span>

[良い](./type-and-script.md#script-prefer-define-options-good)

[カテゴリの例](./type-and-script.md#script-prefer-define-options) · [ルール参照](./reference/script-prefer-define-options.md)

### `script/prefer-import-from-vue`

内部パッケージではなく vue から import します。

<span id="script-prefer-import-from-vue-bad"></span>

[悪い](./type-and-script.md#script-prefer-import-from-vue-bad)

<span id="script-prefer-import-from-vue-good"></span>

[良い](./type-and-script.md#script-prefer-import-from-vue-good)

[カテゴリの例](./type-and-script.md#script-prefer-import-from-vue) · [ルール参照](./reference/script-prefer-import-from-vue.md)

### `script/prefer-ref-over-reactive`

状態管理に reactive() より ref() を使う方針を適用します。

<span id="script-prefer-ref-over-reactive-bad"></span>

[悪い](./type-and-script.md#script-prefer-ref-over-reactive-bad)

<span id="script-prefer-ref-over-reactive-good"></span>

[良い](./type-and-script.md#script-prefer-ref-over-reactive-good)

[カテゴリの例](./type-and-script.md#script-prefer-ref-over-reactive) · [ルール参照](./reference/script-prefer-ref-over-reactive.md)

### `script/prefer-use-attrs`

setup の context.attrs を useAttrs() に置き換えます。

<span id="script-prefer-use-attrs-bad"></span>

[悪い](./type-and-script.md#script-prefer-use-attrs-bad)

<span id="script-prefer-use-attrs-good"></span>

[良い](./type-and-script.md#script-prefer-use-attrs-good)

[カテゴリの例](./type-and-script.md#script-prefer-use-attrs) · [ルール参照](./reference/script-prefer-use-attrs.md)

### `script/prefer-use-id`

一意な ID の生成に Vue 3.5 の useId() を使います。

<span id="script-prefer-use-id-bad"></span>

[悪い](./type-and-script.md#script-prefer-use-id-bad)

<span id="script-prefer-use-id-good"></span>

[良い](./type-and-script.md#script-prefer-use-id-good)

[カテゴリの例](./type-and-script.md#script-prefer-use-id) · [ルール参照](./reference/script-prefer-use-id.md)

### `script/prefer-use-slots`

setup の context.slots を useSlots() に置き換えます。

<span id="script-prefer-use-slots-bad"></span>

[悪い](./type-and-script.md#script-prefer-use-slots-bad)

<span id="script-prefer-use-slots-good"></span>

[良い](./type-and-script.md#script-prefer-use-slots-good)

[カテゴリの例](./type-and-script.md#script-prefer-use-slots) · [ルール参照](./reference/script-prefer-use-slots.md)

### `script/prefer-use-template-ref`

テンプレート参照に Vue 3.5 の useTemplateRef() を使います。

<span id="script-prefer-use-template-ref-bad"></span>

[悪い](./type-and-script.md#script-prefer-use-template-ref-bad)

<span id="script-prefer-use-template-ref-good"></span>

[良い](./type-and-script.md#script-prefer-use-template-ref-good)

[カテゴリの例](./type-and-script.md#script-prefer-use-template-ref) · [ルール参照](./reference/script-prefer-use-template-ref.md)

### `script/require-default-prop`

省略可能な、Boolean 以外の prop に default を用意します。

<span id="script-require-default-prop-bad"></span>

[悪い](./type-and-script.md#script-require-default-prop-bad)

<span id="script-require-default-prop-good"></span>

[良い](./type-and-script.md#script-require-default-prop-good)

[カテゴリの例](./type-and-script.md#script-require-default-prop) · [ルール参照](./reference/script-require-default-prop.md)

### `script/require-explicit-emits`

emit するイベントを defineEmits または emits に宣言します。

<span id="script-require-explicit-emits-bad"></span>

[悪い](./type-and-script.md#script-require-explicit-emits-bad)

<span id="script-require-explicit-emits-good"></span>

[良い](./type-and-script.md#script-require-explicit-emits-good)

[カテゴリの例](./type-and-script.md#script-require-explicit-emits) · [ルール参照](./reference/script-require-explicit-emits.md)

### `script/require-explicit-slots`

useSlots() で使う slot を defineSlots の型で宣言します。

<span id="script-require-explicit-slots-bad"></span>

[悪い](./type-and-script.md#script-require-explicit-slots-bad)

<span id="script-require-explicit-slots-good"></span>

[良い](./type-and-script.md#script-require-explicit-slots-good)

[カテゴリの例](./type-and-script.md#script-require-explicit-slots) · [ルール参照](./reference/script-require-explicit-slots.md)

### `script/require-function-return-type`

関数に戻り値の型注釈を指定します。

<span id="script-require-function-return-type-bad"></span>

[悪い](./type-and-script.md#script-require-function-return-type-bad)

<span id="script-require-function-return-type-good"></span>

[良い](./type-and-script.md#script-require-function-return-type-good)

[カテゴリの例](./type-and-script.md#script-require-function-return-type) · [ルール参照](./reference/script-require-function-return-type.md)

### `script/require-prop-type-constructor`

prop の type に文字列ではなくコンストラクターを指定します。

<span id="script-require-prop-type-constructor-bad"></span>

[悪い](./type-and-script.md#script-require-prop-type-constructor-bad)

<span id="script-require-prop-type-constructor-good"></span>

[良い](./type-and-script.md#script-require-prop-type-constructor-good)

[カテゴリの例](./type-and-script.md#script-require-prop-type-constructor) · [ルール参照](./reference/script-require-prop-type-constructor.md)

### `script/require-prop-types`

各 prop の型を宣言します。

<span id="script-require-prop-types-bad"></span>

[悪い](./type-and-script.md#script-require-prop-types-bad)

<span id="script-require-prop-types-good"></span>

[良い](./type-and-script.md#script-require-prop-types-good)

[カテゴリの例](./type-and-script.md#script-require-prop-types) · [ルール参照](./reference/script-require-prop-types.md)

### `script/require-symbol-provide`

provide / inject のキーに衝突しにくい Symbol を使います。

<span id="script-require-symbol-provide-bad"></span>

[悪い](./type-and-script.md#script-require-symbol-provide-bad)

<span id="script-require-symbol-provide-good"></span>

[良い](./type-and-script.md#script-require-symbol-provide-good)

[カテゴリの例](./type-and-script.md#script-require-symbol-provide) · [ルール参照](./reference/script-require-symbol-provide.md)

### `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

<span id="script-require-typed-object-prop-bad"></span>

[悪い](./type-and-script.md#script-require-typed-object-prop-bad)

<span id="script-require-typed-object-prop-good"></span>

[良い](./type-and-script.md#script-require-typed-object-prop-good)

[カテゴリの例](./type-and-script.md#script-require-typed-object-prop) · [ルール参照](./reference/script-require-typed-object-prop.md)

### `script/require-typed-ref`

引数なし、null、undefined で呼び出す ref() に型引数を指定します。

<span id="script-require-typed-ref-bad"></span>

[悪い](./type-and-script.md#script-require-typed-ref-bad)

<span id="script-require-typed-ref-good"></span>

[良い](./type-and-script.md#script-require-typed-ref-good)

[カテゴリの例](./type-and-script.md#script-require-typed-ref) · [ルール参照](./reference/script-require-typed-ref.md)

### `script/require-valid-default-prop`

prop の default を宣言した型に合う値にします。

<span id="script-require-valid-default-prop-bad"></span>

[悪い](./type-and-script.md#script-require-valid-default-prop-bad)

<span id="script-require-valid-default-prop-good"></span>

[良い](./type-and-script.md#script-require-valid-default-prop-good)

[カテゴリの例](./type-and-script.md#script-require-valid-default-prop) · [ルール参照](./reference/script-require-valid-default-prop.md)

### `script/return-in-computed-property`

computed の getter に値を返す return を用意します。

<span id="script-return-in-computed-property-bad"></span>

[悪い](./type-and-script.md#script-return-in-computed-property-bad)

<span id="script-return-in-computed-property-good"></span>

[良い](./type-and-script.md#script-return-in-computed-property-good)

[カテゴリの例](./type-and-script.md#script-return-in-computed-property) · [ルール参照](./reference/script-return-in-computed-property.md)

### `script/return-in-emits-validator`

Options API の emits validator に戻り値を用意します。

<span id="script-return-in-emits-validator-bad"></span>

[悪い](./type-and-script.md#script-return-in-emits-validator-bad)

<span id="script-return-in-emits-validator-good"></span>

[良い](./type-and-script.md#script-return-in-emits-validator-good)

[カテゴリの例](./type-and-script.md#script-return-in-emits-validator) · [ルール参照](./reference/script-return-in-emits-validator.md)

### `script/valid-define-emits`

defineEmits の重複や型と実行時引数の併用を検出します。

<span id="script-valid-define-emits-bad"></span>

[悪い](./type-and-script.md#script-valid-define-emits-bad)

<span id="script-valid-define-emits-good"></span>

[良い](./type-and-script.md#script-valid-define-emits-good)

[カテゴリの例](./type-and-script.md#script-valid-define-emits) · [ルール参照](./reference/script-valid-define-emits.md)

### `script/valid-define-options`

defineOptions の引数と使用回数を検査します。

<span id="script-valid-define-options-bad"></span>

[悪い](./type-and-script.md#script-valid-define-options-bad)

<span id="script-valid-define-options-good"></span>

[良い](./type-and-script.md#script-valid-define-options-good)

[カテゴリの例](./type-and-script.md#script-valid-define-options) · [ルール参照](./reference/script-valid-define-options.md)

### `script/valid-define-props`

defineProps の重複や型と実行時引数の併用を検出します。

<span id="script-valid-define-props-bad"></span>

[悪い](./type-and-script.md#script-valid-define-props-bad)

<span id="script-valid-define-props-good"></span>

[良い](./type-and-script.md#script-valid-define-props-good)

[カテゴリの例](./type-and-script.md#script-valid-define-props) · [ルール参照](./reference/script-valid-define-props.md)

### `script/valid-next-tick`

nextTick() の完了を await、then、またはコールバックで扱います。

<span id="script-valid-next-tick-bad"></span>

[悪い](./type-and-script.md#script-valid-next-tick-bad)

<span id="script-valid-next-tick-good"></span>

[良い](./type-and-script.md#script-valid-next-tick-good)

[カテゴリの例](./type-and-script.md#script-valid-next-tick) · [ルール参照](./reference/script-valid-next-tick.md)

### `nuxt/no-nuxt-config-test-key`

Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

[悪い](./ecosystem.md#nuxt-no-nuxt-config-test-key-bad)

<span id="nuxt-no-nuxt-config-test-key-good"></span>

[良い](./ecosystem.md#nuxt-no-nuxt-config-test-key-good)

[カテゴリの例](./ecosystem.md#nuxt-no-nuxt-config-test-key) · [ルール参照](./reference/nuxt-no-nuxt-config-test-key.md)

### `nuxt/no-page-meta-runtime-values`

definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

[悪い](./ecosystem.md#nuxt-no-page-meta-runtime-values-bad)

<span id="nuxt-no-page-meta-runtime-values-good"></span>

[良い](./ecosystem.md#nuxt-no-page-meta-runtime-values-good)

[カテゴリの例](./ecosystem.md#nuxt-no-page-meta-runtime-values) · [ルール参照](./reference/nuxt-no-page-meta-runtime-values.md)

### `nuxt/nuxt-config-keys-order`

Nuxt 設定のプロパティを推奨順に並べます。

<span id="nuxt-nuxt-config-keys-order-bad"></span>

[悪い](./ecosystem.md#nuxt-nuxt-config-keys-order-bad)

<span id="nuxt-nuxt-config-keys-order-good"></span>

[良い](./ecosystem.md#nuxt-nuxt-config-keys-order-good)

[カテゴリの例](./ecosystem.md#nuxt-nuxt-config-keys-order) · [ルール参照](./reference/nuxt-nuxt-config-keys-order.md)

### `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

<span id="nuxt-prefer-import-meta-bad"></span>

[悪い](./ecosystem.md#nuxt-prefer-import-meta-bad)

<span id="nuxt-prefer-import-meta-good"></span>

[良い](./ecosystem.md#nuxt-prefer-import-meta-good)

[カテゴリの例](./ecosystem.md#nuxt-prefer-import-meta) · [ルール参照](./reference/nuxt-prefer-import-meta.md)


## プロジェクトの例

### `ecosystem/vue-router-unknown-route`

登録済み router に存在しない名前です。

<span id="ecosystem-vue-router-unknown-route-bad"></span>

[悪い](./cross-file.md#ecosystem-vue-router-unknown-route-bad)

<span id="ecosystem-vue-router-unknown-route-good"></span>

[良い](./cross-file.md#ecosystem-vue-router-unknown-route-good)

[カテゴリの例](./cross-file.md#ecosystem-vue-router-unknown-route) · [ルール参照](./project/ecosystem-vue-router-unknown-route.md)

### `ecosystem/vue-router-extra-param`

パスにない tab は Vue Router に破棄されます。

<span id="ecosystem-vue-router-extra-param-bad"></span>

[悪い](./cross-file.md#ecosystem-vue-router-extra-param-bad)

<span id="ecosystem-vue-router-extra-param-good"></span>

[良い](./cross-file.md#ecosystem-vue-router-extra-param-good)

[カテゴリの例](./cross-file.md#ecosystem-vue-router-extra-param) · [ルール参照](./project/ecosystem-vue-router-extra-param.md)

### `ecosystem/vue-router-param-type`

postId は繰り返しパラメーターではないため配列を渡せません。

<span id="ecosystem-vue-router-param-type-bad"></span>

[悪い](./cross-file.md#ecosystem-vue-router-param-type-bad)

<span id="ecosystem-vue-router-param-type-good"></span>

[良い](./cross-file.md#ecosystem-vue-router-param-type-good)

[カテゴリの例](./cross-file.md#ecosystem-vue-router-param-type) · [ルール参照](./project/ecosystem-vue-router-param-type.md)

### `ecosystem/vue-router-missing-param`

必須の postId がありません。現在のルートに依存する遷移になります。

<span id="ecosystem-vue-router-missing-param-bad"></span>

[悪い](./cross-file.md#ecosystem-vue-router-missing-param-bad)

<span id="ecosystem-vue-router-missing-param-good"></span>

[良い](./cross-file.md#ecosystem-vue-router-missing-param-good)

[カテゴリの例](./cross-file.md#ecosystem-vue-router-missing-param) · [ルール参照](./project/ecosystem-vue-router-missing-param.md)

### `html/cross-component-nesting`

import した子コンポーネントの要素を合成して HTML の入れ子を検査します。

<span id="html-cross-component-nesting-bad"></span>

[悪い](./cross-file.md#html-cross-component-nesting-bad)

<span id="html-cross-component-nesting-good"></span>

[良い](./cross-file.md#html-cross-component-nesting-good)

[カテゴリの例](./cross-file.md#html-cross-component-nesting) · [ルール参照](./project/html-cross-component-nesting.md)

### `vue/cross-file-attrs-fallthrough`

属性を渡す親と、属性を自動継承できず $attrs も使っていない子コンポーネントの関係を検査します。

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

[悪い](./cross-file.md#vue-cross-file-attrs-fallthrough-bad)

<span id="vue-cross-file-attrs-fallthrough-good"></span>

[良い](./cross-file.md#vue-cross-file-attrs-fallthrough-good)

[カテゴリの例](./cross-file.md#vue-cross-file-attrs-fallthrough) · [ルール参照](./project/vue-cross-file-attrs-fallthrough.md)

### `vize:croquis/cf/array-mutation`

現在の生成元がないこの契約では、旧 Vue 2.7 の配列のインデックス代入を例にしています。Vue 2 は直接のインデックス代入による更新を通知しませんが、Vue 3 はその代入を追跡します。悪い例・良い例は Vue 2 の修正を示すもので、Vue 3 の不具合を示す例ではありません。

<span id="vize-croquis-cf-array-mutation-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-array-mutation-bad)

<span id="vize-croquis-cf-array-mutation-good"></span>

[良い](./cross-file.md#vize-croquis-cf-array-mutation-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-array-mutation) · [ルール参照](./project/vize-croquis-cf-array-mutation.md)

### `vize:croquis/cf/async-boundary`

reactive な状態が async 境界を越え、古い値が見えることがあります。

<span id="vize-croquis-cf-async-boundary-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-async-boundary-bad)

<span id="vize-croquis-cf-async-boundary-good"></span>

[良い](./cross-file.md#vize-croquis-cf-async-boundary-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-async-boundary) · [ルール参照](./project/vize-croquis-cf-async-boundary.md)

### `vize:croquis/cf/async-no-suspense`

async コンポーネントが Suspense 境界なしで描画されています。

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-async-no-suspense-bad)

<span id="vize-croquis-cf-async-no-suspense-good"></span>

[良い](./cross-file.md#vize-croquis-cf-async-no-suspense-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-async-no-suspense) · [ルール参照](./project/vize-croquis-cf-async-no-suspense.md)

### `vize:croquis/cf/browser-api-ssr`

サーバーでも描画され得る場所で、ブラウザ専用 API を使っています。

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-browser-api-ssr-bad)

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

[良い](./cross-file.md#vize-croquis-cf-browser-api-ssr-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-browser-api-ssr) · [ルール参照](./project/vize-croquis-cf-browser-api-ssr.md)

### `vize:croquis/cf/circular-dep`

コンポーネントの import が循環しています。

<span id="vize-croquis-cf-circular-dep-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-circular-dep-bad)

<span id="vize-croquis-cf-circular-dep-good"></span>

[良い](./cross-file.md#vize-croquis-cf-circular-dep-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-circular-dep) · [ルール参照](./project/vize-croquis-cf-circular-dep.md)

### `vize:croquis/cf/circular-reactive-dependency`

reactive な計算が互いに循環して依存しています。

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-bad)

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

[良い](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-circular-reactive-dependency) · [ルール参照](./project/vize-croquis-cf-circular-reactive-dependency.md)

### `vize:croquis/cf/closure-captures-reactive`

クロージャが reactive な値を捕捉しており、その後の更新が見えません。

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-closure-captures-reactive-bad)

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

[良い](./cross-file.md#vize-croquis-cf-closure-captures-reactive-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-closure-captures-reactive) · [ルール参照](./project/vize-croquis-cf-closure-captures-reactive.md)

### `vize:croquis/cf/composable-outside-setup`

composable が `setup` の外で呼ばれています。

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-composable-outside-setup-bad)

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-composable-outside-setup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-composable-outside-setup) · [ルール参照](./project/vize-croquis-cf-composable-outside-setup.md)

### `vize:croquis/cf/computed-side-effects`

computed の getter が状態を書いたり、他の副作用を起こしています。

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-computed-side-effects-bad)

<span id="vize-croquis-cf-computed-side-effects-good"></span>

[良い](./cross-file.md#vize-croquis-cf-computed-side-effects-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-computed-side-effects) · [ルール参照](./project/vize-croquis-cf-computed-side-effects.md)

### `vize:croquis/cf/deep-import`

import の連鎖がプロジェクトの許す深さより深くなっています。

<span id="vize-croquis-cf-deep-import-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-deep-import-bad)

<span id="vize-croquis-cf-deep-import-good"></span>

[良い](./cross-file.md#vize-croquis-cf-deep-import-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-deep-import) · [ルール参照](./project/vize-croquis-cf-deep-import.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

reactive オブジェクトの分割代入はフィールドをコピーし、追跡を失います。

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-bad)

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

[良い](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity) · [ルール参照](./project/vize-croquis-cf-destructuring-breaks-reactivity.md)

### `vize:croquis/cf/di-outside-setup`

`provide` または `inject` が `setup` の外で呼ばれています。

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-di-outside-setup-bad)

<span id="vize-croquis-cf-di-outside-setup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-di-outside-setup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-di-outside-setup) · [ルール参照](./project/vize-croquis-cf-di-outside-setup.md)

### `vize:croquis/cf/dom-access-without-next-tick`

Vue が更新を反映する前に DOM を読んでいます。

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-bad)

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

[良い](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick) · [ルール参照](./project/vize-croquis-cf-dom-access-without-next-tick.md)

### `vize:croquis/cf/duplicate-id`

同じ要素 id が複数のコンポーネントで使われています。

<span id="vize-croquis-cf-duplicate-id-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-duplicate-id-bad)

<span id="vize-croquis-cf-duplicate-id-good"></span>

[良い](./cross-file.md#vize-croquis-cf-duplicate-id-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-duplicate-id) · [ルール参照](./project/vize-croquis-cf-duplicate-id.md)

### `vize:croquis/cf/event-listener-leak`

イベントリスナーが登録されたまま、解除されていません。

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-event-listener-leak-bad)

<span id="vize-croquis-cf-event-listener-leak-good"></span>

[良い](./cross-file.md#vize-croquis-cf-event-listener-leak-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-event-listener-leak) · [ルール参照](./project/vize-croquis-cf-event-listener-leak.md)

### `vize:croquis/cf/event-modifier`

emit が対応しない修飾子を、イベントリスナーが使っています。

<span id="vize-croquis-cf-event-modifier-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-event-modifier-bad)

<span id="vize-croquis-cf-event-modifier-good"></span>

[良い](./cross-file.md#vize-croquis-cf-event-modifier-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-event-modifier) · [ルール参照](./project/vize-croquis-cf-event-modifier.md)

### `vize:croquis/cf/hydration-risk`

この診断コードは、prop を ref にコピーする操作など、リアクティビティに関する複数の検出結果をまとめています。ファイル間検査がすべての `Date.now()` 式を検出するという意味ではありません。

<span id="vize-croquis-cf-hydration-risk-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-hydration-risk-bad)

<span id="vize-croquis-cf-hydration-risk-good"></span>

[良い](./cross-file.md#vize-croquis-cf-hydration-risk-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-hydration-risk) · [ルール参照](./project/vize-croquis-cf-hydration-risk.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` が設定されているのに、属性が読まれていません。

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-bad)

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

[良い](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-inherit-attrs-unused) · [ルール参照](./project/vize-croquis-cf-inherit-attrs-unused.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` が `InjectionKey` のシンボルではなく素のキーを使っています。

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-inject-without-symbol-bad)

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

[良い](./cross-file.md#vize-croquis-cf-inject-without-symbol-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-inject-without-symbol) · [ルール参照](./project/vize-croquis-cf-inject-without-symbol.md)

### `vize:croquis/cf/injected-async-mutation-race`

inject した値が、競合し得る async タスクから変更されています。

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-bad)

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

[良い](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-injected-async-mutation-race) · [ルール参照](./project/vize-croquis-cf-injected-async-mutation-race.md)

### `vize:croquis/cf/lifecycle-outside-setup`

ライフサイクルフックが `setup` の外で登録されています。

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-bad)

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup) · [ルール参照](./project/vize-croquis-cf-lifecycle-outside-setup.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

ライフサイクルフックが後片付けなしで処理を始めています。

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-bad)

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup) · [ルール参照](./project/vize-croquis-cf-lifecycle-without-cleanup.md)

### `vize:croquis/cf/missing-required-prop`

必須 prop が渡されていません。

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-missing-required-prop-bad)

<span id="vize-croquis-cf-missing-required-prop-good"></span>

[良い](./cross-file.md#vize-croquis-cf-missing-required-prop-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-missing-required-prop) · [ルール参照](./project/vize-croquis-cf-missing-required-prop.md)

### `vize:croquis/cf/missing-suspense`

async な依存が Suspense 境界の外で使われています。

<span id="vize-croquis-cf-missing-suspense-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-missing-suspense-bad)

<span id="vize-croquis-cf-missing-suspense-good"></span>

[良い](./cross-file.md#vize-croquis-cf-missing-suspense-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-missing-suspense) · [ルール参照](./project/vize-croquis-cf-missing-suspense.md)

### `vize:croquis/cf/module-scope-reactive`

reactive な状態がモジュールスコープで作られ、すべての呼び出し元で共有されます。

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-module-scope-reactive-bad)

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

[良い](./cross-file.md#vize-croquis-cf-module-scope-reactive-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-module-scope-reactive) · [ルール参照](./project/vize-croquis-cf-module-scope-reactive.md)

### `vize:croquis/cf/multi-root-attrs`

複数ルートのコンポーネントが属性を受け取りますが、付ける場所がありません。

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-multi-root-attrs-bad)

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

[良い](./cross-file.md#vize-croquis-cf-multi-root-attrs-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-multi-root-attrs) · [ルール参照](./project/vize-croquis-cf-multi-root-attrs.md)

### `vize:croquis/cf/mutated-after-escape`

reactive オブジェクトが、所有者から漏れた後に変更されています。

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-mutated-after-escape-bad)

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

[良い](./cross-file.md#vize-croquis-cf-mutated-after-escape-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-mutated-after-escape) · [ルール参照](./project/vize-croquis-cf-mutated-after-escape.md)

### `vize:croquis/cf/non-reactive-provide`

provide した値が reactive ではないため、子孫は更新を観測できません。

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-non-reactive-provide-bad)

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

[良い](./cross-file.md#vize-croquis-cf-non-reactive-provide-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-non-reactive-provide) · [ルール参照](./project/vize-croquis-cf-non-reactive-provide.md)

### `vize:croquis/cf/non-unique-id`

ループ内の要素 id が項目ごとに一意ではありません。

<span id="vize-croquis-cf-non-unique-id-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-non-unique-id-bad)

<span id="vize-croquis-cf-non-unique-id-good"></span>

[良い](./cross-file.md#vize-croquis-cf-non-unique-id-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-non-unique-id) · [ルール参照](./project/vize-croquis-cf-non-unique-id.md)

### `vize:croquis/cf/object-identity-comparison`

元のオブジェクトとそのリアクティブな Proxy は別の同一性を持ち、`toRaw()` は元のオブジェクトを返します。この契約の例は、レコードとして同じものかを判定するために安定した ID を比較する方法を示します。

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-object-identity-comparison-bad)

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

[良い](./cross-file.md#vize-croquis-cf-object-identity-comparison-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-object-identity-comparison) · [ルール参照](./project/vize-croquis-cf-object-identity-comparison.md)

### `vize:croquis/cf/pinia-getter`

この契約の例では、setup で Pinia の getter から現在のプリミティブ値をコピーし、更新に追従しないスナップショットを作っています。追跡される計算やテンプレートで getter を直接読む場合はリアクティビティを保てます。この例では `storeToRefs()` を使って値を取り出します。

<span id="vize-croquis-cf-pinia-getter-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-pinia-getter-bad)

<span id="vize-croquis-cf-pinia-getter-good"></span>

[良い](./cross-file.md#vize-croquis-cf-pinia-getter-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-pinia-getter) · [ルール参照](./project/vize-croquis-cf-pinia-getter.md)

### `vize:croquis/cf/prop-type-mismatch`

渡した prop の値が、宣言された型と一致しません。

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-prop-type-mismatch-bad)

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

[良い](./cross-file.md#vize-croquis-cf-prop-type-mismatch-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-prop-type-mismatch) · [ルール参照](./project/vize-croquis-cf-prop-type-mismatch.md)

### `vize:croquis/cf/provide-inject-type`

provide した値と inject の型が一致しません。

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-provide-inject-type-bad)

<span id="vize-croquis-cf-provide-inject-type-good"></span>

[良い](./cross-file.md#vize-croquis-cf-provide-inject-type-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-provide-inject-type) · [ルール参照](./project/vize-croquis-cf-provide-inject-type.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` が `InjectionKey` のシンボルではなく素のキーを使っています。

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-provide-without-symbol-bad)

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

[良い](./cross-file.md#vize-croquis-cf-provide-without-symbol-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-provide-without-symbol) · [ルール参照](./project/vize-croquis-cf-provide-without-symbol.md)

### `vize:croquis/cf/reactive-export`

reactive な状態がモジュールから export されています。

<span id="vize-croquis-cf-reactive-export-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-reactive-export-bad)

<span id="vize-croquis-cf-reactive-export-good"></span>

[良い](./cross-file.md#vize-croquis-cf-reactive-export-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-reactive-export) · [ルール参照](./project/vize-croquis-cf-reactive-export.md)

### `vize:croquis/cf/reactivity-outside-setup`

reactive API が `setup` の外で呼ばれています。

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-bad)

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-reactivity-outside-setup) · [ルール参照](./project/vize-croquis-cf-reactivity-outside-setup.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

reactive な束縛を再代入すると、ただの値に置き換わります。

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-bad)

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

[良い](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity) · [ルール参照](./project/vize-croquis-cf-reassignment-breaks-reactivity.md)

### `vize:croquis/cf/reference-escapes-scope`

reactive な参照が、寿命を持つスコープの外へ漏れています。

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-reference-escapes-scope-bad)

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

[良い](./cross-file.md#vize-croquis-cf-reference-escapes-scope-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-reference-escapes-scope) · [ルール参照](./project/vize-croquis-cf-reference-escapes-scope.md)

### `vize:croquis/cf/setup-context-violation`

実験的な解析器の例では、インスタンスごとの setup ではなく通常の script のモジュールスコープに `ref(0)` を置いています。Vue ではこのようなリアクティブな状態を作れますが、この例ではインスタンスごとに count を持つことを前提とします。CLI はこの個別の診断コードを生成しません。

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-setup-context-violation-bad)

<span id="vize-croquis-cf-setup-context-violation-good"></span>

[良い](./cross-file.md#vize-croquis-cf-setup-context-violation-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-setup-context-violation) · [ルール参照](./project/vize-croquis-cf-setup-context-violation.md)

### `vize:croquis/cf/shallow-deep-access`

`shallowReactive` または `shallowRef` の深いプロパティを、追跡されるものとして読んでいます。

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-shallow-deep-access-bad)

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

[良い](./cross-file.md#vize-croquis-cf-shallow-deep-access-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-shallow-deep-access) · [ルール参照](./project/vize-croquis-cf-shallow-deep-access.md)

### `vize:croquis/cf/spread-breaks-reactivity`

reactive オブジェクトのスプレッドは値をコピーし、追跡を失います。

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-bad)

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

[良い](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity) · [ルール参照](./project/vize-croquis-cf-spread-breaks-reactivity.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` に fallback がありません。

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-suspense-no-fallback-bad)

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

[良い](./cross-file.md#vize-croquis-cf-suspense-no-fallback-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-suspense-no-fallback) · [ルール参照](./project/vize-croquis-cf-suspense-no-fallback.md)

### `vize:croquis/cf/template-ref-timing`

テンプレート ref が、コンポーネントのマウント前に読まれています。

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-template-ref-timing-bad)

<span id="vize-croquis-cf-template-ref-timing-good"></span>

[良い](./cross-file.md#vize-croquis-cf-template-ref-timing-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-template-ref-timing) · [ルール参照](./project/vize-croquis-cf-template-ref-timing.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` した生のオブジェクトを変更しています。

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-toraw-mutation-bad)

<span id="vize-croquis-cf-toraw-mutation-good"></span>

[良い](./cross-file.md#vize-croquis-cf-toraw-mutation-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-toraw-mutation) · [ルール参照](./project/vize-croquis-cf-toraw-mutation.md)

### `vize:croquis/cf/uncaught-error`

コンポーネントが例外を投げても、受け止めるエラー境界がありません。

<span id="vize-croquis-cf-uncaught-error-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-uncaught-error-bad)

<span id="vize-croquis-cf-uncaught-error-good"></span>

[良い](./cross-file.md#vize-croquis-cf-uncaught-error-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-uncaught-error) · [ルール参照](./project/vize-croquis-cf-uncaught-error.md)

### `vize:croquis/cf/undeclared-emit`

宣言されていないイベントを emit しています。

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-undeclared-emit-bad)

<span id="vize-croquis-cf-undeclared-emit-good"></span>

[良い](./cross-file.md#vize-croquis-cf-undeclared-emit-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-undeclared-emit) · [ルール参照](./project/vize-croquis-cf-undeclared-emit.md)

### `vize:croquis/cf/undeclared-prop`

親が、子が宣言していない prop を渡しています。

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-undeclared-prop-bad)

<span id="vize-croquis-cf-undeclared-prop-good"></span>

[良い](./cross-file.md#vize-croquis-cf-undeclared-prop-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-undeclared-prop) · [ルール参照](./project/vize-croquis-cf-undeclared-prop.md)

### `vize:croquis/cf/undefined-slot`

親が、子が公開していないスロットを埋めています。

<span id="vize-croquis-cf-undefined-slot-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-undefined-slot-bad)

<span id="vize-croquis-cf-undefined-slot-good"></span>

[良い](./cross-file.md#vize-croquis-cf-undefined-slot-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-undefined-slot) · [ルール参照](./project/vize-croquis-cf-undefined-slot.md)

### `vize:croquis/cf/unhandled-event`

子が emit したイベントを、どの親も処理していません。

<span id="vize-croquis-cf-unhandled-event-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unhandled-event-bad)

<span id="vize-croquis-cf-unhandled-event-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unhandled-event-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unhandled-event) · [ルール参照](./project/vize-croquis-cf-unhandled-event.md)

### `vize:croquis/cf/unmatched-inject`

`inject` が、どの祖先も provide していないキーを指定しています。

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unmatched-inject-bad)

<span id="vize-croquis-cf-unmatched-inject-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unmatched-inject-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unmatched-inject) · [ルール参照](./project/vize-croquis-cf-unmatched-inject.md)

### `vize:croquis/cf/unmatched-listener`

親が、子が emit しないイベントを購読しています。

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unmatched-listener-bad)

<span id="vize-croquis-cf-unmatched-listener-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unmatched-listener-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unmatched-listener) · [ルール参照](./project/vize-croquis-cf-unmatched-listener.md)

### `vize:croquis/cf/unregistered-component`

テンプレートが、登録も import もされていないコンポーネントを使っています。

<span id="vize-croquis-cf-unregistered-component-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unregistered-component-bad)

<span id="vize-croquis-cf-unregistered-component-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unregistered-component-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unregistered-component) · [ルール参照](./project/vize-croquis-cf-unregistered-component.md)

### `vize:croquis/cf/unresolved-import`

import がモジュールへ解決できません。

<span id="vize-croquis-cf-unresolved-import-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unresolved-import-bad)

<span id="vize-croquis-cf-unresolved-import-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unresolved-import-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unresolved-import) · [ルール参照](./project/vize-croquis-cf-unresolved-import.md)

### `vize:croquis/cf/unused-attrs`

複数ルートのコンポーネントに渡した fallthrough 属性が使われていません。

<span id="vize-croquis-cf-unused-attrs-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unused-attrs-bad)

<span id="vize-croquis-cf-unused-attrs-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unused-attrs-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unused-attrs) · [ルール参照](./project/vize-croquis-cf-unused-attrs.md)

### `vize:croquis/cf/unused-emit`

宣言した emit が一度も使われていません。

<span id="vize-croquis-cf-unused-emit-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unused-emit-bad)

<span id="vize-croquis-cf-unused-emit-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unused-emit-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unused-emit) · [ルール参照](./project/vize-croquis-cf-unused-emit.md)

### `vize:croquis/cf/unused-provide`

provide したキーが一度も inject されていません。

<span id="vize-croquis-cf-unused-provide-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-unused-provide-bad)

<span id="vize-croquis-cf-unused-provide-good"></span>

[良い](./cross-file.md#vize-croquis-cf-unused-provide-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-unused-provide) · [ルール参照](./project/vize-croquis-cf-unused-provide.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

reactive な値をローカルへ取り出すと、その後の更新が失われます。

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-bad)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

[良い](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity) · [ルール参照](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md)

### `vize:croquis/cf/watch-can-be-computed`

ウォッチャが値を状態へ写しているだけで、computed にできます。

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-watch-can-be-computed-bad)

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

[良い](./cross-file.md#vize-croquis-cf-watch-can-be-computed-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-watch-can-be-computed) · [ルール参照](./project/vize-croquis-cf-watch-can-be-computed.md)

### `vize:croquis/cf/watcheffect-async`

この例の非同期の `watchEffect` には、無効化時のクリーンアップや古い応答を防ぐ処理がありません。Vue の `watchEffect` はクリーンアップを扱えます。良い例では `watch` で監視対象を明示し、無効化された処理を中止します。

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-watcheffect-async-bad)

<span id="vize-croquis-cf-watcheffect-async-good"></span>

[良い](./cross-file.md#vize-croquis-cf-watcheffect-async-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-watcheffect-async) · [ルール参照](./project/vize-croquis-cf-watcheffect-async.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` または `watchEffect` が `setup` の外で呼ばれています。

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

[悪い](./cross-file.md#vize-croquis-cf-watcher-outside-setup-bad)

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

[良い](./cross-file.md#vize-croquis-cf-watcher-outside-setup-good)

[カテゴリの例](./cross-file.md#vize-croquis-cf-watcher-outside-setup) · [ルール参照](./project/vize-croquis-cf-watcher-outside-setup.md)
