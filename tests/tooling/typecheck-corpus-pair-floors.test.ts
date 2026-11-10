import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import { frameReader } from "../../tools/benchmarks/scripts/typecheck-corpus-pair-parse.ts";
import {
  ablateResult,
  ablateText,
} from "../../tools/benchmarks/scripts/typecheck-floors/ablate.ts";
import {
  readIndex,
  readSession,
  recordingWriter,
  transformRecording,
} from "../../tools/benchmarks/scripts/typecheck-floors/recording.ts";
import { root } from "./support/github-workflows.ts";

const floors = join(root, "tools/benchmarks/scripts/typecheck-floors");

function scratch(t: TestContext): string {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-floors-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

const frame = (message: unknown) => {
  const body = Buffer.from(JSON.stringify(message));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};

/** Run a floor mapper over one session and parse its replies. */
function converse(script: string, requests: unknown[], env: Record<string, string> = {}) {
  const result = spawnSync(process.execPath, [join(floors, script)], {
    input: Buffer.concat(requests.map(frame)),
    env: { ...process.env, ...env },
  });
  assert.equal(result.status, 0, result.stderr.toString());
  return frameReader()(result.stdout);
}

const session = [
  { jsonrpc: "2.0", id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
  { jsonrpc: "2.0", id: 2, method: "openProject", params: { projectHandle: "p" } },
  { jsonrpc: "2.0", id: 3, method: "transform", params: { fileName: "/p/A.vue", content: "<x/>" } },
  { jsonrpc: "2.0", id: 4, method: "transform", params: { fileName: "/p/B.vue", content: "<y/>" } },
];

void test("the absolute floor projects every SFC to `export default {} as any`", () => {
  const replies = converse("stub-mapper.ts", session);
  assert.deepEqual(
    replies.map((reply) => reply.id),
    [1, 2, 3, 4],
  );
  assert.deepEqual(replies[0]?.result, { diagnosticSource: "vize", positionEncoding: "utf-8" });
  assert.deepEqual(replies[2]?.result, {
    diagnostics: [],
    extension: ".ts",
    mappings: [],
    text: "export default {} as any;\n",
  });
});

void test("a recording replays byte for byte, and a file it lacks is an error", (t) => {
  const dir = join(scratch(t), "rec");
  const writer = recordingWriter(dir);
  const result = {
    text: "const a = 1;\n",
    extension: ".ts",
    mappings: [[0, 5, 0, 5, 0, 1]],
    diagnostics: [],
  };
  writer.add("/p/A.vue", result);
  writer.session({
    initialize: { positionEncoding: "utf-8", diagnosticSource: "vize" },
    openProject: [{ configIdentity: "c" }],
  });
  assert.equal(readIndex(dir).index.size, 1);
  assert.deepEqual(readSession(dir).openProject, [{ configIdentity: "c" }]);
  const replies = converse("replay-mapper.ts", session, { FLOOR_REPLAY_DIR: dir });
  assert.deepEqual(replies[1]?.result, { configIdentity: "c" });
  assert.deepEqual(replies[2], { id: 3, jsonrpc: "2.0", result });
  assert.match(
    String((replies[3]?.error as { message?: string } | undefined)?.message),
    /not recorded: \/p\/B\.vue/u,
  );
});

void test("expressions-only keeps every name and length and drops tuples inside rewritten typing", (t) => {
  const text = [
    "type __vize_Props_12 = { a: string; b: number };",
    "const __vize_props_nav_3 = { a: 1, b: 2 };",
    "// é note",
    "const kept = 1;",
  ].join("\n");
  const ablated = ablateText(text);
  assert.equal(Buffer.byteLength(ablated.text), Buffer.byteLength(text));
  assert.match(ablated.text, /^type __vize_Props_12 = any {2,};$/mu);
  assert.match(ablated.text, /^const __vize_props_nav_3 = 0 as any {2,};$/mu);
  assert.match(ablated.text, /^const kept = 1;$/mu);
  assert.deepEqual(ablated.counts, { type: 1, const: 1, blank: 1 });
  // UTF-16 offsets count the é once.
  assert.equal(ablateText(text, "utf-16").text.length, text.length);
  const kept = Buffer.byteLength(text) - "const kept = 1;".length;
  const result = {
    text,
    mappings: [
      [5, 3, 0, 3, 0, 1],
      [kept, 5, 9, 5, 0, 1],
    ],
    diagnosticDirectives: { directives: [[0, 0, 2, 9, 0, 0]] },
  };
  assert.deepEqual(ablateResult(result), { type: 1, const: 1, blank: 1, "dropped tuples": 2 });
  assert.deepEqual(result.mappings, [[kept, 5, 9, 5, 0, 1]]);
  const dir = scratch(t);
  const writer = recordingWriter(join(dir, "head"));
  writer.add("/p/A.vue", { text, mappings: [], diagnostics: [] });
  writer.session({ initialize: { positionEncoding: "utf-8" }, openProject: [] });
  const summary = transformRecording(join(dir, "head"), join(dir, "expressions"), ablateResult);
  assert.deepEqual(summary.counts, { type: 1, const: 1, blank: 1, "dropped tuples": 0 });
  assert.equal(summary.files, 1);
  assert.equal(summary.bytesAfter, summary.bytesBefore);
});
