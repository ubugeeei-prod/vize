//! Reading the pinned WHATWG HTML snapshot (Davinci P4-11a).
//!
//! Two layers. `extract_*` turn the upstream multipage HTML into the
//! committed plain-text excerpt (`whatwg/*.txt|tsv`): the element and
//! content-category index tables, and the tree-construction sections with
//! their `<dt>`/`<dd>` rule structure kept as `@dt`/`@dd` lines. `Spec` then
//! reads the excerpt and answers the questions the fact-table recipes ask.

use std::collections::BTreeMap;

/// The tree-construction sections the recipes read, by fragment id.
pub const SECTIONS: [&str; 10] = [
    "tree-construction",
    "the-stack-of-open-elements",
    "the-list-of-active-formatting-elements",
    "closing-elements-that-have-implied-end-tags",
    "parsing-main-inhead",
    "parsing-main-inbody",
    "parsing-main-intable",
    "parsing-main-incaption",
    "parsing-main-incolgroup",
    "parsing-main-intbody",
];
/// Sections read after `SECTIONS` (kept separate to stay in spec order).
pub const MORE_SECTIONS: [&str; 2] = ["parsing-main-intr", "parsing-main-inforeign"];

fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        rest = &rest[index..];
        let Some(end) = rest.find(';').filter(|end| *end <= 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                u32::from_str_radix(&entity[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if entity.starts_with('#') => entity[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(ch) => {
                out.push(ch);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Markup to structured text: `<dt>`/`<dd>`/`<li>` start `@dt `/`@dd `/`- `
/// lines, block tags break lines, every other tag is dropped.
fn to_text(raw: &str) -> String {
    let mut out = String::new();
    let mut rest = raw;
    // Source line breaks are formatting; only structure starts a new line.
    let push_text = |out: &mut String, text: &str| out.push_str(&text.replace(['\n', '\r'], " "));
    while let Some(open) = rest.find('<') {
        push_text(&mut out, &rest[..open]);
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        let tag = &rest[open + 1..open + close];
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let closing = tag.starts_with('/');
        match (name.as_str(), closing) {
            ("dt", false) => out.push_str("\n@dt "),
            ("dd", false) => out.push_str("\n@dd "),
            ("li", false) => out.push_str("\n- "),
            ("p" | "br" | "dl" | "ul" | "ol" | "div" | "pre" | "table" | "tr", _) => out.push('\n'),
            (name, _) if name.len() == 2 && name.starts_with('h') => out.push('\n'),
            _ => {}
        }
        rest = &rest[open + close + 1..];
    }
    push_text(&mut out, rest);
    decode_entities(&out)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of the section headed by `id`, up to the next heading.
pub fn extract_section(html: &str, id: &str) -> Result<String, String> {
    let marker = format!("id={id}>");
    let start = html
        .find(&marker)
        .ok_or_else(|| format!("snapshot has no section #{id}"))?;
    let body = start + marker.len();
    let bytes = html.as_bytes();
    let end = (body..bytes.len())
        .find(|&index| {
            bytes[index] == b'<'
                && bytes.get(index + 1) == Some(&b'h')
                && bytes.get(index + 2).is_some_and(u8::is_ascii_digit)
        })
        .unwrap_or(html.len());
    Ok(to_text(&html[body..end]))
}

/// The rows of the index table whose caption is `caption`, cells tab-joined.
pub fn extract_table(html: &str, caption: &str) -> Result<String, String> {
    let at = html
        .find(&format!("<caption>{caption}"))
        .ok_or_else(|| format!("snapshot has no table \"{caption}\""))?;
    let start = html[..at].rfind("<table").ok_or("table start")?;
    let end = at + html[at..].find("</table>").ok_or("table end")?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rest = &html[start..end];
    while let Some(open) = rest.find('<') {
        let text = &rest[..open];
        if let Some(cell) = rows.last_mut().and_then(|row| row.last_mut()) {
            cell.push_str(text);
        }
        let close = rest[open..].find('>').ok_or("unterminated tag")?;
        let tag = rest[open + 1..open + close].to_ascii_lowercase();
        if tag == "tr" || tag.starts_with("tr ") {
            rows.push(Vec::new());
        } else if tag.starts_with("th") || tag.starts_with("td") {
            if let Some(row) = rows.last_mut() {
                row.push(String::new());
            }
        }
        rest = &rest[open + close + 1..];
    }
    Ok(rows
        .into_iter()
        .filter(|row| !row.is_empty())
        .map(|row| {
            row.iter()
                .map(|cell| to_text(cell).replace('\n', " "))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// One tree-construction rule: its token conditions and its steps.
#[derive(Debug, Default)]
pub struct Rule {
    pub conditions: Vec<String>,
    pub steps: String,
}

/// The committed excerpt, parsed.
pub struct Spec {
    sections: BTreeMap<String, String>,
    pub elements: Vec<Vec<String>>,
    pub categories: Vec<Vec<String>>,
}

impl Spec {
    pub fn new(parsing: &str, elements: &str, categories: &str) -> Self {
        let mut sections = BTreeMap::new();
        let mut current: Option<(String, String)> = None;
        for line in parsing.lines() {
            if let Some(id) = line.strip_prefix("## ") {
                if let Some((id, text)) = current.take() {
                    sections.insert(id, text);
                }
                current = Some((id.to_string(), String::new()));
            } else if let Some((_, text)) = current.as_mut() {
                text.push_str(line);
                text.push('\n');
            }
        }
        if let Some((id, text)) = current {
            sections.insert(id, text);
        }
        let table = |tsv: &str| -> Vec<Vec<String>> {
            tsv.lines()
                .filter(|line| !line.starts_with('#'))
                .map(|line| line.split('\t').map(str::to_string).collect())
                .collect()
        };
        Self {
            sections,
            elements: table(elements),
            categories: table(categories),
        }
    }

    pub fn section(&self, id: &str) -> Result<&str, String> {
        self.sections
            .get(id)
            .map(String::as_str)
            .ok_or_else(|| format!("excerpt has no section {id}"))
    }

    /// The `@dt`…`@dd` rules of an insertion-mode section, in order.
    pub fn rules(&self, id: &str) -> Result<Vec<Rule>, String> {
        let mut rules: Vec<Rule> = Vec::new();
        let mut in_steps = false;
        for line in self.section(id)?.lines() {
            if let Some(condition) = line.strip_prefix("@dt ") {
                if in_steps || rules.is_empty() {
                    rules.push(Rule::default());
                }
                in_steps = false;
                rules
                    .last_mut()
                    .ok_or("rule")?
                    .conditions
                    .push(condition.to_string());
            } else if let Some(steps) = line.strip_prefix("@dd").map(str::trim_start) {
                in_steps = true;
                if let Some(rule) = rules.last_mut() {
                    rule.steps.push_str(steps);
                    rule.steps.push('\n');
                }
            } else if in_steps && let Some(rule) = rules.last_mut() {
                rule.steps.push_str(line);
                rule.steps.push('\n');
            }
        }
        Ok(rules)
    }
}

/// The quoted tag names of a start-tag condition (`A start tag whose tag
/// name is one of: "a", "b"`); empty for any other condition. A trailing
/// qualifier (`, if the token has any attributes named "color", …`) names
/// attributes, not tags, and is not read.
pub fn start_tags(condition: &str) -> Vec<String> {
    if !condition.starts_with("A start tag whose tag name is") {
        return Vec::new();
    }
    let names = condition.split(", if ").next().unwrap_or(condition);
    names
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}
