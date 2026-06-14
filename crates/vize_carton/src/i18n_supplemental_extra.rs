//! Additional supplemental i18n entries.
//!
//! `i18n_supplemental.rs` is already at the repository's source-length guard
//! baseline, so new keys are registered here instead. The `i18n` module merges
//! these into the global translator at startup alongside the original
//! supplemental entries.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every extra supplemental entry into the locale message maps.
///
/// The maps are indexed by `Locale::index()`: `[0] = En`, `[1] = Ja`, `[2] = Zh`.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// Extra supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/v-on-handler-style
    (
        "vue/v-on-handler-style.description",
        "Enforce writing v-on handlers as a method reference or an inline function",
        "v-onハンドラをメソッド参照またはインライン関数として記述することを強制する",
        "强制将v-on处理函数写成方法引用或内联函数",
    ),
    (
        "vue/v-on-handler-style.message",
        "Prefer a method reference or an inline function over an inline statement for this v-on handler",
        "このv-onハンドラにはインライン文ではなく、メソッド参照またはインライン関数を使用してください",
        "此v-on处理函数应优先使用方法引用或内联函数，而非内联语句",
    ),
    (
        "vue/v-on-handler-style.help",
        "Write the handler as a method reference (e.g. @click=\"handler\") or an inline function (e.g. @click=\"() => count++\") instead of an inline statement.",
        "ハンドラをインライン文ではなく、メソッド参照（例: @click=\"handler\"）またはインライン関数（例: @click=\"() => count++\"）として記述してください。",
        "请将处理函数写成方法引用（例如 @click=\"handler\"）或内联函数（例如 @click=\"() => count++\"），而不是内联语句。",
    ),
    // vue/this-in-template
    (
        "vue/this-in-template.description",
        "Disallow `this.` in template expressions",
        "テンプレート式での `this.` を禁止する",
        "禁止在模板表达式中使用 `this.`",
    ),
    (
        "vue/this-in-template.message",
        "Unexpected usage of 'this.' in a template expression.",
        "テンプレート式で予期しない 'this.' が使われています。",
        "模板表达式中出现了意外的 'this.'。",
    ),
    (
        "vue/this-in-template.help",
        "Remove the 'this.' prefix; Vue resolves template identifiers against the component instance automatically.",
        "'this.' を削除してください。Vue はテンプレートの識別子をコンポーネントインスタンスから自動的に解決します。",
        "请移除 'this.' 前缀；Vue 会自动从组件实例解析模板中的标识符。",
    ),
    // html/no-dupe-style-properties
    (
        "html/no-dupe-style-properties.description",
        "Disallow duplicate properties in inline style attributes",
        "インラインstyle属性内の重複するプロパティを禁止する",
        "禁止内联 style 属性中出现重复的属性",
    ),
    (
        "html/no-dupe-style-properties.message",
        "Duplicate property '{property}' in inline style",
        "インラインstyleにプロパティ '{property}' が重複しています",
        "内联 style 中存在重复的属性 '{property}'",
    ),
    (
        "html/no-dupe-style-properties.help",
        "Remove the duplicate declaration. When a property is declared more than once, only the last value applies, so the earlier ones are dead code.",
        "重複した宣言を削除してください。同じプロパティを複数回宣言しても最後の値だけが適用されるため、それより前の宣言は無効なコードです。",
        "请删除重复的声明。同一属性多次声明时只有最后一个值生效，因此前面的声明是无效代码。",
    ),
    // vue/no-v-text
    (
        "vue/no-v-text.description",
        "Disallow the v-text directive; prefer mustache interpolation",
        "v-textディレクティブを禁止し、マスタッシュ補間を推奨する",
        "禁止使用v-text指令；推荐使用胡子插值",
    ),
    (
        "vue/no-v-text.message",
        "Avoid the 'v-text' directive; use mustache interpolation {{ }} for text content instead",
        "'v-text' ディレクティブは避け、テキスト内容にはマスタッシュ補間 {{ }} を使用してください",
        "请避免使用 'v-text' 指令；文本内容请改用胡子插值 {{ }}",
    ),
    (
        "vue/no-v-text.help",
        "Replace `v-text=\"expr\"` with mustache interpolation in the element's content (e.g. `<div>{{ expr }}</div>`).",
        "`v-text=\"expr\"` を要素の内容のマスタッシュ補間に置き換えてください（例: `<div>{{ expr }}</div>`）。",
        "请将 `v-text=\"expr\"` 替换为元素内容中的胡子插值（例如 `<div>{{ expr }}</div>`）。",
    ),
    // vue/no-root-v-if
    (
        "vue/no-root-v-if.description",
        "Disallow v-if on the single root element of a template",
        "テンプレートの唯一のルート要素への v-if を禁止する",
        "禁止在模板的唯一根元素上使用 v-if",
    ),
    (
        "vue/no-root-v-if.message",
        "v-if on the single root element can make the whole component render nothing",
        "唯一のルート要素への v-if は、コンポーネント全体が何も描画しなくなる可能性があります",
        "在唯一根元素上使用 v-if 可能导致整个组件不渲染任何内容",
    ),
    (
        "vue/no-root-v-if.help",
        "Wrap the content in an always-present root element, or use v-show instead of v-if.",
        "内容を常に存在するルート要素で囲むか、v-if の代わりに v-show を使用してください。",
        "请将内容包裹在始终存在的根元素中，或使用 v-show 代替 v-if。",
    ),
    // vue/no-deprecated-html-element-is
    (
        "vue/no-deprecated-html-element-is.description",
        "Disallow the `is` attribute on native HTML elements",
        "ネイティブHTML要素への`is`属性を禁止する",
        "禁止在原生HTML元素上使用`is`属性",
    ),
    (
        "vue/no-deprecated-html-element-is.message",
        "the `is` attribute on native HTML elements (component substitution) was removed in Vue 3",
        "ネイティブHTML要素の`is`属性（コンポーネント置換）はVue 3で削除されました",
        "原生HTML元素上的`is`属性（组件替换）已在Vue 3中移除",
    ),
    (
        "vue/no-deprecated-html-element-is.help",
        "Use `<component :is=\"...\">` for dynamic components, or prefix the value with `vue:` (e.g. `is=\"vue:MyComponent\"`) for a customized built-in element.",
        "動的コンポーネントには`<component :is=\"...\">`を使うか、カスタマイズされた組み込み要素には値に`vue:`を付けてください（例: `is=\"vue:MyComponent\"`）。",
        "对于动态组件请使用`<component :is=\"...\">`，对于定制内置元素请在值前加`vue:`前缀（例如 `is=\"vue:MyComponent\"`）。",
    ),
    // vue/no-empty-component-block
    (
        "vue/no-empty-component-block.description",
        "Disallow empty SFC blocks such as <template></template>, <script></script>, or <style></style>",
        "<template></template> や <script></script>、<style></style> などの空のSFCブロックを禁止する",
        "禁止空的SFC块，例如 <template></template>、<script></script> 或 <style></style>",
    ),
    (
        "vue/no-empty-component-block.message",
        "The <{block}> block is empty",
        "<{block}> ブロックが空です",
        "<{block}> 块为空",
    ),
    (
        "vue/no-empty-component-block.help",
        "Add meaningful content to the block or remove it entirely.",
        "ブロックに意味のある内容を追加するか、ブロックごと削除してください。",
        "请为该块添加有意义的内容，或将其整体删除。",
    ),
    // vue/no-multiple-objects-in-class
    (
        "vue/no-multiple-objects-in-class.description",
        "Disallow multiple object literals inside a :class array binding",
        ":class配列バインディング内の複数のオブジェクトリテラルを禁止する",
        "禁止在:class数组绑定中使用多个对象字面量",
    ),
    (
        "vue/no-multiple-objects-in-class.message",
        "Multiple object literals in a :class array should be merged into a single object",
        ":class配列内の複数のオブジェクトリテラルは1つのオブジェクトにまとめるべきです",
        ":class数组中的多个对象字面量应合并为单个对象",
    ),
    (
        "vue/no-multiple-objects-in-class.help",
        "Merge the objects into one, e.g. :class=\"[{ a }, { b }]\" becomes :class=\"{ a, b }\".",
        "オブジェクトを1つにまとめてください。例: :class=\"[{ a }, { b }]\" は :class=\"{ a, b }\" になります。",
        "请将这些对象合并为一个，例如 :class=\"[{ a }, { b }]\" 改为 :class=\"{ a, b }\"。",
    ),
    // vue/valid-template-root
    (
        "vue/valid-template-root.description",
        "Enforce a valid `<template>` root for Vue 3 fragment semantics",
        "Vue 3のフラグメントセマンティクスに対して有効な `<template>` ルートを強制する",
        "强制 `<template>` 根符合 Vue 3 的片段语义",
    ),
    (
        "vue/valid-template-root.disallowed_root",
        "`<{tag}>` cannot be used as a root node of the template",
        "`<{tag}>` はテンプレートのルートノードとして使用できません",
        "`<{tag}>` 不能用作模板的根节点",
    ),
    (
        "vue/valid-template-root.help",
        "Wrap the content in a real root element (e.g. <div>) instead of using <template> or <slot> as the root node.",
        "<template> や <slot> をルートノードとして使わず、内容を実要素（例: <div>）で囲んでください。",
        "请用真实的根元素（例如 <div>）包裹内容，而不要把 <template> 或 <slot> 作为根节点。",
    ),
    // vue/no-v-for-template-key-on-child
    (
        "vue/no-v-for-template-key-on-child.description",
        "Disallow `key` on the child of a `<template v-for>`",
        "`<template v-for>`の子要素への`key`を禁止する",
        "禁止在`<template v-for>`的子元素上使用`key`",
    ),
    (
        "vue/no-v-for-template-key-on-child.message",
        "the `key` for a `<template v-for>` must be on the `<template>`, not on its child",
        "`<template v-for>`の`key`は子要素ではなく`<template>`自身に置く必要があります",
        "`<template v-for>`的`key`必须放在`<template>`上，而不是其子元素上",
    ),
    (
        "vue/no-v-for-template-key-on-child.help",
        "Move the `:key` up onto the `<template v-for>` element. In Vue 3 the key lives on the template, not the child (the reverse of Vue 2).",
        "`:key`を`<template v-for>`要素に移動してください。Vue 3ではキーは子要素ではなくtemplateに置きます（Vue 2とは逆）。",
        "请将`:key`移到`<template v-for>`元素上。在Vue 3中，key位于template而非子元素上（与Vue 2相反）。",
    ),
    // vue/require-toggle-inside-transition
    (
        "vue/require-toggle-inside-transition.description",
        "Require a toggle on the element wrapped by `<transition>`",
        "`<transition>`で囲む要素にトグルを必須にする",
        "要求`<transition>`包裹的元素具有切换条件",
    ),
    (
        "vue/require-toggle-inside-transition.message",
        "the element inside `<transition>` is expected to have a toggle such as `v-if`, `v-show`, or a bound `:key`",
        "`<transition>`内の要素には`v-if`、`v-show`、またはバインドされた`:key`などのトグルが必要です",
        "`<transition>`内的元素应具有切换条件，例如`v-if`、`v-show`或绑定的`:key`",
    ),
    (
        "vue/require-toggle-inside-transition.help",
        "Add `v-if`, `v-show`, `v-else`, `v-else-if`, or a bound `:key`, or use a dynamic `<component :is>`; otherwise the element never enters or leaves and the `<transition>` does nothing.",
        "`v-if`、`v-show`、`v-else`、`v-else-if`、バインドされた`:key`を追加するか、動的な`<component :is>`を使用してください。そうしないと要素が出入りせず、`<transition>`は何もしません。",
        "请添加`v-if`、`v-show`、`v-else`、`v-else-if`或绑定的`:key`，或使用动态`<component :is>`；否则元素永远不会进入或离开，`<transition>`将不起作用。",
    ),
    // vue/no-array-index-key
    (
        "vue/no-array-index-key.description",
        "Disallow using the v-for index variable directly as the :key",
        "v-forのインデックス変数をそのまま :key に使うことを禁止する",
        "禁止直接将 v-for 的索引变量用作 :key",
    ),
    (
        "vue/no-array-index-key.message",
        "Do not use the v-for index as :key; use a stable, unique identifier instead",
        "v-forのインデックスを :key に使わないでください。安定した一意の識別子を使用してください",
        "请勿将 v-for 的索引用作 :key；请改用稳定且唯一的标识符",
    ),
    (
        "vue/no-array-index-key.help",
        "Bind :key to a stable id from the item (e.g. :key=\"item.id\"). The index changes when the list is reordered or filtered, so Vue reuses the wrong element state.",
        ":key には項目の安定したid（例: :key=\"item.id\"）をバインドしてください。リストの並べ替えやフィルタリングでインデックスは変化するため、Vueが誤った要素の状態を再利用してしまいます。",
        "请将 :key 绑定到项目中稳定的 id（例如 :key=\"item.id\"）。当列表被重新排序或过滤时索引会变化，导致 Vue 复用错误的元素状态。",
    ),
    // vue/no-bare-strings-in-template
    (
        "vue/no-bare-strings-in-template.description",
        "Disallow raw human-readable text in the template that should be internationalized",
        "国際化すべきテンプレート内の生の人間可読テキストを禁止する",
        "禁止在模板中使用应当国际化的原始可读文本",
    ),
    (
        "vue/no-bare-strings-in-template.message",
        "Raw text should be wrapped in a translation function instead of being written directly in the template",
        "生のテキストはテンプレートに直接書くのではなく、翻訳関数で囲んでください",
        "原始文本应使用翻译函数包裹，而不是直接写在模板中",
    ),
    (
        "vue/no-bare-strings-in-template.help",
        "Move the text into a translation function, e.g. {{ $t('key') }} for content or :title=\"$t('key')\" for an attribute.",
        "テキストを翻訳関数に移してください。例: 内容には {{ $t('key') }}、属性には :title=\"$t('key')\" を使用します。",
        "请将文本移入翻译函数，例如内容使用 {{ $t('key') }}，属性使用 :title=\"$t('key')\"。",
    ),
    // vue/no-deprecated-filter
    (
        "vue/no-deprecated-filter.description",
        "Disallow deprecated Vue 2 filter syntax (the `|` pipe)",
        "非推奨の Vue 2 フィルター構文（`|` パイプ）を禁止する",
        "禁止已废弃的 Vue 2 过滤器语法（`|` 管道符）",
    ),
    (
        "vue/no-deprecated-filter.message",
        "Filters were removed in Vue 3; replace the '|' filter with a method call or computed property",
        "フィルターは Vue 3 で削除されました。'|' フィルターをメソッド呼び出しまたは算出プロパティに置き換えてください",
        "过滤器已在 Vue 3 中移除；请将 '|' 过滤器替换为方法调用或计算属性",
    ),
    (
        "vue/no-deprecated-filter.help",
        "Replace the filter with a method call or computed property (e.g. {{ capitalize(message) }} instead of {{ message | capitalize }}).",
        "フィルターをメソッド呼び出しまたは算出プロパティに置き換えてください（例: {{ message | capitalize }} ではなく {{ capitalize(message) }}）。",
        "请将过滤器替换为方法调用或计算属性（例如用 {{ capitalize(message) }} 代替 {{ message | capitalize }}）。",
    ),
    // vue/no-deprecated-functional-template
    (
        "vue/no-deprecated-functional-template.description",
        "Disallow the `functional` attribute on the SFC `<template>`",
        "SFCの`<template>`への`functional`属性を禁止する",
        "禁止在SFC的`<template>`上使用`functional`属性",
    ),
    (
        "vue/no-deprecated-functional-template.message",
        "the `functional` attribute on `<template>` (functional SFC templates) was removed in Vue 3",
        "`<template>`の`functional`属性（関数型SFCテンプレート）はVue 3で削除されました",
        "`<template>`上的`functional`属性（函数式SFC模板）已在Vue 3中移除",
    ),
    (
        "vue/no-deprecated-functional-template.help",
        "Remove `functional` and use a stateful component, or write the functional component as a plain function (e.g. a render function or JSX).",
        "`functional`を削除してステートフルなコンポーネントにするか、関数型コンポーネントを通常の関数（レンダー関数やJSXなど）として記述してください。",
        "请移除`functional`并改用有状态组件，或将函数式组件写成普通函数（例如渲染函数或JSX）。",
    ),
];
