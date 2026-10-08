---
title: 型と script のルール
---

# 型と script のルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`script/component-options-name-casing`](./all.md#script-component-options-name-casing) | [悪い例](./all.md#script-component-options-name-casing-bad) · [良い例](./all.md#script-component-options-name-casing-good) | コンポーネントの name オプションを PascalCase に揃えます。 |
| [`script/custom-event-name-casing`](./all.md#script-custom-event-name-casing) | [悪い例](./all.md#script-custom-event-name-casing-bad) · [良い例](./all.md#script-custom-event-name-casing-good) | emit するカスタムイベント名を指定した形式に揃えます。 |
| [`script/define-emits-declaration`](./all.md#script-define-emits-declaration) | [悪い例](./all.md#script-define-emits-declaration-bad) · [良い例](./all.md#script-define-emits-declaration-good) | defineEmits を型による宣言形式に揃えます。 |
| [`script/define-macros-order`](./all.md#script-define-macros-order) | [悪い例](./all.md#script-define-macros-order-bad) · [良い例](./all.md#script-define-macros-order-good) | script setup のコンパイラーマクロを一定の順に宣言します。 |
| [`script/define-props-declaration`](./all.md#script-define-props-declaration) | [悪い例](./all.md#script-define-props-declaration-bad) · [良い例](./all.md#script-define-props-declaration-good) | defineProps を型による宣言形式に揃えます。 |
| [`script/define-props-destructuring`](./all.md#script-define-props-destructuring) | [悪い例](./all.md#script-define-props-destructuring-bad) · [良い例](./all.md#script-define-props-destructuring-good) | defineProps の分割代入スタイルを指定した方針に揃えます。 |
| [`script/no-arrow-functions-in-watch`](./all.md#script-no-arrow-functions-in-watch) | [悪い例](./all.md#script-no-arrow-functions-in-watch-bad) · [良い例](./all.md#script-no-arrow-functions-in-watch-good) | Options API の watch に this を持たないアロー関数を使う箇所を検出します。 |
| [`script/no-async-in-computed`](./all.md#script-no-async-in-computed) | [悪い例](./all.md#script-no-async-in-computed-bad) · [良い例](./all.md#script-no-async-in-computed-good) | computed の getter で非同期関数を使う箇所を検出します。 |
| [`script/no-boolean-default`](./all.md#script-no-boolean-default) | [悪い例](./all.md#script-no-boolean-default-bad) · [良い例](./all.md#script-no-boolean-default-good) | Boolean prop の冗長な default を検出します。 |
| [`script/no-deep-destructure-in-props`](./all.md#script-no-deep-destructure-in-props) | [悪い例](./all.md#script-no-deep-destructure-in-props-bad) · [良い例](./all.md#script-no-deep-destructure-in-props-good) | defineProps の深い分割代入を検出します。 |
| [`script/no-deprecated-data-object-declaration`](./all.md#script-no-deprecated-data-object-declaration) | [悪い例](./all.md#script-no-deprecated-data-object-declaration-bad) · [良い例](./all.md#script-no-deprecated-data-object-declaration-good) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 |
| [`script/no-deprecated-destroyed-lifecycle`](./all.md#script-no-deprecated-destroyed-lifecycle) | [悪い例](./all.md#script-no-deprecated-destroyed-lifecycle-bad) · [良い例](./all.md#script-no-deprecated-destroyed-lifecycle-good) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。 |
| [`script/no-deprecated-dollar-listeners-api`](./all.md#script-no-deprecated-dollar-listeners-api) | [悪い例](./all.md#script-no-deprecated-dollar-listeners-api-bad) · [良い例](./all.md#script-no-deprecated-dollar-listeners-api-good) | Vue 3 で $attrs に統合された $listeners を検出します。 |
| [`script/no-deprecated-dollar-scopedslots-api`](./all.md#script-no-deprecated-dollar-scopedslots-api) | [悪い例](./all.md#script-no-deprecated-dollar-scopedslots-api-bad) · [良い例](./all.md#script-no-deprecated-dollar-scopedslots-api-good) | Vue 3 で $slots に統合された $scopedSlots を検出します。 |
| [`script/no-deprecated-events-api`](./all.md#script-no-deprecated-events-api) | [悪い例](./all.md#script-no-deprecated-events-api-bad) · [良い例](./all.md#script-no-deprecated-events-api-good) | Vue 3 で削除された $on / $off / $once を検出します。 |
| [`script/no-deprecated-props-default-this`](./all.md#script-no-deprecated-props-default-this) | [悪い例](./all.md#script-no-deprecated-props-default-this-bad) · [良い例](./all.md#script-no-deprecated-props-default-this-good) | prop の default / validator 内で使えなくなった this を検出します。 |
| [`script/no-dupe-keys`](./all.md#script-no-dupe-keys) | [悪い例](./all.md#script-no-dupe-keys-bad) · [良い例](./all.md#script-no-dupe-keys-good) | Options API の props / data / computed などのキー重複を検出します。 |
| [`script/no-duplicate-attr-inheritance`](./all.md#script-no-duplicate-attr-inheritance) | [悪い例](./all.md#script-no-duplicate-attr-inheritance-bad) · [良い例](./all.md#script-no-duplicate-attr-inheritance-good) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 |
| [`script/no-export-in-script-setup`](./all.md#script-no-export-in-script-setup) | [悪い例](./all.md#script-no-export-in-script-setup-bad) · [良い例](./all.md#script-no-export-in-script-setup-good) | script setup 内の export 文を検出します。 |
| [`script/no-get-current-instance`](./all.md#script-no-get-current-instance) | [悪い例](./all.md#script-no-get-current-instance-bad) · [良い例](./all.md#script-no-get-current-instance-good) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-import-compiler-macros`](./all.md#script-no-import-compiler-macros) | [悪い例](./all.md#script-no-import-compiler-macros-bad) · [良い例](./all.md#script-no-import-compiler-macros-good) | 自動的に使える Vue コンパイラーマクロの import を検出します。 |
| [`script/no-internal-imports`](./all.md#script-no-internal-imports) | [悪い例](./all.md#script-no-internal-imports-bad) · [良い例](./all.md#script-no-internal-imports-good) | Vue 内部モジュールからの import を検出します。 |
| [`script/no-multiple-slot-args`](./all.md#script-no-multiple-slot-args) | [悪い例](./all.md#script-no-multiple-slot-args-bad) · [良い例](./all.md#script-no-multiple-slot-args-good) | scoped slot 関数に複数の引数を渡す箇所を検出します。 |
| [`script/no-next-tick`](./all.md#script-no-next-tick) | [悪い例](./all.md#script-no-next-tick-bad) · [良い例](./all.md#script-no-next-tick-good) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](./all.md#script-no-options-api) | [悪い例](./all.md#script-no-options-api-bad) · [良い例](./all.md#script-no-options-api-good) | Vapor で Options API を使用する箇所を検出します。 |
| [`script/no-potential-component-option-typo`](./all.md#script-no-potential-component-option-typo) | [悪い例](./all.md#script-no-potential-component-option-typo-bad) · [良い例](./all.md#script-no-potential-component-option-typo-good) | Options API のオプション名の入力ミスを検出します。 |
| [`script/no-reactive-destructure`](./all.md#script-no-reactive-destructure) | [悪い例](./all.md#script-no-reactive-destructure-bad) · [良い例](./all.md#script-no-reactive-destructure-good) | reactive オブジェクトの反応性を失う分割代入を検出します。 |
| [`script/no-ref-as-operand`](./all.md#script-no-ref-as-operand) | [悪い例](./all.md#script-no-ref-as-operand-bad) · [良い例](./all.md#script-no-ref-as-operand-good) | ref を演算の値として使う際に .value を参照します。 |
| [`script/no-required-prop-with-default`](./all.md#script-no-required-prop-with-default) | [悪い例](./all.md#script-no-required-prop-with-default-bad) · [良い例](./all.md#script-no-required-prop-with-default-good) | required: true と default を同時に持つ prop を検出します。 |
| [`script/no-reserved-identifiers`](./all.md#script-no-reserved-identifiers) | [悪い例](./all.md#script-no-reserved-identifiers-bad) · [良い例](./all.md#script-no-reserved-identifiers-good) | Vue コンパイラーが予約した識別子の宣言を検出します。 |
| [`script/no-reserved-keys`](./all.md#script-no-reserved-keys) | [悪い例](./all.md#script-no-reserved-keys-bad) · [良い例](./all.md#script-no-reserved-keys-good) | Options API のキーに Vue の予約名を使う箇所を検出します。 |
| [`script/no-reserved-props`](./all.md#script-no-reserved-props) | [悪い例](./all.md#script-no-reserved-props-bad) · [良い例](./all.md#script-no-reserved-props-good) | prop 宣言に Vue の予約名を使う箇所を検出します。 |
| [`script/no-restricted-globals`](./all.md#script-no-restricted-globals) | [悪い例](./all.md#script-no-restricted-globals-bad) · [良い例](./all.md#script-no-restricted-globals-good) | 設定で禁止した実行環境のグローバル参照を検出します。 |
| [`script/no-restricted-members`](./all.md#script-no-restricted-members) | [悪い例](./all.md#script-no-restricted-members-bad) · [良い例](./all.md#script-no-restricted-members-good) | 設定で禁止した object.property へのアクセスを検出します。 |
| [`script/no-side-effects-in-computed-properties`](./all.md#script-no-side-effects-in-computed-properties) | [悪い例](./all.md#script-no-side-effects-in-computed-properties-bad) · [良い例](./all.md#script-no-side-effects-in-computed-properties-good) | Options API の computed getter 内の副作用を検出します。 |
| [`script/no-top-level-ref-in-script`](./all.md#script-no-top-level-ref-in-script) | [悪い例](./all.md#script-no-top-level-ref-in-script-bad) · [良い例](./all.md#script-no-top-level-ref-in-script-good) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 |
| [`script/no-unstable-nested-components`](./all.md#script-no-unstable-nested-components) | [悪い例](./all.md#script-no-unstable-nested-components-bad) · [良い例](./all.md#script-no-unstable-nested-components-good) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 |
| [`script/no-unused-emit-declarations`](./all.md#script-no-unused-emit-declarations) | [悪い例](./all.md#script-no-unused-emit-declarations-bad) · [良い例](./all.md#script-no-unused-emit-declarations-good) | 宣言したまま emit していないイベントを検出します。 |
| [`script/no-use-computed-property-like-method`](./all.md#script-no-use-computed-property-like-method) | [悪い例](./all.md#script-no-use-computed-property-like-method-bad) · [良い例](./all.md#script-no-use-computed-property-like-method-good) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 |
| [`script/no-with-defaults`](./all.md#script-no-with-defaults) | [悪い例](./all.md#script-no-with-defaults-bad) · [良い例](./all.md#script-no-with-defaults-good) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 |
| [`script/prefer-computed`](./all.md#script-prefer-computed) | [悪い例](./all.md#script-prefer-computed-bad) · [良い例](./all.md#script-prefer-computed-good) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 |
| [`script/prefer-define-options`](./all.md#script-prefer-define-options) | [悪い例](./all.md#script-prefer-define-options-bad) · [良い例](./all.md#script-prefer-define-options-good) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 |
| [`script/prefer-import-from-vue`](./all.md#script-prefer-import-from-vue) | [悪い例](./all.md#script-prefer-import-from-vue-bad) · [良い例](./all.md#script-prefer-import-from-vue-good) | 内部パッケージではなく vue から import します。 |
| [`script/prefer-ref-over-reactive`](./all.md#script-prefer-ref-over-reactive) | [悪い例](./all.md#script-prefer-ref-over-reactive-bad) · [良い例](./all.md#script-prefer-ref-over-reactive-good) | 状態管理に reactive() より ref() を使う方針を適用します。 |
| [`script/prefer-use-attrs`](./all.md#script-prefer-use-attrs) | [悪い例](./all.md#script-prefer-use-attrs-bad) · [良い例](./all.md#script-prefer-use-attrs-good) | setup の context.attrs を useAttrs() に置き換えます。 |
| [`script/prefer-use-id`](./all.md#script-prefer-use-id) | [悪い例](./all.md#script-prefer-use-id-bad) · [良い例](./all.md#script-prefer-use-id-good) | 一意な ID の生成に Vue 3.5 の useId() を使います。 |
| [`script/prefer-use-slots`](./all.md#script-prefer-use-slots) | [悪い例](./all.md#script-prefer-use-slots-bad) · [良い例](./all.md#script-prefer-use-slots-good) | setup の context.slots を useSlots() に置き換えます。 |
| [`script/prefer-use-template-ref`](./all.md#script-prefer-use-template-ref) | [悪い例](./all.md#script-prefer-use-template-ref-bad) · [良い例](./all.md#script-prefer-use-template-ref-good) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 |
| [`script/require-default-prop`](./all.md#script-require-default-prop) | [悪い例](./all.md#script-require-default-prop-bad) · [良い例](./all.md#script-require-default-prop-good) | 任意指定で Boolean ではない prop に default を用意します。 |
| [`script/require-explicit-emits`](./all.md#script-require-explicit-emits) | [悪い例](./all.md#script-require-explicit-emits-bad) · [良い例](./all.md#script-require-explicit-emits-good) | emit するイベントを defineEmits または emits に宣言します。 |
| [`script/require-explicit-slots`](./all.md#script-require-explicit-slots) | [悪い例](./all.md#script-require-explicit-slots-bad) · [良い例](./all.md#script-require-explicit-slots-good) | useSlots() で使う slot を defineSlots の型で宣言します。 |
| [`script/require-function-return-type`](./all.md#script-require-function-return-type) | [悪い例](./all.md#script-require-function-return-type-bad) · [良い例](./all.md#script-require-function-return-type-good) | 関数に戻り値の型注釈を指定します。 |
| [`script/require-prop-type-constructor`](./all.md#script-require-prop-type-constructor) | [悪い例](./all.md#script-require-prop-type-constructor-bad) · [良い例](./all.md#script-require-prop-type-constructor-good) | prop の type に文字列ではなくコンストラクターを指定します。 |
| [`script/require-prop-types`](./all.md#script-require-prop-types) | [悪い例](./all.md#script-require-prop-types-bad) · [良い例](./all.md#script-require-prop-types-good) | 各 prop の型を宣言します。 |
| [`script/require-symbol-provide`](./all.md#script-require-symbol-provide) | [悪い例](./all.md#script-require-symbol-provide-bad) · [良い例](./all.md#script-require-symbol-provide-good) | provide / inject のキーに衝突しにくい Symbol を使います。 |
| [`script/require-typed-object-prop`](./all.md#script-require-typed-object-prop) | [悪い例](./all.md#script-require-typed-object-prop-bad) · [良い例](./all.md#script-require-typed-object-prop-good) | Object / Array の prop に具体的な型を指定します。 |
| [`script/require-typed-ref`](./all.md#script-require-typed-ref) | [悪い例](./all.md#script-require-typed-ref-bad) · [良い例](./all.md#script-require-typed-ref-good) | 空・null・undefined で初期化する ref() に型引数を指定します。 |
| [`script/require-valid-default-prop`](./all.md#script-require-valid-default-prop) | [悪い例](./all.md#script-require-valid-default-prop-bad) · [良い例](./all.md#script-require-valid-default-prop-good) | prop の default を宣言した型に合う値にします。 |
| [`script/return-in-computed-property`](./all.md#script-return-in-computed-property) | [悪い例](./all.md#script-return-in-computed-property-bad) · [良い例](./all.md#script-return-in-computed-property-good) | computed の getter に値を返す return を用意します。 |
| [`script/return-in-emits-validator`](./all.md#script-return-in-emits-validator) | [悪い例](./all.md#script-return-in-emits-validator-bad) · [良い例](./all.md#script-return-in-emits-validator-good) | Options API の emits validator に戻り値を用意します。 |
| [`script/valid-define-emits`](./all.md#script-valid-define-emits) | [悪い例](./all.md#script-valid-define-emits-bad) · [良い例](./all.md#script-valid-define-emits-good) | defineEmits の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-define-options`](./all.md#script-valid-define-options) | [悪い例](./all.md#script-valid-define-options-bad) · [良い例](./all.md#script-valid-define-options-good) | defineOptions の引数と使用回数を検査します。 |
| [`script/valid-define-props`](./all.md#script-valid-define-props) | [悪い例](./all.md#script-valid-define-props-bad) · [良い例](./all.md#script-valid-define-props-good) | defineProps の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-next-tick`](./all.md#script-valid-next-tick) | [悪い例](./all.md#script-valid-next-tick-bad) · [良い例](./all.md#script-valid-next-tick-good) | nextTick() の完了を await、then、または callback で扱います。 |
| [`type/no-floating-promises`](./all.md#type-no-floating-promises) | [悪い例](./all.md#type-no-floating-promises-bad) · [良い例](./all.md#type-no-floating-promises-good) | 処理しないまま放置された Promise を検出します。 |
| [`type/no-reactivity-loss`](./all.md#type-no-reactivity-loss) | [悪い例](./all.md#type-no-reactivity-loss-bad) · [良い例](./all.md#type-no-reactivity-loss-good) | 代入や呼び出しによって反応性を失うスナップショットを検出します。 |
| [`type/no-unsafe-template-binding`](./all.md#type-no-unsafe-template-binding) | [悪い例](./all.md#type-no-unsafe-template-binding-bad) · [良い例](./all.md#type-no-unsafe-template-binding-good) | テンプレートで安全でない型の値を使用する箇所を検出します。 |
| [`type/require-typed-emits`](./all.md#type-require-typed-emits) | [悪い例](./all.md#type-require-typed-emits-bad) · [良い例](./all.md#type-require-typed-emits-good) | defineEmits に型定義を指定します。 |
| [`type/require-typed-props`](./all.md#type-require-typed-props) | [悪い例](./all.md#type-require-typed-props-bad) · [良い例](./all.md#type-require-typed-props-good) | defineProps に型定義を指定します。 |
| [`type/strict-boolean-expressions`](./all.md#type-strict-boolean-expressions) | [悪い例](./all.md#type-strict-boolean-expressions-bad) · [良い例](./all.md#type-strict-boolean-expressions-good) | script とテンプレートの条件式で安全な真偽判定を使います。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
