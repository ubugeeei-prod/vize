import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { validateBuildReceipt } from "../../../differential/build-receipt.ts";
import { auditRss } from "./rss.ts";
import { completionLabels } from "../../../tooling/support/lsp/assertions.ts";
import { loadLspChurnBudget } from "../churn-metrics.ts";
import { positionInsideTemplateSymbol } from "../lsp-oracle.ts";
import { loadLspIncrementalBudget } from "../incremental-metrics.ts";
import {
  elapsedMs,
  hash,
  object,
  responseFor,
  statistics,
  timedMessages,
  type Chunk,
  type TimedMessage,
} from "./wire.ts";

export const readJson = (directory: string, file: string): Record<string, unknown> =>
  object(JSON.parse(fs.readFileSync(path.join(directory, file), "utf8")));
export function readLines(directory: string, file: string): Record<string, unknown>[] {
  const text = fs.readFileSync(path.join(directory, file), "utf8");
  assert(text.endsWith("\n"), "partial JSONL footer cannot qualify");
  return text
    .trimEnd()
    .split("\n")
    .filter(Boolean)
    .map((line) => object(JSON.parse(line)));
}

export function cycleObservations(
  cycles: Record<string, unknown>[],
  client: TimedMessage[],
  server: TimedMessage[],
  authored: Record<string, unknown>,
) {
  assert.equal(cycles.length, 40);
  const lanes: Record<string, number[]> = {
    leafBroken: [],
    leafRepaired: [],
    sharedBrokenCompatible: [],
    sharedRepairedCompatible: [],
    completion: [],
  };
  const observations: unknown[] = [];
  const primaryVersions = new Set<string>();
  let previousEnd = -1;
  for (const [index, cycle] of cycles.entries()) {
    assert.equal(
      cycle.label,
      `${index < 20 ? "A" : "B"}${index % 20}`,
      "closed original cycle order",
    );
    assert(typeof cycle.start === "number" && typeof cycle.end === "number");
    assert(
      cycle.start >= previousEnd && cycle.end > cycle.start,
      "cycle byte windows cannot overlap",
    );
    previousEnd = cycle.end;
    const changes = client.filter(
      (row) => row.start >= Number(cycle.start) && row.end <= Number(cycle.end),
    );
    assert.equal(changes.length, 4, "exact original four edits, no extra RPC in cycle");
    assert(Array.isArray(cycle.consumed) && cycle.consumed.length === 4);
    const consumed = cycle.consumed.map(object);
    const completion = object(cycle.completion);
    const versions: number[] = [];
    for (const [edit, request] of changes.entries()) {
      assert.equal(request.message.method, "textDocument/didChange");
      const params = object(request.message.params);
      const document = object(params.textDocument);
      assert(typeof document.uri === "string" && Number.isSafeInteger(document.version));
      assert(Array.isArray(params.contentChanges) && params.contentChanges.length === 1);
      const content = object(params.contentChanges[0]);
      const key = ["leafBrokenSource", "cleanSource", "brokenDependency", "cleanDependency"][edit];
      assert.equal(content.text, object(authored[key]).text, "whole authored source must match");
      assert.deepEqual(Object.keys(content), ["text"], "original full-document edit shape");
      assert.equal(object(authored[key]).sha256, hash(String(content.text)));
      const next = changes[edit + 1];
      const matching = server.filter(
        (row) =>
          row.message.method === "textDocument/publishDiagnostics" &&
          BigInt(row.ns) >= BigInt(request.ns) &&
          (!next || BigInt(row.ns) <= BigInt(next.ns)) &&
          JSON.stringify(row.message.params) === JSON.stringify(consumed[edit]),
      );
      assert(
        matching.length > 0,
        "complete original diagnostic response must arrive in its wire window",
      );
      const publish = matching[0];
      const target = consumed[edit];
      assert(Array.isArray(target.diagnostics), "whole diagnostics array is required");
      assert.equal(target.uri, completion.uri);
      if (edit < 2) {
        assert.equal(document.uri, target.uri);
        assert.equal(document.version, target.version, "stale document versions cannot qualify");
        const identity = `${String(document.uri)}@${String(document.version)}`;
        assert(!primaryVersions.has(identity), "unique primary edit version required");
        primaryVersions.add(identity);
      } else {
        assert.notEqual(document.uri, completion.uri);
        assert.equal(target.version, completion.version);
      }
      versions.push(Number(document.version));
      const lane = Object.keys(lanes)[edit];
      const ms = elapsedMs(request.ns, publish.ns);
      lanes[lane].push(ms);
      observations.push({
        label: cycle.label,
        lane,
        request,
        publish,
        ms,
        qualification:
          edit < 2
            ? "unique-version leaf transport latency"
            : "same-version compatible shared refresh; causal application unproven",
      });
    }
    assert.equal(versions[1], versions[0] + 1);
    assert.equal(versions[3], versions[2] + 1);
    assert.equal(completion.version, versions[1]);
    assert.equal(completion.sourceSha256, object(authored.cleanSource).sha256);
    assert.equal(completion.requiredSymbol, "churnMirror");
    const requests = client.filter(
      (row) => row.message.id === completion.id && row.message.method === "textDocument/completion",
    );
    assert.equal(requests.length, 1);
    const request = requests[0];
    assert(
      request.start >= Number(cycle.end),
      "extra completion is outside original four-edit cycle",
    );
    assert.deepEqual(request.message.params, {
      textDocument: { uri: completion.uri },
      position: positionInsideTemplateSymbol(
        String(object(authored.cleanSource).text),
        "churnMirror",
        "churnM",
      ),
    });
    const response = responseFor(request, server);
    assert(completionLabels(response.message.result as never).includes("churnMirror"));
    const ms = elapsedMs(request.ns, response.ns);
    assert(
      ms <= loadLspIncrementalBudget("misskey-lsp-incremental").budget.laneBudgetsMs.completion,
      "unchanged Misskey completion ceiling",
    );
    lanes.completion.push(ms);
    observations.push({
      label: cycle.label,
      lane: "completion",
      request,
      response,
      ms,
      document: completion,
    });
  }
  assert.equal(primaryVersions.size, 80);
  assert.equal(
    client.filter((row) => row.message.method === "textDocument/completion").length,
    40,
    "no unregistered completion samples",
  );
  return {
    lanes,
    observations,
    statistics: Object.fromEntries(
      Object.entries(lanes).map(([lane, values]) => [lane, statistics(values)]),
    ),
  };
}

export function auditSession(
  directory: string,
  expected: Record<string, unknown>,
  custody: Record<string, unknown>,
) {
  const launch = readJson(directory, "launch.json");
  validateBuildReceipt(launch.receipt, expected);
  assert.deepEqual(launch.expected, expected);
  const probe = object(launch.versionProbe);
  assert.equal(probe.exitStatus, 0);
  assert.equal(probe.signal, null);
  assert.equal(probe.processError, null);
  assert.equal(
    Buffer.from(String(probe.stdoutBase64), "base64").toString("utf8").trim(),
    expected.cliVersion,
  );
  const process = readJson(directory, "process.json");
  assert.equal(process.exitStatus, 0);
  assert.equal(process.signal, null);
  assert.equal(process.processError, null);
  assert.deepEqual(process.sampler, { code: 0, signal: null, error: null });
  const chunks = readLines(directory, "chunks.jsonl");
  const streams = object(process.streams);
  const wires = Object.fromEntries(
    ["client", "server", "stderr"].map((stream) => {
      const bytes = fs.readFileSync(path.join(directory, `${stream}.bin`));
      assert.equal(bytes.length, object(streams[stream]).bytes);
      assert.equal(hash(bytes), object(streams[stream]).sha256);
      return [
        stream,
        stream === "stderr"
          ? []
          : timedMessages(bytes, chunks.filter((row) => row.stream === stream) as Chunk[]),
      ];
    }),
  );
  const metrics = readJson(directory, "churn-metrics.json");
  assert.equal(metrics.status, "passed");
  assert.equal(metrics.failure, null);
  assert.equal(metrics.commit, expected.sourceRevision);
  const { budget } = loadLspChurnBudget("misskey-lsp-churn");
  assert.deepEqual(metrics.budget, { scale: 1, ...budget });
  assert.equal(object(metrics.cycleStats).cycles, 40);
  const fixture = readJson(directory, "fixture.json");
  assert.equal(object(fixture.entry).fixturePath, object(custody.fixture).path);
  assert.equal(object(fixture.entry).revision, object(custody.fixture).revision);
  assert.equal(object(fixture.entry).revision, metrics.fixtureRevision);
  const selected = object(fixture.selected);
  assert.equal(selected.sha256, hash(JSON.stringify(selected.files)));
  const cycles = cycleObservations(
    readLines(directory, "cycles.jsonl"),
    wires.client,
    wires.server,
    object(fixture.authoredSources),
  );
  const initialize = wires.client.filter((row) => row.message.method === "initialize");
  assert.equal(initialize.length, 1);
  const ready = responseFor(initialize[0], wires.server);
  assert(ready.message.result != null);
  const start = readJson(directory, "process-start.json");
  const coldOpen = wires.client.find(
    (row) =>
      row.message.method === "textDocument/didOpen" &&
      String(object(object(row.message.params).textDocument).uri).endsWith("/MkDivider.vue"),
  );
  assert(coldOpen);
  const cold = wires.server.find(
    (row) =>
      row.message.method === "textDocument/publishDiagnostics" &&
      object(row.message.params).uri === object(object(coldOpen.message.params).textDocument).uri &&
      object(row.message.params).version === 1,
  );
  assert(cold && Array.isArray(object(cold.message.params).diagnostics));
  assert.deepEqual(object(cold.message.params).diagnostics, []);
  const rss = auditRss(
    readLines(directory, "rss.jsonl"),
    start,
    String(expected.binarySha256),
    object(custody.backend),
  );
  return {
    ...cycles,
    startup: {
      spawnToReadyMs: elapsedMs(String(start.spawnNs), ready.ns),
      initializeRpcMs: elapsedMs(initialize[0].ns, ready.ns),
      coldOpenMs: elapsedMs(coldOpen.ns, cold.ns),
      initialize,
      ready,
      coldOpen,
      cold,
      versionProbe: launch.versionProbe,
      resolveLaunchMs: launch.resolveLaunchMs,
    },
    rss,
  };
}
