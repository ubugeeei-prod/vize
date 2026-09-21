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
    for &(key, en, ja, zh) in ENTRIES.iter().chain(crate::i18n_explain::ENTRIES) {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
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
    // Witness-derived "why" notes: `= note: because <fact>`. Japanese and
    // Chinese label the footer as the grounds and state the fact plainly.
    ("render.why", "note", "根拠", "依据"),
    ("render.because", "because {fact}", "{fact}", "{fact}"),
    (
        "render.fact_fallback",
        "fact group {group} holds for {subject}",
        "{subject} についてファクトグループ {group} が成立",
        "事实组 {group} 对 {subject} 成立",
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
