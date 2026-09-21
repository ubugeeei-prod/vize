//! Catalogue entries for Vapor, petite-vue, script, Musea, SSR and HTML lint rules, part 2 of 2.
//! Registered by [`crate::i18n_messages`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "html/no-empty-palpable-content.message",
        "<{tag}> element is empty but expects visible content",
        "<{tag}>要素は空ですが、表示コンテンツが期待されています",
        "<{tag}>元素为空，但应包含可见内容",
    ),
    (
        "html/no-empty-palpable-content.help",
        "Add text content, child elements, or use aria-label for accessible content.",
        "テキストコンテンツ、子要素を追加するか、アクセシブルなコンテンツにはaria-labelを使用してください。",
        "添加文本内容、子元素，或使用aria-label提供无障碍内容。",
    ),
    (
        "html/no-duplicate-dt.message",
        "Duplicate <dt> term \"{term}\" in <dl>",
        "<dl>内に重複する<dt>用語「{term}」があります",
        "<dl>中存在重复的<dt>术语\"{term}\"",
    ),
    (
        "html/no-duplicate-dt.help",
        "Each term in a definition list should be unique. Combine definitions under a single <dt> or use distinct terms.",
        "定義リスト内の各用語は一意であるべきです。1つの<dt>に定義をまとめるか、異なる用語を使用してください。",
        "定义列表中的每个术语应该是唯一的。将定义合并到单个<dt>下或使用不同的术语。",
    ),
];
