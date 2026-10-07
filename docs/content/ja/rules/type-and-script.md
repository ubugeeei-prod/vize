---
title: 型と script のルール
---

# 型と script のルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| Rule | 目的 |
| --- | --- |
| [`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | コンポーネントの name オプションを PascalCase に揃えます。 |
| [`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | emit するカスタムイベント名を指定した形式に揃えます。 |
| [`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | defineEmits を型による宣言形式に揃えます。 |
| [`script/define-macros-order`](./reference/script-define-macros-order.md) | script setup のコンパイラーマクロを一定の順に宣言します。 |
| [`script/define-props-declaration`](./reference/script-define-props-declaration.md) | defineProps を型による宣言形式に揃えます。 |
| [`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | defineProps の分割代入スタイルを指定した方針に揃えます。 |
| [`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Options API の watch に this を持たないアロー関数を使う箇所を検出します。 |
| [`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | computed の getter で非同期関数を使う箇所を検出します。 |
| [`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Boolean prop の冗長な default を検出します。 |
| [`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | defineProps の深い分割代入を検出します。 |
| [`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 |
| [`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。 |
| [`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Vue 3 で $attrs に統合された $listeners を検出します。 |
| [`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Vue 3 で $slots に統合された $scopedSlots を検出します。 |
| [`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Vue 3 で削除された $on / $off / $once を検出します。 |
| [`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | prop の default / validator 内で使えなくなった this を検出します。 |
| [`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Options API の props / data / computed などのキー重複を検出します。 |
| [`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 |
| [`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | script setup 内の export 文を検出します。 |
| [`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | 自動的に使える Vue コンパイラーマクロの import を検出します。 |
| [`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Vue 内部モジュールからの import を検出します。 |
| [`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | scoped slot 関数に複数の引数を渡す箇所を検出します。 |
| [`script/no-next-tick`](./reference/script-no-next-tick.md) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](./reference/script-no-options-api.md) | Vapor で Options API を使用する箇所を検出します。 |
| [`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Options API のオプション名の入力ミスを検出します。 |
| [`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | reactive オブジェクトの反応性を失う分割代入を検出します。 |
| [`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | ref を演算の値として使う際に .value を参照します。 |
| [`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | required: true と default を同時に持つ prop を検出します。 |
| [`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Vue コンパイラーが予約した識別子の宣言を検出します。 |
| [`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Options API のキーに Vue の予約名を使う箇所を検出します。 |
| [`script/no-reserved-props`](./reference/script-no-reserved-props.md) | prop 宣言に Vue の予約名を使う箇所を検出します。 |
| [`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | 設定で禁止した実行環境のグローバル参照を検出します。 |
| [`script/no-restricted-members`](./reference/script-no-restricted-members.md) | 設定で禁止した object.property へのアクセスを検出します。 |
| [`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Options API の computed getter 内の副作用を検出します。 |
| [`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 |
| [`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 |
| [`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | 宣言したまま emit していないイベントを検出します。 |
| [`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 |
| [`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 |
| [`script/prefer-computed`](./reference/script-prefer-computed.md) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 |
| [`script/prefer-define-options`](./reference/script-prefer-define-options.md) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 |
| [`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | 内部パッケージではなく vue から import します。 |
| [`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | 状態管理に reactive() より ref() を使う方針を適用します。 |
| [`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | setup の context.attrs を useAttrs() に置き換えます。 |
| [`script/prefer-use-id`](./reference/script-prefer-use-id.md) | 一意な ID の生成に Vue 3.5 の useId() を使います。 |
| [`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | setup の context.slots を useSlots() に置き換えます。 |
| [`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 |
| [`script/require-default-prop`](./reference/script-require-default-prop.md) | 任意指定で Boolean ではない prop に default を用意します。 |
| [`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | emit するイベントを defineEmits または emits に宣言します。 |
| [`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | useSlots() で使う slot を defineSlots の型で宣言します。 |
| [`script/require-function-return-type`](./reference/script-require-function-return-type.md) | 関数に戻り値の型注釈を指定します。 |
| [`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | prop の type に文字列ではなくコンストラクターを指定します。 |
| [`script/require-prop-types`](./reference/script-require-prop-types.md) | 各 prop の型を宣言します。 |
| [`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | provide / inject のキーに衝突しにくい Symbol を使います。 |
| [`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Object / Array の prop に具体的な型を指定します。 |
| [`script/require-typed-ref`](./reference/script-require-typed-ref.md) | 空・null・undefined で初期化する ref() に型引数を指定します。 |
| [`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | prop の default を宣言した型に合う値にします。 |
| [`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | computed の getter に値を返す return を用意します。 |
| [`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Options API の emits validator に戻り値を用意します。 |
| [`script/valid-define-emits`](./reference/script-valid-define-emits.md) | defineEmits の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-define-options`](./reference/script-valid-define-options.md) | defineOptions の引数と使用回数を検査します。 |
| [`script/valid-define-props`](./reference/script-valid-define-props.md) | defineProps の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-next-tick`](./reference/script-valid-next-tick.md) | nextTick() の完了を await、then、または callback で扱います。 |
| [`type/no-floating-promises`](./reference/type-no-floating-promises.md) | 処理しないまま放置された Promise を検出します。 |
| [`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | 代入や呼び出しによって反応性を失うスナップショットを検出します。 |
| [`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | テンプレートで安全でない型の値を使用する箇所を検出します。 |
| [`type/require-typed-emits`](./reference/type-require-typed-emits.md) | defineEmits に型定義を指定します。 |
| [`type/require-typed-props`](./reference/type-require-typed-props.md) | defineProps に型定義を指定します。 |
| [`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | script とテンプレートの条件式で安全な真偽判定を使います。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
