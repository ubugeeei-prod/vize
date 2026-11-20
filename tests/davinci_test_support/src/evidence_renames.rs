//! Pinned evidence read through the current names.
//!
//! Reviewed outputs are pinned byte for byte, but some of them record `Debug`
//! text, and `Debug` text names types. When a type is renamed, the pinned bytes
//! stay untouched: `docs/davinci/plan/evidence-renames.json` registers the
//! rename (ordered, one entry per rename) with the evidence files that predate
//! it, and a custody gate compares [`spell_back`] of the current output with
//! the pinned file. Spelling back undoes, newest first, every registered rename
//! that lists the file, word for word.
//!
//! [`validate`] keeps that exact: a listed file must spell every old name of
//! its entry somewhere and no new name anywhere, so spelling back can only
//! restore what the rename changed. `tests/tooling/support/evidence-renames.ts`
//! is the same reader for the hosted TS judges.
use std::path::Path;
use std::sync::OnceLock;

use serde_json::Value;
use vize_l0::{CompactString, cstr};

/// The registered map, relative to the repository root.
pub const MAP_PATH: &str = "docs/davinci/plan/evidence-renames.json";
const MAP: &str = include_str!("../../../docs/davinci/plan/evidence-renames.json");

/// One registered rename: `(old, new)` names and the evidence spelled with the old ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rename {
    pub id: CompactString,
    pub names: Vec<(CompactString, CompactString)>,
    pub evidence: Vec<CompactString>,
}

type Outcome<T> = Result<T, CompactString>;

fn sorted_keys(value: &Value) -> Option<Vec<&str>> {
    let mut keys: Vec<&str> = value.as_object()?.keys().map(|key| key.as_str()).collect();
    keys.sort_unstable();
    Some(keys)
}

fn strings(value: &Value, what: &str) -> Outcome<Vec<CompactString>> {
    value
        .as_array()
        .ok_or_else(|| cstr!("evidence renames: {what} must be an array"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(CompactString::from)
                .ok_or_else(|| cstr!("evidence renames: {what} holds a non-string"))
        })
        .collect()
}

fn entry(value: &Value) -> Outcome<Rename> {
    if sorted_keys(value).as_deref() != Some(&["evidence", "id", "names"][..]) {
        return Err("evidence renames: entry fields are `id`, `names`, `evidence`".into());
    }
    let names = value
        .get("names")
        .and_then(Value::as_array)
        .ok_or("evidence renames: `names` array")?
        .iter()
        .map(|pair| match strings(pair, "a name pair")?.as_slice() {
            [old, new] => Ok((old.clone(), new.clone())),
            _ => Err(CompactString::from(
                "evidence renames: a name pair is [old, new]",
            )),
        })
        .collect::<Outcome<_>>()?;
    Ok(Rename {
        id: value
            .get("id")
            .and_then(Value::as_str)
            .ok_or("evidence renames: `id` string")?
            .into(),
        names,
        evidence: strings(value.get("evidence").unwrap_or(&Value::Null), "`evidence`")?,
    })
}

fn parse(text: &str) -> Outcome<Vec<Rename>> {
    let map: Value = serde_json::from_str(text).map_err(|error| cstr!("{MAP_PATH}: {error}"))?;
    if sorted_keys(&map).as_deref() != Some(&["renames", "schema", "version"][..])
        || map.get("schema").and_then(Value::as_str) != Some("vize.evidence-renames")
        || map.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err("evidence renames: schema `vize.evidence-renames` version 1 expected".into());
    }
    map.get("renames")
        .and_then(Value::as_array)
        .ok_or("evidence renames: `renames` array")?
        .iter()
        .map(entry)
        .collect()
}

/// The registered renames, oldest first.
pub fn renames() -> Outcome<&'static [Rename]> {
    static RENAMES: OnceLock<Outcome<Vec<Rename>>> = OnceLock::new();
    RENAMES
        .get_or_init(|| parse(MAP))
        .as_deref()
        .map_err(Clone::clone)
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Replace whole-word `from` (identifier boundaries) with `to`.
pub fn replace_word(text: &str, from: &str, to: &str) -> CompactString {
    let bytes = text.as_bytes();
    let mut out = CompactString::with_capacity(text.len());
    let mut last = 0;
    for (at, _) in text.match_indices(from) {
        let end = at + from.len();
        let before = at
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied();
        if !before.is_some_and(is_word) && !bytes.get(end).copied().is_some_and(is_word) {
            out.push_str(text.get(last..at).unwrap_or_default());
            out.push_str(to);
            last = end;
        }
    }
    out.push_str(text.get(last..).unwrap_or_default());
    out
}

/// Whether `text` spells `name` as a whole word.
pub fn mentions(text: &str, name: &str) -> bool {
    replace_word(text, name, "") != text
}

fn lists(rename: &Rename, evidence: &str) -> bool {
    rename.evidence.iter().any(|path| path == evidence)
}

fn spell_text(renames: &[Rename], text: &str, evidence: &str) -> CompactString {
    renames
        .iter()
        .rev()
        .filter(|rename| lists(rename, evidence))
        .fold(CompactString::from(text), |text, rename| {
            rename
                .names
                .iter()
                .fold(text, |text, (old, new)| replace_word(&text, new, old))
        })
}

/// Text spelled forward, old names to new, oldest rename first.
fn spell_forward(renames: &[Rename], text: &str, evidence: &str) -> CompactString {
    renames
        .iter()
        .filter(|rename| lists(rename, evidence))
        .fold(CompactString::from(text), |text, rename| {
            rename
                .names
                .iter()
                .fold(text, |text, (old, new)| replace_word(&text, old, new))
        })
}

fn spell_value(renames: &[Rename], value: &Value, evidence: &str) -> Value {
    match value {
        Value::String(text) => Value::String(spell_text(renames, text, evidence).into()),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| spell_value(renames, item, evidence))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, item)| {
                    (
                        spell_text(renames, key, evidence).into(),
                        spell_value(renames, item, evidence),
                    )
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Current text spelled with the names the pinned file `evidence` recorded.
pub fn spell_back_text(text: &str, evidence: &str) -> Outcome<CompactString> {
    Ok(spell_text(renames()?, text, evidence))
}

/// A current JSON value spelled with the names the pinned file `evidence` recorded.
pub fn spell_back(value: &Value, evidence: &str) -> Outcome<Value> {
    Ok(spell_value(renames()?, value, evidence))
}

/// Check the map against the repository at `root`: ids are unique, names are
/// identifiers that rename to new spellings, and every listed file exists,
/// spells each old name of its entry and no new one, and reads back unchanged
/// after spelling forward and back.
pub fn validate(root: &Path) -> Outcome<()> {
    validate_renames(renames()?, root)
}

fn identifier(name: &str) -> bool {
    name.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && name.bytes().all(is_word)
}

fn read(root: &Path, path: &str) -> Outcome<CompactString> {
    std::fs::read_to_string(root.join(path))
        .map(CompactString::from)
        .map_err(|error| cstr!("evidence renames: {path}: {error}"))
}

fn validate_renames(renames: &[Rename], root: &Path) -> Outcome<()> {
    for (index, rename) in renames.iter().enumerate() {
        let id = &rename.id;
        if renames.iter().take(index).any(|earlier| earlier.id == *id) {
            return Err(cstr!("evidence renames: `{id}` is registered twice"));
        }
        if rename.names.is_empty() || rename.evidence.is_empty() {
            return Err(cstr!("evidence renames: `{id}` needs names and evidence"));
        }
        for (old, new) in &rename.names {
            if !identifier(old) || !identifier(new) || old == new {
                return Err(cstr!("evidence renames: `{id}` renames `{old}` to `{new}`"));
            }
            if rename.names.iter().any(|(other, _)| other == new) {
                return Err(cstr!("evidence renames: `{id}` renames `{new}` again"));
            }
        }
        let later = renames.get(index + 1..).unwrap_or_default();
        for path in &rename.evidence {
            // A later rename may also list this file; it is checked in the
            // spelling this entry renames from.
            let at_entry = spell_text(later, &read(root, path)?, path);
            for (old, new) in &rename.names {
                if !mentions(&at_entry, old) {
                    return Err(cstr!(
                        "evidence renames: {path} never spells `{old}` (`{id}`)"
                    ));
                }
                if mentions(&at_entry, new) {
                    return Err(cstr!(
                        "evidence renames: {path} already spells `{new}`, so `{id}` cannot be spelled back"
                    ));
                }
            }
        }
    }
    for path in renames.iter().flat_map(|rename| rename.evidence.iter()) {
        let text = read(root, path)?;
        let forward = spell_forward(renames, &text, path);
        if spell_text(renames, &forward, path) != text {
            return Err(cstr!(
                "evidence renames: {path} does not read back after spelling forward"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Rename, mentions, parse, replace_word, spell_text, validate, validate_renames};
    use std::path::Path;

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn every_registered_rename_spells_back_to_its_pinned_bytes() {
        assert_eq!(validate(root()), Ok(()));
    }

    #[test]
    fn replacement_respects_identifier_boundaries() {
        assert_eq!(
            replace_word("Owned { a: OwnedX, b: _Owned, c: Owned }", "Owned", "N"),
            "N { a: OwnedX, b: _Owned, c: N }"
        );
        assert!(mentions("x Owned y", "Owned"));
        assert!(!mentions("OwnedX", "Owned"));
    }

    #[test]
    fn spelling_back_undoes_newer_renames_first_and_only_for_listed_files() {
        let renames = parse(
            r#"{"schema":"vize.evidence-renames","version":1,"renames":[
              {"id":"one","names":[["A","B"]],"evidence":["e.json"]},
              {"id":"two","names":[["B","C"]],"evidence":["e.json","f.json"]}]}"#,
        )
        .unwrap();
        assert_eq!(spell_text(&renames, "C { B }", "e.json"), "A { A }");
        assert_eq!(spell_text(&renames, "C", "f.json"), "B");
        assert_eq!(spell_text(&renames, "C", "g.json"), "C");
    }

    #[test]
    fn evidence_that_already_spells_a_new_name_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("e.json"), r#"{"debug":"Old { x: New }"}"#).unwrap();
        let rename = |names: &[(&str, &str)]| Rename {
            id: "r".into(),
            names: names
                .iter()
                .map(|(old, new)| ((*old).into(), (*new).into()))
                .collect(),
            evidence: vec!["e.json".into()],
        };
        assert_eq!(
            validate_renames(&[rename(&[("Old", "Fresh")])], dir.path()),
            Ok(())
        );
        assert_eq!(
            validate_renames(&[rename(&[("Old", "New")])], dir.path()),
            Err(
                "evidence renames: e.json already spells `New`, so `r` cannot be spelled back"
                    .into()
            )
        );
        assert_eq!(
            validate_renames(&[rename(&[("Gone", "Fresh")])], dir.path()),
            Err("evidence renames: e.json never spells `Gone` (`r`)".into())
        );
    }
}
