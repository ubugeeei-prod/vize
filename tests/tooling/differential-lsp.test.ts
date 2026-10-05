import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { loadLspManifest, runLspPack, validateLspReport } from "../differential/lsp.ts";
import {
  referenceObservation,
  referenceReport,
  rewriteFrames,
} from "./support/lsp/differential-reference.ts";
import type { JsonRpcMessage, WireObservation, LspReport } from "../differential/lsp-types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pack = path.join(root, "tests/_fixtures/differential/lsp");
const manifestPath = path.join(pack, "manifest.json");
const loaded = loadLspManifest(manifestPath);
const build = {
  sourceRevision: loaded.manifest.baseRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0",
};

function paramsAt(messages: JsonRpcMessage[], index: number): Record<string, unknown> {
  const params = messages[index].params;
  assert(params);
  return params;
}
function resultArray(message: JsonRpcMessage): unknown[] {
  assert(Array.isArray(message.result));
  return message.result as unknown[];
}
function linkAt(message: JsonRpcMessage, index: number) {
  return resultArray(message)[index] as {
    tooltip?: string;
    range: { start: { character: number } };
  };
}

void test("LSP manifest binds original complete contracts and every original request", () => {
  assert.equal(loaded.cases.length, 15);
  assert.deepEqual(
    loaded.cases.map((fixture) => fixture.requests.length),
    [1, 4, 1, 1, 1, 1, 1, 1, 4, 1, 1, 2, 2, 1, 1],
  );
  assert.equal(loaded.cases[1].files[0].bytes.filter((byte) => byte === 13).length, 18);
  for (const relative of [
    "inactive-imports/case.json",
    "inactive-imports/response.expected.json",
    "on-type-crlf/App.vue.txt",
    "component-hover-readability/LongComponent.vue.txt",
    "component-hover-readability/responses.expected.json",
  ]) {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-manifest-law-"));
    try {
      fs.cpSync(pack, dir, { recursive: true });
      fs.appendFileSync(path.join(dir, relative), "corrupted");
      assert.throws(() => loadLspManifest(path.join(dir, "manifest.json")), /SHA256 mismatch/);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }
});

void test("LSP contract rejects stale readiness, duplicate replies and incomplete lifecycle", () => {
  for (const mutate of [
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        delete paramsAt(messages, 1).version;
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        paramsAt(messages, 1).version = 0;
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        const params = paramsAt(messages, 1);
        assert(typeof params.uri === "string");
        params.uri += ".other";
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        messages.push(messages[2]);
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        messages.push({ jsonrpc: "2.0", id: 999, result: null });
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "server", (messages: JsonRpcMessage[]) => {
        messages.splice(1, 1);
      }),
    (observation: WireObservation) =>
      rewriteFrames(observation, "client", (messages: JsonRpcMessage[]) => {
        messages.splice(3, 1);
      }),
    (observation: WireObservation) => {
      observation.exitStatus = 1;
    },
    (observation: WireObservation) => {
      observation.signal = "SIGTERM";
    },
    (observation: WireObservation) => {
      observation.serverWireSha256 = "1".repeat(64);
    },
  ]) {
    const observation = referenceObservation(loaded.cases[0]);
    mutate(observation);
    assert.throws(() => inspectLspObservation(loaded.cases[0], observation));
  }
});

void test("LSP complete JSON comparison preserves link order, extra fields and null versus array", () => {
  for (const [index, mutate] of [
    [0, (messages: JsonRpcMessage[]) => resultArray(messages[2]).reverse()],
    [
      0,
      (messages: JsonRpcMessage[]) => {
        linkAt(messages[2], 0).tooltip = "extra";
      },
    ],
    [
      0,
      (messages: JsonRpcMessage[]) => {
        linkAt(messages[2], 0).range.start.character += 1;
      },
    ],
    [
      1,
      (messages: JsonRpcMessage[]) => {
        messages[5].result = [];
      },
    ],
    [
      1,
      (messages: JsonRpcMessage[]) => {
        messages[4].result = null;
      },
    ],
  ] as Array<[number, (messages: JsonRpcMessage[]) => void]>) {
    const fixture = loaded.cases[index];
    const observation = referenceObservation(fixture);
    rewriteFrames(observation, "server", mutate);
    assert(
      inspectLspObservation(fixture, observation).some(
        (comparison) => comparison.state === "different",
      ),
    );
  }
});

void test("LSP report cannot invent native credit, references, coordinates or source identity", () => {
  validateLspReport(loaded, referenceReport(loaded, build), build);
  for (const mutate of [
    (report: LspReport) => report.rows.pop(),
    (report: LspReport) => {
      report.rows[1] = report.rows[0];
    },
    (report: LspReport) => {
      report.rows[0].id = "lsp/unplanned";
    },
    (report: LspReport) => {
      report.rows[0].native.state = "completed";
    },
    (report: LspReport) => {
      assert(report.rows[0].legacy.responses);
      report.rows[0].legacy.responses.pop();
    },
    (report: LspReport) => {
      assert(report.buildReceipt);
      report.buildReceipt.sourceRevision = "1".repeat(40);
    },
    (report: LspReport) => {
      assert(report.binary);
      report.binary.sha256 = "1".repeat(64);
    },
    (report: LspReport) => {
      report.buildReceipt = null;
    },
    (report: LspReport) => {
      report.summary.nativeHandled = 1;
    },
    (report: LspReport) => {
      assert(report.rows[0].legacy.observation);
      rewriteFrames(report.rows[0].legacy.observation, "server", (messages: JsonRpcMessage[]) => {
        messages[2].result = [];
      });
    },
  ]) {
    const report = referenceReport(loaded, build);
    mutate(report);
    assert.throws(() => validateLspReport(loaded, report, build));
  }
});

void test("LSP unavailable executables fail every planned case without trying a fallback", async () => {
  for (const binaryPath of [path.join(pack, "missing-vize"), "vize"]) {
    const report = await runLspPack({
      manifestPath,
      binaryPath,
      sourceRevision: build.sourceRevision,
    });
    assert.equal(report.binary, null);
    assert.equal(report.summary.legacyFailures, 15);
    assert.equal(report.summary.nativeHandled, 0);
    assert(
      report.rows.every(
        (row) => typeof row.legacy.error === "string" && row.legacy.error.length > 0,
      ),
    );
  }
});
