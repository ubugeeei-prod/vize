import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// Python, Lean, MoonBit and `.mjs` tooling were retired by the maintainer on
// 2026-10-13 (#8840). Python, MoonBit and `.mjs` tooling are rewritten in strictly
// typed TypeScript and Lean proofs become Rust Kani harnesses. The ledger lists
// every file that still waits for its port, and this test makes the ledger a ratchet.
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ledgerPath = "tools/config/retired-tooling-languages.json";
const languages = ["py", "lean", "mbt", "mjs"] as const;
type Language = (typeof languages)[number];

const trackedPatterns: Readonly<Record<Language, readonly string[]>> = {
  py: ["*.py"],
  lean: ["*.lean", "lean-toolchain", "*/lean-toolchain"],
  mbt: ["*.mbt"],
  mjs: ["*.mjs"],
};
const replacement: Readonly<Record<Language, string>> = {
  py: "write TypeScript instead",
  lean: "write a Rust Kani harness instead",
  mbt: "write TypeScript instead",
  mjs: "write strictly typed TypeScript that Node runs directly instead",
};

interface PermanentPrefix {
  readonly language: Language;
  readonly prefix: string;
  readonly reason: string;
}
interface Ledger {
  readonly permanent: readonly PermanentPrefix[];
  readonly retiring: Readonly<Record<Language, readonly string[]>>;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isLanguage(value: unknown): value is Language {
  return languages.some((language) => language === value);
}

function stringList(value: unknown, label: string): readonly string[] {
  assert.ok(Array.isArray(value), `${label} must be an array`);
  const entries: string[] = [];
  for (const entry of value as readonly unknown[]) {
    assert.equal(typeof entry, "string", `${label} entries must be strings`);
    entries.push(entry as string);
  }
  return entries;
}

function parseLedger(text: string): Ledger {
  const parsed: unknown = JSON.parse(text);
  assert.ok(isRecord(parsed), "the ledger must be an object");
  assert.ok(Array.isArray(parsed.permanent), "permanent must be an array");
  const permanent: PermanentPrefix[] = [];
  for (const entry of parsed.permanent as readonly unknown[]) {
    assert.ok(isRecord(entry), "permanent entries must be objects");
    const { language, prefix, reason } = entry;
    assert.ok(isLanguage(language), "permanent entries name a retired language");
    assert.ok(typeof prefix === "string" && prefix.endsWith("/"), "prefix is a directory");
    assert.ok(typeof reason === "string" && reason.length > 0, "permanent entries give a reason");
    permanent.push({ language, prefix, reason });
  }
  assert.ok(isRecord(parsed.retiring), "retiring must be an object");
  assert.deepEqual(Object.keys(parsed.retiring).sort(), [...languages].sort());
  const retiring = parsed.retiring;
  return {
    permanent,
    retiring: {
      py: stringList(retiring.py, "retiring.py"),
      lean: stringList(retiring.lean, "retiring.lean"),
      mbt: stringList(retiring.mbt, "retiring.mbt"),
      mjs: stringList(retiring.mjs, "retiring.mjs"),
    },
  };
}

function git(args: readonly string[]): { readonly status: number | null; readonly stdout: string } {
  const run = spawnSync("git", [...args], { cwd: repoRoot, encoding: "utf8" });
  return { status: run.status, stdout: run.stdout };
}

function comparePaths(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function retiringFiles(ledger: Ledger, language: Language): readonly string[] {
  const listed = git(["ls-files", "-z", "--", ...trackedPatterns[language]]);
  assert.equal(listed.status, 0, "git ls-files must succeed");
  const prefixes = ledger.permanent
    .filter((entry) => entry.language === language)
    .map((entry) => entry.prefix);
  return listed.stdout
    .split("\0")
    .filter((file) => file.length > 0 && !prefixes.some((prefix) => file.startsWith(prefix)))
    .sort(comparePaths);
}

const ledger = parseLedger(readFileSync(path.join(repoRoot, ledgerPath), "utf8"));

void test("every Python, Lean, MoonBit and .mjs tooling file is listed for retirement", () => {
  for (const language of languages) {
    const tracked = retiringFiles(ledger, language);
    const listed = ledger.retiring[language];
    assert.deepEqual(
      [...listed].sort(comparePaths),
      listed,
      `retiring.${language} must stay sorted so the ledger diffs cleanly`,
    );
    const unlisted = tracked.filter((file) => !listed.includes(file));
    assert.deepEqual(
      unlisted,
      [],
      `.${language} is retired (#8840): ${replacement[language]} of adding these files`,
    );
    const stale = listed.filter((file) => !tracked.includes(file));
    assert.deepEqual(
      stale,
      [],
      `retiring.${language} lists files that are gone; delete their ${ledgerPath} entries`,
    );
  }
});

void test("the retirement ledger never accepts a file the comparison base lacks", (t) => {
  const base = process.env.BASE_SHA ?? "";
  if (!/^[0-9a-f]{40}$/u.test(base) || /^0+$/u.test(base)) {
    t.skip("no comparison base in this run");
    return;
  }
  if (git(["cat-file", "-e", `${base}^{commit}`]).status !== 0) {
    t.skip("the comparison base is not fetched in this checkout");
    return;
  }
  for (const language of languages) {
    const listed = ledger.retiring[language];
    if (listed.length === 0) continue;
    // Only the listed paths are read, so the output stays small on a large tree.
    const baseTree = git(["ls-tree", "-r", "-z", "--name-only", base, "--", ...listed]);
    assert.equal(baseTree.status, 0, "git ls-tree must read the comparison base");
    const baseFiles = new Set(baseTree.stdout.split("\0"));
    const added = listed.filter((file) => !baseFiles.has(file));
    assert.deepEqual(
      added,
      [],
      `retiring.${language} only shrinks: ${replacement[language]} of adding or moving these files`,
    );
  }
});
