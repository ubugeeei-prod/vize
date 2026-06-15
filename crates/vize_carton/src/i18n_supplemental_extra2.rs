//! Additional supplemental i18n entries split out to keep source files small.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every extra supplemental entry into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// Extra supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/no-invalid-html-attribute
    (
        "vue/no-invalid-html-attribute.description",
        "Disallow invalid static values for HTML attributes",
        "HTML属性の無効な静的値を禁止する",
        "禁止HTML属性使用无效的静态值",
    ),
    (
        "vue/no-invalid-html-attribute.empty",
        "The `rel` attribute must not be empty",
        "`rel`属性を空にしてはいけません",
        "`rel`属性不能为空",
    ),
    (
        "vue/no-invalid-html-attribute.wrong_tag",
        "The `rel` attribute is not valid on `<{tag}>`",
        "`rel`属性は`<{tag}>`では有効ではありません",
        "`rel`属性在`<{tag}>`上无效",
    ),
    (
        "vue/no-invalid-html-attribute.invalid",
        "`{value}` is not a valid `rel` value",
        "`{value}`は有効な`rel`値ではありません",
        "`{value}`不是有效的`rel`值",
    ),
    (
        "vue/no-invalid-html-attribute.invalid_for_tag",
        "`{value}` is not a valid `rel` value on `<{tag}>`",
        "`{value}`は`<{tag}>`で有効な`rel`値ではありません",
        "`{value}`不是`<{tag}>`上的有效`rel`值",
    ),
    (
        "vue/no-invalid-html-attribute.shortcut",
        "`shortcut` in `rel` must be followed by `icon`",
        "`rel`内の`shortcut`は`icon`の直前に置く必要があります",
        "`rel`中的`shortcut`后面必须跟随`icon`",
    ),
    (
        "vue/no-invalid-html-attribute.help",
        "Use only standard `rel` tokens that are allowed for this element, such as `noopener noreferrer` on links or `stylesheet` on link elements.",
        "この要素で許可されている標準の`rel`トークンだけを使ってください。例: リンクの`noopener noreferrer`、link要素の`stylesheet`。",
        "请只使用此元素允许的标准`rel`标记，例如链接上的`noopener noreferrer`或link元素上的`stylesheet`。",
    ),
];
