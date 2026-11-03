//! P4-15a: every exact/sound rule needs a seeded defect class.
//!
//! Classes come from the HTML-nesting generator and the snippet table in
//! `seed_exact_gen.rs`. A rule of tier `exact` or `sound` that has neither a
//! class nor a row in the ledger's `p4-15a-unclassed` list fails closed.
//! Listing a rule there triages the miss; it does not make this check pass.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::common;
use crate::seed_exact_gen;

pub const TABLE_REL: &str = "crates/vize_patina/src/rule_contracts/table.rs";
pub const LEDGER_REL: &str = "docs/davinci/plan/ledger-fn.md";
const LEDGER_START: &str = "<!-- p4-15a-unclassed -->";
const LEDGER_END: &str = "<!-- /p4-15a-unclassed -->";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Exact,
    Sound,
    Complete,
    Heuristic,
}

impl Tier {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "Exact" => Some(Self::Exact),
            "Sound" => Some(Self::Sound),
            "Complete" => Some(Self::Complete),
            "Heuristic" => Some(Self::Heuristic),
            _ => None,
        }
    }

    fn recalls(self) -> bool {
        matches!(self, Self::Exact | Self::Sound)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "Error" => Some(Self::Error),
            "Warning" => Some(Self::Warning),
            _ => None,
        }
    }

    pub fn lint_code(self) -> i64 {
        match self {
            Self::Error => 2,
            Self::Warning => 1,
        }
    }
}

pub struct Contract {
    pub name: String,
    pub tier: Tier,
    pub severity: Severity,
}

pub fn load_contracts(path: &Path) -> Result<Vec<Contract>, String> {
    let text = common::read_text(path)?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let Some(row) = parse_row(line, index + 1, path)? else {
            continue;
        };
        if !seen.insert(row.name.clone()) {
            return Err(format!(
                "{}: duplicate contract row \"{}\"",
                path.display(),
                row.name
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

pub fn load_ledger(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = common::read_text(path)?;
    let Some(start) = text.find(LEDGER_START) else {
        return Err(format!(
            "{} is missing the {LEDGER_START} list",
            path.display()
        ));
    };
    let body_at = start + LEDGER_START.len();
    let Some(end_rel) = text[body_at..].find(LEDGER_END) else {
        return Err(format!(
            "{} is missing the {LEDGER_END} marker",
            path.display()
        ));
    };
    let mut names = BTreeSet::new();
    for line in text[body_at..body_at + end_rel].lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some(name) = line
            .strip_prefix("- `")
            .and_then(|rest| rest.strip_suffix('`'))
        else {
            return Err(format!(
                "{}: unreadable unclassed row {line:?}",
                path.display()
            ));
        };
        if name.is_empty() || !names.insert(name.to_string()) {
            return Err(format!(
                "{}: duplicate or empty unclassed rule {name:?}",
                path.display()
            ));
        }
    }
    Ok(names)
}

pub fn check(
    repo_root: &Path,
    contracts: Option<&Path>,
    ledger: Option<&Path>,
) -> Result<u8, String> {
    let contracts_path = contracts
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join(TABLE_REL));
    let ledger_path = ledger
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join(LEDGER_REL));
    let rows = load_contracts(&contracts_path)?;
    let classes = seed_exact_gen::bindings()?;
    let triaged = load_ledger(&ledger_path)?;
    let summary = summarize(&rows, &classes, &triaged)?;
    for rule in &summary.classed_rules {
        println!("classed-rule {rule}");
    }
    for line in &summary.stray {
        println!("STRAY {line}");
    }
    for rule in &summary.untriaged {
        println!("UNTRIAGED {rule}");
    }
    for rule in &summary.stale {
        println!("STALE {rule}");
    }
    println!(
        "scope-proof: exact={} sound={} classed-rules={} classed-classes={} unclassed={} untriaged={} stale={}",
        summary.exact,
        summary.sound,
        summary.classed_rules.len(),
        summary.class_count,
        summary.missing.len(),
        summary.untriaged.len(),
        summary.stale.len()
    );
    Ok(summary.code())
}

struct Summary {
    exact: usize,
    sound: usize,
    class_count: usize,
    classed_rules: BTreeSet<String>,
    missing: BTreeSet<String>,
    untriaged: BTreeSet<String>,
    stale: BTreeSet<String>,
    stray: Vec<String>,
}

impl Summary {
    fn code(&self) -> u8 {
        if !self.stray.is_empty() || !self.untriaged.is_empty() || !self.stale.is_empty() {
            2
        } else if !self.missing.is_empty() {
            1
        } else {
            0
        }
    }
}

fn summarize(
    rows: &[Contract],
    classes: &[seed_exact_gen::ClassBinding],
    triaged: &BTreeSet<String>,
) -> Result<Summary, String> {
    let mut by_name = BTreeMap::<&str, &Contract>::new();
    for row in rows {
        if by_name.insert(row.name.as_str(), row).is_some() {
            return Err(format!("duplicate contract row \"{}\"", row.name));
        }
    }
    let mut id_rule = BTreeMap::<&str, &str>::new();
    for class in classes {
        if let Some(previous) = id_rule.insert(class.id, class.rule)
            && previous != class.rule
        {
            return Err(format!(
                "class \"{}\" cites both \"{previous}\" and \"{}\"",
                class.id, class.rule
            ));
        }
    }
    let mut classed_rules = BTreeSet::new();
    let mut stray = Vec::new();
    for (id, rule) in &id_rule {
        match by_name.get(rule).filter(|row| row.tier.recalls()) {
            Some(_) => {
                classed_rules.insert((*rule).to_string());
            }
            None => stray.push(format!(
                "class \"{id}\" cites \"{rule}\", which is not an exact or sound rule"
            )),
        }
    }
    let mut exact = 0usize;
    let mut sound = 0usize;
    let mut missing = BTreeSet::new();
    for row in rows {
        if !row.tier.recalls() {
            continue;
        }
        match row.tier {
            Tier::Exact => exact += 1,
            Tier::Sound => sound += 1,
            Tier::Complete | Tier::Heuristic => {}
        }
        if !classed_rules.contains(&row.name) {
            missing.insert(row.name.clone());
        }
    }
    let untriaged = missing.difference(triaged).cloned().collect();
    let stale = triaged.difference(&missing).cloned().collect();
    Ok(Summary {
        exact,
        sound,
        class_count: id_rule.len(),
        classed_rules,
        missing,
        untriaged,
        stale,
        stray,
    })
}

fn parse_row(line: &str, line_no: usize, path: &Path) -> Result<Option<Contract>, String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("row!(") {
        return Ok(None);
    }
    let err = |detail: &str| {
        format!(
            "{}:{line_no}: unreadable contract row ({detail})",
            path.display()
        )
    };
    let Some(body) = trimmed
        .strip_prefix("row!(")
        .and_then(|rest| rest.strip_suffix("),"))
    else {
        return Err(err("expected row!(...),"));
    };
    let body = body.trim();
    let Some(body) = body.strip_prefix('"') else {
        return Err(err("rule name"));
    };
    let Some((name, rest)) = body.split_once('"') else {
        return Err(err("rule name"));
    };
    if name.is_empty() {
        return Err(err("empty rule name"));
    }
    let Some(rest) = rest.trim_start().strip_prefix(',') else {
        return Err(err("tier"));
    };
    let Some((tier, rest)) = split_token(rest.trim_start()) else {
        return Err(err("tier"));
    };
    let Some(tier) = Tier::parse(tier) else {
        return Err(err("tier"));
    };
    let Some(rest) = rest.trim_start().strip_prefix(',') else {
        return Err(err("domain"));
    };
    let Some((domain, rest)) = split_token(rest.trim_start()) else {
        return Err(err("domain"));
    };
    if domain.is_empty()
        || !domain
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(err("domain"));
    }
    let Some(severity) = rest.trim_start().strip_prefix(',') else {
        return Err(err("severity"));
    };
    let Some(severity) = Severity::parse(severity.trim()) else {
        return Err(err("severity"));
    };
    Ok(Some(Contract {
        name: name.to_string(),
        tier,
        severity,
    }))
}

fn split_token(input: &str) -> Option<(&str, &str)> {
    let end = input
        .find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .unwrap_or(input.len());
    (end > 0).then_some((&input[..end], &input[end..]))
}
