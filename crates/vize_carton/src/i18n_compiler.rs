//! Catalog of the template compiler's diagnostic codes (`vize_relief::ErrorCode`).
//!
//! Every code has a `compiler/<name>.message` — its headline, whose `en` text
//! is byte-identical to `ErrorCode::message()` so English output never moves —
//! and a `compiler/<name>.help` saying what to do about it. `<name>` is
//! `ErrorCode::name()`. `tests/tooling/davinci-diagnostic-catalog.test.ts`
//! enumerates the enum's variants from source and fails when a code lacks
//! either entry in any locale or when an `en` message drifts from the enum.
//!
//! The table is split by the enum's own sections: HTML tokenizer errors here,
//! Vue template syntax in [`crate::i18n_compiler_template`], directive and
//! mode errors in [`crate::i18n_compiler_directive`].

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every compiler-code entry into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    let tables = [
        ENTRIES,
        crate::i18n_compiler_template::ENTRIES,
        crate::i18n_compiler_directive::ENTRIES,
    ];
    for &(key, en, ja, zh) in tables.into_iter().flatten() {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// HTML tokenizer errors, `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "compiler/abrupt-closing-of-empty-comment.message",
        "Illegal comment.",
        "不正なコメントです。",
        "非法的注释。",
    ),
    (
        "compiler/abrupt-closing-of-empty-comment.help",
        "write a comment as `<!-- … -->`; `<!-->` and `<!--->` end it too early",
        "コメントは `<!-- … -->` の形で書いてください。`<!-->` や `<!--->` ではコメントが途中で閉じてしまいます",
        "注释必须写成 `<!-- … -->`；`<!-->` 和 `<!--->` 会过早结束注释",
    ),
    (
        "compiler/cdata-in-html-content.message",
        "CDATA section is allowed only in XML context.",
        "CDATA セクションは XML コンテキストでのみ使用できます。",
        "CDATA 区块只能在 XML 上下文中使用。",
    ),
    (
        "compiler/cdata-in-html-content.help",
        "`<![CDATA[ … ]]>` only works inside `<svg>` or `<math>`; escape the text instead",
        "`<![CDATA[ … ]]>` は `<svg>` や `<math>` の中でしか使えません。代わりにテキストをエスケープしてください",
        "`<![CDATA[ … ]]>` 只能用在 `<svg>` 或 `<math>` 内部；请改为转义文本",
    ),
    (
        "compiler/duplicate-attribute.message",
        "Duplicate attribute.",
        "属性が重複しています。",
        "属性重复。",
    ),
    (
        "compiler/duplicate-attribute.help",
        "remove one of the two, or merge their values",
        "どちらか一方を削除するか、値を 1 つにまとめてください",
        "请删除其中一个，或将它们的值合并",
    ),
    (
        "compiler/end-tag-with-attributes.message",
        "End tag cannot have attributes.",
        "終了タグには属性を指定できません。",
        "结束标签不能带有属性。",
    ),
    (
        "compiler/end-tag-with-attributes.help",
        "move the attributes to the matching start tag",
        "属性は対応する開始タグに移してください",
        "请把这些属性移到对应的开始标签上",
    ),
    (
        "compiler/end-tag-with-trailing-solidus.message",
        "Trailing solidus not allowed in end tags.",
        "終了タグの末尾にスラッシュは付けられません。",
        "结束标签末尾不能有斜杠。",
    ),
    (
        "compiler/end-tag-with-trailing-solidus.help",
        "write `</div>`, not `</div/>`",
        "`</div/>` ではなく `</div>` と書いてください",
        "请写成 `</div>`，而不是 `</div/>`",
    ),
    (
        "compiler/eof-before-tag-name.message",
        "Unexpected EOF in tag.",
        "タグ名の前でファイルが終わっています。",
        "标签名之前意外遇到文件结尾。",
    ),
    (
        "compiler/eof-before-tag-name.help",
        "the file ends right after `<`; finish the tag or remove the stray `<`",
        "`<` の直後でファイルが終わっています。タグを書き終えるか、余分な `<` を削除してください",
        "文件在 `<` 之后就结束了；请补全标签或删除多余的 `<`",
    ),
    (
        "compiler/eof-in-cdata.message",
        "EOF in CDATA section.",
        "CDATA セクションの途中でファイルが終わっています。",
        "CDATA 区块中意外遇到文件结尾。",
    ),
    (
        "compiler/eof-in-cdata.help",
        "close the section with `]]>`",
        "`]]>` でセクションを閉じてください",
        "请用 `]]>` 结束该区块",
    ),
    (
        "compiler/eof-in-comment.message",
        "EOF in comment.",
        "コメントの途中でファイルが終わっています。",
        "注释中意外遇到文件结尾。",
    ),
    (
        "compiler/eof-in-comment.help",
        "close the comment with `-->`",
        "`-->` でコメントを閉じてください",
        "请用 `-->` 结束注释",
    ),
    (
        "compiler/eof-in-script-html-comment-like-text.message",
        "EOF in script.",
        "スクリプトの途中でファイルが終わっています。",
        "脚本中意外遇到文件结尾。",
    ),
    (
        "compiler/eof-in-script-html-comment-like-text.help",
        "a `<!--` inside `<script>` is never closed; remove it or close it with `-->`",
        "`<script>` 内の `<!--` が閉じられていません。削除するか `-->` で閉じてください",
        "`<script>` 中的 `<!--` 没有闭合；请删除它或用 `-->` 闭合",
    ),
    (
        "compiler/eof-in-tag.message",
        "EOF in tag.",
        "タグの途中でファイルが終わっています。",
        "标签中意外遇到文件结尾。",
    ),
    (
        "compiler/eof-in-tag.help",
        "finish the tag with `>` before the end of the file",
        "ファイルが終わる前に `>` でタグを閉じてください",
        "请在文件结束前用 `>` 闭合标签",
    ),
    (
        "compiler/incorrectly-closed-comment.message",
        "Incorrectly closed comment.",
        "コメントの閉じ方が正しくありません。",
        "注释的闭合方式不正确。",
    ),
    (
        "compiler/incorrectly-closed-comment.help",
        "close comments with `-->`, not `--!>`",
        "コメントは `--!>` ではなく `-->` で閉じてください",
        "请用 `-->` 而不是 `--!>` 闭合注释",
    ),
    (
        "compiler/incorrectly-opened-comment.message",
        "Incorrectly opened comment.",
        "コメントの開き方が正しくありません。",
        "注释的开始方式不正确。",
    ),
    (
        "compiler/incorrectly-opened-comment.help",
        "open comments with `<!--`; `<!` alone starts a bogus comment",
        "コメントは `<!--` で始めてください。`<!` だけでは不正なコメントとして扱われます",
        "请用 `<!--` 开始注释；单独的 `<!` 会被当作无效注释",
    ),
    (
        "compiler/invalid-first-character-of-tag-name.message",
        "Invalid first character of tag name.",
        "タグ名の先頭の文字が不正です。",
        "标签名的首字符无效。",
    ),
    (
        "compiler/invalid-first-character-of-tag-name.help",
        "a tag name starts with an ASCII letter; write `&lt;` for a literal `<`",
        "タグ名は英字で始めてください。`<` という文字そのものを表示するには `&lt;` と書きます",
        "标签名必须以 ASCII 字母开头；如需显示字面量 `<`，请写成 `&lt;`",
    ),
    (
        "compiler/missing-attribute-value.message",
        "Attribute value expected.",
        "属性値がありません。",
        "缺少属性值。",
    ),
    (
        "compiler/missing-attribute-value.help",
        "give the attribute a value after `=`, or remove the `=`",
        "`=` の後に値を書くか、`=` を削除してください",
        "请在 `=` 后写上属性值，或删除 `=`",
    ),
    (
        "compiler/missing-end-tag-name.message",
        "End tag name expected.",
        "終了タグの名前がありません。",
        "缺少结束标签名。",
    ),
    (
        "compiler/missing-end-tag-name.help",
        "`</>` is not an end tag; name the element, as in `</div>`",
        "`</>` は終了タグになりません。`</div>` のように要素名を書いてください",
        "`</>` 不是有效的结束标签；请写上元素名，例如 `</div>`",
    ),
    (
        "compiler/missing-whitespace-between-attributes.message",
        "Whitespace expected between attributes.",
        "属性の間に空白が必要です。",
        "属性之间需要空白。",
    ),
    (
        "compiler/missing-whitespace-between-attributes.help",
        "separate the attributes with a space",
        "属性と属性の間を空白で区切ってください",
        "请用空格分隔各个属性",
    ),
    (
        "compiler/nested-comment.message",
        "Nested comments are not allowed.",
        "コメントを入れ子にすることはできません。",
        "不允许嵌套注释。",
    ),
    (
        "compiler/nested-comment.help",
        "a comment cannot contain `<!--`; close the outer comment first",
        "コメントの中に `<!--` は書けません。外側のコメントを先に閉じてください",
        "注释中不能包含 `<!--`；请先闭合外层注释",
    ),
    (
        "compiler/unexpected-character-in-attribute-name.message",
        "Unexpected character in attribute name.",
        "属性名に使えない文字が含まれています。",
        "属性名中出现了意外的字符。",
    ),
    (
        "compiler/unexpected-character-in-attribute-name.help",
        "`\"`, `'` and `<` cannot appear in an attribute name; check for a missing `=` or quote",
        "属性名に `\"`、`'`、`<` は使えません。`=` や引用符の書き忘れがないか確認してください",
        "属性名中不能出现 `\"`、`'` 或 `<`；请检查是否漏写了 `=` 或引号",
    ),
    (
        "compiler/unexpected-character-in-unquoted-attribute-value.message",
        "Unexpected character in unquoted attribute value.",
        "引用符で囲まれていない属性値に使えない文字が含まれています。",
        "未加引号的属性值中出现了意外的字符。",
    ),
    (
        "compiler/unexpected-character-in-unquoted-attribute-value.help",
        "quote the value: one containing `\"`, `'`, `<`, `=` or `` ` `` must be quoted",
        "値を引用符で囲んでください。`\"`、`'`、`<`、`=`、`` ` `` を含む値には引用符が必要です",
        "请给属性值加上引号；包含 `\"`、`'`、`<`、`=` 或 `` ` `` 的值必须加引号",
    ),
    (
        "compiler/unexpected-equals-sign-before-attribute-name.message",
        "Unexpected equals sign before attribute name.",
        "属性名の前に予期しない `=` があります。",
        "属性名前出现了意外的等号。",
    ),
    (
        "compiler/unexpected-equals-sign-before-attribute-name.help",
        "an attribute name cannot start with `=`; remove the stray `=`",
        "属性名を `=` で始めることはできません。余分な `=` を削除してください",
        "属性名不能以 `=` 开头；请删除多余的 `=`",
    ),
    (
        "compiler/unexpected-null-character.message",
        "Unexpected null character.",
        "予期しない NULL 文字があります。",
        "出现了意外的空字符。",
    ),
    (
        "compiler/unexpected-null-character.help",
        "remove the U+0000 character; it usually means a corrupted file or stray binary data",
        "U+0000 文字を削除してください。多くの場合、ファイルの破損やバイナリデータの混入が原因です",
        "请删除 U+0000 字符；它通常意味着文件已损坏或混入了二进制数据",
    ),
    (
        "compiler/unexpected-question-mark-instead-of-tag-name.message",
        "Invalid tag name.",
        "タグ名が不正です。",
        "无效的标签名。",
    ),
    (
        "compiler/unexpected-question-mark-instead-of-tag-name.help",
        "`<?` starts a processing instruction, which HTML does not support; remove it",
        "`<?` は処理命令の書き出しですが、HTML では使えません。削除してください",
        "`<?` 表示处理指令，而 HTML 不支持它；请将其删除",
    ),
    (
        "compiler/unexpected-solidus-in-tag.message",
        "Unexpected solidus in tag.",
        "タグの中に予期しないスラッシュがあります。",
        "标签中出现了意外的斜杠。",
    ),
    (
        "compiler/unexpected-solidus-in-tag.help",
        "a `/` is only allowed right before `>`, to self-close the tag",
        "`/` を書けるのは、タグを自己終了させる `>` の直前だけです",
        "`/` 只能出现在 `>` 之前，用于自闭合标签",
    ),
];
