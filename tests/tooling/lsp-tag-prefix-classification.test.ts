import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { binaryRelativePath } from "../differential/build-receipt.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { resolveFreshVuePackage } from "./support/lsp/fresh-vue-project.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { JsonRpcMessage } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  fixtureDirectory,
  inputHashes,
  nativeCaptures,
  prefixParams,
  preparePrefixWorkspace,
  profiles,
  sha256,
  type NativeCapture,
  type PrefixProfile,
  type PrefixQuery,
} from "./support/tag-prefix-classification-10622.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

type BaselineRow = {
  name: string;
  position: { line: number; character: number };
  packet: JsonRpcMessage;
  nativeRequests: Array<{ positions: number[]; sourceSha256: string; vueModulePosition: number }>;
};
type Baseline = {
  schema: string;
  version: number;
  authority: string;
  releaseVersion: string;
  inputs: Record<string, string>;
  evidence: { path: string; sha256: string };
  profiles: Array<{ profile: string; sourceSha256: string; rows: BaselineRow[] }>;
};
const baselineBytes = readPinnedArtifact(fixtureDirectory, {
  path: "public-0.567.0.json",
  sha256: "cd4e0f3c8a1b6bf171fdcd4cf821843e0599a64385c2bc89d95ac09b98288077",
});
const baseline = JSON.parse(baselineBytes.toString("utf8")) as Baseline;
const output = path.join(testOutputRoot, "tag-prefix-classification-10622");

function writeJson(file: string, value: unknown) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}

type NativeRequest = {
  uri: string;
  source: string;
  positions: number[];
  vueModulePosition: number;
};
function assertNativeRequests(captures: NativeCapture[], query: PrefixQuery, hasVue: boolean) {
  const records = captures.flatMap((capture) => capture.records);
  const requests = records.filter((record) => record.stage === "request");
  if (!query.requestedBindings.length) {
    assert.deepEqual(
      captures,
      [],
      `${query.name}: zero matching local bindings must not query native`,
    );
    return;
  }
  assert.equal(requests.length, 1, `${query.name}: exactly one native classification`);
  const request = requests[0].value as NativeRequest;
  assert.equal(request.positions.length, query.requestedBindings.length, query.name);
  assert.equal(
    new Set(request.positions).size,
    request.positions.length,
    "distinct authored spans",
  );
  const names = request.positions.map((position) => {
    const matches = query.requestedBindings.filter(
      (name) =>
        request.source.slice(position, position + name.length) === name &&
        !/[\w$]/u.test(request.source.charAt(position + name.length)),
    );
    assert.equal(matches.length, 1, `${query.name}: exact UTF-16 binding position ${position}`);
    return matches[0];
  });
  const compare = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);
  assert.deepEqual([...names].sort(compare), [...query.requestedBindings].sort(compare));
  const admissions = records.filter((record) => record.stage === "clientSourceAdmission");
  assert.equal(admissions.length, 1);
  const admission = admissions[0].value as {
    matches: boolean;
    uri: string;
    requestedSource: string;
    retainedSource: string;
  };
  assert.deepEqual(admission, {
    matches: true,
    uri: request.uri,
    requestedSource: request.source,
    retainedSource: request.source,
  });
  const classifications = records.filter((record) => record.stage === "classification");
  assert.equal(classifications.length, 1);
  assert.deepEqual(
    classifications[0].value,
    { result: hasVue ? names.map((name) => name !== "FakeCard") : null },
    `${query.name}: actual checker outcomes, including false and unavailable evidence`,
  );
}

test("frozen public tag packets preserve all original native requests separately", () => {
  assert.equal(baseline.schema, "vize.lsp.tag-prefix.public-baseline");
  assert.equal(baseline.version, 1);
  assert.equal(baseline.authority, "published-npm-consumer");
  assert.equal(baseline.releaseVersion, "0.567.0");
  assert.deepEqual(baseline.inputs, inputHashes);
  assert.equal(
    baseline.evidence.sha256,
    "f37a02dafee9ac4c5d409c49b8d877dd07a4b6ce95b2b630c419fa2e32dfc0f4",
  );
  readPinnedArtifact(fixtureDirectory, baseline.evidence);
  assert.deepEqual(
    baseline.profiles.map((profile) => profile.profile),
    profiles.map(({ name }) => name),
  );
  let rows = 0;
  for (const [index, profile] of profiles.entries()) {
    const original = baseline.profiles[index];
    assert.deepEqual(
      original.rows.map(({ name }) => name),
      profile.queries.map(({ name }) => name),
    );
    for (const row of original.rows) {
      assert.equal(row.nativeRequests.length, 1);
      assert.equal(row.nativeRequests[0].positions.length, profile.historicalCandidates);
      assert.equal(row.packet.jsonrpc, "2.0");
      assert.ok("result" in row.packet && !("error" in row.packet));
      rows++;
    }
  }
  assert.equal(rows, 27);
});

test("current native position oracle rejects a broad query and a zero-match query", () => {
  const query = profiles[0].queries[0];
  assert.throws(() =>
    assertNativeRequests(
      [
        {
          file: "broad.json",
          sha256: "",
          records: [{ stage: "request", value: { positions: [1, 2, 3] } }],
        },
      ],
      query,
      true,
    ),
  );
  assert.throws(() =>
    assertNativeRequests(
      [{ file: "unexpected.json", sha256: "", records: [{ stage: "request", value: {} }] }],
      profiles[0].queries[4],
      true,
    ),
  );
});

async function initializePrefixSession(
  session: LspSession,
  workspace: string,
  uri: string,
  source: string,
) {
  await session.initialize(workspace, {
    editor: true,
    lint: false,
    typecheck: true,
    ecosystem: false,
  });
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
  });
  await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
    45000,
  );
}

async function closePrefixSession(session: LspSession, evidence: string) {
  try {
    await session.shutdown();
    await session.waitForClosed();
  } finally {
    fs.writeFileSync(path.join(evidence, "stderr.log"), session.stderrText);
  }
}

function newSession() {
  // Keep the production source-required launcher and receipt validation intact.
  return new LspSession({
    repoRoot: root,
    binary: process.env.VIZE_LSP_BIN ?? binaryRelativePath(),
  });
}

test("source tag prefixes retain complete public packets with exact native binding membership", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "real Corsa runtime",
    "runtime unavailable",
  );
  const vue = requireTypecheckDependency(
    t,
    resolveFreshVuePackage(),
    "real Vue package",
    "Vue unavailable",
  );
  if (!runtime || !vue) return;
  fs.mkdirSync(output, { recursive: true });
  const run = fs.mkdtempSync(path.join(output, "correctness-"));
  writeJson(path.join(run, "dependencies.json"), {
    runtime: {
      path: fs.realpathSync(runtime),
      sha256: sha256(fs.readFileSync(runtime)),
      version: execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(),
    },
    vue: {
      path: fs.realpathSync(vue),
      packageJsonSha256: sha256(fs.readFileSync(path.join(vue, "package.json"))),
      version: JSON.parse(fs.readFileSync(path.join(vue, "package.json"), "utf8")).version,
    },
    meaning:
      "Complete-packet and native-membership correctness with capture enabled; not latency evidence",
  });
  const previous = process.env.VIZE_COMPONENT_TYPE_CAPTURE;
  try {
    for (const [profileIndex, profile] of profiles.entries()) {
      await t.test(profile.name, async () => {
        const evidence = path.join(run, profile.name);
        const captureDirectory = path.join(evidence, "native");
        fs.mkdirSync(captureDirectory, { recursive: true });
        process.env.VIZE_COMPONENT_TYPE_CAPTURE = captureDirectory;
        const { workspace, source, uri } = preparePrefixWorkspace(profile, runtime, vue, evidence);
        const expected = baseline.profiles[profileIndex];
        assert.equal(sha256(source), expected.sourceSha256);
        const session = newSession();
        const packets: JsonRpcMessage[] = [];
        const rows: unknown[] = [];
        session.responseObservers.push((packet) => packets.push(packet));
        try {
          await initializePrefixSession(session, workspace, uri, source);
          for (const [index, query] of profile.queries.entries()) {
            const before = new Set(nativeCaptures(captureDirectory).map(({ file }) => file));
            const params = prefixParams(source, uri, query);
            let requestId: number | string | undefined;
            const result = await session.request("textDocument/completion", params, 60000, (id) => {
              requestId = id;
              return [];
            });
            const packet = packets.find(({ id }) => id === requestId);
            const captures = nativeCaptures(captureDirectory).filter(
              ({ file }) => !before.has(file),
            );
            rows.push({
              name: query.name,
              params,
              packet,
              result,
              captures,
              expectedBindings: query.requestedBindings,
            });
            writeJson(path.join(evidence, "observations.json"), rows);
            assert.deepEqual(params.position, expected.rows[index].position);
            assert.deepEqual(
              packet,
              expected.rows[index].packet,
              `${profile.name}/${query.name}: whole packet`,
            );
            assert.deepEqual(packet, { jsonrpc: "2.0", id: requestId, result });
            assertNativeRequests(captures, query, profile.vue);
          }
        } finally {
          await closePrefixSession(session, evidence);
          fs.rmSync(workspace, { recursive: true, force: true });
        }
      });
    }
  } finally {
    if (previous === undefined) delete process.env.VIZE_COMPONENT_TYPE_CAPTURE;
    else process.env.VIZE_COMPONENT_TYPE_CAPTURE = previous;
  }
});

test("uncaptured source tag observations retain every complete first and warm response", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "real Corsa runtime",
    "runtime unavailable",
  );
  const vue = requireTypecheckDependency(
    t,
    resolveFreshVuePackage(),
    "real Vue package",
    "Vue unavailable",
  );
  if (!runtime || !vue) return;
  fs.mkdirSync(output, { recursive: true });
  const evidence = fs.mkdtempSync(path.join(output, "uncaptured-observation-"));
  const profile: PrefixProfile = profiles[0];
  const { workspace, source, uri } = preparePrefixWorkspace(profile, runtime, vue, evidence);
  const previous = process.env.VIZE_COMPONENT_TYPE_CAPTURE;
  delete process.env.VIZE_COMPONENT_TYPE_CAPTURE;
  let session: LspSession;
  try {
    session = newSession();
  } catch (error) {
    if (previous === undefined) delete process.env.VIZE_COMPONENT_TYPE_CAPTURE;
    else process.env.VIZE_COMPONENT_TYPE_CAPTURE = previous;
    throw error;
  }
  const packets: JsonRpcMessage[] = [];
  session.responseObservers.push((packet) => packets.push(packet));
  const rows: unknown[] = [];
  const summaries: unknown[] = [];
  writeJson(path.join(evidence, "methodology.json"), {
    sourceOnly: true,
    nativeCaptureEnabled: false,
    startupMeaning:
      "Requests follow initialization and initial diagnostics in this separate process; not globally cold startup",
    scope:
      "One first request and eight unchanged warm requests for Comp7 and zero-match; no performance budget or universal claim",
  });
  try {
    await initializePrefixSession(session, workspace, uri, source);
    for (const index of [0, 4]) {
      const query = profile.queries[index];
      const elapsedSamples: number[] = [];
      for (let repetition = 0; repetition < 9; repetition++) {
        const params = prefixParams(source, uri, query);
        let requestId: number | string | undefined;
        const started = performance.now();
        const result = await session.request("textDocument/completion", params, 60000, (id) => {
          requestId = id;
          return [];
        });
        const elapsedMs = performance.now() - started;
        elapsedSamples.push(elapsedMs);
        const packet = packets.find(({ id }) => id === requestId);
        rows.push({
          name: query.name,
          phase: repetition === 0 ? "first-after-initial-diagnostics" : "unchanged-warm",
          repetition,
          params,
          packet,
          elapsedMs,
        });
        writeJson(path.join(evidence, "observations.json"), rows);
        assert.deepEqual(packet, { jsonrpc: "2.0", id: requestId, result });
        assert.deepEqual(result, baseline.profiles[0].rows[index].packet.result);
      }
      const warm = elapsedSamples.slice(1).sort((a, b) => a - b);
      const p50Ms = warm[Math.ceil(warm.length * 0.5) - 1];
      const p95Ms = warm[Math.ceil(warm.length * 0.95) - 1];
      summaries.push({
        name: query.name,
        firstAfterInitialDiagnosticsMs: elapsedSamples[0],
        warmSamples: warm.length,
        warmP50Ms: p50Ms,
        warmP95Ms: p95Ms,
        percentileMethod: "nearest rank over eight unchanged warm requests",
      });
      writeJson(path.join(evidence, "summaries.json"), summaries);
      t.diagnostic(
        `${query.name}: first after initial diagnostics=${elapsedSamples[0].toFixed(3)} ms; eight warm requests p50=${p50Ms.toFixed(3)} ms, p95=${p95Ms.toFixed(3)} ms (uncaptured source session; no budget)`,
      );
    }
    assert.equal(rows.length, 18);
    assert.equal(session.stderrText.includes("component classification guard custody"), false);
  } finally {
    try {
      await closePrefixSession(session, evidence);
      fs.rmSync(workspace, { recursive: true, force: true });
    } finally {
      if (previous === undefined) delete process.env.VIZE_COMPONENT_TYPE_CAPTURE;
      else process.env.VIZE_COMPONENT_TYPE_CAPTURE = previous;
    }
  }
});
