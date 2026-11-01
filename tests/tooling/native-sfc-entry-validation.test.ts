import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mts";
import { captureWholeValue, loadSfcEntryCases } from "./support/sfc-entry-validation-reference.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}
interface Binding {
  compileSfc(source: string, options: Record<string, unknown>): unknown;
  parseSfc(source: string, options: Record<string, unknown>): unknown;
}
function native(): { binding: Binding; build: unknown; image: string; imageSha256: string } {
  const directory = path.join(root, "npm/native");
  const bytes = fs.readFileSync(nativeHistoryReceipt(directory));
  const receipt: unknown = JSON.parse(bytes.toString("utf8"));
  validateNativeHistoryBuild(directory, receipt);
  record(receipt);
  record(receipt.source);
  record(receipt.frozen);
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  assert.equal(typeof receipt.frozen.path, "string");
  const image = receipt.frozen.path as string;
  const require = createRequire(import.meta.url);
  const previous = [
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
  ].map((key) => [key, process.env[key]] as const);
  let loaded: unknown;
  try {
    process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
    delete process.env.NAPI_RS_FORCE_WASI;
    loaded = require(path.join(directory, "index.js"));
  } finally {
    for (const [key, value] of previous) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
  record(loaded);
  assert.equal(typeof loaded.compileSfc, "function");
  assert.equal(typeof loaded.parseSfc, "function");
  assert.ok(require.cache[fs.realpathSync(image)]);
  return {
    binding: loaded as unknown as Binding,
    build: { receiptSha256: hash(bytes), receipt },
    image,
    imageSha256: hash(fs.readFileSync(image)),
  };
}

test("original SFC entry failures and type-only controls cross the actual public addon", () => {
  const provider = native();
  const cases = loadSfcEntryCases(root);
  const observations: {
    row: (typeof cases)[number];
    vapor: boolean;
    parsed: unknown;
    options: Record<string, unknown>;
    before: string;
    after: string;
    first: unknown;
    repeated: unknown;
  }[] = [];
  const directory = path.join(root, "target/differential/sfc-entry-validation-11262-11263");
  fs.mkdirSync(directory, { recursive: true });
  const persist = () =>
    fs.writeFileSync(
      path.join(directory, "native.json"),
      JSON.stringify(
        {
          schema: "vize.sfc-entry-validation.public.v1",
          source: process.env.GITHUB_SHA,
          build: provider.build,
          physicalImage: { path: provider.image, sha256: provider.imageSha256 },
          observations: observations.map(({ row, ...observation }) => ({
            row,
            ...observation,
            parsed: captureWholeValue(observation.parsed),
            first: captureWholeValue(observation.first),
            repeated: captureWholeValue(observation.repeated),
          })),
        },
        null,
        2,
      ) + "\n",
    );
  const observe = (run: () => unknown): unknown => {
    try {
      return { kind: "returned", value: run() };
    } catch (error) {
      return { kind: "threw", error };
    }
  };
  for (const row of cases) {
    const parsed = observe(() => provider.binding.parseSfc(row.source, { filename: row.filename }));
    for (const vapor of [false, true]) {
      const options = {
        filename: row.filename,
        vapor,
        templateSyntax: "standard",
        sourceMap: false,
      };
      const before = JSON.stringify(options);
      const first = observe(() => provider.binding.compileSfc(row.source, options));
      const repeated = observe(() => provider.binding.compileSfc(row.source, options));
      observations.push({
        row,
        vapor,
        parsed,
        options,
        before,
        after: JSON.stringify(options),
        first,
        repeated,
      });
      persist();
      console.log(
        "sfc-entry-native-packet " +
          JSON.stringify({
            id: row.id,
            vapor,
            parsed: captureWholeValue(parsed),
            first: captureWholeValue(first),
            repeated: captureWholeValue(repeated),
          }),
      );
    }
  }
  assert.equal(observations.length, 40);
  assert.equal(hash(fs.readFileSync(provider.image)), provider.imageSha256);
  const allowed = [
    "code",
    "map",
    "css",
    "errors",
    "warnings",
    "templateHash",
    "styleHash",
    "scriptHash",
    "hasScoped",
    "styles",
    "customBlocks",
    "macroArtifacts",
    "dependencies",
    "moduleShape",
  ];
  for (const observation of observations) {
    const { row, parsed, first, repeated } = observation;
    assert.equal(observation.after, observation.before);
    assert.deepEqual(captureWholeValue(repeated), captureWholeValue(first));
    record(parsed);
    assert.equal(parsed.kind, "returned");
    record(parsed.value);
    assert.equal(parsed.value.source, row.source);
    record(first);
    assert.equal(first.kind, "returned");
    record(first.value);
    const result = first.value;
    for (const key of Reflect.ownKeys(result))
      assert.ok(typeof key === "string" && allowed.includes(key));
    assert.ok(typeof result.code === "string");
    assert.deepEqual(result.warnings, []);
    assert.deepEqual(result.macroArtifacts, []);
    if (row.expect === "valid") {
      assert.deepEqual(result.errors, []);
      assert.ok(result.code.length > 0);
      const parsedModule = spawnSync(process.execPath, ["--input-type=module", "--check"], {
        input: result.code,
        timeout: 10_000,
        maxBuffer: 1024 * 1024,
      });
      console.log(
        "sfc-entry-module-parser " +
          JSON.stringify({
            id: row.id,
            vapor: observation.vapor,
            status: parsedModule.status,
            signal: parsedModule.signal,
            stdout: parsedModule.stdout?.toString("base64"),
            stderr: parsedModule.stderr?.toString("base64"),
            error: captureWholeValue(parsedModule.error),
          }),
      );
      assert.equal(parsedModule.error, undefined);
      assert.equal(parsedModule.status, 0);
      assert.equal(parsedModule.signal, null);
      assert.deepEqual(parsedModule.stdout, Buffer.alloc(0));
      assert.deepEqual(parsedModule.stderr, Buffer.alloc(0));
    } else {
      assert.deepEqual(result.errors, [
        row.expect === "export-error"
          ? "<script setup> cannot contain ES module exports."
          : "At least one <template> or <script> is required in a single file component.",
      ]);
      assert.equal(result.code, "");
      for (const key of ["map", "css", "moduleShape", "dependencies"])
        assert.equal(result[key], undefined);
    }
    assert.equal(result.hasScoped, row.id === "styleonly");
    assert.ok(Array.isArray(result.styles) && Array.isArray(result.customBlocks));
    assert.equal(result.styles.length, row.id === "styleonly" ? 1 : 0);
    assert.equal(result.customBlocks.length, row.id === "customonly" ? 1 : 0);
    for (const [key, block] of [
      ["templateHash", "template"],
      ["scriptHash", "scriptSetup"],
      ["styleHash", "styles"],
    ] as const) {
      const present = block === "styles" ? result.styles.length > 0 : parsed.value[block] !== null;
      if (key === "scriptHash" && parsed.value.script !== null)
        assert.match(String(result[key]), /^[0-9a-f]+$/);
      else if (present) assert.match(String(result[key]), /^[0-9a-f]+$/);
      else assert.equal(result[key], undefined);
    }
  }
  persist();
});
