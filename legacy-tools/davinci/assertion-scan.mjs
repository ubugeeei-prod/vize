// Scanner half of the banned weak-assertion lint (Davinci P0-12); the CLI,
// allowlist, and doctrine notes live in tools/davinci/assertion-lint.mjs.
//
// Everything here is pure detection: mask a Rust source, find the assert
// spans that count as test code, and report banned patterns inside them.

import fs from "node:fs";
import path from "node:path";

const SKIP_DIRECTORIES = new Set(["target", "node_modules", "dist"]);
const ASSERT_MACRO_RE = /\b(?:debug_)?assert(?:_eq|_ne)?!\s*\(/g;
const CFG_TEST_RE = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
const CFG_TEST_MOD_HEAD_RE = /^(?:pub\s*(?:\([^)]*\)\s*)?)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{/;
const JSON_MACRO_RE = /\bjson!\s*[([{]/;
const BANNED_PATTERNS = [
  { category: "contains", re: /\.\s*contains\s*\(/g },
  { category: "starts-with", re: /\.\s*starts_with\s*\(/g },
  { category: "ends-with", re: /\.\s*ends_with\s*\(/g },
  { category: "regex", re: /\bRegex\s*::\s*new\b/g },
];

// --- source masking ---------------------------------------------------------
// Returns a same-length copy of `source` where comment bodies and
// string/char-literal contents are replaced with spaces (newlines kept, so
// offsets and line numbers survive). All structural matching below runs on
// the masked text; findings are reported from the original text.
export function maskRustSource(source) {
  const out = source.split("");
  const blank = (from, to) => {
    for (let k = from; k < to && k < out.length; k += 1) {
      if (out[k] !== "\n") out[k] = " ";
    }
  };
  const isIdent = (ch) => ch != null && /[A-Za-z0-9_]/.test(ch);
  let i = 0;
  const n = source.length;
  while (i < n) {
    const ch = source[i];
    const next = source[i + 1];
    if (ch === "/" && next === "/") {
      let j = i;
      while (j < n && source[j] !== "\n") j += 1;
      blank(i, j);
      i = j;
      continue;
    }
    if (ch === "/" && next === "*") {
      // Rust block comments nest.
      let depth = 1;
      let j = i + 2;
      while (j < n && depth > 0) {
        if (source[j] === "/" && source[j + 1] === "*") {
          depth += 1;
          j += 2;
        } else if (source[j] === "*" && source[j + 1] === "/") {
          depth -= 1;
          j += 2;
        } else {
          j += 1;
        }
      }
      blank(i, j);
      i = j;
      continue;
    }
    // Raw strings r"…", r#"…"#, with optional b/c prefixes — only when the
    // prefix does not continue an identifier (`error"` is not a raw string).
    if ((ch === "r" || ch === "b" || ch === "c") && !isIdent(source[i - 1])) {
      const head = /^(?:br|cr|b|c)?r(#*)"/.exec(source.slice(i, i + 8));
      if (head != null) {
        const hashes = head[1];
        const bodyStart = i + head[0].length;
        const terminator = `"${hashes}`;
        const end = source.indexOf(terminator, bodyStart);
        const j = end === -1 ? n : end + terminator.length;
        blank(bodyStart, end === -1 ? n : end);
        i = j;
        continue;
      }
    }
    if (ch === '"') {
      let j = i + 1;
      while (j < n) {
        if (source[j] === "\\") j += 2;
        else if (source[j] === '"') {
          j += 1;
          break;
        } else j += 1;
      }
      blank(i + 1, j - 1);
      i = j;
      continue;
    }
    if (ch === "'") {
      // Char literal vs lifetime: a char literal closes within a few chars.
      const lookahead = source.slice(i, i + 12);
      const charLit = /^'(?:\\(?:u\{[0-9a-fA-F_]{1,6}\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'/.exec(
        lookahead,
      );
      if (charLit != null) {
        blank(i + 1, i + charLit[0].length - 1);
        i += charLit[0].length;
        continue;
      }
      i += 1; // lifetime marker
      continue;
    }
    i += 1;
  }
  return out.join("");
}

// --- structural helpers -----------------------------------------------------

function matchDelimiter(masked, openIndex, open, close) {
  let depth = 0;
  for (let i = openIndex; i < masked.length; i += 1) {
    const ch = masked[i];
    if (ch === open) depth += 1;
    else if (ch === close) {
      depth -= 1;
      if (depth === 0) return i + 1;
    }
  }
  return -1;
}

// Regions of `masked` that are `#[cfg(test)] mod … { … }` bodies.
export function cfgTestModRegions(masked) {
  const regions = [];
  CFG_TEST_RE.lastIndex = 0;
  let attr;
  while ((attr = CFG_TEST_RE.exec(masked)) != null) {
    let i = attr.index + attr[0].length;
    // Skip whitespace and any further outer attributes between the cfg
    // attribute and the item it gates.
    for (;;) {
      while (i < masked.length && /\s/.test(masked[i])) i += 1;
      if (masked[i] === "#") {
        let bracket = i;
        while (bracket < masked.length && masked[bracket] !== "[") bracket += 1;
        const end = matchDelimiter(masked, bracket, "[", "]");
        if (end === -1) break;
        i = end;
        continue;
      }
      break;
    }
    const head = CFG_TEST_MOD_HEAD_RE.exec(masked.slice(i, i + 256));
    if (head == null) continue; // not an inline mod — conservatively skipped
    const open = i + head[0].length - 1;
    const end = matchDelimiter(masked, open, "{", "}");
    if (end === -1) continue;
    regions.push([open + 1, end - 1]);
  }
  return regions;
}

// Argument spans (interior of the parens) of assert-family macros that
// start inside one of `regions`.
export function assertSpans(masked, regions) {
  const spans = [];
  for (const [start, end] of regions) {
    ASSERT_MACRO_RE.lastIndex = start;
    let m;
    while ((m = ASSERT_MACRO_RE.exec(masked)) != null && m.index < end) {
      const open = m.index + m[0].length - 1;
      const close = matchDelimiter(masked, open, "(", ")");
      if (close === -1 || close > end) continue; // cannot delimit — do not flag
      spans.push([open + 1, close - 1]);
      ASSERT_MACRO_RE.lastIndex = close;
    }
  }
  return spans;
}

function lineStarts(text) {
  const starts = [0];
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === "\n") starts.push(i + 1);
  }
  return starts;
}

function lineNumberAt(starts, offset) {
  let low = 0;
  let high = starts.length - 1;
  while (low < high) {
    const mid = (low + high + 1) >> 1;
    if (starts[mid] <= offset) low = mid;
    else high = mid - 1;
  }
  return low + 1;
}

export function scanFile(source, { wholeFile }) {
  const masked = maskRustSource(source);
  const regions = wholeFile ? [[0, masked.length]] : cfgTestModRegions(masked);
  if (regions.length === 0) return [];
  const spans = assertSpans(masked, regions);
  if (spans.length === 0) return [];
  const starts = lineStarts(source);
  const findings = [];
  const seen = new Set();
  for (const [from, to] of spans) {
    const spanText = masked.slice(from, to);
    const partialJson = JSON_MACRO_RE.test(spanText);
    for (const { category, re } of BANNED_PATTERNS) {
      re.lastIndex = 0;
      let hit;
      while ((hit = re.exec(spanText)) != null) {
        const offset = from + hit.index;
        const line = lineNumberAt(starts, offset);
        const reported = category === "contains" && partialJson ? "partial-json" : category;
        const key = `${line}:${reported}`;
        if (seen.has(key)) continue;
        seen.add(key);
        const lineEnd = line < starts.length ? starts[line] - 1 : source.length;
        findings.push({
          line,
          category: reported,
          text: source.slice(starts[line - 1], lineEnd).trim(),
          offset,
        });
      }
    }
  }
  findings.sort((a, b) => a.offset - b.offset);
  return findings;
}

// --- filesystem walk --------------------------------------------------------

function* walkRustFiles(dir) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  entries.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  for (const entry of entries) {
    if (entry.name.startsWith(".") || SKIP_DIRECTORIES.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) yield* walkRustFiles(full);
    else if (entry.isFile() && entry.name.endsWith(".rs")) yield full;
  }
}

export function collectTargets(root, defaultTree) {
  const targets = [];
  if (defaultTree) {
    const cratesDir = path.join(root, "crates");
    for (const file of walkRustFiles(cratesDir)) {
      const rel = path.relative(root, file).split(path.sep).join("/");
      if (/^crates\/[^/]+\/tests\//.test(rel)) targets.push({ file, rel, wholeFile: true });
      else if (/^crates\/[^/]+\/src\//.test(rel)) targets.push({ file, rel, wholeFile: false });
      // benches/, examples/, build.rs: out of scope for the test-assertion lint.
    }
  } else {
    for (const file of walkRustFiles(root)) {
      const rel = path.relative(root, file).split(path.sep).join("/");
      targets.push({ file, rel, wholeFile: rel.split("/").includes("tests") });
    }
  }
  return targets;
}
