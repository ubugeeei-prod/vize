//! Parsing the committed fact table into [`Facts`].

use vize_s0::{FxHashMap, String, cstr};

use super::{Attr, Cond, ElemId, Facts, Members, Ns};
use crate::html_content_model::rows::ROWS;

impl Facts {
    /// Parse a fact table, skipping malformed, unknown or duplicate rows. The
    /// table is committed data, so the unit tests assert that
    /// [`Self::parse_reporting`] finds no defect before any lint runs.
    pub fn parse(tsv: &'static str) -> Self {
        Self::parse_reporting(tsv, &mut Vec::new())
    }

    /// [`Self::parse`], recording every table defect in `defects`.
    pub(in crate::html_content_model) fn parse_reporting(
        tsv: &'static str,
        defects: &mut Vec<String>,
    ) -> Self {
        let mut facts = Self {
            names: Vec::new(),
            ids: FxHashMap::default(),
            cased: Vec::new(),
            rows: vec![Members::default(); ROWS.len()],
            children: FxHashMap::default(),
            empty: Members::default(),
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
                defects.push(cstr!("malformed row `{line}`"));
                continue;
            };
            let mut row = Members {
                anchor,
                ..Members::default()
            };
            for member in members.split(' ') {
                if let Err(defect) = facts.add_member(&mut row, member, &categories) {
                    defects.push(defect);
                }
            }
            if kind == "children" {
                let Some(id) = facts.intern(Ns::Html, name) else {
                    defects.push(cstr!("element universe overflow at `{name}`"));
                    continue;
                };
                if facts.children.insert(id, row).is_some() {
                    defects.push(cstr!("duplicate children row `{name}`"));
                }
                continue;
            }
            let Some((index, seen)) = ROWS
                .iter()
                .zip(seen.iter_mut())
                .enumerate()
                .find(|(_, ((_, row_kind, row_name), _))| *row_kind == kind && *row_name == name)
                .map(|(index, (_, seen))| (index, seen))
            else {
                defects.push(cstr!("unknown row `{kind} {name}`"));
                continue;
            };
            if std::mem::replace(seen, true) {
                defects.push(cstr!("duplicate row `{kind} {name}`"));
                continue;
            }
            if kind == "category" {
                categories.insert(name, row.clone());
            }
            if let Some(slot) = facts.rows.get_mut(index) {
                *slot = row;
            }
        }
        for ((_, kind, name), seen) in ROWS.iter().zip(seen) {
            if !seen {
                defects.push(cstr!("missing row `{kind} {name}`"));
            }
        }
        facts
    }

    fn add_member(
        &mut self,
        row: &mut Members,
        member: &'static str,
        categories: &FxHashMap<&'static str, Members>,
    ) -> Result<(), String> {
        if member == "#text" {
            row.text = true;
            return Ok(());
        }
        if let Some(category) = member.strip_prefix('@') {
            let category = categories
                .get(category)
                .ok_or_else(|| cstr!("`@{category}` used before its row"))?;
            row.absorb(category);
            return Ok(());
        }
        let (name, cond) = match member.split_once('?') {
            Some((name, cond)) => (
                name,
                Some(parse_cond(cond).ok_or_else(|| cstr!("unknown condition `{cond}`"))?),
            ),
            None => (member, None),
        };
        let (ns, local) = if let Some(local) = name.strip_prefix("svg:") {
            (Ns::Svg, local)
        } else if let Some(local) = name.strip_prefix("math:") {
            (Ns::MathMl, local)
        } else {
            (Ns::Html, name)
        };
        let id = self
            .intern(ns, local)
            .ok_or_else(|| cstr!("element universe overflow at `{name}`"))?;
        match cond {
            Some(cond) => row.conditional.push((id, cond)),
            None => row.always.insert(id),
        }
        Ok(())
    }

    /// The id of `(ns, local)`, allocating one; `None` once the universe
    /// outgrows the 256-bit set width.
    fn intern(&mut self, ns: Ns, local: &'static str) -> Option<ElemId> {
        if let Some(id) = self.ids.get(&(ns, local)) {
            return Some(*id);
        }
        let id = ElemId::try_from(self.names.len())
            .ok()
            .filter(|id| *id < 256)?;
        self.names.push((ns, local));
        self.ids.insert((ns, local), id);
        if local.bytes().any(|byte| byte.is_ascii_uppercase()) {
            self.cased.push((ns, local, id));
        }
        Some(id)
    }
}

fn parse_cond(text: &str) -> Option<Cond> {
    Some(match text {
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
        _ => return None,
    })
}
