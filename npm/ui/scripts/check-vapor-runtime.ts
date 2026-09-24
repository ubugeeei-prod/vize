/**
 * Vapor runtime conformance for every `@vizejs/ui` runtime fixture.
 *
 * `check-renderers.ts` proves every SFC *compiles* through the Vapor lane;
 * this check executes the output. For every runtime conformance fixture
 * (`src/conformance/runtime-conformance*.ts`, which cover every shipped SFC):
 *
 * 1. `ssr` — a child process renders the fixture with Vue 3.5's server
 *    renderer, compiling each `.vue` through the native VDOM SSR lane, with no
 *    browser globals (a real Node server);
 * 2. `mount` — a child process running Vue 3.6 (`vue-vapor-runtime`)
 *    compiles each `.vue` through the native Vapor lane and client-mounts
 *    the fixture (VDOM root + `vaporInteropPlugin`) in happy-dom;
 * 3. `hydrate` — the same Vapor build hydrates the SSR markup.
 *
 * Any Vue warning, thrown error, replaced root, or failing
 * `assertHydratedDom` fails a fixture. Coverage is catalog-driven: every
 * family in `uiFamilyCatalog` that ships an SFC must be exercised by at least
 * one fixture.
 *
 * Known failures are tracked in `scripts/vapor-runtime/known-failures.json`
 * as a ratchet: a new failure fails the check, and so does a listed fixture
 * that now passes (remove it from the ledger). Requires a native binding.
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { uiFamilyCatalog } from "../src/catalog/family-catalog.ts";

type Lane = "ssr" | "mount" | "hydrate";

interface LaneRecord {
  readonly name: string;
  readonly problems: readonly string[];
}

interface KnownFailures {
  readonly [lane: string]: { readonly [fixture: string]: string };
}

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const laneDirectory = path.join(packageRoot, "scripts/vapor-runtime");
const ledgerFile = path.join(laneDirectory, "known-failures.json");
const updateLedger = process.argv.includes("--update-known-failures");
const workDirectory = mkdtempSync(path.join(tmpdir(), "vize-ui-vapor-"));
const failures: string[] = [];

function runLane(script: string, args: readonly string[], output: string): unknown {
  const child = spawnSync(process.execPath, [path.join(laneDirectory, script), ...args, output], {
    cwd: packageRoot,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
  if (child.status !== 0) {
    throw new Error(
      `${script} ${args.join(" ")} crashed (${String(child.status)}):\n${child.stderr}`,
    );
  }
  return JSON.parse(readFileSync(output, "utf8"));
}

interface SsrOutput {
  readonly records: readonly {
    readonly name: string;
    readonly sourceFile: string;
    readonly html: string | null;
    readonly error: string | null;
  }[];
  readonly compileFailures: readonly string[];
}

interface VaporOutput {
  readonly results: readonly LaneRecord[];
  readonly compileFailures: readonly string[];
}

// 1. Server rendering (VDOM SSR lane, no browser globals).
const ssr = runLane("ssr-lane.ts", [], path.join(workDirectory, "ssr-output.json")) as SsrOutput;
const ssrFile = path.join(workDirectory, "ssr.json");
writeFileSync(ssrFile, JSON.stringify(ssr.records));
for (const failure of ssr.compileFailures) failures.push(`[compile:ssr] ${failure}`);

// 2. Catalog coverage: every family that ships an SFC has a fixture.
const fixtureSources = new Set(ssr.records.map((record) => `src/${record.sourceFile}`));
for (const family of uiFamilyCatalog) {
  const sfcs = family.sourceFiles.filter((file) => file.endsWith(".vue"));
  if (sfcs.length > 0 && !sfcs.some((file) => fixtureSources.has(file))) {
    failures.push(`[coverage] family "${family.canonicalName}" has no runtime fixture`);
  }
}

// 3. Vapor mount and hydration (Vue 3.6, native Vapor lane).
const lanes: Record<Lane, readonly LaneRecord[]> = {
  ssr: ssr.records.map((record) => ({
    name: record.name,
    problems: record.error === null ? [] : [record.error],
  })),
  mount: [],
  hydrate: [],
};
for (const lane of ["mount", "hydrate"] as const) {
  const output = runLane(
    "vapor-lane.ts",
    [ssrFile, lane],
    path.join(workDirectory, `${lane}-output.json`),
  ) as VaporOutput;
  lanes[lane] = output.results;
  for (const failure of output.compileFailures) failures.push(`[compile:vapor] ${failure}`);
}
rmSync(workDirectory, { recursive: true, force: true });

// 4. Ratchet against the known-failure ledger.
const ledger = JSON.parse(readFileSync(ledgerFile, "utf8")) as KnownFailures;
const nextLedger: Record<string, Record<string, string>> = {};
const summary: Record<string, { passed: number; knownFailures: number }> = {};
for (const [lane, records] of Object.entries(lanes)) {
  const known = ledger[lane] ?? {};
  const next: Record<string, string> = {};
  let passed = 0;
  for (const record of records) {
    const failed = record.problems.length > 0;
    if (failed) next[record.name] = known[record.name] ?? classify(record.problems);
    else passed += 1;
    if (failed && !(record.name in known)) {
      failures.push(`[${lane}] ${record.name} fails:\n  ${record.problems.join("\n  ")}`);
    } else if (!failed && record.name in known) {
      failures.push(
        `[${lane}] ${record.name} now passes — remove it from scripts/vapor-runtime/known-failures.json`,
      );
    }
  }
  nextLedger[lane] = Object.fromEntries(
    Object.entries(next).sort(([left], [right]) => left.localeCompare(right)),
  );
  summary[lane] = { passed, knownFailures: Object.keys(next).length };
}

/**
 * Root-cause bucket for a new ledger entry. Thrown errors are classified
 * before the warnings they cascade into (a setup that throws leaves an empty
 * render context behind, which then warns on every property access).
 */
function classify(problems: readonly string[]): string {
  const errors = problems.filter((problem) =>
    /^(setup function|render function|component update|uncaught|unhandled rejection|[A-Za-z]*Error)/.test(
      problem,
    ),
  );
  const text = (errors.length > 0 ? errors : problems).join("\n");
  if (/insertBefore/.test(text)) {
    return "vapor-codegen: placeholder/anchor insertion model differs from the Vue 3.6 runtime";
  }
  if (/VIZE_UI_CONTEXT_MISSING/.test(text)) {
    return "vapor-codegen: provide/inject context lost after an earlier render failure";
  }
  if (/Failed setting prop/.test(text))
    return "vapor-codegen: setProp used for an attribute-only key";
  if (/[Hh]ydration|hydration cursor|current hydration node/.test(text)) {
    return "vapor-hydration: codegen does not use the Vue 3.6 insertion-state hydration model";
  }
  if (/accessed during render but is not defined on instance/.test(text)) {
    return "vapor-codegen: setup state not exposed to the render context";
  }
  const first = (errors[0] ?? problems[0] ?? "").split("\n")[0] ?? "";
  return `unclassified: ${first.slice(0, 160)}`;
}

if (updateLedger) {
  writeFileSync(ledgerFile, `${JSON.stringify(nextLedger, null, 2)}\n`);
}

console.log(
  JSON.stringify({
    check: "@vizejs/ui Vapor runtime conformance",
    fixtures: lanes.ssr.length,
    familiesWithSfcs: uiFamilyCatalog.filter((family) =>
      family.sourceFiles.some((file) => file.endsWith(".vue")),
    ).length,
    lanes: summary,
    failures: updateLedger
      ? failures.filter((failure) => !failure.startsWith("[")).length
      : failures.length,
  }),
);
const blocking = updateLedger
  ? failures.filter((failure) => failure.startsWith("[coverage]") || failure.startsWith("[compile"))
  : failures;
if (blocking.length > 0) {
  console.error(blocking.join("\n\n"));
  process.exit(1);
}
process.exit(0);
