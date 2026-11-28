//! A tag's fact-table ids in all three namespaces with one lookup — the
//! skeleton builder asks this for every element.

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;

use vize_l0::{CompactString, FxHashMap};

use super::facts::{ElemId, Facts, Ns, facts};

const NAMESPACES: [Ns; 3] = [Ns::Html, Ns::Svg, Ns::MathMl];

/// Every table name, ASCII-lowercased, to [`Facts::id`] of that lowercase tag
/// in each namespace.
static BY_TAG: LazyLock<FxHashMap<TagKey, [Option<ElemId>; 3]>> = LazyLock::new(|| by_tag(facts()));

/// A lowercase table name. It hashes exactly as its `str` does, so lookups
/// borrow a `&str`, and the hash is always inlined into the table build:
/// whether LLVM inlines `CompactString`'s own hash there depends on what
/// else shares the codegen unit.
#[derive(Debug, PartialEq, Eq)]
struct TagKey(CompactString);

impl Hash for TagKey {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl Borrow<str> for TagKey {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/// [`BY_TAG`] for `table`, in one pass over its universe.
///
/// A lowercase tag only matches names that lowercase to it, so each slot is
/// decided among the names sharing its key, with [`Facts::id`]'s precedence:
/// the name equal to the key (an exact `ids` hit) wins, and otherwise the
/// first uppercase-bearing name, since `cased` holds those in id order — the
/// order the universe yields them.
fn by_tag(table: &Facts) -> FxHashMap<TagKey, [Option<ElemId>; 3]> {
    let mut map: FxHashMap<TagKey, [Option<ElemId>; 3]> =
        FxHashMap::with_capacity_and_hasher(table.universe().count(), Default::default());
    for (id, ns, name) in table.universe() {
        let exact = !name.bytes().any(|byte| byte.is_ascii_uppercase());
        let mut lower = CompactString::new(name);
        if !exact {
            lower.make_ascii_lowercase();
        }
        let [html, svg, math] = map.entry(TagKey(lower)).or_default();
        let slot = match ns {
            Ns::Html => html,
            Ns::Svg => svg,
            Ns::MathMl => math,
        };
        if exact || slot.is_none() {
            *slot = Some(id);
        }
    }
    map
}

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
    use super::{NAMESPACES, by_tag, tag_ids};
    use crate::html_content_model::{Facts, Ns, facts};

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

    /// Names that collide after lowercasing, which the committed table does
    /// not have: an exact name beats an earlier or later cased one, the first
    /// cased name beats later ones, and namespaces never mix.
    #[test]
    fn one_pass_build_keeps_lookup_precedence() {
        let table = Facts::parse(
            "set\tspecial\tx\tsvg:fooBar svg:foobar svg:FOObar math:Mx math:mX svg:mx b math:q math:Q \
             svg:Zed svg:zED svg:zed math:Zed\n",
        );
        let map = by_tag(&table);
        let mut keys: Vec<&str> = map.keys().map(|key| key.0.as_str()).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["b", "foobar", "mx", "q", "zed"]);
        for (key, ids) in &map {
            assert_eq!(*ids, NAMESPACES.map(|ns| table.id(ns, &key.0)), "{key:?}");
        }
        let id = |ns, name| {
            table
                .universe()
                .find(|(_, name_ns, table_name)| *name_ns == ns && *table_name == name)
                .map(|(id, _, _)| id)
        };
        let get = |key: &str| map.get(key).copied();
        assert_eq!(get("foobar"), Some([None, id(Ns::Svg, "foobar"), None]));
        assert_eq!(
            get("mx"),
            Some([None, id(Ns::Svg, "mx"), id(Ns::MathMl, "Mx")])
        );
        assert_eq!(
            get("zed"),
            Some([None, id(Ns::Svg, "zed"), id(Ns::MathMl, "Zed")])
        );
        assert_eq!(get("q"), Some([None, None, id(Ns::MathMl, "q")]));
        assert_eq!(get("b"), Some([id(Ns::Html, "b"), None, None]));
    }
}
