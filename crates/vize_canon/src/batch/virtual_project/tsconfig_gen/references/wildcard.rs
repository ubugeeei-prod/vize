//! TypeScript's tsconfig `include` / `exclude` spec matching.
//!
//! Mirrors `getSubPatternFromSpec` and `getIncludeBasePath` in TypeScript's
//! `utilities.ts`, which tsgo ports unchanged. Only `*`, `?` and a whole `**`
//! segment are wildcards. Every other character matches itself, `[`, `]`,
//! `{`, `}` and `!` included, so file-based routing pages such as
//! `pages/users/[userId=int].vue` are plain names (#10819).
//!
//! An include spec also carries `tsc`'s implicit rules: a wildcard segment
//! never selects `node_modules`, `bower_components` or `jspm_packages`, `**`
//! never enters those or a dot-directory, a leading `*` or `?` never matches a
//! leading `.`, and `*` never consumes the `.` of a trailing `.min.js`. Literal
//! segments are exempt. An exclude spec has none of these rules and also
//! excludes everything below a matched directory.

use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use vize_carton::String as CompactString;

/// `commonPackageFolders` in TypeScript's `utilities.ts`.
const PACKAGE_FOLDERS: [&str; 3] = ["node_modules", "bower_components", "jspm_packages"];

/// Which tsconfig list a spec belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsconfigWildcardUsage {
    Include,
    Exclude,
}

/// One compiled `include` or `exclude` entry.
#[derive(Debug, Clone)]
pub struct TsconfigWildcard {
    /// The literal directory the first wildcard segment lives in.
    base: PathBuf,
    /// `base` relative to the config directory after leading `..` segments.
    literal_directories: PathBuf,
    segments: Vec<Segment>,
    usage: TsconfigWildcardUsage,
}

#[derive(Debug, Clone)]
enum Segment {
    Literal(CompactString),
    Recursive,
    Wildcard(Vec<Token>),
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Star,
    Question,
    Char(char),
}

impl TsconfigWildcard {
    /// Compile `spec` relative to `config_dir`. `None` is a spec TypeScript
    /// drops: an include ending in `**` (TS5010) or `..` after `**` (TS5065).
    pub fn new(config_dir: &Path, spec: &str, usage: TsconfigWildcardUsage) -> Option<Self> {
        if parent_after_recursive(spec) {
            return None;
        }
        let spec = spec.replace('\\', "/");
        let (mut anchor, rest) = match split_root(&spec) {
            Some((root, rest)) => (PathBuf::from(root), rest),
            None => (config_dir.to_path_buf(), spec.as_str()),
        };
        let mut parts = Vec::new();
        for part in rest.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        anchor.pop();
                    }
                }
                part => parts.push(part),
            }
        }

        let last = match parts.last() {
            Some(last) => Cow::Borrowed(*last),
            None => anchor
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_default(),
        };
        if usage == TsconfigWildcardUsage::Include && last == "**" {
            return None;
        }
        // `isImplicitGlob`: a last segment without `.`, `*` or `?` is a
        // directory, so `src` and `.` mean `src/**/*` and `./**/*`.
        if !last.contains(['.', '*', '?']) {
            parts.extend(["**", "*"]);
        }

        let literal = parts
            .iter()
            .position(|part| is_wildcard(part))
            .unwrap_or(parts.len().saturating_sub(1));
        let literal_directories: PathBuf =
            parts.get(..literal).unwrap_or_default().iter().collect();
        // Callers pass a normalized config directory. Literal directories stay
        // as spelled, so their case still follows the host policy.
        let base = anchor.join(&literal_directories);
        let segments = parts
            .get(literal..)
            .unwrap_or_default()
            .iter()
            .map(|part| Segment::compile(part))
            .collect();
        Some(Self {
            base,
            literal_directories,
            segments,
            usage,
        })
    }

    /// The deepest literal directory, where `tsc` starts walking this spec.
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// The literal directories the spec itself names below its config
    /// directory, e.g. `.vitepress/theme` for `.vitepress/theme/**/*`.
    pub fn literal_directories(&self) -> &Path {
        &self.literal_directories
    }

    /// Whether a segment after the first wildcard is the literal `name`.
    pub fn names_literal_segment(&self, name: &str, case_sensitive: bool) -> bool {
        self.segments.iter().any(|segment| {
            matches!(segment, Segment::Literal(literal) if names_equal(literal, name, case_sensitive))
        })
    }

    /// Membership of an absolute path, already normalized like the config
    /// directory this spec was compiled against.
    pub fn matches(&self, path: &Path, case_sensitive: bool) -> bool {
        let Some(names) = names_below(path, &self.base, case_sensitive) else {
            return false;
        };
        let mut memo = vec![None; (self.segments.len() + 1) * (names.len() + 1)];
        SegmentMatcher {
            spec: self,
            names: &names,
            case_sensitive,
            memo: &mut memo,
        }
        .matches(0, 0)
    }
}

impl Segment {
    fn compile(part: &str) -> Self {
        if part == "**" {
            return Self::Recursive;
        }
        if !is_wildcard(part) {
            return Self::Literal(part.into());
        }
        Self::Wildcard(
            part.chars()
                .map(|character| match character {
                    '*' => Token::Star,
                    '?' => Token::Question,
                    character => Token::Char(character),
                })
                .collect(),
        )
    }
}

struct SegmentMatcher<'a> {
    spec: &'a TsconfigWildcard,
    names: &'a [Cow<'a, str>],
    case_sensitive: bool,
    memo: &'a mut [Option<bool>],
}

impl SegmentMatcher<'_> {
    fn matches(&mut self, segment_index: usize, name_index: usize) -> bool {
        let (spec, names) = (self.spec, self.names);
        let include = spec.usage == TsconfigWildcardUsage::Include;
        let Some(segment) = spec.segments.get(segment_index) else {
            // An exclude also covers everything below what it matched.
            return !include || name_index == names.len();
        };
        let key = segment_index * (names.len() + 1) + name_index;
        if let Some(Some(known)) = self.memo.get(key) {
            return *known;
        }
        let case_sensitive = self.case_sensitive;
        let name = names.get(name_index).map(|name| name.as_ref());
        let is_last = name_index + 1 == names.len();
        let matched = match segment {
            Segment::Recursive => {
                self.matches(segment_index + 1, name_index)
                    || (name.is_some_and(|name| {
                        !include
                            || !(name.is_empty()
                                || name.starts_with('.')
                                || is_package_folder(name, case_sensitive))
                    }) && self.matches(segment_index, name_index + 1))
            }
            Segment::Literal(literal) => {
                name.is_some_and(|name| names_equal(literal, name, case_sensitive))
                    && self.matches(segment_index + 1, name_index + 1)
            }
            Segment::Wildcard(tokens) => {
                name.is_some_and(|name| {
                    (!include || !is_package_folder(name, case_sensitive))
                        && TokenMatcher::new(tokens, name, include && is_last, include)
                            .matches(case_sensitive)
                }) && self.matches(segment_index + 1, name_index + 1)
            }
        };
        if let Some(slot) = self.memo.get_mut(key) {
            *slot = Some(matched);
        }
        matched
    }
}

struct TokenMatcher<'a> {
    tokens: &'a [Token],
    name: &'a str,
    /// The name ends the path, so `*` must leave a trailing `.min.js` alone.
    guards_min_js: bool,
    /// Include rules: a leading `*` or `?` does not match a leading `.`.
    include: bool,
    memo: Vec<Option<bool>>,
}

impl<'a> TokenMatcher<'a> {
    fn new(tokens: &'a [Token], name: &'a str, guards_min_js: bool, include: bool) -> Self {
        Self {
            tokens,
            name,
            guards_min_js,
            include,
            memo: vec![None; (tokens.len() + 1) * (name.len() + 1)],
        }
    }

    fn matches(mut self, case_sensitive: bool) -> bool {
        self.step(0, 0, case_sensitive)
    }

    fn step(&mut self, token_index: usize, offset: usize, case_sensitive: bool) -> bool {
        let name = self.name;
        let rest = name.get(offset..).unwrap_or_default();
        let Some(token) = self.tokens.get(token_index).copied() else {
            return rest.is_empty();
        };
        let key = token_index * (name.len() + 1) + offset;
        if let Some(Some(known)) = self.memo.get(key) {
            return *known;
        }
        let next = rest.chars().next();
        // Only the spec segment's first character gets the dot rule.
        let first = token_index == 0 && offset == 0;
        let matched = match token {
            Token::Star => {
                self.step(token_index + 1, offset, case_sensitive)
                    || next.is_some_and(|character| {
                        self.star_consumes(character, first, rest, case_sensitive)
                            && self.step(token_index, offset + character.len_utf8(), case_sensitive)
                    })
            }
            Token::Question => next.is_some_and(|character| {
                !(self.include && first && character == '.')
                    && self.step(
                        token_index + 1,
                        offset + character.len_utf8(),
                        case_sensitive,
                    )
            }),
            Token::Char(expected) => next.is_some_and(|character| {
                chars_equal(character, expected, case_sensitive)
                    && self.step(
                        token_index + 1,
                        offset + character.len_utf8(),
                        case_sensitive,
                    )
            }),
        };
        if let Some(slot) = self.memo.get_mut(key) {
            *slot = Some(matched);
        }
        matched
    }

    fn star_consumes(
        &self,
        character: char,
        first: bool,
        rest: &str,
        case_sensitive: bool,
    ) -> bool {
        if !self.include || character != '.' {
            return true;
        }
        !first
            && !(self.guards_min_js
                && if case_sensitive {
                    rest == ".min.js"
                } else {
                    rest.eq_ignore_ascii_case(".min.js")
                })
    }
}

fn is_wildcard(part: &str) -> bool {
    part.contains(['*', '?'])
}

/// `invalidDotDotAfterRecursiveWildcard`, checked on the raw spec like `tsc`.
fn parent_after_recursive(spec: &str) -> bool {
    let recursive = if spec.starts_with("**/") {
        Some(0)
    } else {
        spec.find("/**/")
    };
    let Some(recursive) = recursive else {
        return false;
    };
    let parent = if spec.ends_with("/..") {
        Some(spec.len())
    } else {
        spec.rfind("/../")
    };
    parent.is_some_and(|parent| parent > recursive)
}

/// The root of an absolute spec (`/`, `C:/`) and the rest after it.
fn split_root(spec: &str) -> Option<(&str, &str)> {
    if spec.starts_with('/') {
        return Some(spec.split_at(1));
    }
    let bytes = spec.as_bytes();
    let drive = bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':');
    drive.then(|| spec.split_at(if bytes.get(2) == Some(&b'/') { 3 } else { 2 }))
}

/// The names of `path` below `base`, or `None` when it is not below it.
fn names_below<'a>(path: &'a Path, base: &Path, case_sensitive: bool) -> Option<Vec<Cow<'a, str>>> {
    let mut components = path.components();
    for base_part in base.components() {
        let part = components.next()?;
        if !components_equal(part, base_part, case_sensitive) {
            return None;
        }
    }
    Some(
        components
            .map(|component| component.as_os_str().to_string_lossy())
            .collect(),
    )
}

fn components_equal(left: Component<'_>, right: Component<'_>, case_sensitive: bool) -> bool {
    let (left, right) = (left.as_os_str(), right.as_os_str());
    left == right
        || (!case_sensitive
            && names_equal(&left.to_string_lossy(), &right.to_string_lossy(), false))
}

fn names_equal(left: &str, right: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        return left == right;
    }
    left.chars()
        .flat_map(char::to_lowercase)
        .eq(right.chars().flat_map(char::to_lowercase))
}

fn chars_equal(left: char, right: char, case_sensitive: bool) -> bool {
    left == right || (!case_sensitive && left.to_lowercase().eq(right.to_lowercase()))
}

fn is_package_folder(name: &str, case_sensitive: bool) -> bool {
    PACKAGE_FOLDERS.iter().any(|folder| {
        if case_sensitive {
            name == *folder
        } else {
            name.eq_ignore_ascii_case(folder)
        }
    })
}
