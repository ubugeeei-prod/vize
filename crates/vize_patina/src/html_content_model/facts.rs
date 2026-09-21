//! Loader for the committed WHATWG fact table (`whatwg.tsv`).
//!
//! The table is the single source of truth: this module owns no element
//! lists of its own. Every row the checker consults is named by a [`Row`]
//! variant, and loading fails (at first use, and in the crate's unit tests)
//! if the table has a row the checker does not know, lacks one it needs, or
//! repeats one — so data and checker cannot drift silently.

use std::sync::LazyLock;

use vize_s0::{FxHashMap, SmallVec};

use super::rows::ROWS;
pub use super::rows::Row;
use super::tri::Tri;

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

fn parse_cond(text: &str) -> Cond {
    match text {
        "href" => Cond::Has(Attr::Href),
        "controls" => Cond::Has(Attr::Controls),
        "usemap" => Cond::Has(Attr::Usemap),
        "itemprop" => Cond::Has(Attr::Itemprop),
        "type-hidden" => Cond::Has(Attr::TypeHidden),
        "type-not-hidden" => Cond::Lacks(Attr::TypeHidden),
        "font-presentational" => Cond::Has(Attr::FontPresentational),
        "encoding-html" => Cond::Has(Attr::EncodingHtml),
        "in-map" => Cond::InMap,
        "body-ok" => Cond::BodyOk,
        other => panic!("whatwg.tsv: unknown member condition `{other}`"),
    }
}

/// A bitset over [`ElemId`]s.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bits([u64; 4]);

impl Bits {
    fn insert(&mut self, id: ElemId) {
        self.0[usize::from(id / 64)] |= 1 << (id % 64);
    }

    /// Whether `id` is a member.
    pub fn has(&self, id: ElemId) -> bool {
        self.0[usize::from(id / 64)] & (1 << (id % 64)) != 0
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
    rows: Vec<Members>,
    children: FxHashMap<ElemId, Members>,
}

static FACTS: LazyLock<Facts> = LazyLock::new(|| Facts::parse(WHATWG_TSV));

/// The process-wide fact table.
pub fn facts() -> &'static Facts {
    &FACTS
}

impl Facts {
    /// Parse a fact table. Panics on any malformed, unknown, duplicate or
    /// missing row: the table is committed data, so a bad table is a build
    /// defect that the unit tests surface before any lint runs.
    pub fn parse(tsv: &'static str) -> Self {
        let mut facts = Self {
            names: Vec::new(),
            ids: FxHashMap::default(),
            rows: vec![Members::default(); ROWS.len()],
            children: FxHashMap::default(),
        };
        let mut seen = [false; ROWS.len()];
        let mut categories: FxHashMap<&'static str, Members> = FxHashMap::default();
        for line in tsv
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let mut columns = line.split('\t');
            let (Some(kind), Some(name), Some(anchor), Some(members), None) = (
                columns.next(),
                columns.next(),
                columns.next(),
                columns.next(),
                columns.next(),
            ) else {
                panic!("whatwg.tsv: malformed row `{line}`");
            };
            let mut row = Members {
                anchor,
                ..Members::default()
            };
            for member in members.split(' ') {
                facts.add_member(&mut row, member, &categories);
            }
            if kind == "children" {
                let id = facts.intern(Ns::Html, name);
                assert!(
                    facts.children.insert(id, row).is_none(),
                    "whatwg.tsv: duplicate children row `{name}`"
                );
                continue;
            }
            let index = ROWS
                .iter()
                .position(|(_, row_kind, row_name)| *row_kind == kind && *row_name == name)
                .unwrap_or_else(|| panic!("whatwg.tsv: unknown row `{kind} {name}`"));
            assert!(!seen[index], "whatwg.tsv: duplicate row `{kind} {name}`");
            seen[index] = true;
            if kind == "category" {
                categories.insert(name, row.clone());
            }
            facts.rows[index] = row;
        }
        if let Some(missing) = seen.iter().position(|seen| !seen) {
            panic!(
                "whatwg.tsv: missing row `{} {}`",
                ROWS[missing].1, ROWS[missing].2
            );
        }
        facts
    }

    fn add_member(
        &mut self,
        row: &mut Members,
        member: &'static str,
        categories: &FxHashMap<&'static str, Members>,
    ) {
        if member == "#text" {
            row.text = true;
            return;
        }
        if let Some(category) = member.strip_prefix('@') {
            let category = categories
                .get(category)
                .unwrap_or_else(|| panic!("whatwg.tsv: `@{category}` used before its row"));
            row.absorb(category);
            return;
        }
        let (name, cond) = match member.split_once('?') {
            Some((name, cond)) => (name, Some(parse_cond(cond))),
            None => (member, None),
        };
        let (ns, local) = if let Some(local) = name.strip_prefix("svg:") {
            (Ns::Svg, local)
        } else if let Some(local) = name.strip_prefix("math:") {
            (Ns::MathMl, local)
        } else {
            (Ns::Html, name)
        };
        let id = self.intern(ns, local);
        match cond {
            Some(cond) => row.conditional.push((id, cond)),
            None => row.always.insert(id),
        }
    }

    fn intern(&mut self, ns: Ns, local: &'static str) -> ElemId {
        if let Some(id) = self.ids.get(&(ns, local)) {
            return *id;
        }
        let id = ElemId::try_from(self.names.len()).expect("element universe fits u16");
        assert!(
            id < 256,
            "whatwg.tsv: element universe exceeds the 256-bit set width"
        );
        self.names.push((ns, local));
        self.ids.insert((ns, local), id);
        id
    }

    /// Look up an element. HTML and MathML names are matched ASCII
    /// case-insensitively (the tokenizer lowercases tag names); SVG names are
    /// matched after the §13.2.6.5 case adjustment, which is also
    /// case-insensitive for every SVG name the table lists.
    pub fn id(&self, ns: Ns, tag: &str) -> Option<ElemId> {
        if let Some(id) = self.ids.get(&(ns, tag)) {
            return Some(*id);
        }
        self.names
            .iter()
            .position(|(name_ns, name)| *name_ns == ns && name.eq_ignore_ascii_case(tag))
            .map(|index| index as ElemId)
    }

    /// The `(namespace, local name)` of an element id.
    pub fn name(&self, id: ElemId) -> (Ns, &'static str) {
        self.names[usize::from(id)]
    }

    /// A row's members.
    pub fn row(&self, row: Row) -> &Members {
        &self.rows[row as usize]
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
