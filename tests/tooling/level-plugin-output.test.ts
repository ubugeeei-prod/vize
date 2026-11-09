import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compileFunction } from "node:vm";

const require = createRequire(import.meta.url);
const native = require("../../npm/native/index.js");
const Vue = require("vue");
const source = '<button title="日本語🎨" @click="save">{{ label }}</button>';
const options = {
  mode: "function",
  prefixIdentifiers: true,
  sourceMap: true,
  filename: "Output.vue",
};
const plugin = (name: string, family: string, run: (batch: string) => string) => ({
  name,
  family,
  version: "1.0.0",
  fingerprint: name,
  cacheInputs: [],
  run,
});

test("formatter and output hooks feed an executable native render result", () => {
  let formatterCalls = 0;
  let outputCalls = 0;
  const formatter = plugin("executable-formatter", "formatter", (json) => {
    formatterCalls++;
    const batch = JSON.parse(json);
    assert.equal(batch.offsetEncoding, "utf8");
    const start = Buffer.from(batch.compiled.code).indexOf(10);
    return JSON.stringify([{ start, end: start + 1, text: "\n\n" }]);
  });
  const output = plugin("executable-license", "output", () => {
    outputCalls++;
    return JSON.stringify([{ placement: "prepend", comment: "Copyright 日本語🎨\nlicense" }]);
  });
  const expected = native.compile(source, options);
  const compiled = native.compileWithOutputPlugins(source, [formatter, output], options);
  assert.equal(formatterCalls, 2);
  assert.equal(outputCalls, 2);
  assert.deepEqual(compiled.result.ast, expected.ast);
  assert.deepEqual(compiled.result.helpers, expected.helpers);
  assert.equal(compiled.result.preamble, expected.preamble);
  const render = compileFunction(
    `${compiled.result.preamble}\n${compiled.result.code}\nreturn render`,
    ["Vue"],
  )(Vue);
  let events = 0;
  const vnode = render({ label: "ready", save: () => events++ }, []);
  assert.equal(vnode.type, "button");
  assert.equal(vnode.children, "ready");
  assert.equal(vnode.props.title, "日本語🎨");
  vnode.props.onClick();
  assert.equal(events, 1);
  assert.equal(compiled.result.map.sourcesContent[0], source);
  assert.equal(compiled.result.map.sources[0], "Output.vue");
  assert.equal(compiled.plugins.length, 2);
  for (const cost of compiled.plugins) {
    assert.ok(cost.contentKey && cost.batchBytes > 0 && cost.operations === 1);
    assert.ok(cost.elapsedNs > 0 && cost.jsNs > 0 && !cost.cached);
  }
});

test("output cache skips callbacks and includes preceding pipeline output", () => {
  let calls = 0;
  const output = plugin("preceding-output-cache", "output", () => {
    calls++;
    return JSON.stringify([{ placement: "append", comment: "licensed" }]);
  });
  const baseline = native.compile(source, options);
  const first = native.applyOutputPlugins(baseline, [output], { cache: true });
  const second = native.applyOutputPlugins(baseline, [output], { cache: true });
  assert.equal(calls, 2);
  assert.deepEqual(second.result, first.result);
  assert.equal(second.plugins[0].cached, true);
  assert.equal(second.plugins[0].jsNs, 0);
  assert.equal(second.plugins[0].batchBytes, 0);
  const changed = native.applyOutputPlugins(first.result, [output], { cache: true });
  assert.equal(calls, 4);
  assert.notEqual(changed.plugins[0].contentKey, first.plugins[0].contentKey);
});

test("formatter and output cannot change executable text or forge maps", () => {
  const compiled = native.compile(source, options);
  for (const response of [
    [{ start: 0, end: 5, text: "alert(1)" }],
    [{ start: 0, end: 0, text: " ", map: {} }],
  ]) {
    assert.throws(() =>
      native.applyOutputPlugins(compiled, [
        plugin("forged-formatter", "formatter", () => JSON.stringify(response)),
      ]),
    );
  }
  assert.throws(() =>
    native.applyOutputPlugins(compiled, [
      plugin("forged-output", "output", () =>
        JSON.stringify([{ placement: "prepend", comment: "*/ alert(1); /*" }]),
      ),
    ]),
  );
  let calls = 0;
  assert.throws(
    () =>
      native.applyOutputPlugins(
        compiled,
        [
          plugin("nondeterministic-output", "output", () =>
            JSON.stringify([{ placement: "append", comment: String(++calls) }]),
          ),
        ],
        { cache: true, auditDeterminism: false },
      ),
    /nondeterministic/,
  );
  assert.equal(calls, 2);
});

test("Vapor output comments retain templates and map opt-in", () => {
  const pluginOptions = {
    outputMode: "vapor",
    prefixIdentifiers: true,
    sourceMap: true,
    filename: "Vapor.vue",
  };
  const baseline = native.compileVapor(source, pluginOptions);
  const result = native.compileWithOutputPlugins(
    source,
    [plugin("vapor-license", "output", () => "[]")],
    pluginOptions,
  );
  assert.deepEqual(result.result, baseline);
  const withoutMap = native.compileWithOutputPlugins(
    source,
    [
      plugin("no-map-license", "output", () =>
        JSON.stringify([{ placement: "prepend", comment: "license" }]),
      ),
    ],
    { ...pluginOptions, sourceMap: false },
  );
  assert.equal(withoutMap.result.map, undefined);
  assert.deepEqual(withoutMap.result.templates, baseline.templates);
});

test("fresh Node processes validate persistent output entries before skipping JavaScript", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-output-plugin-"));
  const run = (mayCall: boolean) => {
    const program = `
      const native = require(${JSON.stringify(require.resolve("../../npm/native/index.js"))});
      let calls = 0;
      const plugin = {
        name: "persistent-output", family: "output", version: "1", fingerprint: "license-v1", cacheInputs: [],
        run() {
          if (!${JSON.stringify(mayCall)}) throw new Error("cache hit called JavaScript");
          calls++;
          return JSON.stringify([{placement:"prepend",comment:"persistent license"}]);
        }
      };
      const result = native.compileWithOutputPlugins(${JSON.stringify(source)}, [plugin],
        ${JSON.stringify(options)}, {cache:true,cacheDir:${JSON.stringify(directory)}});
      process.stdout.write(JSON.stringify({result:result.result,cost:result.plugins[0],calls}));
    `;
    const child = spawnSync(process.execPath, ["-e", program], { encoding: "utf8" });
    assert.equal(child.status, 0, child.stderr);
    return JSON.parse(child.stdout);
  };
  try {
    const first = run(true);
    const cached = run(false);
    assert.equal(first.calls, 2);
    assert.equal(cached.calls, 0);
    assert.equal(cached.cost.cached, true);
    assert.equal(cached.cost.jsNs, 0);
    assert.deepEqual(cached.result, first.result);
    const entries = readdirSync(directory);
    assert.equal(entries.length, 1);
    const filename = path.join(directory, entries[0]);
    const entry = JSON.parse(readFileSync(filename, "utf8"));
    entry.response = JSON.stringify([{ placement: "replace", comment: "invalid cache" }]);
    writeFileSync(filename, JSON.stringify(entry));
    const repaired = run(true);
    assert.equal(repaired.calls, 2);
    assert.equal(repaired.cost.cached, false);
    assert.deepEqual(repaired.result, first.result);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
