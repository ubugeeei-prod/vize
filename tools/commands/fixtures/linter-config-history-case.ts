// Register linter fix-history cases (#6881) that need host configuration in
// the configured cohort: presets, rule sets, severity and category overrides,
// and typed rule options (the same `linter.ruleOptions` JSON the CLI reads).
//
//   node tools/commands/fixtures/linter-config-history-case.ts register [--counts FILE] SPEC.json...
//   node tools/commands/fixtures/linter-config-history-case.ts finalize
//
// A SPEC.json names one historical fix and its cases:
//   { commit, cases: [{ id, source, filename, entry, preset, enabled_rules?, disabled_rules?,
//       rule_severities?, disabled_categories?, category_severities?, rule_options? }] }
// `register` writes each source beside the cohort, the case wire (diagnostic
// and fix counts come from --counts, a { id: { diagnostics, fixes } } map, else
// the existing case, else 0), the runner's count pin and the ledger row. Then run
//   VIZE_CONFIG_HISTORY_UPDATE=1 cargo test -p vize_patina --test integration config_history
// so the complete observations exist, and `finalize` pins every cohort file in
// the shared linter manifest.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "../../../tests/differential/sha256.ts";
import { sourceInspectedDialectCoverage } from "../../../tests/differential/source-dialect-coverage.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const CONFIG_COHORT = "config-history-6881";
const cohortRoot = path.join(root, "tests/_fixtures/differential/linter", CONFIG_COHORT);
const casesPath = path.join(cohortRoot, "cases.json");
const testPath = path.join(root, "crates/vize_patina/tests/config_history.rs");
const manifestPath = path.join(root, "tests/_fixtures/differential/linter/manifest.json");
const ledgerPath = path.join(root, "docs/davinci/plan/linter-fix-history.tsv");
const ENTRY_EXTENSION = { template: "html", script: "ts", sfc: "vue" } as const;

type Entry = keyof typeof ENTRY_EXTENSION;
export type ConfigWire = {
  id: string;
  history: string;
  source_file: string;
  filename: string;
  entry: Entry;
  preset: string;
  enabled_rules: string[] | null;
  disabled_rules: string[];
  rule_severities: [string, string][];
  disabled_categories: string[];
  category_severities: [string, string][];
  rule_options: Record<string, unknown>;
  diagnostics: number;
  fixes: number;
};
type SpecCase = Partial<Omit<ConfigWire, "history" | "source_file">> & {
  id: string;
  source: string;
};
type Spec = { commit: string; cases: SpecCase[] };

const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;
const read = (file: string) => fs.readFileSync(file, "utf8");

/** The source file of a case: its extension names the authored dialect. */
export function sourceFile(id: string, entry: Entry, filename: string) {
  const extension =
    entry === "script" && /\.(?:[cm]?js)$/.test(filename) ? "js" : ENTRY_EXTENSION[entry];
  return `${id}.${extension}.txt`;
}

function loadSpec(file: string): Spec {
  const spec = JSON.parse(read(file)) as Spec;
  assert.match(spec.commit, /^[0-9a-f]{40}$/);
  execFileSync("git", ["cat-file", "-e", `${spec.commit}^{commit}`], { cwd: root });
  assert(spec.cases.length > 0, `${file}: a fix needs at least one case`);
  for (const item of spec.cases) {
    assert.match(item.id, /^[a-z0-9]+(?:-[a-z0-9]+)*$/, `${file}: case id`);
    assert(Object.hasOwn(ENTRY_EXTENSION, item.entry ?? ""), `${item.id}: entry`);
    assert.equal(typeof item.preset, "string", `${item.id}: preset`);
  }
  return spec;
}

function register(specs: Spec[], counts: Record<string, { diagnostics: number; fixes: number }>) {
  fs.mkdirSync(cohortRoot, { recursive: true });
  const cases = (fs.existsSync(casesPath) ? JSON.parse(read(casesPath)) : []) as ConfigWire[];
  const lines = read(ledgerPath).split("\n");
  for (const spec of specs) {
    for (const item of spec.cases) {
      const previous = cases.find((wire) => wire.id === item.id);
      assert(!previous || previous.history === spec.commit, `${item.id} belongs to another fix`);
      const entry = item.entry as Entry;
      const filename = item.filename ?? "History.vue";
      const wire: ConfigWire = {
        id: item.id,
        history: spec.commit,
        source_file: sourceFile(item.id, entry, filename),
        filename,
        entry,
        preset: item.preset!,
        enabled_rules: item.enabled_rules ?? null,
        disabled_rules: item.disabled_rules ?? [],
        rule_severities: item.rule_severities ?? [],
        disabled_categories: item.disabled_categories ?? [],
        category_severities: item.category_severities ?? [],
        rule_options: item.rule_options ?? {},
        diagnostics: counts[item.id]?.diagnostics ?? previous?.diagnostics ?? 0,
        fixes: counts[item.id]?.fixes ?? previous?.fixes ?? 0,
      };
      fs.writeFileSync(path.join(cohortRoot, wire.source_file), item.source);
      if (previous) cases[cases.indexOf(previous)] = wire;
      else cases.push(wire);
    }
    const at = lines.findIndex((line) => line.startsWith(`${spec.commit}\t`));
    assert(at > 0, `${spec.commit} is not a linter ledger row`);
    const columns = lines[at].split("\t");
    const ids = new Set(columns[3] === "-" ? [] : columns[3].split(","));
    for (const item of spec.cases) ids.add(`${CONFIG_COHORT}/${item.id}`);
    columns[2] = "oracle-pinned";
    columns[3] = [...ids].join(",");
    lines[at] = columns.join("\t");
  }
  cases.sort((a, b) => a.id.localeCompare(b.id));
  fs.writeFileSync(casesPath, json(cases));
  fs.writeFileSync(ledgerPath, lines.join("\n"));
  const test = read(testPath);
  const counted = test.replace(/(cases\.len\(\),\n\s+)\d+(,)/, `$1${cases.length}$2`);
  assert.notEqual(counted.indexOf(`${cases.length},`), -1, "config_history.rs count pin");
  fs.writeFileSync(testPath, counted);
  return cases;
}

/** The manifest registration of the cohort: every file, pinned by content. */
export function configCohortRow(directory = cohortRoot) {
  const cases = JSON.parse(read(path.join(directory, "cases.json"))) as ConfigWire[];
  const files = [
    { path: "cases.json", role: "pinned-configuration-and-counts" },
    ...cases.flatMap((wire) => [
      { path: wire.source_file, role: "authored-fix-history-input" },
      { path: `${wire.id}.observation.txt`, role: "complete-current-observation" },
    ]),
  ].map((file) => ({ ...file, sha256: sha256(fs.readFileSync(path.join(directory, file.path))) }));
  const listed = new Set(files.map((file) => file.path));
  for (const name of fs.readdirSync(directory)) assert(listed.has(name), `unregistered ${name}`);
  return {
    id: `linter/${CONFIG_COHORT}`,
    state: "active",
    targets: ["lint"],
    inputs: { root: CONFIG_COHORT, files },
    dialectCoverage: sourceInspectedDialectCoverage(
      cases.map((wire) => ({
        filename: wire.source_file,
        source: read(path.join(directory, wire.source_file)),
        entry: wire.entry,
      })),
      "linter manifest pinned authored sources",
    ),
  };
}

function finalize() {
  const manifest = JSON.parse(read(manifestPath));
  const row = configCohortRow();
  const at = manifest.cases.findIndex((fixture: any) => fixture.id === row.id);
  if (at >= 0) manifest.cases[at] = row;
  else {
    const next = manifest.cases.findIndex((fixture: any) => fixture.id > row.id);
    manifest.cases.splice(next < 0 ? manifest.cases.length : next, 0, row);
  }
  fs.writeFileSync(manifestPath, json(manifest));
  console.log(`${row.id}: ${row.inputs.files.length} pinned files`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [command, ...rest] = process.argv.slice(2);
  const countsAt = rest.indexOf("--counts");
  const counts = countsAt >= 0 ? JSON.parse(read(rest.splice(countsAt, 2)[1])) : {};
  if (command === "register") console.log(`${register(rest.map(loadSpec), counts).length} cases`);
  else if (command === "finalize") finalize();
  else throw new Error("usage: register|finalize SPEC.json...");
}
