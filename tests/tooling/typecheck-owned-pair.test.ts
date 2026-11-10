import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  batchCommand,
  diagnosticEvidence,
  assertSameEvidence,
  saveRawRun,
  snapshotInputs,
  assertSameInputs,
  freshCache,
  assertNativeReceipt,
} from "../../tools/benchmarks/scripts/typecheck-owned-pair-evidence.ts";

const result = (stdout: string, status = 0, stderr = "") => ({
  stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr),
  status,
  signal: null,
  error: null,
});
const packet = (diagnostics: string[] = [], extra: object = {}) =>
  JSON.stringify({
    files: [{ file: "Probe.vue", diagnostics }],
    errorCount: diagnostics.length,
    warningCount: 0,
    fileCount: 1,
    ...extra,
  });
const temp = <T>(run: (directory: string) => T): T => {
  const directory = mkdtempSync(join(tmpdir(), "owned-pair-test-"));
  try {
    return run(directory);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
};

test("batch commands carry exact widths independently of environment defaults", () => {
  for (const width of [1, 8] as const) {
    const args = batchCommand("/role", "/project", "/host", width);
    assert.equal(args[args.indexOf("--checkers") + 1], String(width));
    assert.ok(args.includes("--corsa-path"));
    assert.equal(args.filter((arg) => arg === "--checkers").length, 1);
  }
  assert.throws(() => batchCommand("/role", "/project", "/host", 0 as 1));
});

test("complete JSON retains unknown fields, diagnostic order, duplicate count and status", () => {
  const original = diagnosticEvidence(
    result(packet(["error: first", "error: second"], { future: { end: 4 } }), 1),
    "/project",
    "json",
  );
  for (const changed of [
    packet(["error: second", "error: first"], { future: { end: 4 } }),
    packet(["error: first", "error: second"], { future: { end: 5 } }),
    packet(["error: first", "error: second", "error: second"], { future: { end: 4 } }),
  ])
    assert.throws(() =>
      assertSameEvidence(original, diagnosticEvidence(result(changed, 1), "/project", "json")),
    );
  assert.throws(() => diagnosticEvidence(result(packet(["error: first"]), 0), "/project", "json"));
});

test("only the declared physical project prefix is canonicalized", () => {
  const a = diagnosticEvidence(
    result(packet([], { detail: "/a/Probe.vue outside=/other/a" })),
    "/a",
    "json",
  );
  const b = diagnosticEvidence(
    result(packet([], { detail: "/b/Probe.vue outside=/other/a" })),
    "/b",
    "json",
  );
  assertSameEvidence(a, b);
  const changed = diagnosticEvidence(
    result(packet([], { detail: "/b/Probe.vue outside=/other/b" })),
    "/b",
    "json",
  );
  assert.throws(() => assertSameEvidence(a, changed));
});

test("host evidence keeps full ordered elaborations while excluding only admitted statistics", () =>
  temp((directory) => {
    const text = `${directory}/Probe.vue(2,3): error TS2322: outer\n  inner detail\nFiles: 1\nTypes: 22\n`;
    const a = diagnosticEvidence(result(text, 2), directory, "text");
    const b = diagnosticEvidence(
      result(text.replace("Types: 22", "Types: 23"), 2),
      directory,
      "text",
    );
    assertSameEvidence(a, b);
    assert.throws(() =>
      assertSameEvidence(
        a,
        diagnosticEvidence(
          result(text.replace("inner detail", "other detail"), 2),
          directory,
          "text",
        ),
      ),
    );
    assert.throws(() =>
      diagnosticEvidence(result("Files: 1\nunknown trailer\n"), directory, "text"),
    );
  }));

test("stderr, invalid UTF8, signal and spawn errors refuse qualification", () => {
  assert.throws(() => diagnosticEvidence(result(packet(), 0, "context canceled"), "/a", "json"));
  assert.throws(() =>
    diagnosticEvidence({ ...result(packet()), stdout: Buffer.from([255]) }, "/a", "json"),
  );
  assert.throws(() => diagnosticEvidence({ ...result(packet()), signal: "SIGTERM" }, "/a", "json"));
  assert.throws(() =>
    diagnosticEvidence({ ...result(packet()), error: "spawn failed" }, "/a", "json"),
  );
});

test("failure bytes are persisted before any diagnostic admission", () =>
  temp((dir) => {
    const raw = { ...result(packet(), 0, "context canceled"), stdout: Buffer.from([0, 255, 10]) };
    saveRawRun(dir, raw, { argv: ["checker", "--checkers", "1"] });
    assert.deepEqual(readFileSync(join(dir, "stdout.raw")), raw.stdout);
    assert.deepEqual(readFileSync(join(dir, "stderr.raw")), raw.stderr);
    assert.throws(() => diagnosticEvidence(raw, dir, "json"));
    assert.equal(JSON.parse(readFileSync(join(dir, "run.json"), "utf8")).status, 0);
  }));

test("every invocation receives a distinct empty cache, even after failure", async () => {
  let first = "";
  await freshCache(async (env) => {
    first = env.XDG_CACHE_HOME;
    writeFileSync(join(first, "replay"), "stale");
  });
  await assert.rejects(
    freshCache(async (env) => {
      assert.notEqual(env.XDG_CACHE_HOME, first);
      throw new Error("failed run");
    }),
  );
  assert.throws(() => readFileSync(join(first, "replay")));
});

test("input closure detects content and added cache artifacts without losing originals", () =>
  temp((dir) => {
    writeFileSync(join(dir, "Probe.vue"), "<template>雪</template>");
    const before = snapshotInputs(dir);
    assertSameInputs(before, snapshotInputs(dir));
    writeFileSync(join(dir, "tsconfig.tsbuildinfo"), "replayed");
    assert.throws(() => assertSameInputs(before, snapshotInputs(dir)));
    rmSync(join(dir, "tsconfig.tsbuildinfo"));
    writeFileSync(join(dir, "Probe.vue"), "<template>changed</template>");
    assert.throws(() => assertSameInputs(before, snapshotInputs(dir)));
  }));

test("native custody rejects fallback, other role, changed image and missing runtime token", () => {
  const expected = {
    source: "a".repeat(40),
    image: "/base/vize.node",
    sha256: "b".repeat(64),
    token: "c".repeat(32),
  };
  assertNativeReceipt(expected, { ...expected, kind: "native" });
  for (const changed of [
    { kind: "process" },
    { image: "/head/vize.node" },
    { sha256: "d".repeat(64) },
    { token: null },
    { source: "e".repeat(40) },
  ])
    assert.throws(() => assertNativeReceipt(expected, { ...expected, kind: "native", ...changed }));
});

test("measurement admission refuses Mac, non-Actions and ambient provider or width overrides", async () => {
  const { requireMeasurementRunner, requireLinuxActionsRunner } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-run.ts");
  const env = { GITHUB_ACTIONS: "true", VIZE_PAIR_RUNNER_LABEL: "blacksmith-32vcpu-ubuntu-2404" };
  requireMeasurementRunner(env, "linux");
  requireLinuxActionsRunner({ GITHUB_ACTIONS: "true" }, "linux");
  assert.throws(() => requireLinuxActionsRunner(env, "darwin"));
  assert.throws(() => requireLinuxActionsRunner({ GITHUB_ACTIONS: "false" }, "linux"));
  assert.throws(() =>
    requireMeasurementRunner(
      { ...env, VIZE_PAIR_RUNNER_LABEL: "blacksmith-8vcpu-ubuntu-2404" },
      "linux",
    ),
  );
  assert.throws(() => requireMeasurementRunner(env, "darwin"));
  assert.throws(() => requireMeasurementRunner({ ...env, GITHUB_ACTIONS: "false" }, "linux"));
  for (const key of [
    "NODE_OPTIONS",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_CHECKERS",
  ])
    assert.throws(() => requireMeasurementRunner({ ...env, [key]: "override" }, "linux"));
});

test("one experiment pairs every huge workload and alternates role order with distinct cold samples", async () => {
  const { checkerTracePolicy, OWNED_WORKLOADS, selectOwnedWorkloads } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  const { PAIR_LANES, selectPairLanes, sampleOrder, requiredSamples } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair.ts");
  assert.deepEqual(
    OWNED_WORKLOADS.map((row) => row.id),
    ["misskey", "n8n", "vue-20000", "nuxt-2500", "nuxt-10000"],
  );
  assert.deepEqual(selectOwnedWorkloads("giant-2500"), [
    { id: "giant-2500", sfcs: 1, kind: "giant", giantBlocks: 2500 },
  ]);
  assert.deepEqual(
    selectOwnedWorkloads(
      "flat-interpolations-2500,flat-interpolations-10000,flat-interpolations-50000",
    ).map((row) => row.id),
    ["flat-interpolations-2500", "flat-interpolations-10000", "flat-interpolations-50000"],
  );
  assert.deepEqual(
    selectOwnedWorkloads(
      "dynamic-union-listeners-16,dynamic-union-listeners-64,dynamic-union-listeners-256",
    ).map((row) => [row.id, row.sfcs]),
    [
      ["dynamic-union-listeners-16", 33],
      ["dynamic-union-listeners-64", 129],
      ["dynamic-union-listeners-256", 513],
    ],
  );
  assert.throws(() => selectOwnedWorkloads("giant-2500,giant-2500"));
  assert.throws(() => selectOwnedWorkloads("missing"));
  assert.deepEqual(sampleOrder(0), ["base", "head"]);
  assert.deepEqual(sampleOrder(1), ["head", "base"]);
  assert.equal(PAIR_LANES.map((row) => row.id).join(","), "tsc,tsrs,batch-8");
  assert.deepEqual(
    selectPairLanes("tsc,tsrs").map((row) => row.id),
    ["tsc", "tsrs"],
  );
  assert.deepEqual(
    selectPairLanes().map((row) => row.id),
    PAIR_LANES.map((row) => row.id),
  );
  assert.throws(() => selectPairLanes("tsc,tsc"));
  assert.throws(() => selectPairLanes("missing"));
  assert.equal(requiredSamples(false).filter((phase) => phase.startsWith("warm-")).length, 3);
  assert.equal(requiredSamples(true).filter((phase) => phase.startsWith("warm-")).length, 5);
  assert.equal(checkerTracePolicy({ generated: null }).enabled, true);
  assert.equal(checkerTracePolicy({ generated: {} as never }).enabled, false);
});

test("real workload SFC custody follows the frozen tsconfig source set", async () => {
  const { expectedOwnedSfcFiles } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  const sha256 = "a".repeat(64);
  const inputs = [
    { file: "src/App.vue", sha256 },
    { file: "src/app/__tests__/fixtures/exampleFeature/ExampleFeatureModal.vue", sha256 },
    { file: "src/App.ts", sha256 },
  ];
  assert.deepEqual(expectedOwnedSfcFiles("n8n", inputs), ["src/App.vue"]);
  assert.deepEqual(expectedOwnedSfcFiles("vue-20000", inputs), [
    "src/App.vue",
    "src/app/__tests__/fixtures/exampleFeature/ExampleFeatureModal.vue",
  ]);
});

test("flat interpolation workloads retain every expression and one planted diagnostic", async () => {
  const { generateFlatInterpolationProject } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  const project = generateFlatInterpolationProject(2500);
  const source = project.files.get("src/App.vue") ?? "";
  assert.equal(source.match(/\{\{ value \+ \d+ \}\}/gu)?.length, 2500);
  assert.deepEqual(project.plants, [
    {
      kind: "script",
      file: "src/App.vue",
      line: 2507,
      codes: ["TS2322"],
    },
  ]);
});

test("dynamic union listener workloads keep distinct constructor unions", async () => {
  const { generateDynamicUnionListenerProject } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  const project = generateDynamicUnionListenerProject(16);
  const source = project.files.get("src/App.vue") ?? "";
  assert.equal(project.sfcs, 33);
  assert.equal(source.match(/<component :is="selected\d+" @change="accept" \/>/gu)?.length, 16);
  assert.equal(source.match(/const selected\d+ =/gu)?.length, 16);
  assert.equal(project.files.has("src/A15.vue"), true);
  assert.equal(project.files.has("src/B15.vue"), true);
  assert.equal(project.plants.length, 1);
  assert.deepEqual(project.plants[0]?.codes, ["TS2322"]);
});

test("declared project cache cleanup refuses linked dependency stores", async () => {
  const { symlinkSync, existsSync } = await import("node:fs");
  const { clearProjectCaches } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  temp((dir) => {
    const cache = join(dir, "cache");
    writeFileSync(cache, "provider data");
    symlinkSync(cache, join(dir, "state.tsbuildinfo"));
    assert.throws(() => clearProjectCaches(dir));
    assert.equal(readFileSync(cache, "utf8"), "provider data");
    rmSync(join(dir, "state.tsbuildinfo"));
    writeFileSync(join(dir, "state.tsbuildinfo"), "replayed");
    assert.deepEqual(clearProjectCaches(dir), ["state.tsbuildinfo"]);
    assert.equal(existsSync(join(dir, "state.tsbuildinfo")), false);
  });
});

test("native observer records the actual loaded image and token without changing exports", async () => {
  const { runInNewContext } = await import("node:vm");
  temp((directory) => {
    const exports = { getNativeBuildToken: () => "a".repeat(32), marker: "same-object" };
    const module = { exports: null as unknown };
    const moduleApi = {
      _extensions: {
        ".node": (target: { exports: unknown }, _filename: string) => {
          target.exports = exports;
        },
      },
    };
    const source = readFileSync(
      new URL("../../tools/benchmarks/scripts/typecheck-owned-pair-native.cjs", import.meta.url),
      "utf8",
    );
    const filesystem = {
      realpathSync: (file: string) => file,
      appendFileSync: (file: string, text: string) => writeFileSync(file, text, { flag: "a" }),
    };
    runInNewContext(source, {
      require: (name: string) => {
        if (name === "node:fs") return filesystem;
        if (name === "node:module") return moduleApi;
        if (name === "node:path")
          return { basename: (file: string) => file.split("/").at(-1), join };
        throw new Error(name);
      },
      process: { pid: 42, env: { VIZE_PAIR_NATIVE_RECEIPTS: directory } },
    });
    moduleApi._extensions[".node"](module, "/role/vize-vitrine.linux-x64-gnu.node");
    assert.equal(module.exports, exports);
    assert.deepEqual(JSON.parse(readFileSync(join(directory, "42.jsonl"), "utf8")), {
      image: "/role/vize-vitrine.linux-x64-gnu.node",
      token: "a".repeat(32),
      kind: "native",
      pid: 42,
    });
  });
});

test("project cache ownership refuses a linked node_modules ancestor before deletion", async () => {
  const { mkdirSync, symlinkSync } = await import("node:fs");
  const { clearProjectCaches } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projects.ts");
  temp((directory) =>
    temp((provider) => {
      mkdirSync(join(provider, ".cache"));
      writeFileSync(join(provider, ".cache/keep"), "provider-owned");
      symlinkSync(provider, join(directory, "node_modules"), "dir");
      assert.throws(() => clearProjectCaches(directory), /cache owner/u);
      assert.equal(readFileSync(join(provider, ".cache/keep"), "utf8"), "provider-owned");
    }),
  );
});

test("same-count unknown projection cannot replace an expected authored SFC", async () => {
  const { ownedProbeRows } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-projection.ts");
  const text = JSON.stringify({
    pid: 1,
    fileName: "/input/Unexpected.vue",
    bytes: 10,
    sourceBytes: 20,
    templateBytes: 8,
  });
  assert.throws(() => ownedProbeRows(text, "/input", 1, ["Expected.vue"]));
});

test("executed package bytes must match every source-published file", async () => {
  const { mkdirSync } = await import("node:fs");
  const { publishedPackageCopy } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-runtime.ts");
  temp((directory) => {
    const source = join(directory, "source"),
      actual = join(directory, "actual");
    for (const root of [source, actual]) {
      mkdirSync(root);
      writeFileSync(
        join(root, "package.json"),
        JSON.stringify({ name: "vize", files: ["mapper.js"] }),
      );
      writeFileSync(join(root, "mapper.js"), "export const projection = 'original';");
    }
    const before = publishedPackageCopy(source, actual);
    assert.equal(before.length, 2);
    writeFileSync(join(actual, "mapper.js"), "export const projection = 'different';");
    assert.throws(() => publishedPackageCopy(source, actual), /actual runtime package/u);
    assert.equal(
      readFileSync(join(source, "mapper.js"), "utf8"),
      "export const projection = 'original';",
    );
  });
});

test("runtime custody rejects copy drift, probe drift and resolution retargeting independently", async () => {
  const { mkdirSync, symlinkSync } = await import("node:fs");
  const { ownedProbeAuthority, runtimeProviderView } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-runtime.ts");
  const { providerClosure } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-authority.ts");
  temp((directory) => {
    const source = join(directory, "source"),
      project = join(directory, "project");
    const actual = join(project, "node_modules/vize");
    for (const root of [source, actual]) {
      mkdirSync(root, { recursive: true });
      writeFileSync(
        join(root, "package.json"),
        JSON.stringify({ name: "vize", files: ["mapper.js"] }),
      );
      writeFileSync(join(root, "mapper.js"), "original");
    }
    for (const mapper of ["vize", "vue"]) {
      const target = join(project, "node_modules", `${mapper}-owned-pair-probe`);
      mkdirSync(target);
      writeFileSync(join(target, "package.json"), JSON.stringify({ exec: "original" }));
    }
    const depA = join(directory, "dep-a"),
      depB = join(directory, "dep-b");
    for (const target of [depA, depB]) {
      mkdirSync(target);
      writeFileSync(join(target, "entry.js"), "same bytes");
    }
    const proofs = providerClosure([depA, depB]),
      probes = ownedProbeAuthority(project);
    const dependency = join(project, "node_modules/dependency");
    symlinkSync(depA, dependency, "dir");
    const before = runtimeProviderView(project, source, proofs, probes);
    rmSync(dependency);
    symlinkSync(depB, dependency, "dir");
    assert.notDeepEqual(runtimeProviderView(project, source, proofs, probes), before);
    writeFileSync(join(actual, "mapper.js"), "mutated");
    assert.throws(
      () => runtimeProviderView(project, source, proofs, probes),
      /actual runtime package/u,
    );
    writeFileSync(join(actual, "mapper.js"), "original");
    writeFileSync(join(project, "node_modules/vize-owned-pair-probe/package.json"), "{}");
    assert.throws(
      () => runtimeProviderView(project, source, proofs, probes),
      /actual probe package/u,
    );
  });
});

test("published copy cannot hide extra nested dependencies outside its manifest", async () => {
  const { mkdirSync } = await import("node:fs");
  const { publishedPackageCopy } =
    await import("../../tools/benchmarks/scripts/typecheck-owned-pair-runtime.ts");
  temp((directory) => {
    const source = join(directory, "source"),
      actual = join(directory, "actual");
    for (const root of [source, actual]) {
      mkdirSync(root);
      writeFileSync(join(root, "package.json"), JSON.stringify({ files: [] }));
    }
    publishedPackageCopy(source, actual);
    mkdirSync(join(actual, "node_modules"));
    writeFileSync(join(actual, "node_modules/hidden.js"), "different provider");
    assert.throws(() => publishedPackageCopy(source, actual));
  });
});
