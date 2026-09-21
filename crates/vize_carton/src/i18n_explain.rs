//! Vocabulary of `vize explain` pages (P4-14c): the labels a page prints
//! around a code's catalogued description and help. The severity words and
//! `help` come from the renderer vocabulary ([`crate::i18n_render`]), so a
//! page and a rendered diagnostic say the same thing the same way.

/// `(key, en, ja, zh)`. Registered by [`crate::i18n_render`].
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    ("explain.kind.rule", "lint rule", "lint ルール", "lint 规则"),
    (
        "explain.kind.compiler",
        "compiler error",
        "コンパイラーエラー",
        "编译器错误",
    ),
    (
        "explain.kind.cross_file",
        "cross-file diagnostic",
        "ファイル横断の診断",
        "跨文件诊断",
    ),
    (
        "explain.kind.verifier",
        "stage verifier invariant",
        "ステージ検証器の不変条件",
        "阶段校验器不变量",
    ),
    (
        "explain.kind.type_check",
        "type-check diagnostic",
        "型チェックの診断",
        "类型检查诊断",
    ),
    (
        "explain.kind.sfc",
        "SFC compiler diagnostic",
        "SFC コンパイラーの診断",
        "SFC 编译器诊断",
    ),
    (
        "explain.kind.typescript",
        "TypeScript error",
        "TypeScript のエラー",
        "TypeScript 错误",
    ),
    ("explain.category", "category", "カテゴリー", "类别"),
    ("explain.severity", "severity", "重大度", "严重级别"),
    (
        "explain.default_severity",
        "default severity",
        "既定の重大度",
        "默认严重级别",
    ),
    ("explain.stage", "stage", "ステージ", "阶段"),
    ("explain.autofix", "autofix", "自動修正", "自动修复"),
    ("explain.autofix.yes", "yes", "あり", "支持"),
    ("explain.autofix.no", "no", "なし", "不支持"),
    ("explain.docs", "docs", "ドキュメント", "文档"),
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
