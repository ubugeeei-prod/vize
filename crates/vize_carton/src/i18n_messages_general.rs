//! Catalogue entries for shared vocabulary: lint categories, CLI and generic compiler strings.
//! Registered by [`crate::i18n_messages`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    ("test.hello", "Hello", "こんにちは", "你好"),
    (
        "test.greeting",
        "Hello, {name}!",
        "こんにちは、{name}さん！",
        "你好，{name}！",
    ),
    (
        "diagnostic.format",
        "[vize:{rule}] {message}",
        "[vize:{rule}] {message}",
        "[vize:{rule}] {message}",
    ),
    (
        "diagnostic.format.ts",
        "[vize:TS{code}] {message}",
        "[vize:TS{code}] {message}",
        "[vize:TS{code}] {message}",
    ),
    ("category.essential", "Essential", "必須", "必需"),
    (
        "category.strongly_recommended",
        "Strongly Recommended",
        "強く推奨",
        "强烈推荐",
    ),
    ("category.recommended", "Recommended", "推奨", "推荐"),
    ("category.vapor", "Vapor", "Vapor", "Vapor"),
    ("category.musea", "Musea", "Musea", "Musea"),
    ("category.script", "Script", "スクリプト", "脚本"),
    ("category.css", "CSS", "CSS", "CSS"),
    (
        "category.a11y",
        "Accessibility",
        "アクセシビリティ",
        "无障碍",
    ),
    ("category.security", "Security", "セキュリティ", "安全"),
    (
        "category.performance",
        "Performance",
        "パフォーマンス",
        "性能",
    ),
    (
        "compiler.parse_error",
        "Parse error: {message}",
        "パースエラー: {message}",
        "解析错误: {message}",
    ),
    (
        "compiler.unexpected_token",
        "Unexpected token '{token}'",
        "予期しないトークン'{token}'",
        "意外的标记'{token}'",
    ),
    (
        "compiler.unclosed_tag",
        "Unclosed tag <{tag}>",
        "閉じられていないタグ<{tag}>",
        "未关闭的标签<{tag}>",
    ),
    (
        "compiler.invalid_directive",
        "Invalid directive syntax: {directive}",
        "無効なディレクティブ構文: {directive}",
        "无效的指令语法: {directive}",
    ),
    (
        "compiler.missing_end_tag",
        "Missing end tag for <{tag}>",
        "<{tag}>の終了タグがありません",
        "<{tag}>缺少结束标签",
    ),
    (
        "cli.compiling",
        "Compiling {count} files...",
        "{count}ファイルをコンパイル中...",
        "正在编译{count}个文件...",
    ),
    (
        "cli.compiled",
        "Compiled {count} files in {time}ms",
        "{count}ファイルを{time}msでコンパイルしました",
        "在{time}ms内编译了{count}个文件",
    ),
    (
        "cli.error_count",
        "{count} error(s) found",
        "{count}個のエラーが見つかりました",
        "发现{count}个错误",
    ),
    (
        "cli.warning_count",
        "{count} warning(s) found",
        "{count}個の警告が見つかりました",
        "发现{count}个警告",
    ),
    (
        "cli.slow_file_warning",
        "Slow compilation detected for {file} ({time}ms)",
        "{file}のコンパイルが遅いです（{time}ms）",
        "{file}编译缓慢（{time}ms）",
    ),
    (
        "cli.suggest_split",
        "Consider splitting large components into smaller ones",
        "大きなコンポーネントを小さなものに分割することを検討してください",
        "考虑将大组件拆分为小组件",
    ),
    (
        "cli.suggest_template",
        "Large template detected ({size} bytes). Consider extracting parts into child components",
        "大きなテンプレートが検出されました（{size}バイト）。一部を子コンポーネントに抽出することを検討してください",
        "检测到大模板（{size}字节）。考虑将部分提取为子组件",
    ),
    (
        "category.html_conformance",
        "HTML Conformance",
        "HTML準拠",
        "HTML合规",
    ),
];
