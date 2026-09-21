//! Translations for the HTML content-model checker (`vue/permitted-contents`,
//! Davinci P4-11a): one message and one help text per violation class.
//!
//! `{child}` and `{parent}` arrive already quoted (`<div>`, `<p>`, `text`).

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every entry into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "html/cross-component-nesting.help",
        "**Why:** the browser re-parses the rendered HTML of the whole page, so a child component's root element is nested exactly where the parent renders the component. Each template is valid on its own; together they are not, and the browser's DOM will not match the virtual DOM (hydration mismatch).\n\n**Fix:** render the component where its root element is permitted, or change the parent element or the child's root element (for example `<div>` instead of `<p>` around block-level components).",
        "**理由:** ブラウザはページ全体のレンダリング済み HTML を再パースするため、子コンポーネントのルート要素は親がそのコンポーネントを描画した位置にそのまま入れ子になります。各テンプレートは単体では正しくても、組み合わせると不正になり、ブラウザの DOM は仮想 DOM と一致しません（ハイドレーションの不一致）。\n\n**修正:** ルート要素が許可される位置でコンポーネントを描画するか、親要素または子のルート要素を変更してください（例: ブロック要素のコンポーネントを囲む `<p>` を `<div>` にする）。",
        "**原因:** 浏览器会重新解析整页渲染后的 HTML，因此子组件的根元素会原样嵌套在父组件渲染该组件的位置。每个模板单独都是正确的，但组合后就不再正确，浏览器的 DOM 与虚拟 DOM 不一致（水合不匹配）。\n\n**修复:** 在允许其根元素的位置渲染该组件，或更换父元素或子组件的根元素（例如用 `<div>` 代替包裹块级组件的 `<p>`）。",
    ),
    (
        "vue/permitted-contents.evidence",
        "{parent} is open here",
        "{parent} はここで開かれています",
        "{parent} 在此处打开",
    ),
    (
        "vue/permitted-contents.paragraph-auto-closed",
        "{child} closes the open {parent}: the browser ends the paragraph before it, so the rendered DOM will not match this template",
        "{child} は開いている {parent} を閉じます。ブラウザはその手前で段落を終了するため、描画される DOM はこのテンプレートと一致しません",
        "{child} 会关闭已打开的 {parent}：浏览器会在它之前结束段落，渲染出的 DOM 与此模板不一致",
    ),
    (
        "vue/permitted-contents.heading-auto-closed",
        "{child} closes the open {parent}: the HTML parser never nests headings",
        "{child} は開いている {parent} を閉じます。HTML パーサーは見出しを入れ子にしません",
        "{child} 会关闭已打开的 {parent}：HTML 解析器从不嵌套标题",
    ),
    (
        "vue/permitted-contents.list-item-auto-closed",
        "{child} closes the open {parent}: the HTML parser ends the previous item first",
        "{child} は開いている {parent} を閉じます。HTML パーサーは先に前の項目を終了します",
        "{child} 会关闭已打开的 {parent}：HTML 解析器会先结束前一个条目",
    ),
    (
        "vue/permitted-contents.nested-form-dropped",
        "{child} is ignored by the HTML parser: {parent} is still open, and forms cannot nest",
        "{child} は HTML パーサーに無視されます。{parent} がまだ開いており、フォームは入れ子にできません",
        "{child} 会被 HTML 解析器忽略：{parent} 仍处于打开状态，表单不能嵌套",
    ),
    (
        "vue/permitted-contents.formatting-adopted",
        "{child} cannot be nested in {parent}: the HTML parser closes the outer one and moves the content",
        "{child} は {parent} の中に入れ子にできません。HTML パーサーは外側を閉じて内容を移動します",
        "{child} 不能嵌套在 {parent} 中：HTML 解析器会关闭外层元素并移动内容",
    ),
    (
        "vue/permitted-contents.button-auto-closed",
        "{child} closes the open {parent}: a button cannot contain a button",
        "{child} は開いている {parent} を閉じます。ボタンはボタンを含められません",
        "{child} 会关闭已打开的 {parent}：按钮不能包含按钮",
    ),
    (
        "vue/permitted-contents.select-auto-closed",
        "{child} closes the open {parent}: the HTML parser ends the select element first",
        "{child} は開いている {parent} を閉じます。HTML パーサーは先に select 要素を終了します",
        "{child} 会关闭已打开的 {parent}：HTML 解析器会先结束 select 元素",
    ),
    (
        "vue/permitted-contents.implied-end-tag-closed",
        "{child} implicitly closes {parent}",
        "{child} は {parent} を暗黙的に閉じます",
        "{child} 会隐式关闭 {parent}",
    ),
    (
        "vue/permitted-contents.table-part-misplaced",
        "{child} is not inserted inside {parent}: the HTML parser only accepts it in its place in a table",
        "{child} は {parent} の中に挿入されません。HTML パーサーはテーブル内の正しい位置でしか受け付けません",
        "{child} 不会被插入到 {parent} 中：HTML 解析器只在表格中的正确位置接受它",
    ),
    (
        "vue/permitted-contents.table-wrapper-inserted",
        "{child} directly inside {parent} gets an implied wrapper (<tbody>, <tr> or <colgroup>) from the HTML parser",
        "{parent} 直下の {child} には、HTML パーサーが暗黙のラッパー（<tbody>・<tr>・<colgroup>）を挿入します",
        "直接位于 {parent} 中的 {child} 会被 HTML 解析器插入隐式包裹元素（<tbody>、<tr> 或 <colgroup>）",
    ),
    (
        "vue/permitted-contents.table-auto-closed",
        "{child} closes the open {parent}",
        "{child} は開いている {parent} を閉じます",
        "{child} 会关闭已打开的 {parent}",
    ),
    (
        "vue/permitted-contents.foster-parented",
        "{child} is moved out of {parent}: the HTML parser places content that is not table structure before the table",
        "{child} は {parent} の外に移動されます。HTML パーサーはテーブル構造でない内容をテーブルの前に配置します",
        "{child} 会被移出 {parent}：HTML 解析器会把非表格结构的内容放到表格之前",
    ),
    (
        "vue/permitted-contents.form-in-table-emptied",
        "{child} is not inserted inside {parent}: within table structure the HTML parser closes a form immediately",
        "{child} は {parent} の中に挿入されません。テーブル構造の中では HTML パーサーがフォームを即座に閉じます",
        "{child} 不会被插入到 {parent} 中：在表格结构内，HTML 解析器会立即关闭表单",
    ),
    (
        "vue/permitted-contents.foreign-content-breakout",
        "{child} breaks out of {parent}: the HTML parser ends SVG/MathML content at this tag",
        "{child} は {parent} から抜け出します。HTML パーサーはこのタグで SVG/MathML コンテンツを終了します",
        "{child} 会跳出 {parent}：HTML 解析器会在此标签处结束 SVG/MathML 内容",
    ),
    (
        "vue/permitted-contents.document-element-dropped",
        "{child} is ignored by the HTML parser inside page content",
        "{child} はページコンテンツ内では HTML パーサーに無視されます",
        "{child} 在页面内容中会被 HTML 解析器忽略",
    ),
    (
        "vue/permitted-contents.image-renamed",
        "{child} is parsed as <img> by the HTML parser",
        "{child} は HTML パーサーによって <img> として解析されます",
        "{child} 会被 HTML 解析器解析为 <img>",
    ),
    (
        "vue/permitted-contents.raw-text-content",
        "{child} inside {parent} is parsed as text, not markup",
        "{parent} 内の {child} はマークアップではなくテキストとして解析されます",
        "{parent} 中的 {child} 会被解析为文本而不是标记",
    ),
    (
        "vue/permitted-contents.void-element-content",
        "{child} cannot be placed inside {parent}: the HTML parser closes that element immediately",
        "{child} は {parent} の中に配置できません。HTML パーサーはその要素を即座に閉じます",
        "{child} 不能放在 {parent} 中：HTML 解析器会立即关闭该元素",
    ),
    (
        "vue/permitted-contents.phrasing-content-expected",
        "{child} is not phrasing content, but {parent} only permits phrasing content",
        "{child} はフレージングコンテンツではありませんが、{parent} にはフレージングコンテンツしか配置できません",
        "{child} 不是短语内容，但 {parent} 只允许短语内容",
    ),
    (
        "vue/permitted-contents.interactive-content-nested",
        "{child} is interactive content and cannot be nested in {parent}",
        "{child} はインタラクティブコンテンツであり、{parent} の中に入れ子にできません",
        "{child} 是交互内容，不能嵌套在 {parent} 中",
    ),
    (
        "vue/permitted-contents.child-not-permitted",
        "{child} is not permitted as a child of {parent}",
        "{child} は {parent} の子として許可されていません",
        "{child} 不允许作为 {parent} 的子节点",
    ),
    (
        "vue/permitted-contents.help.parser",
        "**Why:** the browser builds the DOM by re-parsing the rendered HTML (server-side rendering, static content inserted with `innerHTML`). Where this template disagrees with the HTML parser, the browser's DOM differs from the virtual DOM: hydration mismatches, and styles and scripts see a different tree.\n\n**Fix:** move the element to a place its parent accepts, or change the outer element (for example `<div>` instead of `<p>`, `<tbody>` around table rows).",
        "**理由:** ブラウザはレンダリング済みの HTML を再パースして DOM を構築します（SSR、`innerHTML` で挿入される静的コンテンツ）。このテンプレートが HTML パーサーの解釈と食い違う箇所では、ブラウザの DOM が仮想 DOM と一致せず、ハイドレーションの不一致が起き、スタイルやスクリプトも異なるツリーを扱うことになります。\n\n**修正:** 要素を親が受け付ける位置に移すか、外側の要素を変更してください（例: `<p>` の代わりに `<div>`、テーブル行を `<tbody>` で囲む）。",
        "**原因:** 浏览器会重新解析渲染后的 HTML 来构建 DOM（服务端渲染、通过 `innerHTML` 插入的静态内容）。当此模板与 HTML 解析器不一致时，浏览器的 DOM 与虚拟 DOM 不同：会出现水合不匹配，样式和脚本看到的也是另一棵树。\n\n**修复:** 把元素移到父元素接受的位置，或更换外层元素（例如用 `<div>` 代替 `<p>`，用 `<tbody>` 包裹表格行）。",
    ),
    (
        "vue/permitted-contents.help.content-model",
        "**Why:** the HTML standard's content models define which elements may contain which. The browser builds this DOM as written, but the document is non-conforming: assistive technology, validators and future parsers may treat it differently.\n\n**Fix:** use an element the parent permits (phrasing content such as `<span>` inside `<p>`/`<span>`, `<li>` inside lists), or restructure so interactive elements are not nested.",
        "**理由:** HTML 標準のコンテンツモデルは、どの要素がどの要素を含められるかを定めています。ブラウザはこの DOM を記述どおりに構築しますが、文書は仕様に適合せず、支援技術やバリデーター、将来のパーサーが異なる扱いをする可能性があります。\n\n**修正:** 親が許可する要素を使うか（`<p>`/`<span>` 内には `<span>` などのフレージングコンテンツ、リスト内には `<li>`）、インタラクティブ要素が入れ子にならないよう構造を見直してください。",
        "**原因:** HTML 标准的内容模型规定了哪些元素可以包含哪些元素。浏览器会按原样构建此 DOM，但文档不合规：辅助技术、验证器和未来的解析器可能会有不同处理。\n\n**修复:** 使用父元素允许的元素（`<p>`/`<span>` 中使用 `<span>` 等短语内容，列表中使用 `<li>`），或调整结构避免交互元素嵌套。",
    ),
];
