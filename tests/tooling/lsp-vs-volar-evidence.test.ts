import assert from "node:assert/strict";
import { type ChildProcess, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  type Editor,
  completionLabels,
  lspLocations,
  workspaceEditLocations,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-editors.ts";
import {
  FIXTURES,
  SCRIPT_BROKEN,
  TEMPLATE_CLEAN,
  plantProbes,
  type RequestSpec,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-fixtures.ts";
import {
  killSurvivors,
  machineMemory,
  percentile,
  psTimeSeconds,
  readTree,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-metrics.ts";
import { markdown, summarize } from "../../tools/benchmarks/scripts/lsp-vs-volar-report.ts";
import { judge, runSession } from "../../tools/benchmarks/scripts/lsp-vs-volar-session.ts";
import { OpenDocument } from "../../tools/benchmarks/scripts/lsp-vs-volar-text.ts";
import { session } from "./support/lsp-vs-volar-sessions.ts";
import { FrameReader } from "../../tools/benchmarks/scripts/lsp-vs-volar-wire.ts";

test("frames split across chunks and tsserver's counted newline both decode", () => {
  const messages: unknown[] = [];
  const reader = new FrameReader((message) => messages.push(message));
  const lsp = JSON.stringify({ id: 1, result: "ü" });
  const frame = `Content-Length: ${Buffer.byteLength(lsp)}\r\n\r\n${lsp}`;
  const ts = `${JSON.stringify({ seq: 0, type: "event", event: "x" })}\n`;
  const bytes = Buffer.from(`${frame}Content-Length: ${Buffer.byteLength(ts)}\r\n\r\n${ts}`);
  for (let at = 0; at < bytes.length; at += 7) reader.push(bytes.subarray(at, at + 7));
  assert.deepEqual(messages, [
    { id: 1, result: "ü" },
    { seq: 0, type: "event", event: "x" },
  ]);
});

test("anchors are unique text, and edits keep UTF-16 positions in step", () => {
  const doc = new OpenDocument("/a.vue", "file:///a.vue", "<template>\n  {{ ü.x }}\n</template>\n");
  assert.deepEqual(doc.resolve({ find: "ü.x", at: 2 }), { line: 1, character: 7 });
  assert.throws(() => doc.resolve({ find: "template" }), /not unique/);
  assert.throws(() => doc.resolve({ find: "missing" }), /not found/);
  const edit = doc.replace("ü.x", "ü.yy");
  assert.deepEqual(edit.range, {
    start: { line: 1, character: 5 },
    end: { line: 1, character: 8 },
  });
  assert.equal(doc.version, 2);
  assert.equal(doc.text, "<template>\n  {{ ü.yy }}\n</template>\n");
  assert.equal(doc.offsetAt(doc.positionAt(17)), 17);
});

test("probes plant once per fixture and refuse ambiguous anchors", () => {
  const [spec] = FIXTURES;
  const source = `${spec.scriptProbeAfter}const a = 1;\n</script>\n<template>\n${spec.templateProbeAfter}</template>\n`;
  const planted = plantProbes(spec, source);
  assert.equal(planted.split(SCRIPT_BROKEN).length, 2);
  assert.equal(planted.split(TEMPLATE_CLEAN).length, 2);
  assert.throws(() => plantProbes(spec, `${source}${spec.scriptProbeAfter}`), /exactly once/);
  for (const fixture of FIXTURES) {
    assert.equal(
      new Set(fixture.requests.map((request) => `${request.kind}:${request.name}`)).size,
      fixture.requests.length,
    );
  }
});

test("a fast empty or wrong answer is a failure, not a sample", () => {
  const hover: RequestSpec = { kind: "hover", name: "h", anchor: { find: "x" }, expect: ["x:"] };
  assert.equal(judge(hover, "", "/w").ok, false);
  assert.equal(judge(hover, "const x: number", "/w").ok, true);
  const definition: RequestSpec = {
    kind: "definition",
    name: "d",
    anchor: { find: "x" },
    expectPath: "src/B.vue",
  };
  assert.equal(
    judge(definition, [{ path: "/w/src/A.vue", line: 0, character: 0 }], "/w").ok,
    false,
  );
  assert.equal(judge(definition, [{ path: "/w/src/B.vue", line: 0, character: 0 }], "/w").ok, true);
  const references: RequestSpec = { kind: "references", name: "r", anchor: { find: "x" }, min: 2 };
  const same = { path: "/w/a", line: 1, character: 2 };
  assert.equal(
    judge(references, [same, { ...same }], "/w").ok,
    false,
    "duplicates from two providers count once",
  );
  assert.equal(judge(references, [same, { ...same, character: 9 }], "/w").ok, true);
});

test("a location reached through a workspace symlink is the file it links to", () => {
  const workspace = mkdtempSync(join(os.tmpdir(), "lsp-judge-"));
  try {
    mkdirSync(join(workspace, "packages/board/src"), { recursive: true });
    writeFileSync(join(workspace, "packages/board/src/Level.vue"), "");
    mkdirSync(join(workspace, "node_modules/@mono"), { recursive: true });
    symlinkSync("../../packages/board", join(workspace, "node_modules/@mono/board"), "dir");
    const linked = {
      path: join(workspace, "node_modules/@mono/board/src/Level.vue"),
      line: 0,
      character: 0,
    };
    const definition: RequestSpec = {
      kind: "definition",
      name: "d",
      anchor: { find: "x" },
      expectPath: "packages/board/src/Level.vue",
    };
    assert.equal(judge(definition, [linked], workspace).ok, true);
    const elsewhere = { ...linked, path: join(workspace, "packages/board/src/Other.vue") };
    assert.equal(judge(definition, [elsewhere], workspace).ok, false);
    const references: RequestSpec = {
      kind: "references",
      name: "r",
      anchor: { find: "x" },
      min: 2,
    };
    const real = { ...linked, path: join(workspace, "packages/board/src/Level.vue") };
    assert.equal(judge(references, [linked, real], workspace).ok, false, "one file, two spellings");
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
});

test("survivors of a shutdown are killed and named", async () => {
  const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], { stdio: "ignore" });
  const exit = new Promise((resolve) => child.once("exit", (_, signal) => resolve(signal)));
  const reading = readTree([child.pid!], false, true);
  assert.deepEqual(killSurvivors(reading), [reading.members![0].name]);
  assert.equal(await exit, "SIGKILL");
  assert.deepEqual(killSurvivors(reading), [], "a dead process is not a survivor");
});

const meminfo = machineMemory();
test(
  "below the machine's memory reserve the session is killed and says where memory went",
  { skip: meminfo == null && "no /proc/meminfo" },
  async () => {
    let child: ChildProcess | undefined;
    const never = () => new Promise<never>(() => {});
    const editor: Editor = {
      label: "hog",
      kind: "vize",
      pids: () => (child?.pid ? [child.pid] : []),
      start: async () => {
        child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], { stdio: "ignore" });
      },
      open: () => {},
      change: () => {},
      diagnostics: never,
      hover: never,
      completion: never,
      definition: never,
      references: never,
      rename: never,
      semanticTokens: never,
      // A server that ignores shutdown: only the reserve can end it.
      stop: async () => {},
      stderr: () => "",
      exited: () => null,
    };
    const [fixture] = FIXTURES;
    const source = `${fixture.scriptProbeAfter}\n${fixture.templateProbeAfter}`;
    const options = {
      cycles: 1,
      coldTimeoutMs: 60_000,
      diagnosticsTimeoutMs: 60_000,
      requestTimeoutMs: 60_000,
      warmBudgetMs: 60_000,
      sampleIntervalMs: 20,
      idleMs: 0,
    };
    const exit = () =>
      new Promise((resolve) => child!.once("exit", (_, signal) => resolve(signal)));
    const killed = runSession(
      editor,
      fixture,
      "/w",
      "/w/a.vue",
      plantProbes(fixture, source),
      { ...options, memoryReserveBytes: Number.MAX_SAFE_INTEGER },
      1,
    );
    const signal = exit();
    await assert.rejects(
      killed,
      /machine memory available fell to [\d.]+ GiB \(reserve [\d.]+ GiB\): the tree held [\d.]+ GiB in 1 processes, shmem [\d.]+ GiB, largest outside the tree \(MiB\): .+; the tree was killed/,
    );
    assert.equal(await signal, "SIGKILL");
  },
);

test("protocol answers normalize across both editors' shapes", () => {
  assert.deepEqual(
    completionLabels({
      items: [
        { label: "a" },
        { label: { label: "b" } },
        { label: "channel?", filterText: "channel", insertText: "channel" },
      ],
    }),
    ["a", "b", "channel"],
  );
  const range = { start: { line: 3, character: 4 }, end: { line: 3, character: 5 } };
  assert.deepEqual(
    lspLocations([
      { targetUri: "file:///w/a.ts", targetRange: range, targetSelectionRange: range },
    ]),
    [{ path: "/w/a.ts", line: 3, character: 4 }],
  );
  assert.deepEqual(
    workspaceEditLocations({
      documentChanges: [
        { textDocument: { uri: "file:///w/b.vue" }, edits: [{ range, newText: "y" }] },
      ],
    }),
    [{ path: "/w/b.vue", line: 3, character: 4 }],
  );
});

test("statistics use nearest rank and ps CPU time parses every width", () => {
  assert.equal(percentile([5, 1, 3, 2, 4], 50), 3);
  assert.equal(percentile([5, 1, 3, 2, 4], 99), 5);
  assert.equal(percentile([], 50), null);
  assert.equal(psTimeSeconds("0:01.50"), 1.5);
  assert.equal(psTimeSeconds("1-02:03:04"), 93784);
});

test("reports exclude failed samples from latency and say so", () => {
  const sessions = [session("vize head", 5, true), session("volar 3", 1, false)];
  const [vize, volar] = summarize(sessions);
  assert.equal(vize.diagnostics.script.p50, 5);
  assert.equal(volar.diagnostics.script.p50, null);
  assert.equal(volar.diagnostics.script.ok, 0);
  const text = markdown("fixture", [vize, volar], sessions, { head: "abc" });
  assert.match(
    text,
    /\| diagnostics after script edit \| 5\.0 \/ 5\.0 \(1\) \| – \/ – \(\*\*0\/1 ok\*\*\) \|/,
  );
  assert.match(text, /a \\\| b/);
  assert.match(text, /- volar 3, round 1: cycle 1/);
});

test("the workflow keeps evidence after failure and measures the checked-out source", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/lsp-vs-volar.yml", import.meta.url),
    "utf8",
  );
  const upload = workflow
    .split(/^      - /m)
    .find((step) => step.startsWith("uses: actions/upload-artifact@"));
  assert.match(upload ?? "", /^        if: always\(\)$/m);
  assert.match(workflow, /--meta "head=\$\(git rev-parse HEAD\)"/);
  // Versions come from the workspace catalog, not from the workflow.
  assert.doesNotMatch(workflow, /@vue\/language-server@\d/);
});
