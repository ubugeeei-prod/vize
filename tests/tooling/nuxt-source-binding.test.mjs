import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { verifyNuxtSourceBindingEvents } from "../../tools/support/compat/nuxt/source-binding.mjs";

const custody = {
  schema: "vize.nuxt.source-binding",
  version: 1,
  source: { head: "a".repeat(40) },
  binary: { path: "/source/native.node", sha256: "b".repeat(64) },
  profile: "dev",
  fixtures: [{ filename: "/fixture/App.vue", source: "<template>雪🌸</template>\r\n" }],
};
const event = (kind, fields = {}) => ({
  kind,
  pid: 42,
  sourceHead: custody.source.head,
  binary: custody.binary.path,
  sha256: custody.binary.sha256,
  ...fields,
});
const inputs = [{ path: custody.fixtures[0].filename, source: custody.fixtures[0].source }];
const returned = {
  code: "original complete code",
  map: { sourcesContent: [custody.fixtures[0].source] },
};
const calls = [
  event("load"),
  ...[false, true].map((ssr) =>
    event("call", {
      entrypoint: "compileSfcBatchWithResults",
      args: [inputs, { ssr }],
      outcome: "return",
      result: returned,
    }),
  ),
];

await test("native fixture custody requires original inputs, actual load, and both source backends", () => {
  assert.equal(verifyNuxtSourceBindingEvents(custody, calls).calls, 2);
  assert.equal(verifyNuxtSourceBindingEvents(custody, calls).fixtureTargets.length, 2);
  for (const events of [
    [],
    [calls[0]],
    calls.slice(1),
    calls.slice(0, 2),
    calls.map((row) => ({ ...row, sourceHead: "c".repeat(40) })),
    calls.map((row) => ({ ...row, sha256: "c".repeat(64) })),
    calls.map((row) => ({ ...row, binary: "/published/native.node" })),
    [calls[0], { ...calls[1], outcome: "throw" }, calls[2]],
    [
      calls[0],
      { ...calls[1], args: [[{ ...inputs[0], source: "altered" }], { ssr: false }] },
      calls[2],
    ],
    [
      calls[0],
      { ...calls[1], args: [[{ ...inputs[0], path: "/other/App.vue" }], { ssr: false }] },
      calls[2],
    ],
  ])
    assert.throws(() => verifyNuxtSourceBindingEvents(custody, events));
});

await test("actual preload guard preserves result and exception identity and rejects foreign native bytes", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt-native-guard-"));
  const require = createRequire(import.meta.url);
  try {
    const binary = path.join(temporary, "source.node");
    fs.writeFileSync(binary, "unit-control bytes, not a native compiler witness");
    const proof = {
      ...custody,
      binary: {
        path: binary,
        sha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"),
      },
      calls: path.join(temporary, "calls.jsonl"),
    };
    const configuration = path.join(temporary, "custody.json");
    fs.writeFileSync(configuration, JSON.stringify(proof));
    const failure = new Error("original compiler exception");
    let received;
    const native = Object.defineProperties(
      {},
      {
        compileSfc: {
          value(...args) {
            received = args;
            return returned;
          },
        },
        compileSfcBatchWithResults: {
          value() {
            throw failure;
          },
        },
      },
    );
    const moduleApi = {
      _extensions: {
        ".node": (module) => {
          module.exports = native;
        },
      },
    };
    const context = {
      require: (specifier) => (specifier === "node:module" ? moduleApi : require(specifier)),
      process: {
        pid: 42,
        env: { VIZE_NUXT_NATIVE_CUSTODY: configuration, NAPI_RS_NATIVE_LIBRARY_PATH: binary },
      },
    };
    const source = fs.readFileSync(
      new URL("../../tools/support/compat/nuxt/source-binding-preload.cjs", import.meta.url),
      "utf8",
    );
    vm.runInNewContext(source, context);
    const loaded = { exports: null };
    moduleApi._extensions[".node"](loaded, binary);
    const options = { ssr: false };
    assert.equal(loaded.exports.compileSfc(custody.fixtures[0].source, options), returned);
    assert.equal(received[0], custody.fixtures[0].source);
    assert.equal(received[1], options);
    assert.throws(
      () => loaded.exports.compileSfcBatchWithResults(inputs),
      (error) => error === failure,
    );
    const rows = fs
      .readFileSync(proof.calls, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.deepEqual(
      rows.map((row) => [row.kind, row.outcome]),
      [
        ["load", undefined],
        ["call", "return"],
        ["call", "throw"],
      ],
    );
    assert.deepEqual(
      rows[1].result,
      returned,
      "full original result/map retained in control trace",
    );
    const foreign = path.join(temporary, "vize-vitrine.foreign.node");
    fs.writeFileSync(foreign, "foreign bytes");
    assert.throws(() => moduleApi._extensions[".node"]({ exports: null }, foreign));
    fs.writeFileSync(binary, "stale source bytes");
    assert.throws(() => vm.runInNewContext(source, context));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
