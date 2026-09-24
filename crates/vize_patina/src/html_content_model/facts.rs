//! Loader for the committed WHATWG fact table (`whatwg.tsv`).
//!
//! The table is the single source of truth: this module owns no element
//! lists of its own. Every row the checker consults is named by a [`Row`]
//! variant, and the crate's unit tests fail if the table has a row the
//! checker does not know, lacks one it needs, or repeats one — so data and
//! checker cannot drift silently. At run time such a row is skipped.

use std::sync::LazyLock;

use vize_s0::{FxHashMap, SmallVec};

pub use super::rows::Row;
use super::tri::Tri;

mod load;

/// The committed fact table.
pub const WHATWG_TSV: &str = include_str!("whatwg.tsv");

/// Element namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ns {
    /// HTML namespace.
    Html,
    /// SVG namespace.
    Svg,
    /// MathML namespace.
    MathMl,
}

/// Index of an element in the fact table's universe.
pub type ElemId = u16;

/// Attribute predicates the table's conditional members reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Attr {
    /// `href`.
    Href,
    /// `controls`.
    Controls,
    /// `usemap`.
    Usemap,
    /// `itemprop`.
    Itemprop,
    /// `tabindex`.
    Tabindex,
    /// `type` in the Hidden state (ASCII case-insensitive `hidden`).
    TypeHidden,
    /// `color`, `face` or `size` (the `<font>` foreign-content breakout).
    FontPresentational,
    /// `encoding` of `text/html` or `application/xhtml+xml`.
    EncodingHtml,
}

/// A conditional membership predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    /// The attribute is present.
    Has(Attr),
    /// The attribute is absent (`input?type-not-hidden`).
    Lacks(Attr),
    /// The element has a `map` ancestor.
    InMap,
    /// `link` is allowed in the body: decided by `rel` keyword semantics the
    /// checker does not model, so always unknown.
    BodyOk,
}

/// A bitset over [`ElemId`]s.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bits([u64; 4]);

impl Bits {
    fn insert(&mut self, id: ElemId) {
        if let Some(word) = self.0.get_mut(usize::from(id / 64)) {
            *word |= 1 << (id % 64);
        }
    }

    /// Whether `id` is a member.
    pub fn has(&self, id: ElemId) -> bool {
        self.0
            .get(usize::from(id / 64))
            .is_some_and(|word| word & (1 << (id % 64)) != 0)
    }

    fn union(&mut self, other: &Self) {
        for (left, right) in self.0.iter_mut().zip(other.0) {
            *left |= right;
        }
    }
}

/// One loaded row: unconditional members, conditional members, and whether
/// non-whitespace text is a member.
#[derive(Debug, Clone, Default)]
pub struct Members {
    /// Unconditional members.
    pub always: Bits,
    /// Conditional members.
    pub conditional: SmallVec<[(ElemId, Cond); 4]>,
    /// Whether `#text` is a member.
    pub text: bool,
    /// The row's spec anchor.
    pub anchor: &'static str,
}

impl Members {
    fn absorb(&mut self, other: &Self) {
        self.always.union(&other.always);
        self.conditional.extend(other.conditional.iter().copied());
        self.text |= other.text;
    }

    /// The membership condition for `id`: `None` when never a member.
    pub fn condition(&self, id: ElemId) -> Option<Option<Cond>> {
        if self.always.has(id) {
            return Some(None);
        }
        self.conditional
            .iter()
            .find(|(member, _)| *member == id)
            .map(|(_, cond)| Some(*cond))
    }
}

/// The loaded fact table.
#[derive(Debug)]
pub struct Facts {
    names: Vec<(Ns, &'static str)>,
    ids: FxHashMap<(Ns, &'static str), ElemId>,
    /// Names with ASCII uppercase (SVG camelCase): the only names a
    /// lowercase tag can match without matching exactly.
    cased: Vec<(Ns, &'static str, ElemId)>,
    rows: Vec<Members>,
    children: FxHashMap<ElemId, Members>,
    /// Returned for a row the table failed to provide.
    empty: Members,
}

static FACTS: LazyLock<Facts> = LazyLock::new(|| Facts::parse(WHATWG_TSV));

/// The process-wide fact table.
pub fn facts() -> &'static Facts {
    &FACTS
}

impl Facts {
    /// Look up an element. HTML and MathML names are matched ASCII
    /// case-insensitively (the tokenizer lowercases tag names); SVG names are
    /// matched after the §13.2.6.5 case adjustment, which is also
    /// case-insensitive for every SVG name the table lists.
    pub fn id(&self, ns: Ns, tag: &str) -> Option<ElemId> {
        if let Some(id) = self.ids.get(&(ns, tag)) {
            return Some(*id);
        }
        // A tag without uppercase that missed the exact lookup can only match
        // a name that has uppercase (the hot path: one short scan).
        if !tag.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return self
                .cased
                .iter()
                .find(|(name_ns, name, _)| *name_ns == ns && name.eq_ignore_ascii_case(tag))
                .map(|(_, _, id)| *id);
        }
        self.names
            .iter()
            .position(|(name_ns, name)| *name_ns == ns && name.eq_ignore_ascii_case(tag))
            .map(|index| index as ElemId)
    }

    /// The `(namespace, local name)` of an element id.
    pub fn name(&self, id: ElemId) -> (Ns, &'static str) {
        // Ids only come from `intern`; an unknown one names nothing.
        self.names
            .get(usize::from(id))
            .copied()
            .unwrap_or((Ns::Html, ""))
    }

    /// A row's members.
    pub fn row(&self, row: Row) -> &Members {
        self.rows.get(row as usize).unwrap_or(&self.empty)
    }

    /// Unconditional membership of `id` in `row` (`None` ids are ordinary
    /// elements the table does not list: never a member).
    pub fn is(&self, row: Row, id: Option<ElemId>) -> bool {
        id.is_some_and(|id| self.row(row).always.has(id))
    }

    /// The content-model row of an HTML element, when the table records one.
    pub fn children(&self, id: ElemId) -> Option<&Members> {
        self.children.get(&id)
    }

    /// Every element the table names.
    pub fn universe(&self) -> impl Iterator<Item = (ElemId, Ns, &'static str)> + '_ {
        self.names
            .iter()
            .enumerate()
            .map(|(index, (ns, name))| (index as ElemId, *ns, *name))
    }
}

/// Evaluate a membership condition for an element with the given attribute
/// facts; structural conditions are supplied by the caller.
pub fn eval_attr_cond(cond: Cond, attr: impl Fn(Attr) -> Tri) -> Option<Tri> {
    match cond {
        Cond::Has(name) => Some(attr(name)),
        Cond::Lacks(name) => Some(attr(name).not()),
        Cond::BodyOk => Some(Tri::Maybe),
        Cond::InMap => None,
    }
}
