import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  classifyRaw,
  validateNativeArgs,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-command.ts";
import {
  retainInputBodies,
  recordReplayBeforeInspection,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-custody.ts";
import { createHash } from "node:crypto";
import { coldScaleBuildRecipe } from "../../tools/benchmarks/scripts/cold-typechecker-scale-build.ts";

const config = "/owned/virtual/tsconfig.json";
const args = [
  "--checkers",
  "1",
  "--incremental",
  "--tsBuildInfoFile",
  "/owned/virtual/info",
  "--pretty",
  "false",
  "--project",
  config,
];
test("normal/allocation default recipes stay intact; actual custody feature seals only matching source", () => {
  const source = "a".repeat(40),
    normal = coldScaleBuildRecipe(false, source),
    allocation = coldScaleBuildRecipe(true, source),
    custody = coldScaleBuildRecipe(false, source, source);
  assert.deepEqual(normal.observerEnvironment, {});
  assert.deepEqual(allocation.args, [...normal.args, "--features", "profiling"]);
  assert.deepEqual(custody.args, [...normal.args, "--features", "cache-custody"]);
  assert.deepEqual(custody.observerEnvironment, { VIZE_CACHE_OBSERVER_SOURCE_SHA: source });
  assert.throws(() => coldScaleBuildRecipe(false, source, "b".repeat(40)));
  assert.throws(() => coldScaleBuildRecipe(true, source, source));
});
test("existing one-checker argv is retained whole, not reconstructed", () => {
  assert.equal(validateNativeArgs(args, config), args);
});
test("checker width, option additions, changed incremental policy and missing arguments refuse", () => {
  const changedWidth = [...args];
  changedWidth[1] = "2";
  const changedIncremental = [...args];
  changedIncremental[2] = "--incremental=false";
  const changedConfig = [...args];
  changedConfig[8] = "/other/tsconfig.json";
  const relativeInfo = [...args];
  relativeInfo[4] = "info";
  for (const candidate of [
    changedWidth,
    changedIncremental,
    changedConfig,
    relativeInfo,
    [...args, "--skipLibCheck"],
    args.slice(1),
    null,
  ]) {
    assert.throws(() => validateNativeArgs(candidate, config));
  }
});
test("raw class rows retain CRLF, global findings, continuation and trailing blank", () => {
  const raw = "error TS2688: missing type\r\n  detail \t\r\n";
  assert.deepEqual(classifyRaw(Buffer.from(raw), "/v", "/p", "/d"), {
    rows: [
      { index: 0, raw: "error TS2688: missing type\r\n", kind: "global", code: "2688" },
      { index: 1, raw: "  detail \t\r\n", kind: "other", code: null },
      { index: 2, raw: "", kind: "other", code: null },
    ],
    groups: [['["2688","global"]', 1]],
  });
});
test("native library through materialized symlink is classified by actual physical owner", (context) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "native-class-control-")));
  context.after(() => fs.rmSync(root, { recursive: true }));
  const virtual = path.join(root, "virtual"),
    provider = path.join(root, "provider"),
    project = path.join(root, "project");
  for (const directory of [virtual, provider, project]) fs.mkdirSync(directory);
  fs.writeFileSync(path.join(provider, "lib.d.ts"), "export {};\n");
  fs.symlinkSync(provider, path.join(virtual, "node_modules"), "dir");
  const report = classifyRaw(
    Buffer.from("node_modules/lib.d.ts(1,2): error TS2322: bad\n"),
    virtual,
    project,
    provider,
  );
  assert.deepEqual(report.rows[0], {
    index: 0,
    raw: "node_modules/lib.d.ts(1,2): error TS2322: bad\n",
    kind: "positioned",
    code: "2322",
    file: path.join(virtual, "node_modules/lib.d.ts"),
    physical: path.join(provider, "lib.d.ts"),
    role: "provider-tree",
    line: 1,
    column: 2,
  });
});
test("invalid UTF8 refuses classification while the caller retains original raw bytes", () => {
  assert.throws(() => classifyRaw(Buffer.from([255]), "/v", "/p", "/d"));
});

const sha = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
function owned(context: { after: (callback: () => void) => void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "native-custody-control-")));
  context.after(() => fs.rmSync(root, { recursive: true }));
  return root;
}
function elf() {
  const bytes = Buffer.alloc(64);
  bytes.set([127, 69, 76, 70, 2, 1]);
  bytes.writeUInt16LE(62, 18);
  return bytes;
}
test("explicit physical ELF keeps whole roster but no artifact image; text bodies remain exact", (context) => {
  const root = owned(context),
    image = path.join(root, "tsc"),
    text = path.join(root, "lib.d.ts");
  const bytes = elf();
  fs.writeFileSync(image, bytes);
  fs.writeFileSync(text, "type X = 1;\n");
  const rows = [
    { kind: "file", path: image, sha256: sha(bytes), bytes: bytes.length },
    { kind: "file", path: text, sha256: sha("type X = 1;\n"), bytes: 12 },
  ];
  retainInputBodies(
    { rows },
    root,
    [{ path: image, sha256: sha(bytes), role: "invoked-native" }],
    "control",
  );
  assert.equal(fs.existsSync(path.join(root, "input-blobs", sha(bytes))), false);
  assert.equal(
    fs.readFileSync(path.join(root, "input-blobs", sha("type X = 1;\n")), "utf8"),
    "type X = 1;\n",
  );
  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(root, "control-body-index.json"), "utf8")),
    {
      textBytes: 12,
      rows: [
        { ...rows[0], body: "metadata-only-native-image", imageRole: "invoked-native" },
        { ...rows[1], body: "retained-utf8-text", blob: sha("type X = 1;\n") },
      ],
    },
  );
});
test("unlisted binary fails closed and is never copied; known image hash/type drift refuses", (context) => {
  const root = owned(context),
    image = path.join(root, "unknown");
  const bytes = elf();
  fs.writeFileSync(image, bytes);
  const snapshot = {
    rows: [{ kind: "file", path: image, sha256: sha(bytes), bytes: bytes.length }],
  };
  assert.throws(() => retainInputBodies(snapshot, root, [], "unknown"));
  assert.equal(fs.existsSync(path.join(root, "input-blobs", sha(bytes))), false);
  assert(fs.existsSync(path.join(root, "unknown-body-refusal.json")));
  assert.throws(() =>
    retainInputBodies(
      snapshot,
      root,
      [{ path: image, sha256: "0".repeat(64), role: "bad-hash" }],
      "bad",
    ),
  );
  fs.writeFileSync(image, "ordinary source\n");
  assert.throws(() =>
    retainInputBodies(
      snapshot,
      root,
      [{ path: image, sha256: sha("ordinary source\n"), role: "bad-type" }],
      "bad",
    ),
  );
});
test("post-seal failure preserves exact replay status/error/PID/time and both raw streams first", (context) => {
  const root = owned(context),
    stdout = Buffer.from("raw\r\n"),
    stderr = Buffer.from([255, 0]);
  assert.throws(
    () =>
      recordReplayBeforeInspection(
        root,
        {
          stdout,
          stderr,
          status: null,
          signal: "SIGTERM",
          pid: 4242,
          error: new Error("spawn error"),
        },
        "12345",
        () => {
          assert(fs.existsSync(path.join(root, "native-replay.status.json")));
          throw new Error("post-seal poison");
        },
      ),
    /post-seal poison/u,
  );
  assert.deepEqual(fs.readFileSync(path.join(root, "native-replay.stdout.bin")), stdout);
  assert.deepEqual(fs.readFileSync(path.join(root, "native-replay.stderr.bin")), stderr);
  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(root, "native-replay.status.json"), "utf8")),
    {
      status: null,
      signal: "SIGTERM",
      error: "spawn error",
      invokedPid: 4242,
      elapsedNs: "12345",
    },
  );
});
