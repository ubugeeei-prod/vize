//! Text the terminal cannot print naively: east-asian wide characters and
//! emoji, tabs, controls and bidirectional overrides, CRLF, and the end of
//! the file.

use super::{Case, diagnostic, fix, help, nth, primary, secondary, span, tr};
use vize_davinci::diagnostic::Severity;
use vize_s0::Span;

const WIDE_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue"

const ユーザー名 = ref("うぶげ")
</script>

<template>
  <p class="挨拶">ようこそ、{{ ユーザ名 }}さん！🎉</p>
</template>
"#;

pub const WIDE_CHARACTERS: Case = Case {
    name: "wide_characters",
    path: "src/components/あいさつ.vue",
    source: WIDE_SOURCE,
    diagnostics: |locale| {
        let used = span(WIDE_SOURCE, "ユーザ名");
        let declared = span(WIDE_SOURCE, "ユーザー名");
        let message = tr(
            locale,
            "`ユーザ名` is not defined.",
            "`ユーザ名` は定義されていません。",
            "`ユーザ名` 未定义。",
        );
        let produced = diagnostic(Severity::Error, used, message)
            .with_part(primary(
                used,
                tr(
                    locale,
                    "not declared in this component",
                    "このコンポーネントでは宣言されていません",
                    "该组件中没有声明",
                ),
            ))
            .with_part(secondary(
                declared,
                tr(
                    locale,
                    "a binding with a similar name is declared here",
                    "よく似た名前の変数はここで宣言されています",
                    "此处声明了名称相近的绑定",
                ),
            ))
            .with_part(help(
                used,
                tr(
                    locale,
                    "did you mean `ユーザー名`?",
                    "`ユーザー名` の誤りではありませんか？",
                    "是否应为 `ユーザー名`？",
                ),
            ))
            .with_part(fix(used, "ユーザー名"));
        vec![(Some("vue/no-undefined-refs"), produced)]
    },
};

const CONTROL_SOURCE: &str =
    "<template>\r\n\t<p title=\"\u{202e}fdp.exe\">\tdownload</p>\r\n</template>\r\n";

pub const CONTROL_CHARACTERS: Case = Case {
    name: "control_characters",
    path: "src/components/Download.vue",
    source: CONTROL_SOURCE,
    diagnostics: |locale| {
        let hidden = span(CONTROL_SOURCE, "\u{202e}");
        let value = nth(CONTROL_SOURCE, "fdp.exe", 0);
        let message = tr(
            locale,
            "Bidirectional control character in an attribute value.",
            "属性値に双方向テキストの制御文字が含まれています。",
            "属性值中包含双向文本控制字符。",
        );
        let produced = diagnostic(Severity::Warning, hidden, message)
            .with_part(primary(
                hidden,
                tr(
                    locale,
                    "a right-to-left override hides the real order of what follows",
                    "右から左への上書き文字があり、後に続く文字の本当の並びが見えなくなっています",
                    "从右到左覆盖字符隐藏了其后文本的真实顺序",
                ),
            ))
            .with_part(secondary(
                value,
                tr(
                    locale,
                    "shown reversed, as `exe.pdf`",
                    "画面上では `exe.pdf` と逆順に表示されます",
                    "显示时顺序颠倒，看起来像 `exe.pdf`",
                ),
            ))
            .with_part(fix(hidden, ""));
        vec![(None, produced)]
    },
};

const END_OF_FILE_SOURCE: &str = "<template>\n  <div class=\"card\">\n    <p>Card</p>\n";

pub const END_OF_FILE: Case = Case {
    name: "end_of_file",
    path: "src/components/Card.vue",
    source: END_OF_FILE_SOURCE,
    diagnostics: |locale| {
        let end = END_OF_FILE_SOURCE.len() as u32;
        let eof = Span::new(end, end);
        let opened = span(END_OF_FILE_SOURCE, r#"<div class="card">"#);
        let message = tr(
            locale,
            "Element is missing end tag.",
            "要素の終了タグがありません。",
            "元素缺少结束标签。",
        );
        let produced = diagnostic(Severity::Error, eof, message)
            .with_part(primary(
                eof,
                tr(
                    locale,
                    "the file ends here",
                    "ここでファイルが終わっています",
                    "文件在此处结束",
                ),
            ))
            .with_part(secondary(
                opened,
                tr(
                    locale,
                    "this `<div>` is never closed",
                    "この `<div>` が閉じられていません",
                    "这个 `<div>` 从未闭合",
                ),
            ))
            .with_part(fix(eof, "  </div>"));
        vec![(None, produced)]
    },
};

/// No trailing newline; a tab; a two-byte `é` at bytes 3..5.
const EDGE_SOURCE: &str = "a\tbé\n\nc";

/// Degenerate input the renderer must still draw faithfully: an inverted span,
/// a span past the end, a span splitting a character, an empty headline, a
/// headline with an escape byte, a primary part away from the diagnostic's
/// own span, and overlapping edits (which cannot apply together, so they
/// render as two fixes).
pub const EDGE_CASES: Case = Case {
    name: "edge_cases",
    path: "edge.vue",
    source: EDGE_SOURCE,
    diagnostics: |locale| {
        let inverted = diagnostic(Severity::Error, Span::new(5, 2), "");
        let past_end = diagnostic(
            Severity::Warning,
            Span::new(100, 200),
            tr(
                locale,
                "Past the end.",
                "末尾を越えています。",
                "超出末尾。",
            ),
        );
        let split = diagnostic(
            Severity::Info,
            Span::new(4, 5),
            tr(
                locale,
                "Escape \u{1b}[31mbytes\u{1b}[0m stay visible.",
                "エスケープ \u{1b}[31mバイト\u{1b}[0m も見える形で表示されます。",
                "转义 \u{1b}[31m字节\u{1b}[0m 也会以可见形式显示。",
            ),
        );
        let elsewhere = diagnostic(
            Severity::Hint,
            Span::new(0, 1),
            tr(
                locale,
                "Two primaries.",
                "主な箇所が 2 つあります。",
                "有两个主要位置。",
            ),
        )
        .with_part(primary(
            Span::new(7, 8),
            tr(locale, "labelled", "ラベル付き", "带标签"),
        ))
        .with_part(fix(Span::new(0, 3), "x"))
        .with_part(fix(Span::new(2, 4), "y"));
        vec![
            (None, inverted),
            (Some("edge/past-end"), past_end),
            (Some("edge/split"), split),
            (Some("edge/elsewhere"), elsewhere),
        ]
    },
};
