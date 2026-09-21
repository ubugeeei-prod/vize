//! A tag's fact-table ids in all three namespaces with one lookup — the
//! skeleton builder asks this for every element.

use std::sync::LazyLock;

use vize_s0::{CompactString, FxHashMap};

use super::facts::{ElemId, Ns, facts};

const NAMESPACES: [Ns; 3] = [Ns::Html, Ns::Svg, Ns::MathMl];

/// Every table name, ASCII-lowercased, to [`super::facts::Facts::id`] of
/// that lowercase tag in each namespace.
static BY_TAG: LazyLock<FxHashMap<CompactString, [Option<ElemId>; 3]>> = LazyLock::new(|| {
    let table = facts();
    let mut map = FxHashMap::default();
    for (_, _, name) in table.universe() {
        let lower = CompactString::new(name.to_ascii_lowercase());
        let ids = NAMESPACES.map(|ns| table.id(ns, &lower));
        map.entry(lower).or_insert(ids);
    }
    map
});

/// `[Html, Svg, MathMl].map(|ns| facts().id(ns, tag))`, in one lookup for a
/// lowercase tag: any name a lowercase tag matches (exactly or ASCII
/// case-insensitively) lowercases to that tag.
pub fn tag_ids(tag: &str) -> [Option<ElemId>; 3] {
    if tag.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return NAMESPACES.map(|ns| facts().id(ns, tag));
    }
    BY_TAG.get(tag).copied().unwrap_or([None; 3])
}

#[cfg(test)]
mod tests {
    use super::{NAMESPACES, tag_ids};
    use crate::html_content_model::facts;

    #[test]
    fn one_lookup_agrees_with_three() {
        let mut tags: Vec<&str> = facts().universe().map(|(_, _, name)| name).collect();
        tags.extend(["foreignobject", "FOREIGNOBJECT", "Div", "my-card", "x", ""]);
        for tag in tags {
            let lower = tag.to_ascii_lowercase();
            for probe in [tag, lower.as_str()] {
                assert_eq!(
                    tag_ids(probe),
                    NAMESPACES.map(|ns| facts().id(ns, probe)),
                    "{probe}"
                );
            }
        }
    }
}
