import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  readObservedNativeCall,
  assertSameSourceProviderRows,
  assertSameMaterializedProgram,
  assertSameObservedNative,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-preflight.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
function observe(mutate: boolean, kind: "main" | "probe" | "unknown" = "main") {
  const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-forward-")));
  const project = path.join(temporary, "project"),
    provider = path.join(temporary, "provider"),
    output = path.join(temporary, "capture");
  for (const directory of [project, provider, output]) fs.mkdirSync(directory);
  const observer = path.join(temporary, "observer/.bin");
  fs.mkdirSync(observer, { recursive: true });
  fs.writeFileSync(path.join(observer, "package.json"), '{"type":"module"}\n');
  for (const file of [
    "cold-typechecker-scale-native-forward.ts",
    "cold-typechecker-scale-native-inputs.ts",
    "cold-typechecker-scale-native-custody.ts",
  ])
    fs.copyFileSync(path.join(root, "tools/benchmarks/scripts", file), path.join(observer, file));
  const forwarder = path.join(observer, "cold-typechecker-scale-native-forward.ts");
  fs.chmodSync(forwarder, 0o755);
  fs.writeFileSync(path.join(provider, "package.json"), '{"type":"module"}\n');
  const backend = path.join(provider, "backend.ts");
  fs.copyFileSync(path.join(root, "tests/_fixtures/tooling/native-forward/backend.ts"), backend);
  fs.chmodSync(backend, 0o755);
  const input = path.join(project, "input.ts"),
    config = path.join(project, kind === "probe" ? "tsconfig.options.json" : "tsconfig.json");
  fs.writeFileSync(input, "export const value=1;\n");
  fs.writeFileSync(config, '{"files":["input.ts"]}\n');
  const argv =
    kind === "unknown"
      ? ["--api", "--async", "--cwd", project]
      : kind === "probe"
        ? ["--pretty", "false", "--project", config]
        : [
            "--checkers",
            "1",
            "--incremental",
            "--tsBuildInfoFile",
            path.join(project, "out.tsbuildinfo"),
            "--pretty",
            "false",
            "--project",
            config,
          ];
  const result = spawnSync(forwarder, argv, {
    cwd: project,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: path.dirname(process.execPath) + path.delimiter + process.env.PATH,
      VIZE_COLD_NATIVE: backend,
      VIZE_COLD_CAPTURE: output,
      VIZE_COLD_PROJECT: project,
      VIZE_COLD_PROVIDER: provider,
      ...(mutate ? { VIZE_TEST_MUTATE: input } : {}),
    },
  });
  const calls = fs.readdirSync(output);
  assert.equal(calls.length, 1);
  const call = path.join(output, calls[0]);
  return { temporary, project, provider, result, call, argv };
}

test("untimed forwarding keeps exact child bytes/status and only permits the owned build-info output", () => {
  const observed = observe(false);
  try {
    assert.equal(observed.result.status, 2, observed.result.stderr);
    assert.equal(observed.result.error, undefined);
    assert.equal(
      observed.result.stdout,
      "src/Probe.vue.ts(1,1): error TS5097: exact fixture diagnostic\n",
    );
    assert.equal(observed.result.stderr, "exact fixture stderr\n");
    const status = JSON.parse(
      fs.readFileSync(path.join(observed.call, "native-replay.status.json"), "utf8"),
    ) as { status: number; signal: null; error: null; invokedPid: number };
    assert.deepEqual([status.status, status.signal, status.error], [2, null, null]);
    assert(status.invokedPid > 0);
    const request = JSON.parse(
      fs.readFileSync(path.join(observed.call, "request.json"), "utf8"),
    ) as { argv: string[]; buildInfoInitiallyAbsent: boolean };
    assert.deepEqual(request.argv, observed.argv);
    assert.equal(request.buildInfoInitiallyAbsent, true);
    assert.deepEqual(
      fs.readFileSync(path.join(observed.call, "inputs-before.json")),
      fs.readFileSync(path.join(observed.call, "inputs-after.json")),
    );
    const original = readObservedNativeCall(observed.call);
    assert.equal(original.request.kind, "main-cli");
    assertSameObservedNative(original, original);
    for (const patch of [
      { stdout: Buffer.from(original.stdout.toString() + "extra\n") },
      { stderr: Buffer.alloc(0) },
      { config: Buffer.from("{}") },
      { status: { ...original.status, status: 0 } },
      { request: { ...original.request, imageBefore: "0".repeat(64) } },
      { request: { ...original.request, environment: {} } },
      { request: { ...original.request, argv: ["--api"] } },
    ])
      assert.throws(() => assertSameObservedNative(original, { ...original, ...patch }));
    const before = JSON.parse(
      fs.readFileSync(path.join(observed.call, "inputs-before.json"), "utf8"),
    ) as { rows: { path: string; sha256?: string }[] };
    assertSameSourceProviderRows(
      before,
      structuredClone(before),
      observed.project,
      observed.provider,
    );
    const changedProvider = structuredClone(before);
    const providerBody = changedProvider.rows.find(
      (row) => row.path === path.join(observed.provider, "backend.ts"),
    );
    assert(providerBody);
    providerBody.sha256 = "0".repeat(64);
    assert.throws(
      () =>
        assertSameSourceProviderRows(before, changedProvider, observed.project, observed.provider),
      /potential input closure/u,
    );
  } finally {
    fs.rmSync(observed.temporary, { recursive: true, force: true });
  }
});

test("program equality keeps every path/body/mode/symlink/row; only recreated mtimes differ", () => {
  const root = path.resolve("/owned/materialized");
  const before = {
    rows: [
      { path: root, kind: "directory", mode: 493 },
      {
        path: root + "/index.ts",
        kind: "file",
        bytes: 1,
        sha256: "a".repeat(64),
        mode: 420,
        mtimeNs: "1",
      },
      { path: root + "/link", kind: "symlink", target: "index.ts", physical: root + "/index.ts" },
    ],
  };
  const after = structuredClone(before);
  after.rows[1].mtimeNs = "2";
  assertSameMaterializedProgram(before, after, root);
  for (const [index, patch] of [
    [1, { path: root + "/other.ts" }],
    [1, { sha256: "b".repeat(64) }],
    [1, { mode: 493 }],
    [1, { bytes: 2 }],
    [2, { target: "other.ts" }],
    [2, { physical: root + "/other.ts" }],
  ] as const) {
    const changed = structuredClone(after);
    Object.assign(changed.rows[index], patch);
    assert.throws(() => assertSameMaterializedProgram(before, changed, root));
  }
  assert.throws(() =>
    assertSameMaterializedProgram(before, { rows: [...after.rows, after.rows[1]] }, root),
  );
  assert.throws(() => assertSameMaterializedProgram(before, { rows: after.rows.slice(1) }, root));
});

test("post-native input mutation refuses while the complete native raw/status is already retained", () => {
  const observed = observe(true);
  try {
    assert.equal(observed.result.status, 1);
    const status = JSON.parse(
      fs.readFileSync(path.join(observed.call, "native-replay.status.json"), "utf8"),
    ) as { status: number; signal: null; error: null };
    assert.deepEqual([status.status, status.signal, status.error], [2, null, null]);
    assert.equal(
      fs.readFileSync(path.join(observed.call, "native-replay.stdout.bin"), "utf8"),
      "src/Probe.vue.ts(1,1): error TS5097: exact fixture diagnostic\n",
    );
    assert.equal(
      fs.readFileSync(path.join(observed.call, "native-replay.stderr.bin"), "utf8"),
      "exact fixture stderr\n",
    );
    assert.notDeepEqual(
      fs.readFileSync(path.join(observed.call, "inputs-before.json")),
      fs.readFileSync(path.join(observed.call, "inputs-after.json")),
    );
    assert.throws(() => readObservedNativeCall(observed.call));
  } finally {
    fs.rmSync(observed.temporary, { recursive: true, force: true });
  }
});

test("the exact best-effort option probe is forwarded and recorded independently", () => {
  const observed = observe(false, "probe");
  try {
    assert.equal(observed.result.status, 2, observed.result.stderr);
    assert.equal(
      observed.result.stdout,
      "src/Probe.vue.ts(1,1): error TS5097: exact fixture diagnostic\n",
    );
    assert.equal(observed.result.stderr, "exact fixture stderr\n");
    const request = JSON.parse(
      fs.readFileSync(path.join(observed.call, "request.json"), "utf8"),
    ) as { kind: string; argv: string[]; buildInfo: null; canonicalForwarder: string };
    assert.equal(request.kind, "option-probe");
    assert.deepEqual(request.argv, observed.argv);
    assert.equal(request.buildInfo, null);
    assert.equal(path.basename(path.dirname(request.canonicalForwarder)), ".bin");
    assert.equal(readObservedNativeCall(observed.call).request.kind, "option-probe");
  } finally {
    fs.rmSync(observed.temporary, { recursive: true, force: true });
  }
});

test("an unsupported API invocation remains visible even if an outer caller could swallow its error", () => {
  const observed = observe(false, "unknown");
  try {
    assert.equal(observed.result.status, 1);
    const invocation = JSON.parse(
      fs.readFileSync(path.join(observed.call, "invocation.json"), "utf8"),
    ) as { argv: string[] };
    assert.deepEqual(invocation.argv, ["--api", "--async", "--cwd", observed.project]);
    assert.equal(fs.existsSync(path.join(observed.call, "native-replay.status.json")), false);
    assert.throws(() => readObservedNativeCall(observed.call), /unhandled invocation/u);
  } finally {
    fs.rmSync(observed.temporary, { recursive: true, force: true });
  }
});
