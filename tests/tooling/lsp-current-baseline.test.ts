import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { frameMessage } from "../differential/lsp-wire.ts";
import { cycleObservations } from "../performance/support/current-baseline/audit.ts";
import { auditRss } from "../performance/support/current-baseline/rss.ts";
import { manifest } from "../performance/support/current-baseline/inputs.ts";
import {
  elapsedMs,
  hash,
  responseFor,
  statistics,
  timedMessages,
  type TimedMessage,
} from "../performance/support/current-baseline/wire.ts";
import { positionInsideTemplateSymbol } from "../performance/support/lsp-oracle.ts";

// Synthetic observation controls. They provide no actual LSP timing/runtime credit.
const clean = "<template><span>{{ churnMirror }}</span></template>";
function packet() {
  const client: TimedMessage[] = [];
  const server: TimedMessage[] = [];
  const cycles: Record<string, unknown>[] = [];
  const sourceTexts = {
    leafBrokenSource: "broken leaf",
    cleanSource: clean,
    brokenDependency: "broken shared",
    cleanDependency: "clean shared",
  };
  const authored = Object.fromEntries(
    Object.entries(sourceTexts).map(([key, text]) => [key, { text, sha256: hash(text) }]),
  );
  const leaf = "file:///fixture/MkDivider.vue";
  const dependency = "file:///fixture/MkCodeInline.vue";
  let offset = 0;
  for (let index = 0; index < 40; index += 1) {
    const start = offset;
    const consumed: Record<string, unknown>[] = [];
    for (let edit = 0; edit < 4; edit += 1) {
      const uri = edit < 2 ? leaf : dependency;
      const version = 2 + 2 * index + (edit % 2);
      const text = Object.values(sourceTexts)[edit];
      const ns = String(1000000000n + BigInt(offset) * 1000000n);
      client.push({
        start: offset,
        end: ++offset,
        ns,
        message: {
          jsonrpc: "2.0",
          method: "textDocument/didChange",
          params: { textDocument: { uri, version }, contentChanges: [{ text }] },
        },
      });
      const payload = {
        uri: leaf,
        version: edit < 2 ? version : 3 + 2 * index,
        diagnostics:
          edit % 2 === 0
            ? [
                {
                  message: "whole original diagnostic",
                  code: 42,
                  range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
                  data: null,
                },
              ]
            : [],
      };
      consumed.push(payload);
      server.push({
        start: server.length,
        end: server.length + 1,
        ns: String(BigInt(ns) + 500000n),
        message: { jsonrpc: "2.0", method: "textDocument/publishDiagnostics", params: payload },
      });
    }
    const end = offset;
    const id = index + 1;
    const ns = String(1000000000n + BigInt(offset) * 1000000n);
    client.push({
      start: offset,
      end: ++offset,
      ns,
      message: {
        jsonrpc: "2.0",
        id,
        method: "textDocument/completion",
        params: {
          textDocument: { uri: leaf },
          position: positionInsideTemplateSymbol(clean, "churnMirror", "churnM"),
        },
      },
    });
    server.push({
      start: server.length,
      end: server.length + 1,
      ns: String(BigInt(ns) + 500000n),
      message: {
        jsonrpc: "2.0",
        id,
        result: {
          isIncomplete: false,
          items: [{ label: "churnMirror", detail: "whole result", data: null }],
        },
      },
    });
    cycles.push({
      label: `${index < 20 ? "A" : "B"}${index % 20}`,
      start,
      end,
      consumed,
      completion: {
        id,
        uri: leaf,
        version: 3 + 2 * index,
        sourceSha256: hash(clean),
        requiredSymbol: "churnMirror",
      },
    });
  }
  return { client, server, cycles, authored };
}

void test("split/coalesced UTF-8 frames keep complete objects and end-chunk clocks", () => {
  const objects = [
    { jsonrpc: "2.0", id: 1, result: { text: "🧪", explicitNull: null } },
    { jsonrpc: "2.0", id: 2, result: [] },
  ];
  const wire = Buffer.concat(objects.map(frameMessage));
  const split = wire.indexOf(Buffer.from("🧪")) + 1;
  const rows = timedMessages(wire, [
    { start: 0, end: split, ns: "1" },
    { start: split, end: wire.length, ns: "2" },
  ]);
  assert.deepEqual(
    rows.map((row) => row.message),
    objects,
  );
  assert.deepEqual(
    rows.map((row) => row.ns),
    ["2", "2"],
  );
  assert.equal(rows.at(-1)?.end, wire.length);
  assert.throws(() => timedMessages(wire, [{ start: 1, end: wire.length, ns: "2" }]));
  assert.throws(() =>
    timedMessages(wire.subarray(0, -1), [{ start: 0, end: wire.length - 1, ns: "2" }]),
  );
  assert.throws(() =>
    timedMessages(wire, [
      { start: 0, end: split, ns: "2" },
      { start: split, end: wire.length, ns: "1" },
    ]),
  );
});

void test("request pairing rejects duplicate, missing, pre-request and failed full responses", () => {
  const request: TimedMessage = {
    start: 0,
    end: 1,
    ns: "2",
    message: { jsonrpc: "2.0", id: 3, method: "initialize" },
  };
  const result: TimedMessage = {
    start: 0,
    end: 1,
    ns: "3",
    message: { jsonrpc: "2.0", id: 3, result: null },
  };
  assert.equal(responseFor(request, [result]), result);
  for (const rows of [
    [],
    [result, result],
    [{ ...result, ns: "1" }],
    [
      {
        ...result,
        message: { jsonrpc: "2.0", id: 3, error: { code: 1, message: "failure", data: null } },
      },
    ],
    [{ ...result, message: { jsonrpc: "2.0", id: 3 } }],
  ]) {
    assert.throws(() => responseFor(request, rows));
  }
  assert.throws(() => elapsedMs("2", "1"));
});

void test("whole original 40-cycle packet retains leaf and qualified shared lanes separately", () => {
  const p = packet();
  const result = cycleObservations(p.cycles, p.client, p.server, p.authored);
  assert.equal(result.observations.length, 200);
  for (const values of Object.values(result.lanes)) assert.equal(values.length, 40);
  assert.equal(result.statistics.leafBroken.p95Ms, 0.5);
  assert.equal(result.statistics.sharedBrokenCompatible.n, 40);
});

for (const [name, mutate] of Object.entries({
  "stale leaf version": (p: ReturnType<typeof packet>) => {
    (p.server[0].message.params as Record<string, unknown>).version = -1;
  },
  "lost full diagnostic field": (p: ReturnType<typeof packet>) => {
    p.server[0].message.params = { ...(p.server[0].message.params as object), extra: true };
  },
  "wrong authored source": (p: ReturnType<typeof packet>) => {
    p.authored.cleanSource.sha256 = "0".repeat(64);
  },
  "null completion": (p: ReturnType<typeof packet>) => {
    p.server[4].message.result = null;
  },
  "failed completion": (p: ReturnType<typeof packet>) => {
    p.server[4].message.error = { code: 1, message: "failed" };
  },
  "omitted original cycle": (p: ReturnType<typeof packet>) => {
    p.cycles.pop();
  },
  "swapped phase order": (p: ReturnType<typeof packet>) => {
    [p.cycles[0], p.cycles[1]] = [p.cycles[1], p.cycles[0]];
  },
  "completion inside timer": (p: ReturnType<typeof packet>) => {
    p.client[4].start = 3;
  },
  "extra completion": (p: ReturnType<typeof packet>) => {
    p.client.push(p.client[4]);
  },
})) {
  void test(`complete cycle admission refuses ${name}`, () => {
    const p = packet();
    mutate(p);
    assert.throws(() => cycleObservations(p.cycles, p.client, p.server, p.authored));
  });
}

void test("nearest-rank quantiles preserve failures and refuse invented startup p95", () => {
  assert.deepEqual(statistics([5, 1, 2, 4, 3], 2), {
    n: 5,
    failures: 2,
    p50Ms: 3,
    p95Ms: 5,
    p95Reason: null,
    maxMs: 5,
  });
  assert.equal(statistics([1, 2, 3], 0, true).p95Ms, null);
  assert.equal(statistics([]).p50Ms, null);
  assert.throws(() => statistics([NaN]));
});

function rssPacket() {
  const start = { pid: 1, startTicks: "100", spawnNs: "1" };
  const backend = { path: "/tsgo", sha256: "b".repeat(64) };
  const rows = [
    { kind: "start", pid: 1, startTicks: "100", ns: "2", intervalMs: 50 },
    {
      kind: "sample",
      ns: "3",
      members: [
        {
          pid: 1,
          parentPid: 0,
          startTicks: "100",
          rssKiB: 100,
          executable: "/vize",
          executableSha256: "a".repeat(64),
        },
        {
          pid: 2,
          parentPid: 1,
          startTicks: "101",
          rssKiB: 200,
          executable: "/tsgo",
          executableSha256: backend.sha256,
        },
      ],
    },
    { kind: "finish", ns: "4", stopped: true, failure: null, survivorsBeforeForcedCleanup: [] },
  ];
  return { start, backend, rows };
}

void test("RSS requires exact actual server/backend, positive samples and clean shutdown", () => {
  const p = rssPacket();
  assert.equal(auditRss(p.rows, p.start, "a".repeat(64), p.backend).treeMaxKiB, 300);
  for (const mutation of [
    () => {
      p.start.startTicks = "reused";
    },
    () => {
      p.backend.sha256 = "c".repeat(64);
    },
    () => {
      p.rows[1].members![0].rssKiB = 0;
    },
    () => {
      p.rows[2].failure = "leak" as never;
    },
    () => {
      p.rows.pop();
    },
  ]) {
    const before = structuredClone(p);
    mutation();
    assert.throws(() => auditRss(p.rows, p.start, "a".repeat(64), p.backend));
    Object.assign(p, before);
  }
});

void test("sampler ownership keeps reparented children and refuses reused root PID", () => {
  const program = `import runpy\nm=runpy.run_path('tests/performance/support/current-baseline/sample_rss.py')\nf=m['owned']\nrows={1:{'startTicks':'a','parentPid':0},2:{'startTicks':'b','parentPid':1},3:{'startTicks':'c','parentPid':2}}\nassert f(rows,1,'a',{1:'a'})==[1,2,3]\nassert f({2:{'startTicks':'b','parentPid':0}},1,'a',{1:'a',2:'b'})==[2]\ntry: f({1:{'startTicks':'new','parentPid':0}},1,'a',{1:'a'})\nexcept ValueError: pass\nelse: raise AssertionError('accepted reused root')\n`;
  execFileSync("python3", ["-c", program], { cwd: path.resolve(import.meta.dirname, "../..") });
});

void test("full selected input manifest rejects symlink escapes and keeps exact bytes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-baseline-input-law-"));
  try {
    fs.writeFileSync(path.join(directory, "App.vue"), "🧪\r\n");
    const result = manifest(directory);
    assert.equal(result.files[0].sha256, hash("🧪\r\n"));
    fs.symlinkSync(os.tmpdir(), path.join(directory, "escape"));
    assert.throws(() => manifest(directory));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
