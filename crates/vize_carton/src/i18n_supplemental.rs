//! Supplemental i18n entries that live in Rust source rather than the
//! per-locale `i18n/*.json` files.
//!
//! The `i18n/*.json` files are already past the repository's source-length
//! guard baseline, so new keys cannot grow them. Lint rules whose messages
//! were introduced after that baseline register their translations here and
//! the `i18n` module merges them into the global translator at startup.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every supplemental entry into the locale message maps.
///
/// The maps are indexed by `Locale::index()`: `[0] = En`, `[1] = Ja`, `[2] = Zh`.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// Supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/valid-v-cloak
    (
        "vue/valid-v-cloak.description",
        "Enforce valid v-cloak directives",
        "有効なv-cloakディレクティブを強制する",
        "强制有效的v-cloak指令",
    ),
    (
        "vue/valid-v-cloak.unexpected_value",
        "v-cloak directives require no value",
        "v-cloakディレクティブに値は不要です",
        "v-cloak指令不需要值",
    ),
    (
        "vue/valid-v-cloak.unexpected_argument",
        "v-cloak directives require no argument",
        "v-cloakディレクティブに引数は不要です",
        "v-cloak指令不需要参数",
    ),
    (
        "vue/valid-v-cloak.unexpected_modifier",
        "v-cloak directives require no modifier",
        "v-cloakディレクティブに修飾子は不要です",
        "v-cloak指令不需要修饰符",
    ),
    (
        "vue/valid-v-cloak.help",
        "Remove the value, argument, or modifier from the v-cloak directive",
        "v-cloakディレクティブから値・引数・修飾子を削除してください",
        "请从v-cloak指令中移除值、参数或修饰符",
    ),
    // vue/valid-v-once
    (
        "vue/valid-v-once.description",
        "Enforce valid v-once directives",
        "有効なv-onceディレクティブを強制する",
        "强制有效的v-once指令",
    ),
    (
        "vue/valid-v-once.unexpected_value",
        "v-once directives require no value",
        "v-onceディレクティブに値は不要です",
        "v-once指令不需要值",
    ),
    (
        "vue/valid-v-once.unexpected_argument",
        "v-once directives require no argument",
        "v-onceディレクティブに引数は不要です",
        "v-once指令不需要参数",
    ),
    (
        "vue/valid-v-once.unexpected_modifier",
        "v-once directives require no modifier",
        "v-onceディレクティブに修飾子は不要です",
        "v-once指令不需要修饰符",
    ),
    (
        "vue/valid-v-once.help",
        "Remove the value, argument, or modifier from the v-once directive",
        "v-onceディレクティブから値・引数・修飾子を削除してください",
        "请从v-once指令中移除值、参数或修饰符",
    ),
    // vue/valid-v-text
    (
        "vue/valid-v-text.description",
        "Enforce valid v-text directives",
        "有効なv-textディレクティブを強制する",
        "强制有效的v-text指令",
    ),
    (
        "vue/valid-v-text.missing_expression",
        "v-text directives require that attribute value",
        "v-textディレクティブには式が必要です",
        "v-text指令需要属性值",
    ),
    (
        "vue/valid-v-text.unexpected_argument",
        "v-text directives require no argument",
        "v-textディレクティブに引数は不要です",
        "v-text指令不需要参数",
    ),
    (
        "vue/valid-v-text.unexpected_modifier",
        "v-text directives require no modifier",
        "v-textディレクティブに修飾子は不要です",
        "v-text指令不需要修饰符",
    ),
    (
        "vue/valid-v-text.help",
        "Add an expression to the v-text directive and remove any argument or modifier",
        "v-textディレクティブに式を追加し、引数・修飾子を削除してください",
        "请为v-text指令添加表达式，并移除参数或修饰符",
    ),
    // vue/valid-v-html
    (
        "vue/valid-v-html.description",
        "Enforce valid v-html directives",
        "有効なv-htmlディレクティブを強制する",
        "强制有效的v-html指令",
    ),
    (
        "vue/valid-v-html.missing_expression",
        "v-html directives require that attribute value",
        "v-htmlディレクティブには式が必要です",
        "v-html指令需要属性值",
    ),
    (
        "vue/valid-v-html.unexpected_argument",
        "v-html directives require no argument",
        "v-htmlディレクティブに引数は不要です",
        "v-html指令不需要参数",
    ),
    (
        "vue/valid-v-html.unexpected_modifier",
        "v-html directives require no modifier",
        "v-htmlディレクティブに修飾子は不要です",
        "v-html指令不需要修饰符",
    ),
    (
        "vue/valid-v-html.help",
        "Add an expression to the v-html directive and remove any argument or modifier",
        "v-htmlディレクティブに式を追加し、引数・修飾子を削除してください",
        "请为v-html指令添加表达式，并移除参数或修饰符",
    ),
    // vue/no-useless-v-bind
    (
        "vue/no-useless-v-bind.description",
        "Disallow a v-bind whose value is a plain string literal",
        "値が単なる文字列リテラルのv-bindを禁止する",
        "禁止值为纯字符串字面量的v-bind",
    ),
    (
        "vue/no-useless-v-bind.message",
        "Binding ':{name}' to a constant string is unnecessary; use the static attribute '{name}'",
        "':{name}' に定数文字列をバインドするのは不要です。静的属性 '{name}' を使用してください",
        "将 ':{name}' 绑定到常量字符串是多余的；请使用静态属性 '{name}'",
    ),
    (
        "vue/no-useless-v-bind.help",
        "Replace the binding with a static attribute (foo=bar instead of :foo='bar').",
        "バインディングを静的属性に置き換えてください（:foo='bar' ではなく foo=bar）。",
        "请将该绑定替换为静态属性（用 foo=bar 代替 :foo='bar'）。",
    ),
    // vue/prefer-true-attribute-shorthand
    (
        "vue/prefer-true-attribute-shorthand.description",
        "Prefer the shorthand for a boolean attribute bound to true",
        "trueをバインドする真偽値属性は省略形を推奨する",
        "对绑定为true的布尔属性推荐使用简写",
    ),
    (
        "vue/prefer-true-attribute-shorthand.message",
        "':{name}' bound to true can be written as the shorthand '{name}'",
        "trueをバインドした ':{name}' は省略形 '{name}' で書けます",
        "绑定为true的 ':{name}' 可以写成简写 '{name}'",
    ),
    (
        "vue/prefer-true-attribute-shorthand.help",
        "Drop the ='true' binding and write the attribute name on its own.",
        "='true' のバインディングを削除し、属性名だけを記述してください。",
        "去掉 ='true' 绑定，仅写属性名即可。",
    ),
    // vue/no-useless-mustaches
    (
        "vue/no-useless-mustaches.description",
        "Disallow a mustache interpolation whose expression is a constant string literal",
        "式が定数文字列リテラルのマスタッシュ補間を禁止する",
        "禁止表达式为常量字符串字面量的插值表达式",
    ),
    (
        "vue/no-useless-mustaches.message",
        "Interpolating a constant string is unnecessary; write it as static text",
        "定数文字列を補間するのは不要です。静的なテキストとして記述してください",
        "插值常量字符串是多余的；请将其写为静态文本",
    ),
    (
        "vue/no-useless-mustaches.help",
        "Replace the mustache with the plain text ({{ 'x' }} becomes x).",
        "マスタッシュを素のテキストに置き換えてください（{{ 'x' }} は x になります）。",
        "请将该插值替换为纯文本（{{ 'x' }} 改为 x）。",
    ),
    // vue/html-button-has-type
    (
        "vue/html-button-has-type.description",
        "Require an explicit valid type on button elements",
        "button要素に明示的で有効なtypeを必須にする",
        "要求button元素具有明确且有效的type",
    ),
    (
        "vue/html-button-has-type.missing",
        "'<button>' is missing a 'type' attribute; add type=\"button\", \"submit\", or \"reset\"",
        "'<button>' に 'type' 属性がありません。type=\"button\"、\"submit\"、\"reset\" のいずれかを追加してください",
        "'<button>' 缺少 'type' 属性；请添加 type=\"button\"、\"submit\" 或 \"reset\"",
    ),
    (
        "vue/html-button-has-type.invalid",
        "'{type}' is not a valid 'type' for '<button>'; use \"button\", \"submit\", or \"reset\"",
        "'{type}' は '<button>' の有効な 'type' ではありません。\"button\"、\"submit\"、\"reset\" のいずれかを使用してください",
        "'{type}' 不是 '<button>' 的有效 'type'；请使用 \"button\"、\"submit\" 或 \"reset\"",
    ),
    (
        "vue/html-button-has-type.help",
        "A '<button>' defaults to type=\"submit\", which submits the enclosing form. Set an explicit type=\"button\", \"submit\", or \"reset\".",
        "'<button>' の type は既定で \"submit\" となり、囲っているフォームを送信します。明示的に type=\"button\"、\"submit\"、\"reset\" のいずれかを指定してください。",
        "'<button>' 的 type 默认为 \"submit\"，会提交其所在的表单。请显式设置 type=\"button\"、\"submit\" 或 \"reset\"。",
    ),
    // vue/slot-name-casing
    (
        "vue/slot-name-casing.description",
        "Enforce kebab-case for named slots used via v-slot",
        "v-slotで使用する名前付きスロットにケバブケースを強制する",
        "强制通过v-slot使用的具名插槽采用短横线命名",
    ),
    (
        "vue/slot-name-casing.message",
        "Slot name '{name}' should be kebab-case",
        "スロット名 '{name}' はケバブケースにしてください",
        "插槽名称 '{name}' 应使用短横线命名",
    ),
    (
        "vue/slot-name-casing.help",
        "Rename the slot to kebab-case, e.g. '#my-slot' instead of '#mySlot' or '#my_slot'.",
        "スロット名をケバブケースに変更してください。例: '#mySlot' や '#my_slot' ではなく '#my-slot'。",
        "请将插槽名称改为短横线命名，例如使用 '#my-slot' 而非 '#mySlot' 或 '#my_slot'。",
    ),
    // vue/no-deprecated-router-link-tag-prop
    (
        "vue/no-deprecated-router-link-tag-prop.description",
        "Disallow the deprecated `tag` prop on <router-link>",
        "<router-link> の非推奨な `tag` プロパティを禁止する",
        "禁止 <router-link> 上已弃用的 `tag` 属性",
    ),
    (
        "vue/no-deprecated-router-link-tag-prop.message",
        "The `tag` prop on `<router-link>` was removed in Vue Router 4; use the v-slot API instead",
        "`<router-link>` の `tag` プロパティは Vue Router 4 で削除されました。代わりに v-slot API を使用してください",
        "`<router-link>` 上的 `tag` 属性已在 Vue Router 4 中移除；请改用 v-slot API",
    ),
    (
        "vue/no-deprecated-router-link-tag-prop.help",
        "Remove the `tag` prop and render the element yourself using `v-slot` (it exposes `href`, `navigate`, and `isActive`).",
        "`tag` プロパティを削除し、`v-slot`（`href`・`navigate`・`isActive` を公開します）を使って要素を自分でレンダリングしてください。",
        "请移除 `tag` 属性，并使用 `v-slot`（它会暴露 `href`、`navigate` 和 `isActive`）自行渲染元素。",
    ),
    // vue/no-negated-v-if-condition
    (
        "vue/no-negated-v-if-condition.description",
        "Disallow a negated v-if condition when the chain has a v-else",
        "v-elseを伴う連鎖で否定されたv-if条件を禁止する",
        "当存在v-else分支时，禁止使用取反的v-if条件",
    ),
    (
        "vue/no-negated-v-if-condition.message",
        "Avoid a negated v-if condition when there is a v-else; swap the branches instead",
        "v-elseがある場合は否定されたv-if条件を避け、分岐を入れ替えてください",
        "存在v-else时应避免使用取反的v-if条件；请改为交换两个分支",
    ),
    (
        "vue/no-negated-v-if-condition.help",
        "Remove the leading '!' and swap the v-if and v-else branch contents.",
        "先頭の '!' を削除し、v-ifとv-elseの分岐の中身を入れ替えてください。",
        "请去掉开头的 '!'，并交换 v-if 与 v-else 分支的内容。",
    ),
    // vue/v-on-event-hyphenation
    (
        "vue/v-on-event-hyphenation.description",
        "Enforce hyphenation of custom event names in v-on on components",
        "コンポーネントの v-on におけるカスタムイベント名のハイフネーションを強制する",
        "强制组件上 v-on 自定义事件名称使用连字符",
    ),
    (
        "vue/v-on-event-hyphenation.message",
        "Custom event listeners on components should be hyphenated: use '{name}'",
        "コンポーネントのカスタムイベントリスナーはハイフン区切りにしてください: '{name}' を使用してください",
        "组件上的自定义事件监听器应使用连字符：请使用 '{name}'",
    ),
    (
        "vue/v-on-event-hyphenation.help",
        "Rename the listener to kebab-case so it matches the emitted event name.",
        "発行されるイベント名と一致するように、リスナー名をケバブケースに変更してください。",
        "请将监听器重命名为 kebab-case，以与发出的事件名称匹配。",
    ),
    // html/no-duplicate-class
    (
        "html/no-duplicate-class.description",
        "Disallow duplicate class names in a static class attribute",
        "静的なclass属性内で重複したクラス名を禁止する",
        "禁止在静态 class 属性中出现重复的类名",
    ),
    (
        "html/no-duplicate-class.message",
        "Duplicate class name '{name}'",
        "クラス名 '{name}' が重複しています",
        "类名 '{name}' 重复",
    ),
    (
        "html/no-duplicate-class.help",
        "Remove the duplicate occurrence so each class name appears only once in the class attribute.",
        "重複しているクラス名を削除し、各クラス名がclass属性内で一度だけ現れるようにしてください。",
        "请删除重复出现的类名，使每个类名在 class 属性中只出现一次。",
    ),
];
