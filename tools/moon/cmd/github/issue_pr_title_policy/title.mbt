/// Conventional Commits title parsing and area-name normalization.
///
/// Split out of `main.mbt` so the title grammar lives apart from the GitHub
/// event / API plumbing. All character predicates work on UTF-16 code units,
/// which is what `String` indexing yields.

fn is_lower_ascii(code : UInt16) -> Bool {
  code >= 97 && code <= 122
}

fn is_digit(code : UInt16) -> Bool {
  code >= 48 && code <= 57
}

fn is_type_char(code : UInt16) -> Bool {
  is_lower_ascii(code) || is_digit(code) || code == 45
}

fn is_scope_char(code : UInt16) -> Bool {
  is_lower_ascii(code) || is_digit(code) || code == 45 || code == 46 ||
  code == 47 || code == 95
}

/// Maps a colloquial area name to its internal Vize name. The argument is one
/// whole scope segment, never a fragment of one. `is_scope_char` rejects
/// uppercase, so a scope reaching here is already lower case; `to_lower` only
/// keeps the table total for any future caller.
fn title_replacement(word : String) -> String? {
  match word.to_lower() {
    "check" => Some("canon")
    "compiler" => Some("atelier")
    "lint" => Some("patina")
    "linter" => Some("patina")
    "story" => Some("musea")
    "format" => Some("glyph")
    "fmt" => Some("glyph")
    _ => None
  }
}

/// Rewrites each area name inside a Conventional Commits scope.
///
/// Segment rule: `/` is the ONLY segment separator, so `check/lint` normalizes
/// to `canon/patina`. The other characters `is_scope_char` allows (`-`, `.`,
/// `_`, digits) build compound scope names such as `type-check`, `check.js`, or
/// `vue_compiler`; splitting on them would rewrite a fragment of a name that
/// means something else as a whole. A segment therefore only changes when the
/// entire segment is a known area name -- `checker` stays `checker`.
fn normalize_scope(scope : String) -> String {
  let segments : Array[String] = []
  let mut start = 0
  let mut i = 0
  while i <= scope.length() {
    if i == scope.length() || scope[i] == 47 {
      let segment = scope[start:i].to_owned()
      match title_replacement(segment) {
        Some(replacement) => segments.push(replacement)
        None => segments.push(segment)
      }
      start = i + 1
    }
    i = i + 1
  }
  segments.join("/")
}

/// Returns the half-open `[start, end)` range of the scope text inside a
/// Conventional Commits prefix, or `None` when the prefix carries no scope.
/// Only called on a prefix `is_conventional_prefix` already accepted, so the
/// parentheses are known to be balanced and to enclose at least one character.
fn scope_bounds(prefix : String) -> (Int, Int)? {
  let mut open = 0
  while open < prefix.length() && prefix[open] != 40 {
    open = open + 1
  }
  if open >= prefix.length() {
    return None
  }
  let mut close = open + 1
  while close < prefix.length() && prefix[close] != 41 {
    close = close + 1
  }
  if close >= prefix.length() {
    return None
  }
  Some((open + 1, close))
}

/// Normalizes ONLY the scope of a `type(scope): subject` title.
///
/// The type, the `!` breaking marker, and the whole subject are left exactly as
/// the author wrote them. Area words such as `check`, `format`, `story`, and
/// `compiler` are ordinary English in a subject line ("rank type-check engine
/// classes", "check sources under a hidden directory", "format the output"), so
/// rewriting them there corrupts the title -- and, because the repository
/// squash-merges with the PR title as the commit headline, corrupts permanent
/// history and the generated changelog. A title without a conventional prefix
/// carrying a scope is returned untouched.
fn normalize_title(title : String) -> String {
  let separator = match colon_space_index(title) {
    Some(index) => index
    None => return title
  }
  let prefix = title[0:separator].to_owned()
  if !is_conventional_prefix(prefix) {
    return title
  }
  let (scope_start, scope_end) = match scope_bounds(prefix) {
    Some(bounds) => bounds
    None => return title
  }
  let scope = prefix[scope_start:scope_end].to_owned()
  let normalized_scope = normalize_scope(scope)
  if normalized_scope == scope {
    return title
  }
  let head = title[0:scope_start].to_owned()
  let tail = title[scope_end:].to_owned()
  "\{head}\{normalized_scope}\{tail}"
}

fn has_newline(value : String) -> Bool {
  for i in 0..<value.length() {
    if value[i] == 10 || value[i] == 13 {
      return true
    }
  }
  false
}

fn colon_space_index(value : String) -> Int? {
  let mut i = 0
  while i + 1 < value.length() {
    if value[i] == 58 && value[i + 1] == 32 {
      return Some(i)
    }
    i = i + 1
  }
  None
}

fn is_conventional_prefix(prefix : String) -> Bool {
  if prefix.length() == 0 || !is_lower_ascii(prefix[0]) {
    return false
  }

  let mut i = 1
  while i < prefix.length() && is_type_char(prefix[i]) {
    i = i + 1
  }

  if i < prefix.length() && prefix[i] == 40 {
    i = i + 1
    let scope_start = i
    while i < prefix.length() && is_scope_char(prefix[i]) {
      i = i + 1
    }
    if i == scope_start || i >= prefix.length() || prefix[i] != 41 {
      return false
    }
    i = i + 1
  }

  if i < prefix.length() && prefix[i] == 33 {
    i = i + 1
  }

  i == prefix.length()
}

fn is_conventional_title(title : String) -> Bool {
  if title == "" || has_newline(title) {
    return false
  }
  let separator = match colon_space_index(title) {
    Some(index) => index
    None => return false
  }
  if separator + 2 >= title.length() || title[separator + 2] == 32 {
    return false
  }
  is_conventional_prefix(title[0:separator].to_owned())
}
