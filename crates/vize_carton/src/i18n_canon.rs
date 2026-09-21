//! `<code>.description` for the SFC type checker's codes
//! (`vize_canon::sfc_typecheck`, catalogued as `canon/<code>`) and the SFC
//! compiler's codes (`SfcError.code` in `vize_atelier_sfc` and the
//! `vize_croquis` block parser, catalogued as `sfc/<CODE>`) — P4-14b.
//!
//! Like the cross-file codes, these messages are built per finding, so the
//! catalogue describes each code. The TypeScript-numbered codes
//! (`TypeErrorCode`) already carry `ts/<n>.message` and `ts/<n>.help` in the
//! JSON catalogue; `tests/tooling/davinci-diagnostic-catalog.test.ts`
//! enumerates all three families from their sources and fails on a gap.
//! Registered by [`crate::i18n_cross_file`].

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "canon/untyped-props.description",
        "Report `defineProps()` without a type definition",
        "型定義のない `defineProps()` を報告する",
        "报告没有类型定义的 `defineProps()`",
    ),
    (
        "canon/untyped-prop.description",
        "Report a prop declared without a type",
        "型を指定せずに宣言された props を報告する",
        "报告未指定类型就声明的 prop",
    ),
    (
        "canon/untyped-emits.description",
        "Report `defineEmits()` without a type definition",
        "型定義のない `defineEmits()` を報告する",
        "报告没有类型定义的 `defineEmits()`",
    ),
    (
        "canon/untyped-emit.description",
        "Report an event declared without a payload type",
        "ペイロードの型を指定せずに宣言されたイベントを報告する",
        "报告未指定载荷类型就声明的事件",
    ),
    (
        "canon/undefined-binding.description",
        "Report a template reference that `<script setup>` neither declares nor imports",
        "`<script setup>` で宣言もインポートもされていない、テンプレート内の参照を報告する",
        "报告 `<script setup>` 中既未声明也未导入的模板引用",
    ),
    (
        "canon/reactivity-loss.description",
        "Report destructuring, extracting or reassigning that cuts a value off from its reactive source",
        "分割代入・値の取り出し・再代入によって値がリアクティブな元から切り離されることを報告する",
        "报告因解构、提取或重新赋值而使值脱离其响应式来源",
    ),
    (
        "canon/invalid-export.description",
        "Report an export from `<script setup>`, which cannot export bindings",
        "バインディングをエクスポートできない `<script setup>` からのエクスポートを報告する",
        "报告从不能导出绑定的 `<script setup>` 中进行的导出",
    ),
    (
        "canon/fallthrough-attrs.description",
        "Report a multi-root component that may drop fallthrough attributes",
        "フォールスルー属性を取りこぼすおそれのある、ルート要素が複数のコンポーネントを報告する",
        "报告可能丢失透传属性的多根组件",
    ),
    (
        "canon/parse-error.description",
        "Report an SFC that could not be split into its blocks",
        "ブロックに分割できなかった SFC を報告する",
        "报告无法拆分为各个块的 SFC",
    ),
    (
        "canon/template-parse-error.description",
        "Report a template that could not be parsed for type checking",
        "型チェックのために解析できなかったテンプレートを報告する",
        "报告无法为类型检查而解析的模板",
    ),
    (
        "canon/patterned-template.description",
        "Report invalid `v-match` / `v-when` pattern syntax in a template",
        "テンプレート内の `v-match`・`v-when` のパターン構文の誤りを報告する",
        "报告模板中无效的 `v-match` / `v-when` 模式语法",
    ),
    (
        "canon/script-parse-error.description",
        "Report a script block that could not be parsed",
        "解析できなかった script ブロックを報告する",
        "报告无法解析的 script 块",
    ),
    (
        "canon/module-level-state.description",
        "Report module-level reactive state, which SSR shares across requests",
        "SSR ではリクエスト間で共有されてしまう、モジュールレベルのリアクティブな状態を報告する",
        "报告在 SSR 中会被多个请求共享的模块级响应式状态",
    ),
    (
        "canon/module-level-watch.description",
        "Report a module-level watcher, which is never cleaned up",
        "後始末されることのない、モジュールレベルのウォッチャーを報告する",
        "报告永远不会被清理的模块级侦听器",
    ),
    (
        "canon/module-level-computed.description",
        "Report a module-level `computed`, which is never cleaned up",
        "後始末されることのない、モジュールレベルの `computed` を報告する",
        "报告永远不会被清理的模块级 `computed`",
    ),
    (
        "canon/module-level-provide.description",
        "Report `provide()` outside `setup()` or `<script setup>`",
        "`setup()` や `<script setup>` の外での `provide()` を報告する",
        "报告在 `setup()` 或 `<script setup>` 之外调用的 `provide()`",
    ),
    (
        "canon/module-level-inject.description",
        "Report `inject()` outside `setup()` or `<script setup>`",
        "`setup()` や `<script setup>` の外での `inject()` を報告する",
        "报告在 `setup()` 或 `<script setup>` 之外调用的 `inject()`",
    ),
    (
        "canon/module-level-lifecycle.description",
        "Report lifecycle hooks registered outside `setup()` or `<script setup>`",
        "`setup()` や `<script setup>` の外で登録されるライフサイクルフックを報告する",
        "报告在 `setup()` 或 `<script setup>` 之外注册的生命周期钩子",
    ),
    (
        "sfc/TEMPLATE_ERROR.description",
        "The template failed to compile; the compiler errors it lists say why",
        "テンプレートのコンパイルに失敗しました。理由は一覧されたコンパイラーエラーを参照してください",
        "模板编译失败；原因请参阅其中列出的编译器错误",
    ),
    (
        "sfc/VAPOR_TEMPLATE_ERROR.description",
        "The template failed to compile in Vapor mode",
        "Vapor モードでのテンプレートのコンパイルに失敗しました",
        "模板在 Vapor 模式下编译失败",
    ),
    (
        "sfc/SCRIPT_SETUP_MACRO_SCOPE.description",
        "A compiler macro's arguments reference a variable declared inside `<script setup>`, but they are hoisted outside `setup()`",
        "コンパイラーマクロの引数は `setup()` の外へ巻き上げられるため、`<script setup>` 内で宣言した変数は参照できません",
        "编译器宏的参数会被提升到 `setup()` 之外，因此不能引用 `<script setup>` 中声明的变量",
    ),
    (
        "sfc/DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE.description",
        "A destructured prop's default value does not match its declared type",
        "分割代入した props のデフォルト値が、宣言された型と合っていません",
        "解构 prop 的默认值与其声明的类型不符",
    ),
    (
        "sfc/DEFINE_PROPS_DESTRUCTURE_PARSE.description",
        "A destructured `defineProps()` could not be parsed for transformation",
        "分割代入された `defineProps()` を変換のために解析できませんでした",
        "无法解析解构的 `defineProps()` 以进行转换",
    ),
    (
        "sfc/CSS_MODULE_COMPILE_ERROR.description",
        "A `<style module>` block failed to compile",
        "`<style module>` ブロックのコンパイルに失敗しました",
        "`<style module>` 块编译失败",
    ),
    (
        "sfc/STANDALONE_EXTERNAL_IMPORT.description",
        "Standalone output still imports non-Vue modules that a CDN build must provide separately",
        "スタンドアロン出力に Vue 以外のモジュールのインポートが残っており、CDN で使うにはそれらを別途用意する必要があります",
        "独立输出中仍导入了非 Vue 模块，通过 CDN 使用时需要另行提供这些依赖",
    ),
    (
        "sfc/VAPOR_SSR_FALLBACK.description",
        "Vapor SSR is not supported yet, so the component falls back to standard SSR output",
        "Vapor の SSR にはまだ対応していないため、通常の SSR 出力にフォールバックします",
        "尚不支持 Vapor SSR，因此回退为标准 SSR 输出",
    ),
    (
        "sfc/V_MODEL_CONST_REACTIVE_DEMOTED.description",
        "`v-model` writes to a `const` reactive binding, so the compiler turned the declaration into `let`",
        "`v-model` が `const` のリアクティブなバインディングに書き込むため、コンパイラーが宣言を `let` に変更しました",
        "`v-model` 会写入 `const` 响应式绑定，因此编译器已将该声明改为 `let`",
    ),
    (
        "sfc/V_MATCH_SYNTAX.description",
        "A root `v-match` template is malformed",
        "ルートの `v-match` テンプレートの書き方が正しくありません",
        "根级 `v-match` 模板格式错误",
    ),
    (
        "sfc/DUPLICATE_TEMPLATE.description",
        "An SFC has more than one `<template>` block",
        "SFC に `<template>` ブロックが複数あります",
        "SFC 中有多个 `<template>` 块",
    ),
    (
        "sfc/DUPLICATE_SCRIPT_SETUP.description",
        "An SFC has more than one `<script setup>` block",
        "SFC に `<script setup>` ブロックが複数あります",
        "SFC 中有多个 `<script setup>` 块",
    ),
    (
        "sfc/DUPLICATE_SCRIPT.description",
        "An SFC has more than one plain `<script>` block",
        "SFC に通常の `<script>` ブロックが複数あります",
        "SFC 中有多个普通 `<script>` 块",
    ),
    (
        "sfc/MALFORMED_BLOCK.description",
        "A top-level SFC block is malformed, such as an unterminated tag",
        "SFC のトップレベルのブロックが、閉じられていないタグなどで壊れています",
        "SFC 顶层块格式错误，例如标签未闭合",
    ),
];
