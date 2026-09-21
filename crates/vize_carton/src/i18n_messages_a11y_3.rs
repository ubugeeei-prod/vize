//! Catalogue entries for accessibility lint rules (`a11y/*`), part 3 of 3.
//! Registered by [`crate::i18n_messages`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "a11y/landmark-roles.help_duplicate_main",
        "A page should have exactly one <main> landmark. Remove duplicate <main> elements or use role=\"main\" on only one element.",
        "ページには<main>ランドマークが1つだけ必要です。重複する<main>要素を削除するか、role=\"main\"を1つの要素にのみ使用してください。",
        "页面应该只有一个<main>地标。删除重复的<main>元素或仅在一个元素上使用role=\"main\"。",
    ),
    (
        "a11y/landmark-roles.missing_label",
        "Multiple \"{role}\" landmarks found without distinct labels",
        "複数の\"{role}\"ランドマークがラベルなしで存在します",
        "发现多个\"{role}\"地标但缺少独特标签",
    ),
    (
        "a11y/landmark-roles.help_missing_label",
        "When multiple landmarks of the same type exist, each should have a unique aria-label or aria-labelledby to help users distinguish them.",
        "同じ種類のランドマークが複数ある場合、ユーザーが区別できるようにそれぞれにaria-labelまたはaria-labelledbyを付けてください。",
        "当存在多个相同类型的地标时，每个都应有唯一的aria-label或aria-labelledby以帮助用户区分。",
    ),
];
