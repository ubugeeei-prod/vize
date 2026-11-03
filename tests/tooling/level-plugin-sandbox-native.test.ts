// These integration tests require actual Docker isolation and the native host.
import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire, SourceMap } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { createSandboxRunner, SandboxRuntimeError } from "../../npm/plugin-sdk/sandbox.js";
import type { NativeHook } from "../../npm/plugin-sdk/stages.js";

const require = createRequire(import.meta.url);
const host = () => require("../../npm/native/index.js");
const counted = <T extends NativeHook>(hook: T) => {
  let calls = 0;
  return {
    hook: {
      ...hook,
      run(batch: string) {
        calls += 1;
        return hook.run(batch);
      },
    },
    calls: () => calls,
  };
};
const directory = () => mkdtempSync(path.join(tmpdir(), "vize-sandbox-native-"));

test("missing Docker produces a typed runtime refusal without evaluating guest code", () => {
  const root = directory();
  const sentinel = path.join(root, "guest-was-evaluated");
  const runner = createSandboxRunner({
    name: "unavailable-sandbox",
    version: "1",
    configuration: { sentinel },
    rules: {
      check:
        "(ctx, config) => process.getBuiltinModule('fs').writeFileSync(config.sentinel, 'bad')",
    },
  });
  const previous = process.env.PATH;
  try {
    process.env.PATH = root;
    assert.throws(
      () => runner.run('{"schema":1}'),
      (error: unknown) => {
        assert.ok(error instanceof SandboxRuntimeError);
        assert.equal(error.code, "runtime_unavailable");
        assert.match(error.message, /ENOENT/);
        return true;
      },
    );
    assert.equal(existsSync(sentinel), false);
  } finally {
    if (previous === undefined) delete process.env.PATH;
    else process.env.PATH = previous;
    rmSync(root, { recursive: true, force: true });
  }
});

test("sandbox facts feed native attributed diagnostics, audited caches and configuration invalidation", () => {
  const native = host();
  const cacheDir = directory();
  const source = '<template>日本語😀<button class="danger">Go</button></template>';
  const provider = (color: string) =>
    counted(
      createSandboxRunner({
        name: "sandbox-tokens",
        version: "1",
        family: "provider",
        visit: ["ui.element"],
        provides: ["sandbox-tokens/colors"],
        cacheInputs: [],
        configuration: { color },
        callback:
          "(batch, config) => ({ 'sandbox-tokens/colors': batch.nodes.map(node => [node.id, config.color]) })",
      }),
    );
  const consumer = counted(
    createSandboxRunner({
      name: "sandbox-contrast",
      version: "1",
      visit: ["ui.element"],
      demands: ["sandbox-tokens/colors"],
      cacheInputs: [],
      rules: {
        contrast: `(ctx) => {
      const colors = ctx.facts('sandbox-tokens/colors');
      for (const node of ctx.nodes) ctx.report(node, 'Use contrast for ' + colors.get(node.id));
    }`,
      },
    }),
  );
  const red = provider("red");
  const options = {
    filename: "SandboxProvider.vue",
    cache: true,
    cacheDir,
    factProviders: [red.hook],
  };
  try {
    const first = native.lintWithPlugins(source, [consumer.hook], options);
    assert.equal(first.diagnostics.length, 1);
    const diagnostic = first.diagnostics[0];
    assert.equal(diagnostic.ruleId, "sandbox-contrast/contrast");
    assert.equal(diagnostic.plugin, "sandbox-contrast");
    assert.equal(diagnostic.message, "Use contrast for red");
    assert.equal(
      Buffer.from(source).subarray(diagnostic.start, diagnostic.end).toString(),
      '<button class="danger">Go</button>',
    );
    assert.equal(red.calls(), 2, "native provider audits identical serialized input twice");
    assert.equal(consumer.calls(), 2, "native consumer audits identical serialized input twice");
    assert.equal(first.factProviders[0].audited, true);
    assert.equal(first.factProviders[0].providedGroups, 1);
    for (const cost of [first.factProviders[0], first.plugins[0]]) {
      assert.equal(cost.cached, false);
      assert.ok(cost.contentKey && cost.batchBytes > 0 && cost.jsNs > 0);
      assert.ok(cost.elapsedNs >= cost.jsNs);
    }
    const warm = native.lintWithPlugins(source, [consumer.hook], options);
    assert.deepEqual(warm.diagnostics, first.diagnostics);
    assert.equal(warm.factProviders[0].audited, false);
    for (const cost of [warm.factProviders[0], warm.plugins[0]]) {
      assert.equal(cost.cached, true);
      assert.equal(cost.jsNs, 0);
      assert.equal(cost.batchBytes, 0);
    }
    assert.equal(red.calls(), 2);
    assert.equal(consumer.calls(), 2);
    const blue = provider("blue");
    const changed = native.lintWithPlugins(source, [consumer.hook], {
      ...options,
      factProviders: [blue.hook],
    });
    assert.equal(changed.diagnostics[0].message, "Use contrast for blue");
    assert.equal(blue.calls(), 2);
    assert.equal(consumer.calls(), 4);
    assert.notEqual(changed.factProviders[0].contentKey, first.factProviders[0].contentKey);
    assert.notEqual(changed.plugins[0].contentKey, first.plugins[0].contentKey);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});

test("isolated transforms and output hooks compose into an executable Vue module with authored maps", async () => {
  const native = host();
  const cacheDir = directory();
  const source = '<button class="legacy-btn" title="日本語🎨" @click="save">{{ label }}</button>';
  const options = { filename: "SandboxDesign.vue", sourceMap: true };
  const transform = counted(
    createSandboxRunner({
      name: "sandbox-design-system",
      version: "1",
      family: "transform",
      cacheInputs: [],
      configuration: { replacement: "ds-button" },
      callback: `(batch, config) => batch.nodes.flatMap(node => node.attrs
      .filter(attr => attr.name === 'class' && attr.value === 'legacy-btn')
      .map(() => ({kind: 'replace-static-attribute', node: node.id, name: 'class', value: config.replacement})))`,
    }),
  );
  const formatter = counted(
    createSandboxRunner({
      name: "sandbox-whitespace",
      version: "1",
      family: "formatter",
      cacheInputs: [],
      callback: `batch => {
      const start = Buffer.from(batch.compiled.code).indexOf(10);
      return [{start, end: start + 1, text: '\\n\\n'}];
    }`,
    }),
  );
  const license = counted(
    createSandboxRunner({
      name: "sandbox-license",
      version: "1",
      family: "output",
      cacheInputs: [],
      configuration: { notice: "Copyright 日本語🎨\nverified" },
      callback: "(batch, config) => [{placement: 'prepend', comment: config.notice}]",
    }),
  );
  const hostOptions = { cache: true, cacheDir };
  try {
    const transformed = native.compileWithTransformPlugins(source, [transform.hook], {
      ...options,
      ...hostOptions,
    });
    const expected = native.compile(source.replace("legacy-btn", "ds-button"), {
      mode: "module",
      prefixIdentifiers: true,
    });
    assert.equal(transformed.result.code, expected.code);
    assert.equal(transformed.result.preamble, expected.preamble);
    assert.equal(transformed.plugins[0].edits, 1);
    assert.equal(transform.calls(), 2);
    const transformWarm = native.compileWithTransformPlugins(source, [transform.hook], {
      ...options,
      ...hostOptions,
    });
    assert.deepEqual(transformWarm.result, transformed.result);
    assert.equal(transformWarm.plugins[0].cached, true);
    assert.equal(transformWarm.plugins[0].jsNs, 0);
    assert.equal(transformWarm.plugins[0].nodes, transformed.plugins[0].nodes);
    assert.equal(transform.calls(), 2);
    const output = native.applyOutputPlugins(
      transformed.result,
      [formatter.hook, license.hook],
      hostOptions,
    );
    assert.deepEqual(output.result.ast, transformed.result.ast);
    assert.deepEqual(output.result.helpers, transformed.result.helpers);
    assert.equal(output.result.preamble, transformed.result.preamble);
    assert.deepEqual(output.result.map.sourcesContent, [source]);
    assert.deepEqual(output.result.map.sources, [options.filename]);
    assert.deepEqual(output.result.map.names, transformed.result.map.names);
    assert.deepEqual(
      authoredSegments(output.result.map.mappings),
      authoredSegments(transformed.result.map.mappings),
    );
    const map = new SourceMap(output.result.map);
    for (const name of ["class", "save", "label"]) {
      const before = output.result.code.slice(0, output.result.code.indexOf(name));
      const rows = before.split("\n");
      const entry = map.findEntry(rows.length - 1, rows.at(-1)!.length);
      assert.equal(entry.originalSource, options.filename);
      assert.equal(entry.originalLine, 0);
      assert.equal(entry.originalColumn, source.indexOf(name));
    }
    assert.equal(transformed.plugins[0].cached, false);
    assert.ok(transformed.plugins[0].contentKey && transformed.plugins[0].nodes > 0);
    assert.ok(transformed.plugins[0].jsNs > 0);
    assert.ok(transformed.plugins[0].elapsedNs >= transformed.plugins[0].jsNs);
    for (const cost of output.plugins) {
      assert.equal(cost.cached, false);
      assert.ok(cost.contentKey && cost.batchBytes > 0 && cost.jsNs > 0);
      assert.ok(cost.elapsedNs >= cost.jsNs);
    }
    assert.equal(formatter.calls(), 2);
    assert.equal(license.calls(), 2);
    const warm = native.applyOutputPlugins(
      transformed.result,
      [formatter.hook, license.hook],
      hostOptions,
    );
    assert.deepEqual(warm.result, output.result);
    for (const cost of warm.plugins) {
      assert.equal(cost.cached, true);
      assert.equal(cost.jsNs, 0);
      assert.equal(cost.batchBytes, 0);
    }
    assert.equal(formatter.calls(), 2);
    assert.equal(license.calls(), 2);
    // Evaluate the exact native-validated module, with its ordinary Vue import.
    mkdirSync(path.join(cacheDir, "node_modules"));
    symlinkSync(
      path.dirname(require.resolve("vue/package.json")),
      path.join(cacheDir, "node_modules/vue"),
      "dir",
    );
    const artifact = path.join(cacheDir, "compiled.mjs");
    writeFileSync(artifact, `${output.result.preamble}\n${output.result.code}`);
    const { render } = await import(pathToFileURL(artifact).href);
    let events = 0;
    const vnode = render({ label: "ready", save: () => events++ }, []);
    assert.equal(vnode.type, "button");
    assert.equal(vnode.props.class, "ds-button");
    assert.equal(vnode.props.title, "日本語🎨");
    assert.equal(vnode.children, "ready");
    vnode.props.onClick();
    assert.equal(events, 1);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});

// Compare every mapped segment's authored fields while output whitespace moves
// generated positions. Source/name/line/column deltas remain global across rows.
function authoredSegments(mappings: string) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const state = [0, 0, 0, 0];
  const authored: number[][] = [];
  for (const segment of mappings.split(/[;,]/).filter(Boolean)) {
    const values: number[] = [];
    let value = 0;
    let shift = 0;
    for (const char of segment) {
      const digit = alphabet.indexOf(char);
      assert.ok(digit >= 0);
      value |= (digit & 31) << shift;
      if (digit & 32) shift += 5;
      else {
        values.push(value & 1 ? -(value >>> 1) : value >>> 1);
        value = 0;
        shift = 0;
      }
    }
    assert.equal(shift, 0);
    if (values.length === 1) continue;
    assert.ok(values.length === 4 || values.length === 5);
    for (let index = 1; index < values.length; index++) state[index - 1] += values[index];
    authored.push(state.slice(0, values.length - 1));
  }
  assert.ok(authored.length > 0);
  return authored;
}
