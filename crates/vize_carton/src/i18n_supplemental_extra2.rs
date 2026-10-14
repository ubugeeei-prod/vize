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
    crate::i18n_supplemental_html::register(messages);
    crate::i18n_render::register(messages);
    crate::i18n_compiler::register(messages);
    crate::i18n_l3::register(messages);
    crate::i18n_croquis::register(messages);
    crate::i18n_rules_markup::register(messages);
}

/// Extra supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/no-multiple-template-root
    (
        "vue/no-multiple-template-root.multiple_root",
        "The template root requires exactly one element.",
        "テンプレートのルートには要素が1つだけ必要です。",
        "模板根节点必须恰好包含一个元素。",
    ),
    (
        "vue/no-multiple-template-root.text_root",
        "The template root requires an element rather than texts.",
        "テンプレートのルートにはテキストではなく要素が必要です。",
        "模板根节点需要元素而不是文本。",
    ),
    (
        "vue/no-multiple-template-root.disallowed_element",
        "The template root disallows '<{tag}>' elements.",
        "テンプレートのルートでは '<{tag}>' 要素を使用できません。",
        "模板根节点不允许 '<{tag}>' 元素。",
    ),
    (
        "vue/no-multiple-template-root.disallowed_directive",
        "The template root disallows 'v-for' directives.",
        "テンプレートのルートでは 'v-for' ディレクティブを使用できません。",
        "模板根节点不允许 'v-for' 指令。",
    ),
    (
        "vue/no-multiple-template-root.help",
        "Wrap the template in one rendered element; keep conditional branches in one v-if chain.",
        "テンプレートを1つの描画要素で囲み、条件分岐は1つのv-ifチェーンにまとめてください。",
        "请用一个渲染元素包裹模板，并将条件分支保留在同一个v-if链中。",
    ),
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
    // vue/v-on-event-hyphenation
    (
        "vue/v-on-event-hyphenation.message_never",
        "Custom event listeners on components should not be hyphenated: '{name}'",
        "コンポーネントのカスタムイベントリスナーはハイフン区切りにしないでください: '{name}'",
        "组件上的自定义事件监听器不应使用连字符：'{name}'",
    ),
    (
        "vue/v-on-event-hyphenation.help_never",
        "Use camelCase or a single-word event name instead of kebab-case.",
        "ケバブケースではなく、camelCase または単一語のイベント名を使用してください。",
        "请使用 camelCase 或单词事件名，而不是 kebab-case。",
    ),
    // vue/attribute-hyphenation
    (
        "vue/attribute-hyphenation.message_never",
        "Attribute should not be hyphenated",
        "属性はハイフン区切りにしないでください",
        "属性不应使用连字符命名",
    ),
    (
        "vue/attribute-hyphenation.help_never",
        "Use camelCase or a single-word attribute name instead of kebab-case.",
        "ケバブケースではなく、camelCase または単一語の属性名を使用してください。",
        "请使用 camelCase 或单词属性名，而不是 kebab-case。",
    ),
    // The v-else-if, v-pre and v-is rules use eslint-plugin-vue's English
    // messages, so a migrating project sees the wording it already knows.
    // vue/valid-v-else-if
    (
        "vue/valid-v-else-if.description",
        "Enforce valid v-else-if directives",
        "有効なv-else-ifディレクティブを強制する",
        "强制有效的v-else-if指令",
    ),
    (
        "vue/valid-v-else-if.missing_v_if",
        "'v-else-if' directives require being preceded by the element which has a 'v-if' or 'v-else-if' directive.",
        "v-else-ifは、v-ifまたはv-else-ifを持つ要素の直後に置く必要があります",
        "v-else-if必须紧跟在带有v-if或v-else-if的元素之后",
    ),
    (
        "vue/valid-v-else-if.with_v_if",
        "'v-else-if' and 'v-if' directives can't exist on the same element.",
        "v-else-ifとv-ifは同じ要素に指定できません",
        "v-else-if和v-if不能用在同一个元素上",
    ),
    (
        "vue/valid-v-else-if.with_v_else",
        "'v-else-if' and 'v-else' directives can't exist on the same element.",
        "v-else-ifとv-elseは同じ要素に指定できません",
        "v-else-if和v-else不能用在同一个元素上",
    ),
    (
        "vue/valid-v-else-if.unexpected_argument",
        "'v-else-if' directives require no argument.",
        "v-else-ifディレクティブに引数は指定できません",
        "v-else-if指令不需要参数",
    ),
    (
        "vue/valid-v-else-if.unexpected_modifier",
        "'v-else-if' directives require no modifier.",
        "v-else-ifディレクティブに修飾子は指定できません",
        "v-else-if指令不需要修饰符",
    ),
    (
        "vue/valid-v-else-if.expected_value",
        "'v-else-if' directives require that attribute value.",
        "v-else-ifディレクティブには条件式が必要です",
        "v-else-if指令需要条件表达式",
    ),
    (
        "vue/valid-v-else-if.help",
        "Place v-else-if directly after a v-if or v-else-if element and give it only a condition",
        "v-else-ifはv-ifまたはv-else-if要素の直後に置き、条件式だけを指定してください",
        "请将v-else-if紧跟在v-if或v-else-if元素之后，并且只提供条件表达式",
    ),
    // vue/valid-v-pre
    (
        "vue/valid-v-pre.description",
        "Enforce valid v-pre directives",
        "有効なv-preディレクティブを強制する",
        "强制有效的v-pre指令",
    ),
    (
        "vue/valid-v-pre.unexpected_argument",
        "'v-pre' directives require no argument.",
        "v-preディレクティブに引数は指定できません",
        "v-pre指令不需要参数",
    ),
    (
        "vue/valid-v-pre.unexpected_modifier",
        "'v-pre' directives require no modifier.",
        "v-preディレクティブに修飾子は指定できません",
        "v-pre指令不需要修饰符",
    ),
    (
        "vue/valid-v-pre.unexpected_value",
        "'v-pre' directives require no attribute value.",
        "v-preディレクティブに値は指定できません",
        "v-pre指令不需要属性值",
    ),
    (
        "vue/valid-v-pre.help",
        "Remove the argument, modifier, or value from the v-pre directive",
        "v-preディレクティブから引数・修飾子・値を削除してください",
        "请从v-pre指令中移除参数、修饰符或值",
    ),
    // vue/valid-v-is
    (
        "vue/valid-v-is.description",
        "Enforce valid v-is directives",
        "有効なv-isディレクティブを強制する",
        "强制有效的v-is指令",
    ),
    (
        "vue/valid-v-is.unexpected_argument",
        "'v-is' directives require no argument.",
        "v-isディレクティブに引数は指定できません",
        "v-is指令不需要参数",
    ),
    (
        "vue/valid-v-is.unexpected_modifier",
        "'v-is' directives require no modifier.",
        "v-isディレクティブに修飾子は指定できません",
        "v-is指令不需要修饰符",
    ),
    (
        "vue/valid-v-is.expected_value",
        "'v-is' directives require that attribute value.",
        "v-isディレクティブには値が必要です",
        "v-is指令需要属性值",
    ),
    (
        "vue/valid-v-is.owner_must_be_html_element",
        "'v-is' directive must be owned by a native HTML element, but '{name}' is not.",
        "v-isディレクティブはネイティブのHTML要素に指定する必要がありますが、'{name}'はそうではありません",
        "v-is指令必须用在原生HTML元素上，但'{name}'不是",
    ),
    (
        "vue/valid-v-is.help",
        "Use v-is with a value on a native HTML element, without an argument or modifier",
        "v-isはネイティブのHTML要素に、引数と修飾子なしで値を付けて指定してください",
        "请在原生HTML元素上使用带值的v-is，不要添加参数或修饰符",
    ),
    // vue/no-deprecated-v-is
    (
        "vue/no-deprecated-v-is.description",
        "Disallow the deprecated v-is directive",
        "非推奨のv-isディレクティブを禁止する",
        "禁止使用已废弃的v-is指令",
    ),
    (
        "vue/no-deprecated-v-is.message",
        "`v-is` directive is deprecated.",
        "`v-is`ディレクティブは非推奨です",
        "`v-is`指令已废弃",
    ),
    (
        "vue/no-deprecated-v-is.help",
        "Use the `is` attribute with the `vue:` prefix instead, for example `is=\"vue:MyComponent\"`",
        "代わりに`vue:`プレフィックス付きの`is`属性を使ってください。例: `is=\"vue:MyComponent\"`",
        "请改用带`vue:`前缀的`is`属性，例如`is=\"vue:MyComponent\"`",
    ),
];
