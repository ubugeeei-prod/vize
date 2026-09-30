//! Vocabulary of the Davinci diagnostic renderer (`vize_davinci::render`).
//!
//! One entry per `vize_davinci::render::Phrase`, keyed by `Phrase::key`.
//! `tests/tooling/davinci-diagnostic-catalog.test.ts` enumerates the phrases
//! from the renderer's source and fails when any key is missing here in any
//! locale, or when the `en` column drifts from the renderer's built-in
//! `EnglishCatalog`.
//!
//! The words sit in a rustc-style frame (`error[code]: …`, `= help: …`), so
//! they are the short labels a native reader expects there, not glosses of
//! the English: Japanese reads 警告 and ヒント in that position, and keeps
//! 提案 for a hint-severity finding so it never collides with the help label;
//! Chinese follows the established 错误 / 警告 / 帮助 rendering of compiler
//! output.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert the renderer vocabulary into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
    crate::i18n_explain::register(messages);
}

/// `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    ("render.error", "error", "エラー", "错误"),
    ("render.warning", "warning", "警告", "警告"),
    ("render.info", "info", "情報", "信息"),
    ("render.hint", "hint", "提案", "提示"),
    ("render.help", "help", "ヒント", "帮助"),
    (
        "render.suggested_fix",
        "suggested fix",
        "次のように修正できます",
        "可以这样修复",
    ),
    // Witness-derived why notes: `= note: because <fact>`. The footer word
    // names the grounds; Japanese and Chinese then state the fact plainly.
    ("render.why", "note", "根拠", "依据"),
    ("render.because", "because {fact}", "{fact}", "{fact}"),
    (
        "render.fact_fallback",
        "fact group {group} holds for {subject}",
        "{subject} についてファクトグループ {group} が成立しています",
        "事实组 {group} 对 {subject} 成立",
    ),
    (
        "render.witness.unused_bindings",
        "{subject} is a <script setup> binding that nothing reads",
        "{subject} は <script setup> の束縛で、どこからも読まれていません",
        "{subject} 是 <script setup> 中没有任何代码读取的绑定",
    ),
    (
        "render.witness.html_elements",
        "{subject} is the ancestor element this nesting proof cites",
        "{subject} は、この入れ子の証明が根拠にする祖先要素です",
        "{subject} 是这条嵌套证明所依据的祖先元素",
    ),
    (
        "render.witness.html_composed_nesting",
        "{subject} is rendered where its parent forbids that child",
        "{subject} は、親が禁じる位置に描画されています",
        "{subject} 被渲染在父元素禁止其子内容出现的位置",
    ),
    // The closing line of a `vize lint --format rich` report.
    (
        "render.summary",
        "{errors} and {warnings} in {files}",
        "{files}を検査し、{errors}、{warnings}が見つかりました",
        "检查了 {files}，发现 {errors}、{warnings}",
    ),
    (
        "render.summary.clean",
        "No problems found in {files}",
        "{files}を検査し、問題は見つかりませんでした",
        "检查了 {files}，未发现问题",
    ),
    (
        "render.summary.errors.one",
        "1 error",
        "エラー 1 件",
        "1 个错误",
    ),
    (
        "render.summary.errors.other",
        "{count} errors",
        "エラー {count} 件",
        "{count} 个错误",
    ),
    (
        "render.summary.warnings.one",
        "1 warning",
        "警告 1 件",
        "1 个警告",
    ),
    (
        "render.summary.warnings.other",
        "{count} warnings",
        "警告 {count} 件",
        "{count} 个警告",
    ),
    (
        "render.summary.files.one",
        "1 file",
        "1 ファイル",
        "1 个文件",
    ),
    (
        "render.summary.files.other",
        "{count} files",
        "{count} ファイル",
        "{count} 个文件",
    ),
];
