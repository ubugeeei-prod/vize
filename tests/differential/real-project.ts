// T2 of the shared differential harness (#6891): one planned row per pinned
// real project and product target, in the shared manifest/result format,
// built from the Real Project Matrix shard reports. The legacy lane is the
// shard's source-built CLI or LSP run. No product has a native-only route on
// real projects yet, so every native lane is `unsupported` and earns no
// credit; a product adapter replaces that lane when its route lands.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { renderNativeAcceptanceMarkdown, summarizeNativeAcceptance } from "./acceptance-rates.mjs";
import {
  assertSortedCases,
  loadProductManifest,
  normalizeDialectCoverage,
  validateResultEnvelope,
} from "./harness.mjs";
import { sha256 } from "./sha256.ts";

export const registryPath = "tests/_fixtures/vue-ecosystem-fixtures.json";
export const ledgerPath = "tests/_fixtures/fixture-compatibility-ledger.json";
const shardDirectory = /^real-project-matrix-(\d+)$/;
const revisionPattern = /^[a-f0-9]{40}$/;

// Target names follow docs/davinci/plan/deletion-readiness-policy.json.
// The Matrix compiles the default DOM target only, so `ssr` and `vapor`
// have no T2 rows yet.
export const realProjectProducts = [
  { product: "compiler", target: "dom", coverage: "compiler", tool: "compiler" },
  { product: "linter", target: "lint", coverage: "linter", tool: "linter" },
  { product: "formatter", target: "fmt", coverage: "formatter", tool: "formatter" },
  { product: "typechecker", target: "batch", coverage: "typechecker", tool: "typechecker" },
  { product: "lsp", target: "stdio", coverage: "lsp", tool: null },
] as const;

type Product = (typeof realProjectProducts)[number];
type Json = Record<string, any>;
interface Evidence {
  value: Json;
  sha256: string;
}
export interface Shard {
  index: number;
  directory: string;
  matrix: Evidence;
  lsp: Evidence | null;
}
interface Lane {
  state: string;
  observation?: Json;
}

function readEvidence(file: string): Evidence {
  const bytes = fs.readFileSync(file);
  return { value: JSON.parse(bytes.toString("utf8")), sha256: sha256(bytes) };
}

// One subdirectory per downloaded `real-project-matrix-<shard>` artifact.
export function readShards(artifactsDir: string): Shard[] {
  const shards: Shard[] = [];
  for (const name of fs.readdirSync(artifactsDir).sort()) {
    const match = shardDirectory.exec(name);
    if (!match) continue;
    const directory = path.join(artifactsDir, name);
    const summary = path.join(directory, "summary.json");
    if (!fs.existsSync(summary)) continue;
    const matrix = readEvidence(summary);
    assert.equal(matrix.value.schema, "vize.fixtureToolMatrixReport", `${name}: matrix schema`);
    assert.equal(matrix.value.command?.shardIndex, Number(match[1]), `${name}: shard index`);
    const lspFile = path.join(directory, "lsp-lifecycle-summary.json");
    const lsp = fs.existsSync(lspFile) ? readEvidence(lspFile) : null;
    if (lsp) assert.equal(lsp.value.schema, "vize.realProjectLspLifecycle", `${name}: LSP schema`);
    shards.push({ index: Number(match[1]), directory, matrix, lsp });
  }
  return shards;
}

function dialectCoverage(ledger: Json, fixturePath: string) {
  const entry = ledger.fixtures.find((fixture: Json) => fixture.fixturePath === fixturePath);
  const coverage = entry?.dialectCoverage;
  if (!coverage || coverage.state === "unknown") return undefined;
  const byKind = new Map<string, Set<string>>();
  for (const claim of coverage.evidence) {
    const dialects = byKind.get(claim.kind) ?? new Set<string>();
    for (const dialect of claim.dialects) dialects.add(dialect);
    byKind.set(claim.kind, dialects);
  }
  return {
    state: coverage.state,
    registryRef: `${ledgerPath}#${fixturePath}`,
    evidence: [...byKind].map(([kind, dialects]) => ({ kind, dialects: [...dialects].sort() })),
  };
}

function shardOutput(shard: Shard, outputPath: unknown): string | null {
  if (typeof outputPath !== "string") return null;
  const prefix = `real-project-results/shard-${shard.index}/`;
  assert(outputPath.startsWith(prefix), `shard ${shard.index}: output outside its report`);
  const relative = outputPath.slice(prefix.length);
  assert(relative && !relative.includes("..") && !path.isAbsolute(relative));
  const file = path.join(shard.directory, relative);
  return fs.existsSync(file) ? file : null;
}

function matrixLane(shard: Shard, run: Json | undefined): Lane {
  if (!run) return { state: "missing-evidence" };
  const output = shardOutput(shard, run.outputPath);
  const ran = run.status === "ok" || run.status === "findings";
  return {
    state: ran && output ? "completed" : ran ? "missing-evidence" : run.status,
    observation: {
      status: run.status,
      exitCode: run.exitCode,
      fileCount: run.fileCount,
      outputSha256: output ? sha256(fs.readFileSync(output)) : null,
    },
  };
}

function lspLane(project: Json | undefined): Lane {
  if (!project) return { state: "missing-evidence" };
  return {
    state: project.status === "ok" ? "completed" : project.status,
    observation: {
      status: project.status,
      vueFileCount: project.vueFileCount,
      actualFileDiagnosticsSha256: project.actualFileDiagnostics?.sha256 ?? null,
    },
  };
}

// Every reported project must be pinned and reported once, at its revision.
function indexShards(shards: Shard[], projects: Json[]) {
  const pinned = new Map(projects.map((project) => [project.id, project]));
  const matrix = new Map<string, { shard: Shard; project: Json }>();
  const lsp = new Map<string, Json>();
  const seenShards = new Set<number>();
  for (const shard of shards) {
    assert(!seenShards.has(shard.index), `duplicate shard ${shard.index}`);
    seenShards.add(shard.index);
    for (const project of shard.matrix.value.projects) {
      assert(pinned.has(project.id), `shard ${shard.index}: unpinned project ${project.id}`);
      assert(!matrix.has(project.id), `${project.id}: reported by two shards`);
      assert.equal(project.revision, pinned.get(project.id)?.revision, `${project.id}: revision`);
      matrix.set(project.id, { shard, project });
    }
    for (const project of shard.lsp?.value.projects ?? []) {
      assert.equal(matrix.get(project.id)?.shard, shard, `${project.id}: LSP outside its shard`);
      assert(!lsp.has(project.id), `${project.id}: two LSP rows`);
      lsp.set(project.id, project);
    }
  }
  return { matrix, lsp };
}

function laneFor(product: Product, id: string, index: ReturnType<typeof indexShards>): Lane {
  if (product.tool === null) return lspLane(index.lsp.get(id));
  const reported = index.matrix.get(id);
  if (!reported) return { state: "missing-evidence" };
  const run = reported.project.runs.find((candidate: Json) => candidate.tool === product.tool);
  return matrixLane(reported.shard, run);
}

export function writeRealProjectResults({
  root,
  sourceRevision,
  shards,
  outDir,
}: {
  root: string;
  sourceRevision: string;
  shards: Shard[];
  outDir: string;
}) {
  assert.match(sourceRevision, revisionPattern, "exact source revision is required");
  const registry = readEvidence(path.join(root, registryPath));
  const ledger = readEvidence(path.join(root, ledgerPath));
  const index = indexShards(shards, registry.value.projects);
  const evidence = {
    registrySha256: registry.sha256,
    ledgerSha256: ledger.sha256,
    shards: shards.map((shard) => ({
      index: shard.index,
      matrixSha256: shard.matrix.sha256,
      lspSha256: shard.lsp?.sha256 ?? null,
    })),
  };
  const outputs = [];
  for (const product of realProjectProducts) {
    const projects = registry.value.projects
      .filter((project: Json) => project.coverage.includes(product.coverage))
      .sort((a: Json, b: Json) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
    const cases = projects.map((project: Json) => {
      const fixture: Json = {
        id: `${product.product}/real/${project.id}`,
        state: "active",
        targets: [product.target],
        projectId: project.id,
      };
      const coverage = dialectCoverage(ledger.value, project.fixturePath);
      if (coverage) fixture.dialectCoverage = normalizeDialectCoverage(coverage, fixture.id);
      return fixture;
    });
    assertSortedCases(cases);
    const dir = path.join(outDir, "T2", product.product);
    fs.mkdirSync(dir, { recursive: true });
    const manifestPath = path.join(dir, "manifest.json");
    const manifest = {
      schema: "vize.differential.manifest",
      version: 1,
      product: product.product,
      baseRevision: sourceRevision,
      cases,
    };
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
    const loaded = loadProductManifest(manifestPath, product.product);
    const rows = cases.map((fixture: Json) => ({
      id: fixture.id,
      target: product.target,
      legacy: laneFor(product, fixture.projectId, index),
      native: { state: "unsupported", reason: "no native-only real-project route" },
      comparison: { state: "not-compared" },
    }));
    const legacyCompleted = rows.filter((row: Json) => row.legacy.state === "completed").length;
    const report = {
      schema: "vize.differential.result",
      version: 1,
      tier: "T2",
      product: product.product,
      manifestSha256: loaded.manifestSha256,
      sourceRevision,
      evidence,
      rows,
      summary: {
        plannedCases: cases.length,
        legacyCompleted,
        legacyIncomplete: rows.length - legacyCompleted,
      },
    };
    validateResultEnvelope(loaded, report, sourceRevision);
    const acceptance = summarizeNativeAcceptance(loaded, report, {
      sourceRevision,
      scope: "real-projects",
    });
    fs.writeFileSync(path.join(dir, "result.json"), `${JSON.stringify(report, null, 2)}\n`);
    fs.writeFileSync(path.join(dir, "acceptance.json"), `${JSON.stringify(acceptance, null, 2)}\n`);
    outputs.push({ product: product.product, target: product.target, report, acceptance });
  }
  const markdown = renderRealProjectMarkdown(outputs);
  fs.writeFileSync(path.join(outDir, "summary.md"), markdown);
  return { outputs, markdown };
}

function renderRealProjectMarkdown(outputs: Json[]) {
  const lines = [
    "### Real-project differential (T2)",
    "",
    "Product | Target | Projects | Legacy completed | Legacy incomplete",
    "--- | --- | ---: | ---: | ---:",
  ];
  for (const { product, target, report } of outputs) {
    const { plannedCases, legacyCompleted, legacyIncomplete } = report.summary;
    lines.push(
      `${product} | ${target} | ${plannedCases} | ${legacyCompleted} | ${legacyIncomplete}`,
    );
  }
  lines.push("", renderNativeAcceptanceMarkdown(outputs.map((output) => output.acceptance)));
  return lines.join("\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [artifactsDir, outDir] = process.argv.slice(2);
  assert(artifactsDir && outDir, "usage: real-project.ts SHARD_ARTIFACTS_DIR OUT_DIR");
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
  const { outputs, markdown } = writeRealProjectResults({
    root,
    sourceRevision: process.env.GITHUB_SHA ?? "",
    shards: readShards(path.resolve(artifactsDir)),
    outDir: path.resolve(outDir),
  });
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  for (const { product, report } of outputs) {
    const { plannedCases, legacyCompleted } = report.summary;
    process.stdout.write(`${product}: ${legacyCompleted}/${plannedCases} legacy rows completed\n`);
  }
}
