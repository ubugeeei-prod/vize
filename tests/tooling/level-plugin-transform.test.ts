import assert from "node:assert/strict";
import { createRequire, SourceMap } from "node:module";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const native = createRequire(import.meta.url)("../../npm/native/index.js");
const source = '<main><button class="legacy-btn">é {{ label }}</button></main>';
const plugin = (name = "design-system-transform") => ({
  name,
  version: "1",
  fingerprint: "design-system-static-class-v1",
  cacheInputs: [],
  run: (batchJson: string) => {
    const batch = JSON.parse(batchJson) as {
      schema: number;
      stage: string;
      nodes: Array<{ id: number; attrs: Array<{ name: string; value: string | null }> }>;
    };
    assert.equal(batch.schema, 1);
    assert.equal(batch.stage, "s2-precanonical-static-attributes");
    return JSON.stringify({
      schema: 1,
      edits: batch.nodes.flatMap((node) =>
        node.attrs
          .filter((attr) => attr.name === "class" && attr.value === "legacy-btn")
          .map(() => ({
            kind: "replace-static-attribute",
            node: node.id,
            name: "class",
            value: "ds-button",
          })),
      ),
    });
  },
});

test("native L2 transform compiles real attribute edits and authored source maps", () => {
  const transformed = native.compileWithTransformPlugins(source, [plugin()], {
    filename: "src/Design.vue",
    sourceMap: true,
  });
  const expected = native.compile(source.replace("legacy-btn", "ds-button"), {
    mode: "module",
    prefixIdentifiers: true,
  });
  assert.equal(transformed.result.code, expected.code);
  assert.equal(transformed.result.preamble, expected.preamble);
  assert.deepEqual(transformed.result.map.sourcesContent, [source]);
  const map = new SourceMap(transformed.result.map);
  for (const name of ["class", "label"]) {
    const before = transformed.result.code.slice(0, transformed.result.code.indexOf(name));
    const rows = before.split("\n");
    const entry = map.findEntry(rows.length - 1, rows.at(-1)!.length);
    assert.equal(entry.originalSource, "src/Design.vue");
    assert.equal(entry.originalLine, 0);
    assert.equal(entry.originalColumn, source.indexOf(name));
  }
  assert.equal(transformed.plugins[0].edits, 1);
  assert.equal(transformed.plugins[0].cached, false);
});

test("audited native transform cache reuses edits and invalidates changed plugin code", () => {
  const author = plugin("cache-native-transform");
  let calls = 0;
  const run = author.run;
  author.run = (batch) => {
    calls++;
    return run(batch);
  };
  const first = native.compileWithTransformPlugins(source, [author], { cache: true });
  const second = native.compileWithTransformPlugins(source, [author], { cache: true });
  assert.equal(calls, 2);
  assert.equal(second.plugins[0].cached, true);
  assert.equal(first.result.code, second.result.code);
  author.fingerprint = "design-system-static-class-v2";
  const third = native.compileWithTransformPlugins(source, [author], { cache: true });
  assert.equal(calls, 4);
  assert.equal(third.plugins[0].cached, false);
  assert.notEqual(first.plugins[0].contentKey, third.plugins[0].contentKey);
  const editedSource = source.replace("label", "newLabel");
  const fourth = native.compileWithTransformPlugins(editedSource, [author], { cache: true });
  assert.equal(calls, 6);
  assert.equal(fourth.plugins[0].cached, false);
  assert.match(fourth.result.code, /_ctx\.newLabel/);
  assert.doesNotMatch(fourth.result.code, /_ctx\.label\b/);
});

test("downstream edit keys include the preceding native L2 artifact", () => {
  const upstream = (value: string) => ({
    ...plugin("upstream-color"),
    cacheInputs: [{ name: "color", value }],
    run: () =>
      JSON.stringify({
        schema: 1,
        edits: [{ kind: "replace-static-attribute", node: 1, name: "class", value }],
      }),
  });
  let calls = 0;
  const downstream = {
    ...plugin("downstream-brand"),
    run(batchJson: string) {
      calls++;
      const batch = JSON.parse(batchJson);
      const value = batch.nodes.find((node: { id: number }) => node.id === 1).attrs[0].value;
      return JSON.stringify({
        schema: 1,
        edits: [
          { kind: "replace-static-attribute", node: 1, name: "class", value: `brand-${value}` },
        ],
      });
    },
  };
  const first = native.compileWithTransformPlugins(source, [upstream("red"), downstream], {
    cache: true,
  });
  const second = native.compileWithTransformPlugins(source, [upstream("blue"), downstream], {
    cache: true,
  });
  assert.equal(calls, 4);
  assert.match(first.result.code, /brand-red/);
  assert.match(second.result.code, /brand-blue/);
  assert.equal(second.plugins[1].cached, false);
  assert.notEqual(first.plugins[1].contentKey, second.plugins[1].contentKey);
});

test("native hook refuses invalid edits and unequal repeated callback output", () => {
  const author = plugin("bad-native-transform");
  author.run = () =>
    JSON.stringify({
      schema: 1,
      edits: [{ kind: "replace-static-attribute", node: 1, name: "key", value: "x" }],
    });
  assert.throws(
    () => native.compileWithTransformPlugins(source, [author]),
    /transform attribute `key` is not supported/,
  );
  let count = 0;
  author.run = () =>
    JSON.stringify({
      schema: 1,
      edits: [{ kind: "replace-static-attribute", node: 1, name: "class", value: `${count++}` }],
    });
  assert.throws(
    () => native.compileWithTransformPlugins(source, [author]),
    /nondeterministic transform output/,
  );
});

test("persistent native edits survive a cold Node process", () => {
  const cacheDir = mkdtempSync(path.join(tmpdir(), "vize-transform-cache-"));
  try {
    const first = native.compileWithTransformPlugins(source, [plugin("cold-transform")], {
      cache: true,
      cacheDir,
    });
    const nativePath = fileURLToPath(new URL("../../npm/native/index.js", import.meta.url));
    const script = `
      const native = require(${JSON.stringify(nativePath)});
      const output = native.compileWithTransformPlugins(${JSON.stringify(source)}, [{
        name: 'cold-transform', version: '1', fingerprint: 'design-system-static-class-v1',
        cacheInputs: [], run() { throw new Error('persistent hit must not call JS'); }
      }], { cache: true, cacheDir: ${JSON.stringify(cacheDir)} });
      process.stdout.write(JSON.stringify(output));
    `;
    const child = spawnSync(process.execPath, ["-e", script], { encoding: "utf8" });
    assert.equal(child.status, 0, child.stderr);
    const second = JSON.parse(child.stdout);
    assert.equal(second.plugins[0].cached, true);
    assert.equal(second.plugins[0].jsNs, 0);
    assert.equal(first.result.code, second.result.code);
    assert.equal(first.plugins[0].contentKey, second.plugins[0].contentKey);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});
