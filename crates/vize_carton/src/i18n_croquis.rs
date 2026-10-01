//! Catalog of cross-file diagnostic codes (`vize:croquis/cf/*`), part 1.
//!
//! Call-site messages stay as they are. These entries name the code.
//! The catalog test reads every string in `CrossFileDiagnostic::code`.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every croquis code into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for table in [
        ENTRIES,
        crate::i18n_croquis_more::ENTRIES,
        crate::i18n_croquis_rest::ENTRIES,
        crate::i18n_croquis_last::ENTRIES,
    ] {
        for &(key, en, ja, zh) in table {
            messages[0].insert(key, en);
            messages[1].insert(key, ja);
            messages[2].insert(key, zh);
        }
    }
}

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "vize:croquis/cf/unused-attrs.message",
        "Fallthrough attributes are passed to a multi-root component that does not use them.",
        "複数ルートのコンポーネントに渡した fallthrough 属性が使われていません。",
        "传给多根组件的 fallthrough 属性没有被使用。",
    ),
    (
        "vize:croquis/cf/unused-attrs.help",
        "Bind `$attrs` explicitly or declare the attributes as props.",
        "`$attrs` を明示的にバインドするか、属性を props として宣言してください。",
        "请显式绑定 `$attrs`，或把这些属性声明为 props。",
    ),
    (
        "vize:croquis/cf/inherit-attrs-unused.message",
        "`inheritAttrs: false` is set and the component never reads the attributes.",
        "`inheritAttrs: false` が設定されているのに、属性が読まれていません。",
        "设置了 `inheritAttrs: false`，但组件从未读取这些属性。",
    ),
    (
        "vize:croquis/cf/inherit-attrs-unused.help",
        "Apply `$attrs` on an element, or stop disabling inheritance.",
        "要素に `$attrs` を付けるか、継承の無効化をやめてください。",
        "请在元素上应用 `$attrs`，或不要禁用继承。",
    ),
    (
        "vize:croquis/cf/multi-root-attrs.message",
        "A multi-root component receives attributes and has nowhere to put them.",
        "複数ルートのコンポーネントが属性を受け取りますが、付ける場所がありません。",
        "多根组件收到了属性，但没有可以放置它们的位置。",
    ),
    (
        "vize:croquis/cf/multi-root-attrs.help",
        "Give the component a single root or bind `$attrs` on one root.",
        "単一ルートにするか、どれか一つのルートに `$attrs` をバインドしてください。",
        "请改成单根，或在其中一个根上绑定 `$attrs`。",
    ),
    (
        "vize:croquis/cf/undeclared-emit.message",
        "The component emits an event that is not declared.",
        "宣言されていないイベントを emit しています。",
        "组件 emit 了未声明的事件。",
    ),
    (
        "vize:croquis/cf/undeclared-emit.help",
        "Add the event to the `emits` option or `defineEmits`.",
        "`emits` オプションか `defineEmits` にそのイベントを追加してください。",
        "请把该事件加入 `emits` 选项或 `defineEmits`。",
    ),
    (
        "vize:croquis/cf/unused-emit.message",
        "A declared emit is never used.",
        "宣言した emit が一度も使われていません。",
        "声明的 emit 从未被使用。",
    ),
    (
        "vize:croquis/cf/unused-emit.help",
        "Emit the event or remove it from the declaration.",
        "そのイベントを emit するか、宣言から削除してください。",
        "请 emit 该事件，或从声明中删除它。",
    ),
    (
        "vize:croquis/cf/unmatched-listener.message",
        "A parent listens for an event the child does not emit.",
        "親が、子が emit しないイベントを購読しています。",
        "父组件监听了子组件不会 emit 的事件。",
    ),
    (
        "vize:croquis/cf/unmatched-listener.help",
        "Listen for a declared emit, or declare the event on the child.",
        "宣言された emit を購読するか、子にそのイベントを宣言してください。",
        "请监听已声明的 emit，或在子组件上声明该事件。",
    ),
    (
        "vize:croquis/cf/unhandled-event.message",
        "A child emits an event that no parent handles.",
        "子が emit したイベントを、どの親も処理していません。",
        "子组件 emit 的事件没有任何父组件处理。",
    ),
    (
        "vize:croquis/cf/unhandled-event.help",
        "Handle the event on the parent, or stop emitting it.",
        "親でそのイベントを処理するか、emit をやめてください。",
        "请在父组件处理该事件，或停止 emit。",
    ),
    (
        "vize:croquis/cf/event-modifier.message",
        "An event listener uses a modifier the emit does not support.",
        "emit が対応しない修飾子を、イベントリスナーが使っています。",
        "事件监听使用了该 emit 不支持的修饰符。",
    ),
    (
        "vize:croquis/cf/event-modifier.help",
        "Drop the modifier or emit the event in the form the modifier expects.",
        "修飾子を外すか、修飾子が期待する形でイベントを emit してください。",
        "请去掉修饰符，或按修饰符所期望的形式 emit 事件。",
    ),
    (
        "vize:croquis/cf/unmatched-inject.message",
        "`inject` names a key that no ancestor provides.",
        "`inject` が、どの祖先も provide していないキーを指定しています。",
        "`inject` 使用了没有任何祖先 provide 的键。",
    ),
    (
        "vize:croquis/cf/unmatched-inject.help",
        "Provide the key from an ancestor, or inject a key that is provided.",
        "祖先でそのキーを provide するか、provide されているキーを inject してください。",
        "请从祖先 provide 该键，或 inject 一个已 provide 的键。",
    ),
    (
        "vize:croquis/cf/unused-provide.message",
        "A provided key is never injected.",
        "provide したキーが一度も inject されていません。",
        "被 provide 的键从未被 inject。",
    ),
    (
        "vize:croquis/cf/unused-provide.help",
        "Inject the key in a descendant, or remove the provide.",
        "子孫でそのキーを inject するか、provide を削除してください。",
        "请在后代中 inject 该键，或删除这次 provide。",
    ),
];
