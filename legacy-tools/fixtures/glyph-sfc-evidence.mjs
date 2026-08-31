import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const hashPattern = /^[0-9a-f]{64}$/;
const commitPattern = /^[0-9a-f]{40}$/;
const verdicts = new Set(["equivalent", "semantic-diff", "baseline-unusable"]);
const dialects = new Set(["0.10", "0.11", "1", "2", "2.7", "3"]);
const failureStages = new Set([
  "adapter-load",
  "comparison-harness",
  "sfc-parse",
  "template-compile",
  "semantic-normalize",
]);
const baselineContracts = new Map([
  [
    "unsupported-vue-0.10",
    { dialect: "0.10", package: null, version: null, normalization: "unavailable", options: {} },
  ],
  [
    "unsupported-vue-0.11",
    { dialect: "0.11", package: null, version: null, normalization: "unavailable", options: {} },
  ],
  [
    "unsupported-vue-1",
    { dialect: "1", package: null, version: null, normalization: "unavailable", options: {} },
  ],
  [
    "vue2.6",
    {
      dialect: "2",
      package: "vue-template-compiler",
      version: "2.6.14",
      normalization: "vue2-render-v1",
      options: {
        parse: { pad: false },
        compile: { comments: true, outputSourceRange: true, whitespace: "preserve" },
      },
    },
  ],
  [
    "vue2.7",
    {
      dialect: "2.7",
      package: "@vue/compiler-sfc",
      version: "2.7.16",
      normalization: "vue2-render-v1",
      options: {
        parse: { pad: false },
        compile: {
          isProduction: true,
          prettify: false,
          compilerOptions: { comments: true, outputSourceRange: true, whitespace: "preserve" },
        },
      },
    },
  ],
  [
    "vue3",
    {
      dialect: "3",
      package: "@vue/compiler-sfc",
      version: "3.6.0-beta.10",
      normalization: "vue3-template-ast-v1",
      options: { sourceMap: false },
    },
  ],
]);

export function writeGlyphSfcEquivalenceEvidence(
  input,
  reportDir = process.env.FIXTURE_REPORT_DIR,
) {
  if (reportDir == null || reportDir === "") return null;
  const artifact = createGlyphSfcEquivalenceEvidence(input);
  const output = resolve(repoRoot, reportDir, "glyph-sfc-dialect-equivalence.json");
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  return output;
}

export function createGlyphSfcEquivalenceEvidence(input) {
  const registryPath = resolve(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json");
  const baselines = dedupeBaselines([
    ...(input.availableBaselines ?? []),
    ...input.files.map((file) => file.baseline),
  ]);
  const files = input.files.map(({ baseline: _baseline, ...file }) => file);
  const artifact = {
    schema: "vize.glyphSfcEquivalenceEvidence",
    version: 1,
    sourceCommit: input.sourceCommit,
    registry: {
      path: "tests/_fixtures/vue-ecosystem-fixtures.json",
      sha256: sha256(readFileSync(registryPath)),
    },
    formatter: input.formatter,
    baselines,
    files: files.sort(compareFiles),
    summary: summarize(files, input.waiverValidationError),
  };
  const result = { ...artifact, sha256: sha256(canonicalJson(artifact)) };
  validateGlyphSfcEquivalenceEvidence(result, input.expectedFiles);
  return result;
}

export function validateGlyphSfcEquivalenceEvidence(artifact, expectedFiles = null) {
  if (artifact?.schema !== "vize.glyphSfcEquivalenceEvidence" || artifact.version !== 1) {
    throw new Error("invalid glyph SFC equivalence artifact identity");
  }
  if (!commitPattern.test(artifact.sourceCommit ?? "")) {
    throw new Error("glyph SFC equivalence sourceCommit must be an exact commit");
  }
  if (artifact.registry?.path !== "tests/_fixtures/vue-ecosystem-fixtures.json") {
    throw new Error("glyph SFC equivalence registry path is invalid");
  }
  requireHash(artifact.registry.sha256, "registry sha256");
  requireHash(artifact.formatter?.binarySha256, "formatter binary sha256");
  if (typeof artifact.formatter?.version !== "string" || artifact.formatter.version.length === 0) {
    throw new Error("formatter version must be non-empty");
  }
  const baselines = validateBaselines(artifact.baselines);
  const identities = new Set();
  for (const file of artifact.files ?? []) {
    const identity = `${file.project}\0${file.path}`;
    if (identities.has(identity)) throw new Error(`duplicate glyph SFC evidence: ${identity}`);
    identities.add(identity);
    const baseline = baselines.get(file.baselineId);
    if (baseline == null || baseline.dialect !== file.dialect) {
      throw new Error(`baseline/dialect mismatch for ${file.project}:${file.path}`);
    }
    if (!commitPattern.test(file.revision ?? "")) {
      throw new Error(`fixture revision must be an exact commit for ${identity}`);
    }
    if (!/^[a-z][a-z0-9-]*$/.test(file.routeId ?? "")) {
      throw new Error(`invalid routeId for ${identity}`);
    }
    for (const field of ["originalSha256", "formattedSha256"]) {
      requireHash(file[field], `${identity} ${field}`);
    }
    if (!verdicts.has(file.verdict)) throw new Error(`invalid verdict for ${identity}`);
    if (file.waiver != null)
      throw new Error(`glyph SFC dialect evidence cannot be waived: ${identity}`);
    validateVerdict(file, identity);
  }
  if (expectedFiles != null) {
    const expected = new Map(expectedFiles.map((file) => [`${file.project}\0${file.path}`, file]));
    if (expected.size !== expectedFiles.length)
      throw new Error("expected glyph SFC files are duplicate");
    for (const [identity, expectedFile] of expected) {
      if (!identities.has(identity)) {
        throw new Error(`missing glyph SFC evidence: ${JSON.stringify(identity)}`);
      }
      const actual = artifact.files.find((file) => `${file.project}\0${file.path}` === identity);
      for (const field of ["revision", "routeId", "dialect", "baselineId"]) {
        if (expectedFile[field] != null && actual[field] !== expectedFile[field]) {
          throw new Error(`glyph SFC evidence ${field} mismatch: ${JSON.stringify(identity)}`);
        }
      }
    }
    for (const identity of identities) {
      if (!expected.has(identity)) {
        throw new Error(`unexpected glyph SFC evidence: ${JSON.stringify(identity)}`);
      }
    }
  }
  const summary = summarize(artifact.files, artifact.summary.waiverValidationError);
  if (canonicalJson(summary) !== canonicalJson(artifact.summary)) {
    throw new Error("glyph SFC equivalence summary does not match files");
  }
  const { sha256: recorded, ...unsigned } = artifact;
  requireHash(recorded, "artifact sha256");
  if (recorded !== sha256(canonicalJson(unsigned))) {
    throw new Error("glyph SFC equivalence artifact digest mismatch");
  }
  return artifact;
}

export function formatterEvidence(command, version) {
  if (!existsSync(command)) throw new Error(`formatter evidence binary is missing: ${command}`);
  if (typeof version !== "string" || version.trim() === "") {
    throw new Error("formatter evidence version must be non-empty");
  }
  return { version: version.trim(), binarySha256: sha256(readFileSync(command)) };
}

export function evidenceSourceCommit(environment = process.env, runGit = spawnSync) {
  const environmentSha = environment.GITHUB_SHA;
  if (environmentSha != null) return requireCommit(environmentSha, "GITHUB_SHA");
  const result = runGit("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(`git rev-parse HEAD failed: ${result.stderr.trim()}`);
  }
  return requireCommit(result.stdout.trim(), "git rev-parse HEAD");
}

function validateBaselines(records) {
  const baselines = new Map();
  for (const baseline of records ?? []) {
    if (typeof baseline.id !== "string" || baseline.id.length === 0) {
      throw new Error("baseline id must be non-empty");
    }
    if (baselines.has(baseline.id)) throw new Error(`duplicate baseline id: ${baseline.id}`);
    const contract = baselineContracts.get(baseline.id);
    if (contract == null) throw new Error(`unknown baseline id: ${baseline.id}`);
    if (!dialects.has(baseline.dialect)) {
      throw new Error(`baseline ${baseline.id} has an invalid dialect`);
    }
    if (typeof baseline.normalization !== "string" || baseline.normalization.length === 0) {
      throw new Error(`baseline ${baseline.id} normalization must be non-empty`);
    }
    if (
      baseline.options == null ||
      typeof baseline.options !== "object" ||
      Array.isArray(baseline.options)
    ) {
      throw new Error(`baseline ${baseline.id} options must be an object`);
    }
    for (const field of ["dialect", "package", "version", "normalization"]) {
      if (baseline[field] !== contract[field]) {
        throw new Error(`baseline ${baseline.id} ${field} violates the pinned contract`);
      }
    }
    if (canonicalJson(baseline.options) !== canonicalJson(contract.options)) {
      throw new Error(`baseline ${baseline.id} options violate the pinned contract`);
    }
    if (baseline.package == null) {
      if (baseline.version != null || baseline.entrySha256 != null) {
        throw new Error(`unsupported baseline ${baseline.id} has package provenance`);
      }
    } else {
      if (typeof baseline.version !== "string" || baseline.version.length === 0) {
        throw new Error(`baseline ${baseline.id} version must be non-empty`);
      }
      requireHash(baseline.entrySha256, `${baseline.id} entry sha256`);
    }
    baselines.set(baseline.id, baseline);
  }
  for (const id of baselineContracts.keys()) {
    if (!baselines.has(id)) throw new Error(`missing baseline contract: ${id}`);
  }
  if (baselines.size !== baselineContracts.size) {
    throw new Error("glyph SFC baseline contracts are not an exact partition");
  }
  return baselines;
}

function validateVerdict(file, identity) {
  if (
    !Array.isArray(file.differences) ||
    file.differences.some((value) => typeof value !== "string")
  ) {
    throw new Error(`glyph SFC evidence differences must be strings: ${identity}`);
  }
  const differences = file.differences;
  if (file.verdict === "equivalent") {
    if (
      file.reasonCode != null ||
      file.failure != null ||
      file.waiver != null ||
      differences.length !== 0 ||
      file.beforeSemanticSha256 !== file.afterSemanticSha256
    ) {
      throw new Error(`equivalent glyph SFC evidence is inconsistent: ${identity}`);
    }
    requireHash(file.beforeSemanticSha256, `${identity} semantic sha256`);
    return;
  }
  if (differences.length === 0) {
    throw new Error(`non-equivalent glyph SFC evidence needs differences: ${identity}`);
  }
  if (file.reasonCode === "semantic-signature-changed") {
    if (file.verdict !== "semantic-diff" || file.failure != null) {
      throw new Error(`semantic signature change ownership is invalid: ${identity}`);
    }
    requireHash(file.beforeSemanticSha256, `${identity} before semantic sha256`);
    requireHash(file.afterSemanticSha256, `${identity} after semantic sha256`);
    return;
  }
  const failureContracts = new Map([
    ["original-baseline-unusable", { verdict: "baseline-unusable", side: "original" }],
    ["formatted-baseline-unusable", { verdict: "semantic-diff", side: "formatted" }],
    ["comparison-harness-unusable", { verdict: "baseline-unusable", side: "harness" }],
  ]);
  const contract = failureContracts.get(file.reasonCode);
  if (contract == null) {
    throw new Error(`non-equivalent glyph SFC evidence reason is invalid: ${identity}`);
  }
  if (file.verdict !== contract.verdict || file.failure?.side !== contract.side) {
    throw new Error(`${file.reasonCode} ownership is invalid: ${identity}`);
  }
  if (
    typeof file.failure.stage !== "string" ||
    !failureStages.has(file.failure.stage) ||
    typeof file.failure.message !== "string" ||
    file.failure.message.length === 0
  ) {
    throw new Error(`glyph SFC baseline failure is incomplete: ${identity}`);
  }
  if (
    (file.reasonCode === "comparison-harness-unusable") !==
    (file.failure.stage === "comparison-harness")
  ) {
    throw new Error(`glyph SFC baseline failure stage ownership is invalid: ${identity}`);
  }
  if (!differences.includes(`${file.failure.stage}: ${file.failure.message}`)) {
    throw new Error(`glyph SFC baseline failure detail is missing: ${identity}`);
  }
  if (file.beforeSemanticSha256 != null || file.afterSemanticSha256 != null) {
    throw new Error(`baseline failure cannot claim semantic hashes: ${identity}`);
  }
}

function dedupeBaselines(baselineInputs) {
  const records = new Map();
  for (const baseline of baselineInputs) {
    const encoded = canonicalJson(baseline);
    const existing = records.get(baseline.id);
    if (existing != null && canonicalJson(existing) !== encoded) {
      throw new Error(`conflicting baseline provenance: ${baseline.id}`);
    }
    records.set(baseline.id, baseline);
  }
  return [...records.values()].sort((left, right) => codePointCompare(left.id, right.id));
}

function summarize(files, waiverValidationError) {
  const counts = Object.fromEntries([...verdicts].map((verdict) => [verdict, 0]));
  let waivedDifferenceCount = 0;
  for (const file of files) {
    counts[file.verdict] += 1;
    if (file.waiver != null) waivedDifferenceCount += 1;
  }
  return {
    fileCount: files.length,
    verdictCounts: counts,
    waivedDifferenceCount,
    waiverValidationError,
  };
}

function compareFiles(left, right) {
  return codePointCompare(left.project, right.project) || codePointCompare(left.path, right.path);
}

function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function requireHash(value, label) {
  if (!hashPattern.test(value ?? "")) throw new Error(`${label} must be a sha256`);
}

function requireCommit(value, label) {
  if (!commitPattern.test(value ?? "")) throw new Error(`${label} must be an exact commit`);
  return value;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
