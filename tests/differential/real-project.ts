// T2 of the shared differential harness (#6891): one planned row per pinned
// real project and product target, in the shared manifest/result format,
// built from the Real Project Matrix shard reports. The legacy lane is the
// shard's source-built CLI or LSP run. No product has a native-only route on
// real projects yet: the compiler DOM and SSR lanes read the canonical corpora
// (legacy-backed), every other native lane is `unsupported`, and none earns
// credit. A product adapter replaces its lane when its route lands.
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

// Target names follow the deletion-readiness policy targets. Compiler
// `vapor` has no real-project lane yet, so it has no T2 rows.
export const realProjectProducts = [
  { product: "compiler", targets: ["dom", "ssr"], coverage: "compiler" },
  { product: "linter", targets: ["lint"], coverage: "linter" },
  { product: "formatter", targets: ["fmt"], coverage: "formatter" },
  { product: "typechecker", targets: ["batch"], coverage: "typechecker" },
  { product: "lsp", targets: ["stdio"], coverage: "lsp" },
] as const;

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
// A canonical corpus run: every pinned gitlink's files, and the legacy-vs-level
// divergence total over all of them.
export interface Corpus {
  files: Evidence;
  reportSha256: string;
  canonical: boolean;
  divergences: number | null;
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

// The downloaded canonical DOM corpus evidence, when the corpus job uploaded it.
export function readDomCorpus(dir: string | undefined): Corpus | null {
  if (!dir) return null;
  const summary = path.join(dir, "summary.json");
  const files = path.join(dir, "files.json");
  if (!fs.existsSync(summary) || !fs.existsSync(files)) return null;
  const { value, sha256: reportSha256 } = readEvidence(summary);
  const divergences = value.evidence?.divergences;
  return {
    files: readEvidence(files),
    reportSha256,
    canonical: value.evidence?.canonicalScope === true,
    divergences: Number.isInteger(divergences) ? divergences : null,
  };
}

// The SSR observer logs one template sweep and one production SFC sweep per
// process shard; every line must cover the canonical scope.
const ssrSweep = /SSR (corpus|production) sweep: scope=(\S+) .*\bdivergences=(\d+)/;

export function readSsrCorpus(dir: string | undefined): Corpus | null {
  if (!dir) return null;
  const log = path.join(dir, "ssr-pug.log");
  const files = path.join(dir, "files.json");
  const observer = path.join(dir, "observer.json");
  if (![log, files, observer].every((file) => fs.existsSync(file))) return null;
  const bytes = fs.readFileSync(log);
  const sweeps = bytes
    .toString("utf8")
    .split("\n")
    .map((line) => ssrSweep.exec(line))
    .filter((match) => match !== null);
  const kinds = new Set(sweeps.map((match) => match[1]));
  return {
    files: readEvidence(files),
    reportSha256: sha256(bytes),
    canonical:
      readEvidence(observer).value.outcome === "success" &&
      kinds.size === 2 &&
      sweeps.every((match) => match[2] === "canonical"),
    divergences: sweeps.reduce((total, match) => total + Number(match[3]), 0),
  };
}

// A corpus compares legacy and level output over every pinned gitlink. Its
// template stages run on the levels, but the SFC script still compiles
// through Croquis (and SSR may select a legacy route), so the lane is
// legacy-backed. Zero divergences make every gitlink's compared templates
// equal; otherwise the divergences are not attributed to projects.
function corpusLanes(
  fixturePath: string,
  corpus: Corpus | null,
  contract: string,
): { legacy: Lane; native: Lane; comparison: Json } {
  if (!corpus)
    return {
      legacy: { state: "missing-evidence" },
      native: { state: "missing-evidence" },
      comparison: { state: "not-compared" },
    };
  const gitlink = path.posix.basename(fixturePath);
  const files = corpus.files.value.filter(([file]: [string]) => file.startsWith(`${gitlink}/`));
  const observation = {
    gitlink,
    files: files.length,
    divergences: corpus.divergences,
    reportSha256: corpus.reportSha256,
    filesSha256: corpus.files.sha256,
  };
  const ran = corpus.canonical && corpus.divergences !== null && files.length > 0;
  return {
    legacy: { state: ran ? "completed" : "unverified", observation },
    native: { state: ran ? "legacy-backed" : "unverified", observation },
    comparison: !ran
      ? { state: "not-compared" }
      : corpus.divergences === 0
        ? { state: "equal", contract }
        : { state: "not-attributed" },
  };
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

function matrixRun(id: string, tool: string, index: ReturnType<typeof indexShards>): Lane {
  const reported = index.matrix.get(id);
  if (!reported) return { state: "missing-evidence" };
  const run = reported.project.runs.find((candidate: Json) => candidate.tool === tool);
  return matrixLane(reported.shard, run);
}

const unsupported = {
  native: { state: "unsupported", reason: "no native-only real-project route" },
  comparison: { state: "not-compared" },
};

// The compiler DOM legacy lane is the Matrix build, compared against the DOM
// corpus; the SSR lanes both come from the SSR corpus. The Matrix runs one
// tool per product, and the LSP lifecycle covers `stdio`.
function lanesFor(
  product: string,
  target: string,
  project: Json,
  index: ReturnType<typeof indexShards>,
  corpora: { dom: Corpus | null; ssr: Corpus | null },
) {
  if (target === "dom") {
    const { native, comparison } = corpusLanes(
      project.fixturePath,
      corpora.dom,
      "dom-template-bytes",
    );
    return { legacy: matrixRun(project.id, product, index), native, comparison };
  }
  if (target === "ssr") return corpusLanes(project.fixturePath, corpora.ssr, "ssr-sfc-bytes");
  if (target === "stdio") return { legacy: lspLane(index.lsp.get(project.id)), ...unsupported };
  return { legacy: matrixRun(project.id, product, index), ...unsupported };
}

export function writeRealProjectResults({
  root,
  sourceRevision,
  shards,
  outDir,
  domCorpus = null,
  ssrCorpus = null,
}: {
  root: string;
  sourceRevision: string;
  shards: Shard[];
  outDir: string;
  domCorpus?: Corpus | null;
  ssrCorpus?: Corpus | null;
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
    corpora: Object.fromEntries(
      Object.entries({ dom: domCorpus, ssr: ssrCorpus }).map(([name, corpus]) => [
        name,
        corpus && { reportSha256: corpus.reportSha256, filesSha256: corpus.files.sha256 },
      ]),
    ),
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
        targets: [...product.targets],
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
    const rows = projects.flatMap((project: Json, at: number) =>
      product.targets.map((target) => ({
        id: cases[at].id,
        target,
        ...lanesFor(product.product, target, project, index, { dom: domCorpus, ssr: ssrCorpus }),
      })),
    );
    const targets = Object.fromEntries(
      product.targets.map((target) => {
        const own = rows.filter((row: Json) => row.target === target);
        const legacyCompleted = own.filter((row: Json) => row.legacy.state === "completed").length;
        return [
          target,
          {
            legacyCompleted,
            legacyIncomplete: own.length - legacyCompleted,
            comparedEqual: own.filter((row: Json) => row.comparison.state === "equal").length,
          },
        ];
      }),
    );
    const report = {
      schema: "vize.differential.result",
      version: 1,
      tier: "T2",
      product: product.product,
      manifestSha256: loaded.manifestSha256,
      sourceRevision,
      evidence,
      rows,
      summary: { plannedCases: cases.length, targets },
    };
    validateResultEnvelope(loaded, report, sourceRevision);
    const acceptance = summarizeNativeAcceptance(loaded, report, {
      sourceRevision,
      scope: "real-projects",
    });
    fs.writeFileSync(path.join(dir, "result.json"), `${JSON.stringify(report, null, 2)}\n`);
    fs.writeFileSync(path.join(dir, "acceptance.json"), `${JSON.stringify(acceptance, null, 2)}\n`);
    outputs.push({ product: product.product, report, acceptance });
  }
  const markdown = renderRealProjectMarkdown(outputs);
  fs.writeFileSync(path.join(outDir, "summary.md"), markdown);
  return { outputs, markdown };
}

function renderRealProjectMarkdown(outputs: Json[]) {
  const lines = [
    "### Real-project differential (T2)",
    "",
    "Product | Target | Projects | Legacy completed | Legacy incomplete | Compared equal",
    "--- | --- | ---: | ---: | ---: | ---:",
  ];
  for (const { product, report } of outputs) {
    for (const [target, counts] of Object.entries<Json>(report.summary.targets)) {
      const { legacyCompleted, legacyIncomplete, comparedEqual } = counts;
      lines.push(
        `${product} | ${target} | ${report.summary.plannedCases} | ${legacyCompleted} | ${legacyIncomplete} | ${comparedEqual}`,
      );
    }
  }
  lines.push("", renderNativeAcceptanceMarkdown(outputs.map((output) => output.acceptance)));
  return lines.join("\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [artifactsDir, outDir, domCorpusDir, ssrCorpusDir] = process.argv.slice(2);
  assert(
    artifactsDir && outDir,
    "usage: real-project.ts SHARD_ARTIFACTS_DIR OUT_DIR [DOM_CORPUS_DIR] [SSR_CORPUS_DIR]",
  );
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
  const { outputs, markdown } = writeRealProjectResults({
    root,
    sourceRevision: process.env.SOURCE_REVISION ?? "",
    shards: readShards(path.resolve(artifactsDir)),
    outDir: path.resolve(outDir),
    domCorpus: readDomCorpus(domCorpusDir && path.resolve(domCorpusDir)),
    ssrCorpus: readSsrCorpus(ssrCorpusDir && path.resolve(ssrCorpusDir)),
  });
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  for (const { product, report } of outputs) {
    for (const [target, counts] of Object.entries<Json>(report.summary.targets)) {
      process.stdout.write(
        `${product}/${target}: ${counts.legacyCompleted}/${report.summary.plannedCases} legacy rows completed\n`,
      );
    }
  }
}
