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
  errors: [],
};
const calls = [
  event("load"),
  ...[false, true].map((ssr) =>
    event("call", {
      entrypoint: "compileSfcBatchWithResults",
      args: [inputs, { ssr }],
      outcome: "return",
      result: { results: [{ path: inputs[0].path, ...returned }] },
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
    [calls[0], { ...calls[1], result: { results: [] } }, calls[2]],
    ...[
      { errors: ["native compilation failed; adapter fallback must not count"] },
      { code: "" },
      { path: "/other/App.vue" },
    ].map((override) => [
      calls[0],
      { ...calls[1], result: { results: [{ path: inputs[0].path, ...returned, ...override }] } },
      calls[2],
    ]),
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

await test("explicit SPA custody qualifies only client execution and rejects invalid backend scopes", () => {
  const client = { ...custody, backends: ["client"] };
  assert.deepEqual(verifyNuxtSourceBindingEvents(client, calls.slice(0, 2)).backends, ["client"]);
  assert.throws(() => verifyNuxtSourceBindingEvents(custody, calls.slice(0, 2)));
  for (const backends of [[], ["client", "client"], ["other"], "client"])
    assert.throws(() => verifyNuxtSourceBindingEvents({ ...custody, backends }, calls));
  assert.throws(() => verifyNuxtSourceBindingEvents(client, [calls[0], calls[2]]));
});

await test("actual preload scopes source selection and preserves other addons and call identity", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt-native-guard-"));
  const require = createRequire(import.meta.url);
  try {
    const binary = path.join(temporary, "source.node");
    fs.writeFileSync(binary, "unit-control bytes, not a native compiler witness");
    const proof = {
      ...custody,
      binary: {
        path: fs.realpathSync(binary),
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
    const otherAddon = {
      TsconfigCache: class {
        original = true;
      },
    };
    let physical;
    const moduleApi = {
      _extensions: {
        ".node": (module, filename) => {
          physical = filename;
          module.exports = filename === proof.binary.path ? native : otherAddon;
        },
      },
    };
    const context = {
      require: (specifier) => (specifier === "node:module" ? moduleApi : require(specifier)),
      process: {
        pid: 42,
        env: { VIZE_NUXT_NATIVE_CUSTODY: configuration },
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
    const redirected = { exports: null };
    moduleApi._extensions[".node"](redirected, foreign);
    assert.equal(
      physical,
      proof.binary.path,
      "requested published Vize target never loads foreign bytes",
    );
    assert.equal(redirected.exports.compileSfc(custody.fixtures[0].source, options), returned);
    const other = path.join(temporary, "rolldown.linux-x64-gnu.node");
    fs.writeFileSync(other, "original other-addon unit control");
    const originalOther = { exports: null };
    moduleApi._extensions[".node"](originalOther, other);
    assert.equal(physical, other);
    assert.equal(originalOther.exports, otherAddon);
    assert.equal(new (class extends originalOther.exports.TsconfigCache {})().original, true);
    assert.throws(() =>
      vm.runInNewContext(source, {
        ...context,
        process: {
          ...context.process,
          env: { ...context.process.env, NAPI_RS_NATIVE_LIBRARY_PATH: binary },
        },
      }),
    );
    fs.writeFileSync(binary, "stale source bytes");
    assert.throws(() => vm.runInNewContext(source, context));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
