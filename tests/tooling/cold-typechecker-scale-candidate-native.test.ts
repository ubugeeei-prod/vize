import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createHash } from "node:crypto";
import {
  currentProfileSource,
  assertProfileBuild,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-source.ts";
import {
  assertNormalPhase,
  assertPhaseProgram,
  assertPrimaryArgs,
  assertPrimaryOutput,
  assertProfileEnvironment,
  captureNormalPhase,
  resetOwnedPhaseRuntime,
  sealInputs,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-native-phase.ts";

const sha = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
function build(custody: boolean) {
  return {
    sourceSha: currentProfileSource.sha,
    treeSha: currentProfileSource.tree,
    binarySha256: "image",
    allocation: false,
    targetStartedEmpty: true,
    manifests: Object.fromEntries(currentProfileSource.cargo.map((row) => [row.file, row.sha256])),
    cargoArtifact: {
      features: ["default", "glyph", "maestro", ...(custody ? ["cache-custody"] : [])],
    },
    ...(custody ? { observerSourceSha: currentProfileSource.sha } : {}),
  };
}
test("normal and custody builds each enforce fixed source/Cargo/features, including both-role weakening", () => {
  for (const custody of [false, true]) {
    assertProfileBuild(build(custody), "image", custody);
    for (const change of [
      { sourceSha: "0".repeat(40) },
      { treeSha: "0".repeat(40) },
      { binarySha256: "other" },
      { allocation: true },
      { targetStartedEmpty: false },
      { manifests: {} },
      { cargoArtifact: { features: ["default", "glyph", "maestro", "profiling"] } },
      { observerSourceSha: "0".repeat(40) },
    ])
      assert.throws(() => assertProfileBuild({ ...build(custody), ...change }, "image", custody));
  }
  assert.throws(() => assertProfileBuild(build(true), "image", false));
  assert.throws(() => assertProfileBuild(build(false), "image", true));
});

test("candidate build pin is explicit, source/tree/Cargo exact, and cannot inherit the legacy source", () => {
  const pin = { sha: "1".repeat(40), tree: "2".repeat(40), cargo: currentProfileSource.cargo };
  for (const custody of [false, true]) {
    const changed = {
      ...build(custody),
      sourceSha: pin.sha,
      treeSha: pin.tree,
      ...(custody ? { observerSourceSha: pin.sha } : {}),
    };
    assertProfileBuild(changed, "image", custody, pin);
    assert.throws(() => assertProfileBuild(changed, "image", custody));
    assert.throws(() =>
      assertProfileBuild(changed, "image", custody, { ...pin, tree: "3".repeat(40) }),
    );
    assert.throws(() => assertProfileBuild({ ...changed, manifests: {} }, "image", custody, pin));
  }
});
const profile = () => ({
  schema_version: 1,
  tool: "vize",
  tool_version: "fixture",
  command: "check",
  allocation: null,
  budget: {},
  counters: [],
  spans: [{ key: "canon.corsa.cli.command", count: 1, wall_ns: { total: 1 } }],
  truncation: { dropped_spans: 0, dropped_counters: 0 },
});
test("normal primary flags, environment and nonempty coarse profile refuse debug/allocation/malformed data", () => {
  const args = [
    "check",
    "--no-config",
    "--format",
    "json",
    "--tsconfig",
    "tsconfig.json",
    "--corsa-path",
    "/owned/native",
  ];
  assertPrimaryArgs(args, "/owned/native");
  assertProfileEnvironment({});
  for (const extra of ["--quiet", "--show-virtual-ts", "--profile"])
    assert.throws(() => assertPrimaryArgs([...args, extra], "/owned/native"));
  for (const key of ["GOGC", "GOMEMLIMIT", "NODE_OPTIONS", "NODE_PATH"])
    assert.throws(() => assertProfileEnvironment({ [key]: "" }));
  assertNormalPhase(profile());
  for (const change of [
    { allocation: {} },
    { command: "lint" },
    { spans: [] },
    { spans: [{ key: "other", count: 1, wall_ns: { total: 1 } }] },
    { spans: [{ key: "canon.corsa.cli.command", count: 0, wall_ns: { total: 1 } }] },
    { truncation: { dropped_spans: 1, dropped_counters: 0 } },
    { extra: true },
    { micro: [] },
  ])
    assert.throws(() => assertNormalPhase({ ...profile(), ...change }));
});
function fixture(t: { after: (fn: () => void) => void }) {
  const out = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "normal-phase-")));
  t.after(() => fs.rmSync(out, { recursive: true }));
  const project = path.join(out, "project"),
    provider = path.join(out, "provider"),
    runtime = path.join(out, "runtime");
  const xdg = path.join(runtime, "xdg"),
    tmp = path.join(runtime, "tmp");
  for (const dir of [project, provider, runtime, xdg, tmp]) fs.mkdirSync(dir);
  fs.writeFileSync(path.join(project, "App.vue"), "<template/>\n");
  fs.writeFileSync(path.join(provider, "lib.d.ts"), "type Lib = 1;\n");
  fs.writeFileSync(path.join(xdg, "App.vue.ts"), "const x = 1;\n");
  fs.writeFileSync(path.join(xdg, "tsconfig.json"), "{}\n");
  const roots = [project, provider, xdg, tmp];
  return { out, project, provider, runtime, xdg, tmp, roots };
}
test("normal raw/status and COMPLETE tree persist before invalid profile/error and before reset", (t) => {
  for (const failure of ["none", "profile", "spawn"]) {
    const f = fixture(t);
    const capture = () =>
      captureNormalPhase(
        {
          executable: "/fake/vize",
          args: ["check"],
          cwd: f.project,
          env: {},
          out: f.out,
          images: [],
          snapshot: () => {
            assert(fs.existsSync(path.join(f.out, "phase-vize.status.json")));
            return sealInputs(f.roots, null);
          },
        },
        (_program, args) => {
          fs.writeFileSync(
            args.at(-1)!,
            failure === "profile" ? "{bad" : JSON.stringify(profile()),
          );
          return {
            status: failure === "spawn" ? null : 0,
            signal: null,
            pid: 123,
            stdout: Buffer.from("whole\r\n"),
            stderr: Buffer.from("progress\n"),
            ...(failure === "spawn" ? { error: new Error("spawn failed") } : {}),
          };
        },
      );
    if (failure === "none") {
      const result = capture();
      assert(fs.existsSync(path.join(f.xdg, "App.vue.ts")));
      resetOwnedPhaseRuntime(f.out, result.snapshot);
      assert.deepEqual(fs.readdirSync(f.xdg), []);
      assert(fs.existsSync(path.join(f.out, "input-blobs", sha("const x = 1;\n"))));
    } else {
      assert.throws(capture);
      assert(fs.existsSync(path.join(f.xdg, "App.vue.ts")));
    }
    assert.deepEqual(
      fs.readFileSync(path.join(f.out, "phase-vize.stdout.bin")),
      Buffer.from("whole\r\n"),
    );
    assert(fs.existsSync(path.join(f.out, "phase-complete-inputs.json")));
    assert(fs.existsSync(path.join(f.out, "phase-body-index.json")));
  }
});
test("whole program join allows ONLY runtime recreation mtime and exact separately-bound build info", (t) => {
  const f = fixture(t),
    before = sealInputs(f.roots, null),
    info = path.join(f.xdg, "info");
  assertPhaseProgram(before, structuredClone(before), f.runtime, info, { exists: false });
  const reseal = (row: typeof before) => {
    row.sha256 = sha(JSON.stringify(row.rows));
    row.files = row.rows.filter((r) => r.kind === "file").length;
    row.bytes = row.rows.filter((r) => r.kind === "file").reduce((n, r) => n + Number(r.bytes), 0);
    return row;
  };
  const recreated = structuredClone(before);
  recreated.rows
    .filter((r) => r.kind === "file" && String(r.path).startsWith(f.xdg))
    .forEach((r) => {
      r.mtimeNs = "123";
    });
  assertPhaseProgram(before, reseal(recreated), f.runtime, info, { exists: false });
  for (const mutate of [
    (x: typeof before) => {
      x.roots.push("/extra");
    },
    (x: typeof before) => {
      x.rows.push({
        path: path.join(f.xdg, "extra"),
        kind: "file",
        bytes: 1,
        mode: 33188,
        mtimeNs: "1",
        sha256: "x",
      });
    },
    (x: typeof before) => {
      x.rows.find((r) => r.path === path.join(f.xdg, "App.vue.ts"))!.sha256 = "changed";
    },
    (x: typeof before) => {
      x.rows.find((r) => r.path === path.join(f.xdg, "App.vue.ts"))!.mode = 33261;
    },
    (x: typeof before) => {
      x.rows.find((r) => r.path === path.join(f.xdg, "App.vue.ts"))!.unknown = true;
    },
    (x: typeof before) => {
      x.rows.find((r) => r.path === path.join(f.provider, "lib.d.ts"))!.mtimeNs = "123";
    },
    (x: typeof before) => {
      x.rows.push({
        path: path.join(f.xdg, "link"),
        kind: "symlink",
        target: "/foreign",
        physical: "/foreign",
      });
    },
  ]) {
    const changed = structuredClone(before);
    mutate(changed);
    assert.throws(() =>
      assertPhaseProgram(before, reseal(changed), f.runtime, info, { exists: false }),
    );
  }
  assert.throws(() => assertPhaseProgram(before, before, f.runtime, info, { exists: true }));
  const result = {
    status: 0,
    signal: null,
    stdout: Buffer.from("{}\n"),
    stderr: Buffer.from("progress\n"),
  };
  assertPrimaryOutput(result, { ...result });
  for (const change of [
    { status: 1 },
    { stdout: Buffer.from("{}") },
    { stderr: Buffer.alloc(0) },
    { error: new Error("spawn") },
  ])
    assert.throws(() => assertPrimaryOutput(result, { ...result, ...change }));
});

test("selected paired runtime roots never excuse authored/provider mtime changes under their parent", (t) => {
  const f = fixture(t),
    old = sealInputs(f.roots, null),
    info = path.join(f.xdg, "info");
  const roots = [f.xdg, f.tmp] as const;
  const reseal = (snapshot: typeof old) => {
    snapshot.sha256 = sha(JSON.stringify(snapshot.rows));
    return snapshot;
  };
  const recreated = structuredClone(old);
  recreated.rows
    .filter((row) => row.kind === "file" && String(row.path).startsWith(f.xdg + path.sep))
    .forEach((row) => {
      row.mtimeNs = "123";
    });
  assertPhaseProgram(old, reseal(recreated), f.out, info, { exists: false }, roots);
  const authored = structuredClone(old);
  authored.rows.find((row) => row.path === path.join(f.project, "App.vue"))!.mtimeNs = "123";
  assert.throws(() =>
    assertPhaseProgram(old, reseal(authored), f.out, info, { exists: false }, roots),
  );
  assert.throws(() => resetOwnedPhaseRuntime(f.out, old, [f.tmp, f.xdg]));
  assert(fs.existsSync(path.join(f.xdg, "App.vue.ts")));
  resetOwnedPhaseRuntime(f.out, old, roots);
  assert.deepEqual(fs.readdirSync(f.xdg), []);
  assert(fs.existsSync(path.join(f.project, "App.vue")));
});
