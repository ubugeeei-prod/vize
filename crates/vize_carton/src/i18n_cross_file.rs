//! `<code>.description` for every cross-file diagnostic code
//! (`vize_croquis_cf`, `vize:croquis/cf/<name>`) and every stage-verifier
//! code (`S2V…` in `vize_s2`, `S3V…` in `vize_impeto`) — P4-14b.
//!
//! These producers build their messages from the facts at hand, so the
//! catalogue describes each code rather than templating its message: the
//! description is what `vize explain` shows and what a reader needs before
//! the specific message makes sense. Attributes, emits, provide/inject, IDs,
//! SSR boundaries, dependencies, props and slots are here; setup context and
//! reactivity in [`crate::i18n_cross_file_reactivity`]; verifier codes in
//! [`crate::i18n_verifier`]; type-checker and SFC compiler codes in
//! [`crate::i18n_canon`]. `tests/tooling/davinci-diagnostic-catalog.test.ts`
//! enumerates the codes from their producers' sources and fails on one
//! without a description in every locale.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert the cross-file and verifier descriptions into the message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    let tables = [
        ENTRIES,
        crate::i18n_cross_file_reactivity::ENTRIES,
        crate::i18n_verifier::ENTRIES,
        crate::i18n_canon::ENTRIES,
    ];
    for &(key, en, ja, zh) in tables.into_iter().flatten() {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "vize:croquis/cf/unused-attrs.description",
        "Report attributes passed to a component that neither declares them nor renders `$attrs`",
        "props として宣言も `$attrs` での描画もされない属性をコンポーネントに渡していることを報告する",
        "报告传给组件、但既未声明为 prop 也未通过 `$attrs` 渲染的属性",
    ),
    (
        "vize:croquis/cf/inherit-attrs-unused.description",
        "Report `inheritAttrs: false` on a component that never uses `$attrs`",
        "`inheritAttrs: false` なのに `$attrs` を使っていないコンポーネントを報告する",
        "报告设置了 `inheritAttrs: false` 却从未使用 `$attrs` 的组件",
    ),
    (
        "vize:croquis/cf/multi-root-attrs.description",
        "Report a multi-root component that receives attributes without `v-bind=\"$attrs\"`",
        "ルート要素が複数あるのに `v-bind=\"$attrs\"` を指定していないコンポーネントを報告する",
        "报告有多个根元素却没有绑定 `v-bind=\"$attrs\"` 的组件",
    ),
    (
        "vize:croquis/cf/undeclared-emit.description",
        "Report events emitted without being declared in `defineEmits`",
        "`defineEmits` で宣言せずに発行しているイベントを報告する",
        "报告未在 `defineEmits` 中声明就触发的事件",
    ),
    (
        "vize:croquis/cf/unused-emit.description",
        "Report declared events that are never emitted",
        "宣言したのに一度も発行されないイベントを報告する",
        "报告已声明却从未触发的事件",
    ),
    (
        "vize:croquis/cf/unmatched-listener.description",
        "Report a parent listening for an event the child never emits",
        "子コンポーネントが発行しないイベントを親が購読していることを報告する",
        "报告父组件监听了子组件从不触发的事件",
    ),
    (
        "vize:croquis/cf/unhandled-event.description",
        "Report an emitted event that no ancestor listens for",
        "発行したイベントを購読している祖先コンポーネントがないことを報告する",
        "报告触发的事件没有任何祖先组件监听",
    ),
    (
        "vize:croquis/cf/event-modifier.description",
        "Report event modifiers such as `.stop` or `.prevent` that may change behavior across components",
        "`.stop` や `.prevent` などのイベント修飾子がコンポーネントをまたいで挙動を変えるおそれを報告する",
        "报告 `.stop`、`.prevent` 等事件修饰符可能跨组件改变行为",
    ),
    (
        "vize:croquis/cf/unmatched-inject.description",
        "Report an `inject()` key that no ancestor provides",
        "どの祖先でも `provide()` されていない `inject()` のキーを報告する",
        "报告没有任何祖先 `provide()` 的 `inject()` 键",
    ),
    (
        "vize:croquis/cf/unused-provide.description",
        "Report a `provide()` key that no descendant injects",
        "どの子孫でも `inject()` されない `provide()` のキーを報告する",
        "报告没有任何后代 `inject()` 的 `provide()` 键",
    ),
    (
        "vize:croquis/cf/provide-inject-type.description",
        "Report a type mismatch between a provided value and its injection",
        "`provide()` した値と `inject()` 側の型の不一致を報告する",
        "报告 `provide()` 的值与 `inject()` 端的类型不一致",
    ),
    (
        "vize:croquis/cf/provide-without-symbol.description",
        "Report `provide()` with a string key instead of a `Symbol` or `InjectionKey`",
        "`Symbol` や `InjectionKey` ではなく文字列キーで `provide()` していることを報告する",
        "报告使用字符串键而不是 `Symbol` 或 `InjectionKey` 调用 `provide()`",
    ),
    (
        "vize:croquis/cf/inject-without-symbol.description",
        "Report `inject()` with a string key instead of a `Symbol` or `InjectionKey`",
        "`Symbol` や `InjectionKey` ではなく文字列キーで `inject()` していることを報告する",
        "报告使用字符串键而不是 `Symbol` 或 `InjectionKey` 调用 `inject()`",
    ),
    (
        "vize:croquis/cf/non-reactive-provide.description",
        "Report a `provide()` of a non-reactive value that consumers may expect to update",
        "利用側が更新を期待しうる非リアクティブな値を `provide()` していることを報告する",
        "报告 `provide()` 了使用方可能期望会更新的非响应式值",
    ),
    (
        "vize:croquis/cf/duplicate-id.description",
        "Report the same element `id` rendered by more than one component",
        "複数のコンポーネントが同じ要素 `id` を描画していることを報告する",
        "报告多个组件渲染了相同的元素 `id`",
    ),
    (
        "vize:croquis/cf/non-unique-id.description",
        "Report an `id` inside `v-for` that may repeat across items",
        "`v-for` の中で項目ごとに重複しうる `id` を報告する",
        "报告 `v-for` 中可能在各项之间重复的 `id`",
    ),
    (
        "vize:croquis/cf/browser-api-ssr.description",
        "Report browser-only APIs used where the code may run during SSR",
        "SSR 中に実行されうる箇所でのブラウザー専用 API の使用を報告する",
        "报告在可能于 SSR 期间运行的代码中使用仅限浏览器的 API",
    ),
    (
        "vize:croquis/cf/async-no-suspense.description",
        "Report an async component rendered without a `<Suspense>` boundary",
        "`<Suspense>` 境界なしで描画される非同期コンポーネントを報告する",
        "报告在没有 `<Suspense>` 边界的情况下渲染的异步组件",
    ),
    (
        "vize:croquis/cf/hydration-risk.description",
        "Report client-only content that risks a hydration mismatch",
        "ハイドレーションの不一致を招くおそれのあるクライアント専用の内容を報告する",
        "报告可能导致水合不匹配的仅客户端内容",
    ),
    (
        "vize:croquis/cf/uncaught-error.description",
        "Report errors thrown with no `onErrorCaptured` in any ancestor",
        "どの祖先にも `onErrorCaptured` がないまま投げられるエラーを報告する",
        "报告在任何祖先中都没有 `onErrorCaptured` 的情况下抛出的错误",
    ),
    (
        "vize:croquis/cf/missing-suspense.description",
        "Report async work in `setup` with no `<Suspense>` boundary above it",
        "上位に `<Suspense>` 境界がないまま `setup` で行われる非同期処理を報告する",
        "报告上方没有 `<Suspense>` 边界时在 `setup` 中进行的异步操作",
    ),
    (
        "vize:croquis/cf/suspense-no-fallback.description",
        "Report a nested `<Suspense>` without a fallback",
        "fallback を持たない入れ子の `<Suspense>` を報告する",
        "报告没有 fallback 的嵌套 `<Suspense>`",
    ),
    (
        "vize:croquis/cf/circular-dep.description",
        "Report a circular dependency between modules",
        "モジュール間の循環依存を報告する",
        "报告模块之间的循环依赖",
    ),
    (
        "vize:croquis/cf/deep-import.description",
        "Report import chains deep enough to slow loading",
        "読み込みを遅くするほど深いインポートの連鎖を報告する",
        "报告深到足以拖慢加载的导入链",
    ),
    (
        "vize:croquis/cf/unregistered-component.description",
        "Report a component used in a template but neither imported nor registered",
        "テンプレートで使われているのにインポートも登録もされていないコンポーネントを報告する",
        "报告在模板中使用却既未导入也未注册的组件",
    ),
    (
        "vize:croquis/cf/unresolved-import.description",
        "Report an import specifier that resolves to no file",
        "どのファイルにも解決できないインポート指定子を報告する",
        "报告无法解析到任何文件的导入说明符",
    ),
    (
        "vize:croquis/cf/undeclared-prop.description",
        "Report a prop passed to a component that does not declare it in `defineProps`",
        "`defineProps` で宣言されていない props をコンポーネントに渡していることを報告する",
        "报告向组件传入其 `defineProps` 中未声明的 prop",
    ),
    (
        "vize:croquis/cf/missing-required-prop.description",
        "Report a required prop that is not passed",
        "必須の props が渡されていないことを報告する",
        "报告未传入必需的 prop",
    ),
    (
        "vize:croquis/cf/prop-type-mismatch.description",
        "Report a literal prop value whose type does not match the declaration",
        "宣言された型と合わないリテラルの props の値を報告する",
        "报告与声明类型不符的字面量 prop 值",
    ),
    (
        "vize:croquis/cf/undefined-slot.description",
        "Report a slot the child component does not define in `defineSlots`",
        "子コンポーネントが `defineSlots` で定義していないスロットの使用を報告する",
        "报告使用了子组件未在 `defineSlots` 中定义的插槽",
    ),
];
