//! Source files that only compile under `cfg(test)`.
//!
//! Inline `#[cfg(test)] mod tests { .. }` bodies are scanned in place; this
//! module finds the out-of-line ones (`mod tests;`, `#[path = ".."] mod
//! tests;`) and every module they declare, so a test file is scanned the same
//! way wherever it lives.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use super::{mask_rust_source, match_delimiter, match_outer_attr, match_word, skip_ws};

struct ModDecl {
    name: String,
    path: Option<String>,
    cfg_test: bool,
}

/// Every file in `sources` reached through a `cfg(test)` module declaration.
pub(super) fn test_only_files(sources: &[PathBuf]) -> BTreeSet<PathBuf> {
    let mut walk = Walk {
        known: sources.iter().cloned().collect(),
        path_loaded: BTreeSet::new(),
        test_only: BTreeSet::new(),
        queue: Vec::new(),
    };
    for file in sources {
        for decl in declarations(file).into_iter().filter(|decl| decl.cfg_test) {
            walk.enqueue(file, &decl);
        }
    }
    while let Some(file) = walk.queue.pop() {
        for decl in declarations(&file) {
            walk.enqueue(&file, &decl);
        }
    }
    walk.test_only
}

struct Walk {
    known: BTreeSet<PathBuf>,
    /// Files loaded through `#[path]`: rustc resolves their children like a
    /// `mod.rs`, next to the file instead of in a directory named after it.
    path_loaded: BTreeSet<PathBuf>,
    test_only: BTreeSet<PathBuf>,
    queue: Vec<PathBuf>,
}

impl Walk {
    fn enqueue(&mut self, parent: &Path, decl: &ModDecl) {
        let mod_rs = self.path_loaded.contains(parent);
        for candidate in module_files(parent, decl, mod_rs) {
            if self.known.contains(&candidate) {
                if decl.path.is_some() {
                    self.path_loaded.insert(candidate.clone());
                }
                if self.test_only.insert(candidate.clone()) {
                    self.queue.push(candidate);
                }
                return;
            }
        }
    }
}

/// Rust's file lookup for an out-of-line module declared at the top level of
/// `parent`.
fn module_files(parent: &Path, decl: &ModDecl, path_loaded: bool) -> Vec<PathBuf> {
    let Some(dir) = parent.parent() else {
        return Vec::new();
    };
    if let Some(path) = &decl.path {
        return vec![dir.join(path)];
    }
    let mod_rs = path_loaded
        || parent
            .file_name()
            .is_some_and(|name| matches!(name.to_str(), Some("mod.rs" | "lib.rs" | "main.rs")));
    let base = if mod_rs {
        dir.to_path_buf()
    } else {
        parent.with_extension("")
    };
    vec![
        base.join(format!("{}.rs", decl.name)),
        base.join(&decl.name).join("mod.rs"),
    ]
}

fn declarations(file: &Path) -> Vec<ModDecl> {
    let Ok(source) = fs::read_to_string(file) else {
        return Vec::new();
    };
    let masked = mask_rust_source(&source);
    let bytes = masked.as_bytes();
    let mut decls = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let boundary = index == 0 || !super::is_ident_byte(bytes.get(index - 1).copied());
        if !boundary || !matches!(bytes[index], b'#' | b'p' | b'm') {
            index += 1;
            continue;
        }
        let mut cursor = index;
        let mut cfg_test = false;
        let mut path = None;
        while bytes.get(cursor) == Some(&b'#') {
            let Some(end) = match_outer_attr(bytes, cursor) else {
                break;
            };
            let attr = attribute_inner(&masked, cursor, end);
            cfg_test |= is_cfg_test(&attr);
            if attr.starts_with("path=") {
                path = quoted(&source[cursor..end]);
            }
            cursor = skip_ws(bytes, end);
        }
        match mod_decl(bytes, cursor) {
            Some((name_start, name_end, end)) => {
                decls.push(ModDecl {
                    name: masked[name_start..name_end].to_owned(),
                    path,
                    cfg_test,
                });
                index = end;
            }
            None => index = cursor.max(index + 1),
        }
    }
    decls
}

fn attribute_inner(masked: &str, start: usize, end: usize) -> String {
    let text = &masked[start..end];
    let open = text.find('[').map_or(0, |open| open + 1);
    text[open..text.len().saturating_sub(1)]
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

fn is_cfg_test(attr: &str) -> bool {
    attr == "cfg(test)"
        || (attr.starts_with("cfg(all(")
            && attr
                .split(|ch| matches!(ch, '(' | ')' | ','))
                .any(|token| token == "test"))
}

fn quoted(attr: &str) -> Option<String> {
    let start = attr.find('"')? + 1;
    let end = start + attr[start..].find('"')?;
    Some(attr[start..end].to_owned())
}

/// `[pub[(..)]] mod name;` starting at `start`: name span and end offset.
fn mod_decl(bytes: &[u8], start: usize) -> Option<(usize, usize, usize)> {
    let mut index = start;
    if match_word(bytes, index, b"pub") {
        index = skip_ws(bytes, index + 3);
        if bytes.get(index) == Some(&b'(') {
            index = skip_ws(bytes, match_delimiter(bytes, index, b'(', b')')?);
        }
    }
    if !match_word(bytes, index, b"mod") {
        return None;
    }
    let name_start = skip_ws(bytes, index + 3);
    let mut name_end = name_start;
    while super::is_ident_byte(bytes.get(name_end).copied()) {
        name_end += 1;
    }
    let end = skip_ws(bytes, name_end);
    (name_end > name_start && bytes.get(end) == Some(&b';')).then_some((
        name_start,
        name_end,
        end + 1,
    ))
}
