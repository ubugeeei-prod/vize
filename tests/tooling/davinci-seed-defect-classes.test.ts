// P4-15a — exact/sound rules must have a seeded defect class.
//
// `--check-classes` enumerates Exact/Sound rows from rule_contracts.rs.
// A rule with no class fails the check. Rows in ledger-fn.md are triaged
// misses: the check still exits 1 until a class exists, and exits 2 when
// the ledger and the generator disagree. Snippet classes are recalled by
// diagnostic identity, not by count. This slice does not claim TS-37 100%.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const seedTool = path.join(root, "tools/commands/davinci/seed-defects.rs");
const tablePath = path.join(root, "crates/vize_patina/src/rule_contracts/table.rs");
const ledgerPath = path.join(root, "davinci-road/plan/ledger-fn.md");
const ROW =
  /^\s*row!\("(?<name>[^"]+)",\s*(?<tier>Exact|Sound|Complete|Heuristic),\s*[A-Z0-9_]+,\s*(?:Error|Warning)\),\s*$/u;

type TierName = "Exact" | "Sound" | "Complete" | "Heuristic";
type Span = {
  path: string;
  ruleId: string;
  severity: number;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
};
type Manifest = {
  classes: { class: string; rule: string; path: string; expected: Span[] }[];
};

function runTool(args: string[]) {
  const result = spawnSync("rust-script", [seedTool, ...args], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.error) throw result.error;
  return result;
}

const temps: string[] = [];
process.on("exit", () => {
  for (const dir of temps) fs.rmSync(dir, { recursive: true, force: true });
});

function tempDir(label: string): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), `davinci-exact-${label}-`));
  temps.push(dir);
  return dir;
}

function readTiers(source = fs.readFileSync(tablePath, "utf8")): Map<string, string> {
  const tiers = new Map<string, string>();
  for (const [index, line] of source.split("\n").entries()) {
    if (!line.trimStart().startsWith("row!(")) continue;
    const match = ROW.exec(line);
    assert.ok(match?.groups, `${tablePath}:${index + 1} is not a contract row`);
    assert.equal(tiers.has(match.groups.name), false, match.groups.name);
    tiers.set(match.groups.name, match.groups.tier.toLowerCase());
  }
  return tiers;
}

function ledgerRules(text: string): string[] {
  const start = text.indexOf("<!-- p4-15a-unclassed -->");
  const end = text.indexOf("<!-- /p4-15a-unclassed -->");
  assert.ok(start !== -1 && end > start, "ledger is missing the p4-15a-unclassed list");
  return text
    .slice(start, end)
    .split("\n")
    .slice(1)
    .filter((line) => line.length > 0)
    .map((line) => {
      const match = /^- `([^`]+)`$/u.exec(line);
      assert.ok(match, line);
      return match[1];
    });
}

function classedRules(stdout: string): string[] {
  return stdout
    .split("\n")
    .filter((line) => line.startsWith("classed-rule "))
    .map((line) => line.slice("classed-rule ".length));
}

function scopeOf(stdout: string): Record<string, number> {
  const line = stdout.split("\n").find((entry) => entry.startsWith("scope-proof: exact="));
  assert.ok(line, stdout);
  return Object.fromEntries(
    [...line.matchAll(/([a-z-]+)=(\d+)/g)].map((match) => [match[1], Number(match[2])]),
  );
}

function snippetRules(): string[] {
  const source = fs.readFileSync(
    path.join(root, "tools/support/davinci/seed_exact_snippets.rs"),
    "utf8",
  );
  return [...source.matchAll(/snippet\(\s*"([^"]+)",\s*"([^"]+)"/g)].map((match) => match[2]);
}

function nestingClassIds(): string[] {
  const source = fs.readFileSync(path.join(root, "tools/support/davinci/seed_html.rs"), "utf8");
  const body = source.slice(
    source.indexOf("pub const NESTING:"),
    source.indexOf("pub struct Expected"),
  );
  return [...body.matchAll(/"([a-z0-9-]+)"/g)].map((match) => match[1]);
}

function writeTable(dir: string, rows: [string, TierName][]): string {
  const file = path.join(dir, "table.rs");
  const body = rows
    .map(([name, tier]) => `    row!("${name}", ${tier}, HTML, Warning),`)
    .join("\n");
  fs.writeFileSync(file, `${body}\n`);
  return file;
}

function writeLedger(dir: string, rules: string[]): string {
  const file = path.join(dir, "ledger.md");
  const lines = rules.map((rule) => `- \`${rule}\``).join("\n");
  fs.writeFileSync(
    file,
    `<!-- p4-15a-unclassed -->\n${lines}${lines.length > 0 ? "\n" : ""}<!-- /p4-15a-unclassed -->\n`,
  );
  return file;
}

function checkAgainst(rows: [string, TierName][], triaged: string[]) {
  const dir = tempDir("contracts");
  const contracts = writeTable(dir, rows);
  const ledger = writeLedger(dir, triaged);
  return runTool(["--check-classes", "--contracts", contracts, "--ledger", ledger]);
}

const check = runTool(["--check-classes"]);
const classed = classedRules(check.stdout);

test("exact/sound rules are classed or triaged, and unclassed rules still fail the check", () => {
  const tiers = readTiers();
  const exactSound = [...tiers]
    .filter(([, tier]) => tier === "exact" || tier === "sound")
    .map(([name]) => name)
    .sort();
  const missing = exactSound.filter((name) => !classed.includes(name));
  const ledger = ledgerRules(fs.readFileSync(ledgerPath, "utf8")).sort();
  const scope = scopeOf(check.stdout);
  const snippets = snippetRules();
  assert.deepEqual(ledger, missing);
  assert.deepEqual(classed, [...new Set([...snippets, "vue/permitted-contents"])].sort());
  for (const rule of classed) {
    const tier = tiers.get(rule);
    assert.ok(tier === "exact" || tier === "sound", rule);
  }
  assert.equal(scope.exact, [...tiers.values()].filter((tier) => tier === "exact").length);
  assert.equal(scope.sound, [...tiers.values()].filter((tier) => tier === "sound").length);
  assert.equal(scope["classed-rules"], classed.length);
  assert.equal(scope["classed-classes"], new Set(nestingClassIds()).size + snippets.length);
  assert.equal(scope.unclassed, missing.length);
  assert.equal(scope.untriaged, 0);
  assert.equal(scope.stale, 0);
  assert.equal(check.status, missing.length === 0 ? 0 : 1, `${check.stdout}\n${check.stderr}`);
  assert.doesNotMatch(check.stdout, /^(?:UNTRIAGED|STALE|STRAY) /m);
});

test("the check passes when every exact rule in the table has a class", () => {
  const result = checkAgainst(
    classed.map((name) => [name, "Exact"]),
    [],
  );
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.equal(scopeOf(result.stdout).unclassed, 0);
});

test("a new exact rule without a class is untriaged", () => {
  const result = checkAgainst(
    [...classed.map((name) => [name, "Exact"] as [string, TierName]), ["zz/brand-new", "Exact"]],
    [],
  );
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /^UNTRIAGED zz\/brand-new$/m);
  assert.equal(scopeOf(result.stdout).untriaged, 1);
});

test("triage records the miss and does not make the check pass", () => {
  const result = checkAgainst(
    [...classed.map((name) => [name, "Exact"] as [string, TierName]), ["zz/brand-new", "Exact"]],
    ["zz/brand-new"],
  );
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  const scope = scopeOf(result.stdout);
  assert.equal(scope.unclassed, 1);
  assert.equal(scope.untriaged, 0);
  assert.doesNotMatch(result.stdout, /^UNTRIAGED /m);
});

test("a new sound rule without a class is untriaged", () => {
  const result = checkAgainst(
    [...classed.map((name) => [name, "Exact"] as [string, TierName]), ["zz/sound-new", "Sound"]],
    [],
  );
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /^UNTRIAGED zz\/sound-new$/m);
});

test("a class whose rule is not exact or sound fails", () => {
  const demoted = classed.map(
    (name) => [name, name === "vue/no-v-html" ? "Heuristic" : "Exact"] as [string, TierName],
  );
  const result = checkAgainst(demoted, []);
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /STRAY class "no-v-html" cites "vue\/no-v-html"/);
});

test("a ledger row for a rule that is not an unclassed exact/sound rule is stale", () => {
  const result = checkAgainst(
    classed.map((name) => [name, "Exact"]),
    ["zz/gone"],
  );
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /^STALE zz\/gone$/m);
});

test("an unreadable contract row fails the check", () => {
  const dir = tempDir("bad-row");
  const contracts = path.join(dir, "table.rs");
  fs.writeFileSync(contracts, '    row!("vue/no-v-html", NotATier, HTML, Warning),\n');
  const ledger = writeLedger(dir, []);
  const result = runTool(["--check-classes", "--contracts", contracts, "--ledger", ledger]);
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stderr, /unreadable contract row/);
});

function lintFrom(manifest: Manifest, mutate?: (row: Span, index: number) => void) {
  return manifest.classes.map((entry) => {
    const messages = entry.expected.map((row, index) => {
      const copy = { ...row };
      mutate?.(copy, index);
      const { path: _path, ...message } = copy;
      return message;
    });
    return {
      file: entry.path,
      messages,
      errorCount: messages.filter((message) => message.severity === 2).length,
      warningCount: messages.filter((message) => message.severity === 1).length,
    };
  });
}

function assertExact(mutate?: (row: Span, index: number) => void) {
  const out = tempDir("seed");
  const seeded = runTool(["--exact-classes", "--out", out]);
  assert.equal(seeded.status, 0, `${seeded.stdout}\n${seeded.stderr}`);
  const manifest = JSON.parse(fs.readFileSync(path.join(out, "manifest.json"), "utf8")) as Manifest;
  const lintPath = path.join(out, "lint.json");
  fs.writeFileSync(lintPath, JSON.stringify(lintFrom(manifest, mutate)));
  const result = runTool([
    "--exact-classes",
    "--out",
    out,
    "--assert",
    "--seeded-lint-json",
    lintPath,
  ]);
  const report = JSON.parse(fs.readFileSync(path.join(out, "exact-report.json"), "utf8")) as {
    verdict: string;
    misses: unknown[];
    unexpected: unknown[];
    classes: { class: string; expected: number; detected: number }[];
  };
  return { result, report, manifest };
}

test("snippet classes pass when the diagnostic set matches by identity", () => {
  const { result, report, manifest } = assertExact();
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.equal(report.verdict, "pass");
  assert.equal(manifest.classes.length, snippetRules().length);
  for (const recall of report.classes) assert.equal(recall.detected, recall.expected, recall.class);
  assert.deepEqual(report.misses, []);
  assert.deepEqual(report.unexpected, []);
});

test("identity, not count: a moved span fails the snippet assert", () => {
  const { result, report } = assertExact((row, index) => {
    if (index === 0 && row.ruleId === "vue/no-v-html") row.column += 1;
  });
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  assert.equal(report.verdict, "fail");
  assert.equal(report.misses.length, 1);
  assert.equal(report.unexpected.length, 1);
});

test("a linter that finds nothing misses every snippet injection", () => {
  const { result, report } = assertExact((row) => {
    row.ruleId = "not-a-seeded-rule";
  });
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  assert.equal(report.misses.length, snippetRules().length);
  assert.equal(report.unexpected.length, 0);
});

test("vize lint recalls every snippet class by identity", () => {
  const out = tempDir("vize");
  const result = runTool(["--exact-classes", "--out", out, "--assert"]);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /assert: detected=19\/19 unexpected=0 verdict=pass/);
  const report = JSON.parse(fs.readFileSync(path.join(out, "exact-report.json"), "utf8")) as {
    verdict: string;
    misses: unknown[];
    unexpected: unknown[];
    classes: { detected: number; expected: number }[];
  };
  assert.equal(report.verdict, "pass");
  assert.deepEqual(report.misses, []);
  assert.deepEqual(report.unexpected, []);
  for (const recall of report.classes) assert.equal(recall.detected, recall.expected);
});
