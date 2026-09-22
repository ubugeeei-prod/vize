//! Vocabulary of `vize explain` pages (P4-14c): the labels a page prints
//! around a code's catalogued description, its contract and its help.
//!
//! The severity words and `help` come from the renderer vocabulary
//! ([`crate::i18n_render`]), so a page and a rendered diagnostic say them
//! the same way. Registered by [`crate::i18n_render`].

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert the explain-page vocabulary into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    ("explain.kind.rule", "lint rule", "lint ルール", "lint 规则"),
    (
        "explain.kind.compiler",
        "compiler error",
        "コンパイラーエラー",
        "编译器错误",
    ),
    ("explain.tier", "tier", "精度", "精度层级"),
    ("explain.domain", "domain", "対象", "适用范围"),
    ("explain.severity", "severity", "重大度", "严重级别"),
    ("explain.category", "category", "カテゴリー", "类别"),
    ("explain.stage", "stage", "ステージ", "阶段"),
    ("explain.autofix", "autofix", "自動修正", "自动修复"),
    ("explain.autofix.yes", "yes", "あり", "支持"),
    ("explain.autofix.no", "no", "なし", "不支持"),
    ("explain.docs", "docs", "ドキュメント", "文档"),
    ("explain.example.invalid", "invalid", "不正な例", "错误示例"),
    ("explain.example.valid", "valid", "正しい例", "正确示例"),
    (
        "explain.unknown",
        "no diagnostic has the code `{code}`",
        "`{code}` という診断コードはありません",
        "没有代码为 `{code}` 的诊断",
    ),
    (
        "explain.did_you_mean",
        "did you mean `{code}`?",
        "`{code}` のことですか？",
        "你是指 `{code}` 吗？",
    ),
    (
        "explain.usage",
        "give a code such as `vue/require-v-for-key` or `compiler/v-if-no-expression`, or `--list` for every code",
        "`vue/require-v-for-key` や `compiler/v-if-no-expression` のようにコードを指定するか、`--list` ですべてのコードを表示してください",
        "请指定代码，例如 `vue/require-v-for-key` 或 `compiler/v-if-no-expression`，或使用 `--list` 列出所有代码",
    ),
];
