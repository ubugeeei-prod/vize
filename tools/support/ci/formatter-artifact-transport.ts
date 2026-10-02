import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

// Read-only transport of an actual existing Actions artifact. Never execute a
// product, construct observations, change baselines or grant semantic credit.
const root = path.resolve(process.argv[2]);
const sourceHead = "5a56fded1621006fc7ac65d522351daebb0f9085";
const number = (name: string) => {
  const value = Number(process.env[name]);
  assert(Number.isSafeInteger(value) && value > 0, `invalid ${name}`);
  return value;
};
const runId = number("FORMATTER_SOURCE_RUN");
const attempt = number("FORMATTER_SOURCE_ATTEMPT");
const jobId = number("FORMATTER_SOURCE_JOB");
const artifactId = number("FORMATTER_ARTIFACT_ID");
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const api = async (endpoint: string) => {
  const response = await fetch(`https://api.github.com/repos/ubugeeei-prod/vize/${endpoint}`, {
    headers: {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    redirect: "error",
  });
  assert.equal(response.status, 200, `metadata request failed: ${response.status}`);
  return await response.json();
};
const [run, job, artifact]: any[] = await Promise.all([
  api(`actions/runs/${runId}/attempts/${attempt}`),
  api(`actions/jobs/${jobId}`),
  api(`actions/artifacts/${artifactId}`),
]);
assert.equal(run.id, runId);
assert.equal(run.run_attempt, attempt);
assert.equal(run.repository.full_name, "ubugeeei-prod/vize");
assert.equal(run.head_sha, sourceHead);
assert.equal(run.head_branch, "ci/formatter-original-controls-78cf-20261003");
assert.equal(run.event, "workflow_dispatch");
assert.equal(run.path, ".github/workflows/check.yml");
assert.equal(run.status, "completed", "whole source workflow must be terminal before transport");
assert.equal(job.id, jobId);
assert.equal(job.run_id, runId);
assert.equal(job.run_attempt, attempt);
assert.equal(job.head_sha, sourceHead);
assert.equal(job.name, "formatter-history");
assert.equal(job.status, "completed");
assert.equal(artifact.id, artifactId);
assert.equal(artifact.workflow_run.id, runId);
assert.equal(artifact.workflow_run.head_sha, sourceHead);
assert.equal(artifact.expired, false);
assert.equal(
  artifact.name,
  `formatter-api-corpus-${runId}-${attempt}-formatter-history-formatter-original-controls`,
);
const names = [
  "script",
  "prepared",
  "literal",
  "literal-extra",
  "vue-version",
  "capture",
  "capture-extra",
  "capture-final",
];
const reportNames = [
  "build-receipt.json",
  "cargo.jsonl",
  "cargo.stderr.txt",
  "cli-history-report.json",
  ...names.map((name) => (name === "script" ? "report" : name + "-report") + ".json"),
];
const campaignPrefix = "differential/formatter-campaign/";
const expected = [
  ...[
    "plan.json",
    "authority.json",
    "processes.json",
    "campaign-receipt.json",
    "evidence-files.json",
    "cli-artifact.json",
    "rust-binaries.json",
    "rust-law-results.json",
    "rust-law-source-proofs.json",
    "control-source-proofs.json",
    "engineering-control-results.json",
    "glyph-module-source.json",
    "benchmark-binary.json",
  ].map((name) => campaignPrefix + name),
  ...[1, 2].flatMap((repeat) => [
    campaignPrefix + `repeat-${repeat}/cli-history-report.json`,
    ...reportNames
      .filter((name) => name !== "cli-history-report.json")
      .map((name) => campaignPrefix + `repeat-${repeat}/formatter-api/` + name),
  ]),
  "ci/vize.differential-build.json",
  "differential/formatter.json",
];
const selected = new Set(expected.filter((name) => fs.existsSync(path.join(root, name))));
const owned = (name: string) =>
  name.startsWith(campaignPrefix) ||
  name.startsWith("differential/formatter-campaign-admission-failure/") ||
  name.startsWith("differential/formatter-api/") ||
  ["ci/vize.differential-build.json", "differential/formatter.json"].includes(name);
const frozenBinary = (name: string) =>
  /^differential\/formatter-campaign\/(?:vize|benchmark-formatter|rust-binary-\d+-[^/]+)$/.test(
    name,
  ) || /(?:^|\/)formatter_observe(?:\.exe)?$/.test(name);
const binaryHashes: any[] = [];
const safePath = (relative: string) => {
  assert(!path.isAbsolute(relative) && !relative.includes("\\") && !relative.includes("\0"));
  assert(relative.split("/").every((part) => part && part !== "." && part !== ".."));
  return path.join(root, relative);
};
const inventory: { path: string; bytes: number; sha256: string; selected: boolean }[] = [];
const walk = (relative: string) => {
  const directory = relative ? safePath(relative) : root;
  if (!fs.existsSync(directory)) return;
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const name = relative ? `${relative}/${entry.name}` : entry.name;
    assert(!entry.isSymbolicLink(), "artifact symlinks are not transported");
    if (entry.isDirectory()) walk(name);
    else {
      assert(entry.isFile());
      const bytes = fs.readFileSync(safePath(name));
      if (owned(name) && !frozenBinary(name)) selected.add(name);
      if (owned(name) && frozenBinary(name)) {
        selected.delete(name);
        binaryHashes.push({
          path: name,
          bytes: bytes.length,
          sha256: digest(bytes),
          transported: false,
        });
      }
      inventory.push({
        path: name,
        bytes: bytes.length,
        sha256: digest(bytes),
        selected: selected.has(name),
      });
      assert(inventory.length <= 8192, "complete artifact inventory bound");
    }
  }
};
walk("");
assert(selected.size > 0 && selected.size <= 4096, "transport file count bound");
const files = [...selected].sort().map((relative) => {
  const file = safePath(relative);
  assert(fs.lstatSync(file).isFile());
  const bytes = fs.readFileSync(file);
  assert(bytes.length <= 16 * 1024 * 1024, "transport single-file bound");
  return {
    path: relative,
    bytes: bytes.length,
    sha256: digest(bytes),
    base64: bytes.toString("base64"),
  };
});
const bindings: any[] = [];
const bindingErrors: string[] = [];
for (const file of files.filter((file) => file.path.endsWith(".json"))) {
  try {
    const value = JSON.parse(Buffer.from(file.base64, "base64").toString());
    const revision = value.sourceRevision ?? value.source?.sourceRevision;
    if (revision !== undefined) assert.equal(revision, sourceHead, `source mismatch: ${file.path}`);
    bindings.push({ path: file.path, schema: value.schema, sourceRevision: revision ?? null });
  } catch (error) {
    bindingErrors.push(`${file.path}: ${String(error)}`);
  }
}
const binaryBindings: any[] = [];
const bindBinary = (name: string, sha256: string, bytes?: number) => {
  const observed = binaryHashes.find((row) => row.path === name);
  assert(observed, "receipt references missing frozen binary: " + name);
  assert.equal(observed.sha256, sha256, "frozen binary hash disagrees: " + name);
  if (bytes !== undefined) assert.equal(observed.bytes, bytes);
  binaryBindings.push({ path: name, sha256, bytes: observed.bytes, transported: false });
};
for (const file of files.filter((file) => file.path.endsWith(".json"))) {
  try {
    const value = JSON.parse(Buffer.from(file.base64, "base64").toString());
    if (/\/formatter-api\/build-receipt.json$/.test(file.path)) {
      bindBinary(path.posix.dirname(file.path) + "/formatter_observe", value.artifact.sha256);
    }
    if (file.path === campaignPrefix + "rust-binaries.json") {
      for (const row of value)
        bindBinary(campaignPrefix + row.frozenPath, row.binarySha256, row.binaryBytes);
    }
    if (file.path === campaignPrefix + "benchmark-binary.json") {
      bindBinary(campaignPrefix + value.frozenPath, value.sha256, value.bytes);
    }
    if (file.path === "ci/vize.differential-build.json") {
      bindBinary(campaignPrefix + "vize", value.binarySha256);
    }
  } catch (error) {
    bindingErrors.push(`binary receipt ${file.path}: ${String(error)}`);
  }
}
const payload = Buffer.from(
  JSON.stringify({
    schema: "vize.formatter.actual-artifact.transport",
    version: 1,
    source: {
      runId,
      attempt,
      jobId,
      artifactId,
      sourceHead,
      artifactName: artifact.name,
      authenticatedArtifactDigest: artifact.digest,
      sourceRunStatus: run.status,
      sourceRunConclusion: run.conclusion,
      sourceJobConclusion: job.conclusion,
    },
    metadata: { run, job, artifact },
    selection:
      "Complete actual5a56 campaign source/control witnesses, repeated API/CLI rows and all raw process streams/receipts, with full artifact path/byte/SHA256 inventory. Original PNG/source objects are lossless byte transport. Frozen CLI/observer/Rust-law/benchmark binaries are independently rehashed and receipt-bound without executing or transporting binaries. Missing expected files and receipt binding errors remain explicit; no observations or absent data are reconstructed.",
    files,
    inventory: inventory.sort((a, b) => a.path.localeCompare(b.path)),
    binaryHashes,
    binaryBindings,
    bindings,
    bindingErrors,
    missingExpectedFiles: expected.filter((name) => !selected.has(name)),
    claims: {
      productExecutedByTransport: false,
      semanticVerificationPerformed: false,
      wholeFixesClosed: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
      currentConsumerAcceptance: false,
      literalZipDigestRecomputed: false,
    },
  }),
);
assert(payload.length <= 64 * 1024 * 1024, "transport JSON bound");
const gzip = gzipSync(payload, { level: 9 });
assert(gzip.length <= 8 * 1024 * 1024, "transport gzip bound; split explicitly, never truncate");
const chunks = gzip.toString("base64").match(/.{1,12000}/g) ?? [];
console.log(
  `VIZE_FORMATTER_TRANSPORT_BEGIN ${JSON.stringify({ version: 1, sourceHead, runId, attempt, jobId, artifactId, chunks: chunks.length, files: files.length, jsonBytes: payload.length, jsonSha256: digest(payload), gzipBytes: gzip.length, gzipSha256: digest(gzip) })}`,
);
for (const [index, chunk] of chunks.entries())
  console.log(`VIZE_FORMATTER_TRANSPORT_CHUNK ${index} ${chunk}`);
console.log("VIZE_FORMATTER_TRANSPORT_END");
