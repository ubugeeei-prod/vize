// Register typechecker fix-history packs (#6879): pack files, manifest
// projects, registry entries, the Rust test body and the ledger state.
//
//   node tools/commands/fixtures/typechecker-fix-history-pack.ts [--capture DIR] SPEC.json|PACK...
//
// A SPEC.json names a new or changed pack:
//   { pack, module, test, regressionCommit, historicalIssue, checkerOptions?,
//     projectOptions?, sources?, cases? }
// `sources` maps fixture file names to their bytes; `cases` lists
// `{ id, inputs: [{ file, source }] }`. Without `cases` the pack's
// cases.json supplies them. A bare PACK re-registers an existing pack from
// its cases.json (for example after taking main's manifest in a conflict).
// Diagnostics come from DIR/<pack>/<case>.json when present (arrays of
// runner diagnostics), else from the existing cases.json, else stay empty.
// Run `cargo fmt -p vize_canon` afterwards.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "../../../tests/differential/sha256.ts";
import { typecheckerDialectCoverage } from "../../../tests/differential/typechecker-dialect.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const fixtures = path.join(root, "tests/_fixtures/differential/typechecker");
const manifestPath = path.join(fixtures, "manifest.json");
const registryPath = path.join(fixtures, "fix-history-packs.json");
const casesDir = path.join(root, "crates/vize_canon/tests/fix_history_cases");
const ledgerPath = path.join(root, "docs/davinci/plan/typechecker-fix-history.tsv");
const admitted = "batch-start-actions-verified";
const contract = {
  requiredTier: "T1",
  positionBase: 1,
  positions: "authored UTF16 start",
  comparison: "all production diagnostics in returned order",
  missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
  native: "unsupported",
};

type Input = { file: string; source: string };
type Diagnostic = {
  file: string;
  line: number;
  column: number;
  severity: number;
  code: number | null;
  message: string;
};
type Case = { id: string; inputs: Input[]; diagnostics?: Diagnostic[] };
type CheckerOptions = {
  optionsApi?: boolean;
  jsxTypecheck?: boolean;
  legacyVue2?: boolean;
  virtualTsChecks?: { props: boolean; templateBindings: boolean; emits: boolean };
  // What `vize check` derives from a Nuxt project's generated types.
  strictInstanceGlobals?: boolean;
  autoImportStubs?: string[];
  // Package routes `vize check` installs, by importer and specifier.
  packageRoutes?: Array<{ importer: string; specifier: string }>;
  // Workspace packages linked into node_modules.
  workspaceLinks?: Array<{ package: string; link: string }>;
  // Explicit authored check roots, as selected by the CLI's owning input scope.
  scanPaths?: string[];
};
type Spec = {
  pack: string;
  module: string;
  test: string;
  regressionCommit: string;
  historicalIssue: number | null;
  checkerOptions?: CheckerOptions;
  projectOptions?: { vuePackage: "absent" };
  sources?: Record<string, string>;
  cases?: Case[];
};
type Entry = Record<string, unknown> & { id: string; pack?: string };

const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;
const read = (file: string) => fs.readFileSync(file, "utf8");

function safeName(name: string) {
  assert(
    name.split("/").every((part) => part !== "" && part !== "." && part !== ".."),
    `unsafe fixture path ${name}`,
  );
  return name;
}

type Registered = {
  test: string;
  sourceRevision: string;
  checkerOptions?: CheckerOptions;
  projectOptions?: { vuePackage: "absent" };
};
const readRegistry = (): Record<string, Registered> => JSON.parse(read(registryPath));

function register(pack: string, entry: Registered) {
  const registry = readRegistry();
  registry[pack] = entry;
  const sorted = Object.fromEntries(
    Object.entries(registry).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
  );
  fs.writeFileSync(registryPath, json(sorted));
}

function loadSpec(argument: string): Spec {
  if (argument.endsWith(".json")) return JSON.parse(read(argument)) as Spec;
  const pack = JSON.parse(read(path.join(fixtures, argument, "cases.json")));
  const test = readRegistry()[argument]?.test;
  assert(test, `${argument} has no registered test body`);
  const [, module, body] = /^fix_history_cases::([a-z0-9_]+)::([a-z0-9_]+)$/.exec(test) ?? [];
  assert(module && body, `${argument} is not a fix_history_cases pack`);
  return {
    pack: argument,
    module,
    test: body,
    regressionCommit: pack.regressionCommit,
    historicalIssue: pack.historicalIssue,
    checkerOptions: pack.checkerOptions,
    projectOptions: pack.projectOptions,
  };
}

function diagnosticsFor(spec: Spec, item: Case, previous: Case[], capture?: string) {
  const captured = capture && path.join(capture, spec.pack, `${item.id}.json`);
  if (captured && fs.existsSync(captured)) return JSON.parse(read(captured)) as Diagnostic[];
  return item.diagnostics ?? previous.find((old) => old.id === item.id)?.diagnostics ?? [];
}

function writePack(spec: Spec, capture?: string) {
  assert.match(spec.pack, /^[a-z0-9]+(?:-[a-z0-9]+)*$/);
  assert.match(spec.module, /^[a-z][a-z0-9_]*$/);
  assert.match(spec.test, /^[a-z][a-z0-9_]*$/);
  assert.match(spec.regressionCommit, /^[0-9a-f]{40}$/);
  execFileSync("git", ["cat-file", "-e", `${spec.regressionCommit}^{commit}`], { cwd: root });
  const dir = path.join(fixtures, spec.pack);
  fs.mkdirSync(dir, { recursive: true });
  for (const [name, bytes] of Object.entries(spec.sources ?? {})) {
    fs.writeFileSync(path.join(dir, safeName(name)), bytes);
  }
  const casesPath = path.join(dir, "cases.json");
  const previous: Case[] = fs.existsSync(casesPath) ? JSON.parse(read(casesPath)).cases : [];
  const cases = (spec.cases ?? previous).map((item) => ({
    id: item.id,
    inputs: item.inputs.map(({ file, source }) => ({
      file: safeName(file),
      source: safeName(source),
    })),
    diagnostics: diagnosticsFor(spec, item, previous, capture),
  }));
  assert(cases.length > 0, `${spec.pack} needs at least one case`);
  assert.equal(new Set(cases.map(({ id }) => id)).size, cases.length, "duplicate case ids");
  for (const item of cases) {
    for (const input of item.inputs) {
      assert(fs.statSync(path.join(dir, input.source)).isFile(), `${input.source} is missing`);
    }
  }
  const pack = {
    version: 1,
    issue: 6879,
    historicalIssue: spec.historicalIssue,
    regressionCommit: spec.regressionCommit,
    sourceRevision: spec.regressionCommit,
    checkerOptions: spec.checkerOptions,
    projectOptions: spec.projectOptions,
    diagnosticContract: contract,
    cases,
  };
  fs.writeFileSync(casesPath, json(pack));
  return { cases, packSha256: sha256(fs.readFileSync(casesPath)) };
}

function manifestEntries(spec: Spec, cases: Case[], packSha256: string, old: Entry[]): Entry[] {
  const test = `fix_history_cases::${spec.module}::${spec.test}`;
  return cases.map((item) => {
    const inputs = item.inputs.map(({ file, source }) => {
      const artifact = `${spec.pack}/${source}`;
      return {
        file,
        path: artifact,
        sha256: sha256(fs.readFileSync(path.join(fixtures, artifact))),
      };
    });
    const entry: Entry = {
      id: `typechecker/${spec.pack}/${item.id}`,
      state: "active",
      targets: ["batch"],
      adapters: { legacy: "batch-typechecker-fixture-v1", native: null },
      pack: spec.pack,
      case: item.id,
      test,
      regressionCommit: spec.regressionCommit,
      sourceRevision: spec.regressionCommit,
      checkerOptions: spec.checkerOptions,
      projectOptions: spec.projectOptions,
      packReference: { path: `${spec.pack}/cases.json`, sha256: packSha256 },
      inputs,
      dialectCoverage: typecheckerDialectCoverage(fixtures, inputs),
    };
    if (old.find(({ id }) => id === entry.id)?.diagnostics !== undefined) {
      entry.diagnostics = item.diagnostics;
    }
    return entry;
  });
}

function updateManifest(spec: Spec, cases: Case[], packSha256: string) {
  const manifest = JSON.parse(read(manifestPath)) as { cases: Entry[] };
  const old = manifest.cases.filter((entry) => entry.pack === spec.pack);
  const rest = manifest.cases.filter((entry) => entry.pack !== spec.pack);
  // The manifest sorts by pack, or by id for corpus inputs without a pack
  // (assertSortedCases); insert at the same key.
  let at = rest.findIndex((entry) => (entry.pack ?? entry.id) > spec.pack);
  if (at < 0) at = rest.length;
  rest.splice(at, 0, ...manifestEntries(spec, cases, packSha256, old));
  manifest.cases = rest;
  fs.writeFileSync(manifestPath, json(manifest));
}

function updateRust(spec: Spec, cases: Case[]) {
  const file = path.join(casesDir, `${spec.module}.rs`);
  const issue = spec.historicalIssue === null ? "None" : `Some(${spec.historicalIssue})`;
  const ids = cases.map(({ id }) => `"${id}"`).join(", ");
  fs.writeFileSync(
    file,
    `use crate::support::fix_history_runner::check_pack;\n\n#[test]\nfn ${spec.test}() {\n    check_pack(\n        "${spec.pack}",\n        "fix_history_cases::${spec.module}::${spec.test}",\n        "${spec.regressionCommit}",\n        ${issue},\n        &[${ids}],\n    );\n}\n`,
  );
  const modPath = path.join(casesDir, "mod.rs");
  const lines = new Set(read(modPath).split("\n").filter(Boolean));
  lines.add(`pub(super) mod ${spec.module};`);
  fs.writeFileSync(modPath, `${[...lines].sort().join("\n")}\n`);
}

function updateLedger(spec: Spec) {
  const rows = read(ledgerPath).split("\n");
  const index = rows.findIndex((row) => row.split("\t")[0] === spec.regressionCommit);
  assert(index > 0, `${spec.regressionCommit} is not a ledger row`);
  const columns = rows[index]!.split("\t");
  columns[3] = admitted;
  rows[index] = columns.join("\t");
  fs.writeFileSync(ledgerPath, rows.join("\n"));
}

const args = process.argv.slice(2);
const captureAt = args.indexOf("--capture");
const capture = captureAt >= 0 ? args.splice(captureAt, 2)[1] : undefined;
assert(args.length > 0, "name at least one SPEC.json or PACK");
for (const argument of args) {
  const spec = loadSpec(argument);
  const { cases, packSha256 } = writePack(spec, capture);
  updateManifest(spec, cases, packSha256);
  register(spec.pack, {
    test: `fix_history_cases::${spec.module}::${spec.test}`,
    sourceRevision: spec.regressionCommit,
    checkerOptions: spec.checkerOptions,
    projectOptions: spec.projectOptions,
  });
  updateRust(spec, cases);
  updateLedger(spec);
  console.log(`${spec.pack}: ${cases.length} cases`);
}
