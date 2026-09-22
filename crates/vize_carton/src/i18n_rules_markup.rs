//! `<rule>.description` for the lint rules that had none in any locale
//! (P4-14b): accessibility, CSS, HTML, Musea, Vapor and Vue template rules
//! here; script and type rules in [`crate::i18n_rules_script`] and
//! [`crate::i18n_rules_script_more`]; ecosystem and Nuxt rules in
//! [`crate::i18n_rules_ecosystem`].
//!
//! The `en` column is the rule's own `RuleMeta` description, byte for byte —
//! `tests/tooling/davinci-diagnostic-catalog.test.ts` reads both from source
//! and fails on drift, and on any of the 248 rules without a description in
//! every locale. Japanese follows the "〜を禁止する / 〜を必須にする / 〜を推奨する"
//! register rule catalogues use; Chinese the matching 禁止 / 要求 / 推荐.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every rule-description table into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    let tables = [
        ENTRIES,
        crate::i18n_rules_script::ENTRIES,
        crate::i18n_rules_script_more::ENTRIES,
        crate::i18n_rules_ecosystem::ENTRIES,
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
        "a11y/heading-levels.description",
        "Disallow skipping heading levels",
        "見出しレベルを飛ばすことを禁止する",
        "禁止跳过标题级别",
    ),
    (
        "a11y/landmark-roles.description",
        "Validate landmark role placement and uniqueness",
        "ランドマークロールの配置と一意性を検証する",
        "校验地标角色的位置与唯一性",
    ),
    (
        "a11y/placeholder-label-option.description",
        "Require disabled or hidden on select placeholder option",
        "select のプレースホルダー用 option に disabled か hidden を必須にする",
        "要求 select 的占位 option 带有 disabled 或 hidden",
    ),
    (
        "a11y/use-list.description",
        "Suggest using list elements for bullet-like text",
        "箇条書きのようなテキストにはリスト要素を使うよう提案する",
        "建议对类似项目符号的文本使用列表元素",
    ),
    (
        "css/no-display-none.description",
        "Suggest using v-show instead of display: none",
        "display: none の代わりに v-show を使うよう提案する",
        "建议使用 v-show 代替 display: none",
    ),
    (
        "css/no-hardcoded-values.description",
        "Suggest using CSS variables instead of hardcoded values",
        "ハードコードされた値の代わりに CSS 変数を使うよう提案する",
        "建议使用 CSS 变量代替硬编码的值",
    ),
    (
        "css/no-id-selectors.description",
        "Discourage use of ID selectors in CSS",
        "CSS での ID セレクターの使用を非推奨とする",
        "不建议在 CSS 中使用 ID 选择器",
    ),
    (
        "css/no-important.description",
        "Discourage use of !important in CSS",
        "CSS での !important の使用を非推奨とする",
        "不建议在 CSS 中使用 !important",
    ),
    (
        "css/no-utility-classes.description",
        "Warn against implementing utility classes in component styles",
        "コンポーネントのスタイルでユーティリティクラスを実装することを警告する",
        "对在组件样式中实现工具类发出警告",
    ),
    (
        "css/no-v-bind-performance.description",
        "Warn about performance cost of CSS v-bind()",
        "CSS の v-bind() によるパフォーマンスコストを警告する",
        "对 CSS v-bind() 的性能开销发出警告",
    ),
    (
        "css/prefer-logical-properties.description",
        "Recommend CSS logical properties for better i18n support",
        "国際化に対応しやすい CSS 論理プロパティを推奨する",
        "推荐使用 CSS 逻辑属性，以更好地支持国际化",
    ),
    (
        "css/prefer-nested-selectors.description",
        "Recommend using CSS nesting for descendant selectors",
        "子孫セレクターには CSS ネストを推奨する",
        "推荐对后代选择器使用 CSS 嵌套",
    ),
    (
        "css/prefer-slotted.description",
        "Recommend ::v-slotted() for styling slot content",
        "スロットの内容のスタイル指定には ::v-slotted() を推奨する",
        "推荐使用 ::v-slotted() 为插槽内容设置样式",
    ),
    (
        "css/require-font-display.description",
        "Require font-display in @font-face rules",
        "@font-face ルールで font-display を必須にする",
        "要求在 @font-face 规则中指定 font-display",
    ),
    (
        "html/deprecated-attr.description",
        "Disallow deprecated HTML attributes",
        "非推奨の HTML 属性を禁止する",
        "禁止使用已弃用的 HTML 属性",
    ),
    (
        "html/deprecated-element.description",
        "Disallow deprecated HTML elements",
        "非推奨の HTML 要素を禁止する",
        "禁止使用已弃用的 HTML 元素",
    ),
    (
        "html/id-duplication.description",
        "Disallow duplicate element IDs",
        "要素 ID の重複を禁止する",
        "禁止重复的元素 ID",
    ),
    (
        "html/no-consecutive-br.description",
        "Disallow consecutive <br> elements",
        "<br> 要素を連続して置くことを禁止する",
        "禁止连续的 <br> 元素",
    ),
    (
        "html/no-duplicate-dt.description",
        "Disallow duplicate <dt> names in <dl>",
        "<dl> 内で <dt> の名前が重複することを禁止する",
        "禁止 <dl> 中出现重复的 <dt> 名称",
    ),
    (
        "html/no-empty-palpable-content.description",
        "Disallow empty elements that expect visible content",
        "目に見える内容を持つべき要素が空であることを禁止する",
        "禁止本应包含可见内容的元素为空",
    ),
    (
        "html/require-datetime.description",
        "Require datetime attribute on <time> element",
        "<time> 要素に datetime 属性を必須にする",
        "要求 <time> 元素带有 datetime 属性",
    ),
    (
        "musea/no-empty-variant.description",
        "Disallow empty <variant> blocks",
        "空の <variant> ブロックを禁止する",
        "禁止空的 <variant> 块",
    ),
    (
        "musea/prefer-design-tokens.description",
        "Prefer design token CSS variables over hardcoded primitive values",
        "ハードコードされたプリミティブ値よりデザイントークンの CSS 変数を推奨する",
        "优先使用设计令牌 CSS 变量，而不是硬编码的原始值",
    ),
    (
        "musea/require-component.description",
        "Require component attribute in <art> block",
        "<art> ブロックに component 属性を必須にする",
        "要求 <art> 块带有 component 属性",
    ),
    (
        "musea/unique-variant-names.description",
        "Require unique variant names",
        "バリアント名が一意であることを必須にする",
        "要求变体名称唯一",
    ),
    (
        "musea/valid-variant.description",
        "Require name attribute in <variant> blocks",
        "<variant> ブロックに name 属性を必須にする",
        "要求 <variant> 块带有 name 属性",
    ),
    (
        "vapor/require-vapor-attribute.description",
        "Suggest adding vapor attribute to script setup",
        "script setup に vapor 属性を付けるよう提案する",
        "建议为 script setup 添加 vapor 属性",
    ),
    (
        "vue/no-boolean-attr-value.description",
        "Disallow explicit values for boolean HTML attributes",
        "HTML の真偽値属性に明示的な値を指定することを禁止する",
        "禁止为布尔型 HTML 属性显式指定值",
    ),
    (
        "vue/no-multiple-template-root.description",
        "Disallow multiple root nodes in a template",
        "テンプレートに複数のルートノードを置くことを禁止する",
        "禁止模板中出现多个根节点",
    ),
    (
        "vue/no-mutating-props.description",
        "Disallow mutating component props",
        "コンポーネントの props を変更することを禁止する",
        "禁止修改组件的 props",
    ),
    (
        "vue/no-preprocessor-lang.description",
        "Discourage CSS preprocessor usage in favor of modern CSS",
        "CSS プリプロセッサーを非推奨とし、モダンな CSS を推奨する",
        "不建议使用 CSS 预处理器，推荐使用现代 CSS",
    ),
    (
        "vue/no-script-non-standard-lang.description",
        "Discourage non-standard script lang values",
        "標準外の script の lang 値を非推奨とする",
        "不建议使用非标准的 script lang 值",
    ),
    (
        "vue/no-src-attribute.description",
        "Discourage src attribute on SFC blocks",
        "SFC ブロックでの src 属性を非推奨とする",
        "不建议在 SFC 块上使用 src 属性",
    ),
    (
        "vue/no-static-inline-styles.description",
        "Disallow static inline style attributes",
        "静的なインラインの style 属性を禁止する",
        "禁止使用静态的内联 style 属性",
    ),
    (
        "vue/no-template-lang.description",
        "Discourage lang attribute on template block",
        "template ブロックでの lang 属性を非推奨とする",
        "不建议在 template 块上使用 lang 属性",
    ),
    (
        "vue/no-unused-components.description",
        "Disallow registering components that are not used inside templates",
        "テンプレートで使われないコンポーネントの登録を禁止する",
        "禁止注册模板中未使用的组件",
    ),
    (
        "vue/no-unused-properties.description",
        "Disallow unused properties defined in defineProps",
        "defineProps で定義したのに使われない props を禁止する",
        "禁止 defineProps 中定义却未使用的属性",
    ),
    (
        "vue/no-unused-refs.description",
        "Report template refs (ref=\"x\") never referenced in <script>",
        "<script> で一度も参照されないテンプレート参照（ref=\"x\"）を報告する",
        "报告从未在 <script> 中引用的模板引用（ref=\"x\"）",
    ),
    (
        "vue/require-component-registration.description",
        "Require explicit import or registration for components",
        "コンポーネントの明示的なインポートか登録を必須にする",
        "要求显式导入或注册组件",
    ),
    (
        "vue/sfc-element-order.description",
        "Enforce consistent order of SFC top-level elements",
        "SFC のトップレベル要素の順序を統一する",
        "强制 SFC 顶层元素的顺序保持一致",
    ),
    (
        "vue/single-style-block.description",
        "Recommend having a single style block",
        "style ブロックを 1 つにまとめることを推奨する",
        "推荐只使用一个 style 块",
    ),
    (
        "vue/warn-custom-block.description",
        "Warn about custom blocks in SFC files",
        "SFC ファイル内のカスタムブロックを警告する",
        "对 SFC 文件中的自定义块发出警告",
    ),
    (
        "vue/warn-custom-directive.description",
        "Warn about custom directives that need registration",
        "登録が必要なカスタムディレクティブを警告する",
        "对需要注册的自定义指令发出警告",
    ),
];
