// Register linter fix-history cases (#6881) in the current-API cohort: the
// case wire, its complete Debug snapshot oracle, the oracle index, the shared
// manifest rows and pins, and the history ledger.
//
//   node tools/commands/fixtures/linter-fix-history-case.ts register [--counts FILE] SPEC.json...
//   node tools/commands/fixtures/linter-fix-history-case.ts finalize SPEC.json...
//
// A SPEC.json names one historical fix and its cases:
//   { commit, cases: [{ id, source, filename, entry, rule, vue_version?, vapor? }] }
// `register` writes the cases into the cohort (diagnostic and fix counts come
// from --counts, a { id: { diagnostics, fixes } } map, else the existing
// case, else 0) and the ledger row. Then run
//   INSTA_UPDATE=always cargo test -p vize_patina --test integration lint_history
// so the snapshots exist, and `finalize` pins them in the oracle index and the
// manifest and records each SFC case's native outcome for the SFC contract test.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "../../../tests/differential/sha256.ts";
import { loadLinterManifest } from "../../../tests/differential/linter-api.ts";
import { expectedNativeReason } from "../../../tests/differential/linter-native.ts";
import { sourceInspectedDialectCoverage } from "../../../tests/differential/source-dialect-coverage.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const casesPath = path.join(root, "crates/vize_patina/tests/fixtures/lint-history/cases.json");
const testPath = path.join(root, "crates/vize_patina/tests/lint_history.rs");
const snapshots = "crates/vize_patina/tests/snapshots";
const linterRoot = path.join(root, "tests/_fixtures/differential/linter");
const manifestPath = path.join(linterRoot, "manifest.json");
const oraclePath = path.join(linterRoot, "oracle-index.json");
const ledgerPath = path.join(root, "docs/davinci/plan/linter-fix-history.tsv");
const nativeReasonsPath = path.join(linterRoot, "history-native-reasons.json");
const nativeCohortReaders = [
  "crates/vize_patina/src/rules/a11y/form_control_has_label/native/tests/support.rs",
  "crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option/native/tests/support.rs",
];
const pinnedTests = [
  "tests/tooling/differential-linter-api.test.ts",
  "tests/tooling/differential-linter-sfc-contract.test.ts",
];

type Wire = {
  id: string;
  history: string;
  source: string;
  filename: string;
  entry: "template" | "script" | "sfc";
  rule: string;
  vue_version: string | null;
  vapor: boolean | null;
  diagnostics: number;
  fixes: number;
};
type Spec = { commit: string; cases: Array<Partial<Wire> & { id: string; source: string }> };

const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;
const read = (file: string) => fs.readFileSync(file, "utf8");
const snapshotPath = (id: string) => `${snapshots}/integration__lint_history__${id}.snap`;

function loadSpec(file: string): Spec {
  const spec = JSON.parse(read(file)) as Spec;
  assert.match(spec.commit, /^[0-9a-f]{40}$/);
  execFileSync("git", ["cat-file", "-e", `${spec.commit}^{commit}`], { cwd: root });
  assert(spec.cases.length > 0, `${file}: a fix needs at least one case`);
  for (const item of spec.cases) {
    assert.match(item.id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/, `${file}: case id`);
    assert(["template", "script", "sfc"].includes(item.entry ?? ""), `${item.id}: entry`);
    assert.match(item.rule ?? "", /^[a-z0-9-]+\/[a-z0-9-]+$/, `${item.id}: rule`);
  }
  return spec;
}

function ledgerRows() {
  const lines = read(ledgerPath).split("\n");
  return {
    header: lines[0],
    rows: lines
      .slice(1)
      .filter(Boolean)
      .map((line) => line.split("\t")),
  };
}

function register(specs: Spec[], counts: Record<string, { diagnostics: number; fixes: number }>) {
  const cases = JSON.parse(read(casesPath)) as Wire[];
  const ledger = ledgerRows();
  for (const spec of specs) {
    for (const item of spec.cases) {
      const previous = cases.find((wire) => wire.id === item.id);
      assert(!previous || previous.history === spec.commit, `${item.id} belongs to another fix`);
      const wire: Wire = {
        id: item.id,
        history: spec.commit,
        source: item.source,
        filename: item.filename ?? "History.vue",
        entry: item.entry!,
        rule: item.rule!,
        vue_version: item.vue_version ?? null,
        vapor: item.vapor ?? null,
        diagnostics: counts[item.id]?.diagnostics ?? previous?.diagnostics ?? 0,
        fixes: counts[item.id]?.fixes ?? previous?.fixes ?? 0,
      };
      if (previous) cases[cases.indexOf(previous)] = wire;
      else cases.push(wire);
    }
    const row = ledger.rows.find((columns) => columns[0] === spec.commit);
    assert(row, `${spec.commit} is not a linter ledger row`);
    const ids = new Set(row[3] === "-" ? [] : row[3].split(","));
    for (const item of spec.cases) ids.add(`current-api/${item.id}`);
    row[2] = "oracle-pinned";
    row[3] = [...ids].join(",");
  }
  fs.writeFileSync(casesPath, json(cases));
  fs.writeFileSync(
    ledgerPath,
    `${[ledger.header, ...ledger.rows.map((columns) => columns.join("\t"))].join("\n")}\n`,
  );
  const test = read(testPath);
  const counted = test.replace(/(cases\.len\(\),\n\s+)\d+(,)/, `$1${cases.length}$2`);
  assert.notEqual(counted.indexOf(`${cases.length},`), -1, "lint_history.rs count pin");
  fs.writeFileSync(testPath, counted);
  // Native rule tests read the same cohort and pin its size.
  for (const file of nativeCohortReaders) {
    const full = path.join(root, file);
    const pinned = read(full).replace(
      /assert_eq!\((cases|all)\.len\(\), \d+\);/,
      `assert_eq!($1.len(), ${cases.length});`,
    );
    assert.notEqual(pinned.indexOf(`.len(), ${cases.length});`), -1, `${file} count pin`);
    fs.writeFileSync(full, pinned);
  }
  return cases;
}

function finalize(specs: Spec[]) {
  const cases = JSON.parse(read(casesPath)) as Wire[];
  const oracles = JSON.parse(read(oraclePath)) as Record<string, { path: string; sha256: string }>;
  const manifest = JSON.parse(read(manifestPath));
  const before = {
    cohort: manifest.cohorts.find((cohort: any) => cohort.id === "current-api").cases.sha256,
    oracles: manifest.oracles.sha256,
    planned: Object.keys(oracles).length,
    current: Object.keys(oracles).filter((id) => id.startsWith("linter/current-api/")).length,
  };
  for (const spec of specs) {
    for (const item of spec.cases) {
      const wire = cases.find((candidate) => candidate.id === item.id)!;
      const id = `linter/current-api/${item.id}`;
      const snapshot = snapshotPath(item.id);
      oracles[id] = { path: snapshot, sha256: sha256(fs.readFileSync(path.join(root, snapshot))) };
      const row = {
        id,
        state: "active",
        targets: ["lint"],
        dialectCoverage: sourceInspectedDialectCoverage(
          [
            {
              filename: wire.filename,
              source: wire.source,
              entry: wire.entry,
              vueVersion: wire.vue_version,
            },
          ],
          "current-api pinned authored sources",
        ),
      };
      const at = manifest.cases.findIndex((fixture: any) => fixture.id === id);
      if (at >= 0) manifest.cases[at] = { ...manifest.cases[at], ...row };
      else {
        const next = manifest.cases.findIndex((fixture: any) => fixture.id > id);
        manifest.cases.splice(next < 0 ? manifest.cases.length : next, 0, row);
      }
    }
  }
  fs.writeFileSync(oraclePath, json(oracles));
  const cohort = manifest.cohorts.find((entry: any) => entry.id === "current-api");
  cohort.cases.sha256 = sha256(fs.readFileSync(casesPath));
  manifest.oracles.sha256 = sha256(fs.readFileSync(oraclePath));
  fs.writeFileSync(manifestPath, json(manifest));
  const after = {
    cohort: cohort.cases.sha256,
    oracles: manifest.oracles.sha256,
    planned: Object.keys(oracles).length,
    current: Object.keys(oracles).filter((id) => id.startsWith("linter/current-api/")).length,
  };
  for (const file of pinnedTests) {
    const full = path.join(root, file);
    const updated = read(full)
      .replaceAll(before.cohort, after.cohort)
      .replaceAll(before.oracles, after.oracles)
      .replace(
        `assert.equal(loaded.cases.length, ${before.planned});`,
        `assert.equal(loaded.cases.length, ${after.planned});`,
      )
      .replace(`"current-api": ${before.current},`, `"current-api": ${after.current},`);
    fs.writeFileSync(full, updated);
  }
  // The SFC contract test pins every SFC wire's native outcome; history
  // cases record theirs beside the cohort instead of in the test body.
  const loaded = loadLinterManifest(manifestPath, root);
  const reasons: Record<string, unknown> = fs.existsSync(nativeReasonsPath)
    ? JSON.parse(read(nativeReasonsPath))
    : {};
  for (const spec of specs) {
    for (const item of spec.cases) {
      if (item.entry !== "sfc") {
        // A case that moved off the SFC entry no longer has an SFC outcome.
        delete reasons[`current-api/${item.id}`];
        continue;
      }
      const fixture = loaded.cases.find((row: any) => row.id === `linter/current-api/${item.id}`);
      reasons[`current-api/${item.id}`] = expectedNativeReason(fixture);
    }
  }
  fs.writeFileSync(
    nativeReasonsPath,
    json(Object.fromEntries(Object.entries(reasons).sort(([a], [b]) => a.localeCompare(b)))),
  );
  console.log(`current-api ${before.current} -> ${after.current}; planned ${after.planned}`);
}

const [command, ...rest] = process.argv.slice(2);
const countsAt = rest.indexOf("--counts");
const counts = countsAt >= 0 ? JSON.parse(read(rest.splice(countsAt, 2)[1])) : {};
const specs = rest.map(loadSpec);
if (command === "register") console.log(`${register(specs, counts).length} cases`);
else if (command === "finalize") finalize(specs);
else throw new Error("usage: register|finalize SPEC.json...");
