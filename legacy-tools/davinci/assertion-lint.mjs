// Banned weak-assertion lint for Rust test code (Davinci P0-12).
//
// Doctrine: davinci-road/assurance.md, "Strict oracles — no partial
// matching". Test assertions compare whole normalized artifacts; substring,
// prefix/suffix, regex, and partial-JSON probes are banned in test code.
//
// This file is the CLI and allowlist half; the detection itself lives in
// tools/davinci/assertion-scan.mjs.
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
// Allowlist: davinci-road/plan/assertion-allowlist.toml. Each `[[allow]]`
// group carries one justification and one expiry date plus the `paths` it
// covers (repo-root-relative, forward slashes); a listed path suppresses
// all findings in that file until the group expires. Expired groups stop
// suppressing (the findings come back); paths that match no finding produce
// a stderr warning so the list only ever shrinks.
//
// Exit codes: 0 = no unlisted findings, 1 = unlisted findings,
// 2 = usage/config error. `--list` prints every finding while ignoring
// the allowlist and always exits 0 (informational mode).

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { collectTargets, scanFile } from "./assertion-scan.mjs";
import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const defaultAllowlistPath = path.join(
  repoRoot,
  "davinci-road",
  "plan",
  "assertion-allowlist.toml",
);

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
  const groups = parsed.allow ?? [];
  if (!Array.isArray(groups)) {
    throw new Error(`malformed allowlist ${allowlistPath}: [[allow]] groups expected`);
  }
  const byPath = new Map();
  groups.forEach((group, index) => {
    const where = `${allowlistPath} group ${index + 1}`;
    if (typeof group.justification !== "string" || group.justification.trim().length === 0) {
      throw new Error(`${where}: justification is required`);
    }
    if (typeof group.expires !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(group.expires)) {
      throw new Error(`${where}: expires must be a quoted YYYY-MM-DD date`);
    }
    if (!Array.isArray(group.paths) || group.paths.length === 0) {
      throw new Error(`${where}: paths must be a non-empty array`);
    }
    for (const entryPath of group.paths) {
      if (typeof entryPath !== "string" || entryPath.length === 0 || entryPath.includes("\\")) {
        throw new Error(`${where}: paths must be repo-root-relative forward-slash strings`);
      }
      if (byPath.has(entryPath)) {
        throw new Error(`${where}: duplicate path ${entryPath}`);
      }
      byPath.set(entryPath, {
        justification: group.justification,
        expires: group.expires,
        used: false,
      });
    }
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

const USAGE = `Usage: rust-script tools/commands/davinci/assertion-lint.rs [--list] [--root <dir>] [--allowlist <file>]

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
