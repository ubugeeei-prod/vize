#!/usr/bin/env node
// Banned weak-assertion lint for Rust test code (Davinci P0-12).
//
// Doctrine: davinci-road/assurance.md, "Strict oracles — no partial
// matching". Test assertions compare whole normalized artifacts; substring,
// prefix/suffix, regex, and partial-JSON probes are banned in test code.
//
// What is scanned
//   default (no --root), rooted at the repo root:
//     crates/<crate>/src/**/*.rs   — only code inside `#[cfg(test)] mod … { … }`
//     crates/<crate>/tests/**/*.rs — whole file (integration tests)
//   with --root <dir> (self-test hook):
//     every **/*.rs under <dir>; files with a `tests` path segment are
//     whole-file test code, everything else is scanned for `#[cfg(test)]`
//     modules.
//
// What is flagged — only INSIDE the argument span of assert!/assert_eq!/
// assert_ne!/debug_assert!/debug_assert_eq!/debug_assert_ne! invocations
// that lie in test code:
//   [contains]      `.contains(`
//   [starts-with]   `.starts_with(`
//   [ends-with]     `.ends_with(`
//   [regex]         `Regex::new`
//   [partial-json]  a `.contains(` hit whose assert span also invokes a
//                   `json!(…)` macro (partial JSON comparison idiom)
//
// Detection is deliberately conservative and line-heuristic-free where it
// matters (low false positives, per the P0-12 brief):
//   - a character-level scanner blanks comments and string/char-literal
//     contents first, so banned tokens inside strings or commented-out
//     code never match;
//   - only the exact assert macro family above opens a span (custom
//     macros such as insta::assert_snapshot! are ignored);
//   - `#[cfg(test)]` is honored only in that exact form attached to an
//     inline `mod name { … }`; cfg-gated functions, `mod name;` file
//     declarations, and `cfg(all(test, …))` variants are not scanned —
//     a documented gap chosen over guessing;
//   - when a span cannot be delimited confidently (unbalanced parens,
//     macro invoked with `[]`/`{}` delimiters), nothing is flagged.
//
// Allowlist: davinci-road/plan/assertion-allowlist.toml. One entry per
// file path (repo-root-relative, forward slashes) with a justification and
// an expiry date; a listed path suppresses all findings in that file until
// the entry expires. Expired entries stop suppressing (the findings come
// back); entries that match no finding produce a stderr warning so the
// list only ever shrinks.
//
// Exit codes: 0 = no unlisted findings, 1 = unlisted findings,
// 2 = usage/config error. `--list` prints every finding while ignoring
// the allowlist and always exits 0 (informational mode).

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const defaultAllowlistPath = path.join(
  repoRoot,
  "davinci-road",
  "plan",
  "assertion-allowlist.toml",
);

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

function collectTargets(root, defaultTree) {
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

// --- allowlist --------------------------------------------------------------

function loadAllowlist(allowlistPath) {
  let text;
  try {
    text = fs.readFileSync(allowlistPath, "utf8");
  } catch (error) {
    throw new Error(`cannot read allowlist ${allowlistPath}: ${error.message}`);
  }
  let parsed;
  try {
    parsed = parseTomlLite(text);
  } catch (error) {
    if (error instanceof TomlLiteError) {
      throw new Error(`malformed allowlist ${allowlistPath}: ${error.message}`);
    }
    throw error;
  }
  const entries = parsed.allow ?? [];
  if (!Array.isArray(entries)) {
    throw new Error(`malformed allowlist ${allowlistPath}: [[allow]] entries expected`);
  }
  const byPath = new Map();
  entries.forEach((entry, index) => {
    const where = `${allowlistPath} entry ${index + 1}`;
    if (typeof entry.path !== "string" || entry.path.length === 0 || entry.path.includes("\\")) {
      throw new Error(`${where}: path must be a repo-root-relative forward-slash string`);
    }
    if (typeof entry.justification !== "string" || entry.justification.trim().length === 0) {
      throw new Error(`${where}: justification is required`);
    }
    if (typeof entry.expires !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(entry.expires)) {
      throw new Error(`${where}: expires must be a quoted YYYY-MM-DD date`);
    }
    if (byPath.has(entry.path)) {
      throw new Error(`${where}: duplicate path ${entry.path}`);
    }
    byPath.set(entry.path, { ...entry, used: false });
  });
  return byPath;
}

// --- CLI --------------------------------------------------------------------

function parseArgs(argv) {
  const args = { list: false, root: null, allowlist: null };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--list") args.list = true;
    else if (arg === "--root") {
      args.root = argv[i + 1];
      i += 1;
      if (args.root == null) throw new Error("--root requires a directory argument");
    } else if (arg === "--allowlist") {
      args.allowlist = argv[i + 1];
      i += 1;
      if (args.allowlist == null) throw new Error("--allowlist requires a file argument");
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`unknown argument ${arg}`);
    }
  }
  return args;
}

const USAGE = `Usage: node tools/davinci/assertion-lint.mjs [--list] [--root <dir>] [--allowlist <file>]

Scans Rust test code for banned weak-assertion patterns (Davinci assurance
doctrine). Without flags: scans crates/**, applies the committed allowlist,
exits 1 on unlisted findings. --list ignores the allowlist and exits 0.
--root scans an alternate directory tree (self-test hook) with no default
allowlist.`;

function main() {
  let args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    console.error(USAGE);
    return 2;
  }
  if (args.help) {
    console.log(USAGE);
    return 0;
  }

  const defaultTree = args.root == null;
  const root = defaultTree ? repoRoot : path.resolve(args.root);
  if (!fs.existsSync(root)) {
    console.error(`scan root does not exist: ${root}`);
    return 2;
  }

  const findings = [];
  for (const target of collectTargets(root, defaultTree)) {
    const source = fs.readFileSync(target.file, "utf8");
    for (const finding of scanFile(source, { wholeFile: target.wholeFile })) {
      findings.push({ path: target.rel, ...finding });
    }
  }
  // Walk order is sorted, so findings are already (path, offset) ordered.

  if (args.list) {
    for (const finding of findings) {
      console.log(`${finding.path}:${finding.line} [${finding.category}] ${finding.text}`);
    }
    const files = new Set(findings.map((finding) => finding.path));
    console.log(
      `assertion-lint: ${findings.length} findings in ${files.size} files (allowlist ignored)`,
    );
    return 0;
  }

  let allowlist = new Map();
  const allowlistPath =
    args.allowlist != null
      ? path.resolve(args.allowlist)
      : defaultTree && fs.existsSync(defaultAllowlistPath)
        ? defaultAllowlistPath
        : null;
  if (allowlistPath != null) {
    try {
      allowlist = loadAllowlist(allowlistPath);
    } catch (error) {
      console.error(error.message);
      return 2;
    }
  }

  const today = new Date().toISOString().slice(0, 10);
  const unlisted = [];
  const expired = new Set();
  let suppressed = 0;
  const suppressedFiles = new Set();
  for (const finding of findings) {
    const entry = allowlist.get(finding.path);
    if (entry != null && entry.expires >= today) {
      entry.used = true;
      suppressed += 1;
      suppressedFiles.add(finding.path);
      continue;
    }
    if (entry != null) expired.add(finding.path);
    unlisted.push(finding);
  }

  for (const finding of unlisted) {
    const note = expired.has(finding.path) ? " (allowlist entry expired)" : "";
    console.log(`${finding.path}:${finding.line} [${finding.category}] ${finding.text}${note}`);
  }
  for (const [entryPath, entry] of allowlist) {
    if (!entry.used && !expired.has(entryPath)) {
      console.warn(
        `warning: allowlist entry ${entryPath} matched no finding — remove it (the list only shrinks)`,
      );
    }
  }

  if (unlisted.length > 0) {
    console.log(
      `assertion-lint: ${unlisted.length} unlisted findings — fix the assertion (exact oracles only) or triage via davinci-road/plan/assertion-allowlist.toml`,
    );
    return 1;
  }
  console.log(
    `assertion-lint: OK (${suppressed} findings in ${suppressedFiles.size} files suppressed by allowlist)`,
  );
  return 0;
}

const invokedDirectly =
  process.argv[1] != null && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);
if (invokedDirectly) {
  process.exitCode = main();
}
