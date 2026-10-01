import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

// Read-only transport of bytes from an existing actual source-built Actions
// capture. It does not launch a server, generate a response or grant acceptance.
const root = path.resolve(process.argv[2]);
const runId = Number(process.env.LSP_SOURCE_RUN);
const artifactId = Number(process.env.LSP_ARTIFACT_ID);
const sourceHead = process.env.LSP_SOURCE_HEAD;
assert(Number.isSafeInteger(runId) && runId > 0);
assert(Number.isSafeInteger(artifactId) && artifactId > 0);
assert.match(sourceHead ?? "", /^[a-f0-9]{40}$/);
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const json = (relative: string) => JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
const headers = {
  Authorization: `Bearer ${process.env.GH_TOKEN}`,
  Accept: "application/vnd.github+json",
  "X-GitHub-Api-Version": "2022-11-28",
};
const api = async (endpoint: string) => {
  const result = await fetch(`https://api.github.com/repos/ubugeeei-prod/vize/${endpoint}`, {
    headers,
    redirect: "error",
  });
  assert.equal(result.status, 200, `metadata request failed: ${result.status}`);
  return await result.json();
};
const run: any = await api(`actions/runs/${runId}`);
const artifact: any = await api(`actions/artifacts/${artifactId}`);
assert.equal(run.head_sha, sourceHead);
assert.equal(run.event, "workflow_dispatch");
assert.equal(run.path, ".github/workflows/check.yml");
assert.equal(artifact.workflow_run.id, runId);
assert.equal(artifact.workflow_run.head_sha, sourceHead);
assert.equal(artifact.expired, false);
assert.equal(artifact.name, `formatter-api-corpus-${runId}-${run.run_attempt}-test-scripts-full`);
const receipt = json("ci/vize.differential-build.json");
const report = json("differential/lsp.json");
assert.equal(receipt.sourceRevision, sourceHead);
assert.equal(report.sourceRevision, sourceHead);
assert.deepEqual(report.buildReceipt, receipt);
assert.equal(report.summary.plannedCases, 7);
assert.equal(report.summary.legacyMatches, 7);
assert.equal(report.summary.legacyFailures, 0);
assert.equal(report.summary.baselineDrift, 0);
assert.equal(report.summary.nativeHandled, 0);
assert.equal(report.summary.nativeEquivalent, 0);
const selected = ["ci/vize.differential-build.json", "differential/lsp.json"];
const sessionRoot = path.join(root, "differential/lsp-sessions");
const passive: any[] = [];
if (fs.existsSync(sessionRoot)) {
  for (const entry of fs
    .readdirSync(sessionRoot, { withFileTypes: true })
    .sort((a, b) => a.name.localeCompare(b.name))) {
    assert(entry.isDirectory());
    const relative = `differential/lsp-sessions/${entry.name}`;
    const observation = json(`${relative}/observation.json`);
    if (observation.sourceOrigins.candidates.length === 0) continue;
    assert.equal(observation.sourceRevision, sourceHead);
    assert.equal(observation.state, "process-closed");
    selected.push(
      ...["observation.json", "client.bin", "server.bin", "stderr.bin"].map(
        (file) => `${relative}/${file}`,
      ),
    );
    passive.push({
      session: entry.name,
      process: observation.process,
      candidates: observation.sourceOrigins.candidates,
    });
  }
}
assert(selected.length <= 128, "transport file limit exceeded");
const files = selected.sort().map((relative) => {
  assert(!fs.lstatSync(path.join(root, relative)).isSymbolicLink());
  const bytes = fs.readFileSync(path.join(root, relative));
  assert(bytes.length <= 16 * 1024 * 1024, "transport single-file limit exceeded");
  return {
    path: relative,
    bytes: bytes.length,
    sha256: digest(bytes),
    base64: bytes.toString("base64"),
  };
});
const payload = Buffer.from(
  JSON.stringify({
    schema: "vize.lsp.actual-artifact.transport",
    version: 1,
    source: {
      runId,
      artifactId,
      sourceHead,
      name: artifact.name,
      artifactArchiveDigest: artifact.digest,
      runConclusion: run.conclusion,
    },
    selection:
      "Complete seven-case report and build receipt; all passive sessions associated with the pinned historical catalog. Other passive sessions and product artifacts remain in the original artifact.",
    claims: {
      responseProducer: "original-source-built-runtime",
      wholeFixesClosed: 0,
      nativeEquivalent: 0,
      literalZipDigestRecomputed: false,
    },
    files,
    passive,
  }),
);
assert(payload.length <= 32 * 1024 * 1024, "transport JSON limit exceeded");
const gzip = gzipSync(payload, { level: 9 });
assert(gzip.length <= 2 * 1024 * 1024, "transport gzip limit exceeded");
const encoded = gzip.toString("base64");
const chunks = encoded.match(/.{1,12000}/g) ?? [];
console.log(
  `VIZE_LSP_TRANSPORT_BEGIN ${JSON.stringify({ version: 1, sourceHead, runId, artifactId, chunks: chunks.length, files: files.length, jsonBytes: payload.length, jsonSha256: digest(payload), gzipBytes: gzip.length, gzipSha256: digest(gzip) })}`,
);
for (const [index, chunk] of chunks.entries())
  console.log(`VIZE_LSP_TRANSPORT_CHUNK ${index} ${chunk}`);
console.log("VIZE_LSP_TRANSPORT_END");
