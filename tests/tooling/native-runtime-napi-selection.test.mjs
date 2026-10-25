import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import {
  loadNativeProviders,
  prepareNativeRuntimeConsumer,
  runtimeChildEnvironment,
  captureConsumerInputs,
  assertProviderFiles,
  CORSA_OVERRIDES,
  sha256,
} from "./support/native-runtime-selection/providers.mjs";
import {
  napiNativeExecutions,
  runNapiObservation,
} from "./support/native-runtime-selection/napi-custody.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const floating =
  '<script setup lang="ts">\nasync function loadData(): Promise<number> {\n  return 1\n}\n\nloadData()\n</script>\n';
const ignored = floating.replace("loadData()\n</script>", "void loadData()\n</script>");
const runtimeHelp =
  "Type-aware lint rules were skipped because the Corsa runtime could not be started. Configure `typeChecker.corsaPath` or install `typescript@^7`.";
function empty(filename) {
  return { filename, errorCount: 0, warningCount: 0, diagnostics: [] };
}
function expectedFloating(filename, source) {
  const start = source.lastIndexOf("loadData()"),
    prefix = source.slice(0, start);
  const line = prefix.split("\n").length;
  const column = prefix.slice(prefix.lastIndexOf("\n") + 1).length + 1;
  const offset = Buffer.byteLength(prefix);
  return {
    filename,
    errorCount: 0,
    warningCount: 1,
    diagnostics: [
      {
        rule: "type/no-floating-promises",
        severity: "warning",
        message: "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
        help: "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
        location: {
          start: { line, column, offset },
          end: { line, column: column + 10, offset: offset + 10 },
        },
      },
    ],
  };
}
function expectedMissing(filename, message) {
  return {
    filename,
    errorCount: 0,
    warningCount: 1,
    diagnostics: [
      {
        rule: "type/corsa-runtime",
        severity: "warning",
        message,
        help: runtimeHelp,
        location: {
          start: { line: 1, column: 1, offset: 0 },
          end: { line: 1, column: 1, offset: 0 },
        },
      },
    ],
  };
}

test("NAPI process custody rejects absent, foreign and failed API owner records", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-napi-custody-unit-"));
  try {
    const binary = path.join(directory, "marker");
    fs.writeFileSync(binary, "selection marker only");
    const provider = { path: fs.realpathSync(binary), sha256: sha256(fs.readFileSync(binary)) };
    const node = `execve(${JSON.stringify(process.execPath)}, [${JSON.stringify(process.execPath)}], 0x1) = 0\n+++ exited with 0 +++\n`;
    const api = `execve(${JSON.stringify(binary)}, [${JSON.stringify(binary)}, "--api"], 0x1) = 0\n+++ exited with 0 +++\n`;
    assert.equal(
      napiNativeExecutions(
        [
          { file: "node", text: node },
          { file: "api", text: api },
        ],
        process.execPath,
        provider,
      ).length,
      1,
    );
    assert.throws(() =>
      napiNativeExecutions([{ file: "node", text: node }], process.execPath, provider),
    );
    assert.throws(() =>
      napiNativeExecutions(
        [
          { file: "node", text: node },
          { file: "api", text: api },
        ],
        process.execPath,
        null,
      ),
    );
    assert.throws(() =>
      napiNativeExecutions(
        [
          { file: "node", text: node },
          { file: "api", text: api.replace("exited with 0", "exited with 1") },
        ],
        process.execPath,
        provider,
      ),
    );
    assert.deepEqual(empty("App.vue"), {
      filename: "App.vue",
      errorCount: 0,
      warningCount: 0,
      diagnostics: [],
    });
    assert.equal(expectedFloating("App.vue", floating).diagnostics[0].location.start.offset, 84);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("source type-aware NAPI follows real user compiler graphs and preserves whole DTOs", async (t) => {
  if (process.env.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION !== "1") {
    t.skip("explicit source NAPI/provider qualification not selected; no runtime credit");
    return;
  }
  assert.equal(process.platform, "linux");
  assert(process.env.VIZE_NATIVE_STRACE, "absolute strace path required");
  const nativeDir = path.join(root, "npm/native"),
    buildBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const build = validateNativeHistoryBuild(nativeDir, JSON.parse(buildBytes));
  const addon = { path: fs.realpathSync(build.frozen.path), sha256: build.frozen.sha256 };
  assert.equal(sha256(fs.readFileSync(addon.path)), addon.sha256);
  const providers = loadNativeProviders();
  assert(
    providers.frozen702,
    "qualified user-installed 7.0.2 provider is required for this API capability",
  );
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-runtime-napi-10694-"));
  const archive = path.join(root, "target/vize-tests/native-runtime-napi-selection-10694");
  fs.mkdirSync(archive, { recursive: true });
  const output = fs.mkdtempSync(path.join(archive, "source-"));
  const write = (name, value) =>
    fs.writeFileSync(path.join(output, name), `${JSON.stringify(value, null, 2)}\n`);
  write("identities.json", {
    source: build.source,
    buildReceiptSha256: sha256(buildBytes),
    addon,
    providers,
    consumerBase: base,
    scope:
      "source NAPI compiler selection with qualified 7.0.2; no beta API or new schema/lifecycle compatibility claim",
  });
  const worker = fileURLToPath(
    new URL("./support/native-runtime-selection/napi-worker.mjs", import.meta.url),
  );
  for (const kind of [
    "canonical-702",
    "alias-702",
    "explicit-702",
    "absent",
    "stale-decoys",
    "explicit-missing",
  ]) {
    await t.test(kind, () => {
      const consumer = prepareNativeRuntimeConsumer({
        directory: path.join(base, kind === "alias-702" ? `${kind}-利用者` : kind),
        kind: kind === "explicit-missing" ? "explicit-702" : kind,
        providers,
      });
      const caseOutput = path.join(output, kind);
      fs.mkdirSync(caseOutput);
      const env = runtimeChildEnvironment(process.env, path.join(caseOutput, "environment"));
      for (const name of CORSA_OVERRIDES) assert.equal(env[name], undefined);
      const missing = !consumer.expectedExecutable || kind === "explicit-missing";
      const errorSource =
        kind === "alias-702" ? floating.replace("\nasync", "\n// 日本語 🧭\nasync") : floating;
      const validSource =
        kind === "alias-702" ? ignored.replace("\nasync", "\n// 日本語 🧭\nasync") : ignored;
      fs.writeFileSync(consumer.entry, validSource);
      const options = {
        filename: consumer.entry,
        preset: "opinionated",
        enabledRules: ["type/no-floating-promises"],
        typeAware: true,
      };
      if (consumer.explicitExecutable) options.corsaPath = consumer.explicitExecutable.path;
      if (kind === "explicit-missing")
        options.corsaPath = path.join(consumer.workspace, "missing-native");
      const steps = missing
        ? [{ id: "missing", source: errorSource, options }]
        : [
            { id: "valid", source: validSource, options },
            { id: "error", source: errorSource, options },
            { id: "repair", source: validSource, options },
          ];
      fs.writeFileSync(
        path.join(caseOutput, "inputs.json"),
        `${JSON.stringify({ files: captureConsumerInputs(consumer), steps }, null, 2)}\n`,
      );
      assertProviderFiles(providers);
      const observed = runNapiObservation({
        strace: fs.realpathSync(process.env.VIZE_NATIVE_STRACE),
        worker,
        input: { addon, source: build.source.head, steps },
        cwd: consumer.workspace,
        env,
        directory: path.join(caseOutput, "process"),
        expected: missing ? null : consumer.expectedExecutable,
      });
      const message =
        kind === "explicit-missing"
          ? `Configured Corsa executable does not exist: ${options.corsaPath} (from configuration)`
          : "No native TypeScript/tsgo executable found. Install a native TypeScript runtime in the selected project, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to its executable.";
      assert.deepEqual(
        observed.packet.results,
        missing
          ? [{ id: "missing", result: expectedMissing(consumer.entry, message) }]
          : [
              { id: "valid", result: empty(consumer.entry) },
              { id: "error", result: expectedFloating(consumer.entry, errorSource) },
              { id: "repair", result: empty(consumer.entry) },
            ],
      );
      assertProviderFiles(providers);
      assert.equal(sha256(fs.readFileSync(addon.path)), addon.sha256);
    });
  }
});
