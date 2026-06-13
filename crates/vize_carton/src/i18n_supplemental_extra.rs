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
];
