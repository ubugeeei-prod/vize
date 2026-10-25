import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { test } from "node:test";

const capture = process.env.VIZE_VUE2_IR_CAPTURE;
const required = process.env.VIZE_VUE2_IR_REQUIRED === "1";
if (required) assert.ok(capture, "the required native capture is missing");
const packet = capture ? JSON.parse(readFileSync(capture, "utf8")) : null;
const results = [];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

function plain(value) {
  if (value === undefined) return { kind: "undefined" };
  if (value === null || typeof value !== "object") {
    assert.notEqual(typeof value, "function");
    assert.notEqual(typeof value, "symbol");
    return value;
  }
  if (Array.isArray(value)) return value.map(plain);
  return Object.fromEntries(
    Reflect.ownKeys(value).map((key) => {
      assert.equal(typeof key, "string");
      return [key, plain(value[key])];
    }),
  );
}

// Observe every own VNode field and descriptor in this native-element family.
// Only the context field has process-local identity: it must be the very same
// owning instance, rather than an arbitrary object or a dropped field.
function vnode(node, owner) {
  return Object.fromEntries(
    Reflect.ownKeys(node).map((key) => {
      assert.equal(typeof key, "string");
      const descriptor = Object.getOwnPropertyDescriptor(node, key);
      assert.ok(descriptor);
      assert.equal(typeof descriptor.get, "undefined");
      assert.equal(typeof descriptor.set, "undefined");
      let value;
      if (key === "context" && node[key] !== undefined) {
        assert.equal(node[key], owner);
        value = { kind: "actual-owning-Vue-instance" };
      } else if (key === "children" && node[key] !== undefined) {
        value = node[key].map((child) => vnode(child, owner));
      } else {
        value = plain(node[key]);
      }
      return [
        key,
        {
          value,
          enumerable: descriptor.enumerable,
          configurable: descriptor.configurable,
          writable: descriptor.writable,
        },
      ];
    }),
  );
}

function host(Vue, template) {
  const component = new Vue({ data: () => ({ a: 6, b: 3, 日本: 4 }), ...template });
  const events = [];
  const values = { a: 6, b: 3, 日本: 4 };
  for (const name of Object.keys(values)) {
    Object.defineProperty(component, name, {
      configurable: true,
      get() {
        events.push(`read:${name}`);
        return values[name];
      },
    });
  }
  return { component, events, values };
}

for (const [name, version] of Object.entries({ vue26: "2.6.14", vue27: "2.7.16" })) {
  await test(
    `actual Vue ${version} compares complete native VNodes, evaluation and SSR`,
    { skip: !capture },
    async () => {
      assert.equal(packet.schema, "vize.native-vue2-template-target.v1");
      assert.equal(packet.rows.length, 16);
      const directory = join(process.env.VIZE_VUE2_IR_ORACLES_ROOT, name);
      const require = createRequire(join(directory, "package.json"));
      const Vue = require("vue");
      const compiler = require("vue-template-compiler");
      const renderer = require("vue-server-renderer").createRenderer();
      assert.equal(Vue.version, version);
      assert.equal(require("vue-template-compiler/package.json").version, version);
      assert.equal(require("vue-server-renderer/package.json").version, version);
      const identities = Object.fromEntries(
        ["vue", "vue-template-compiler", "vue-server-renderer"].map((dependency) => {
          const path = require.resolve(`${dependency}/package.json`);
          return [
            dependency,
            { path, sha256: hash(readFileSync(path)), version: require(path).version },
          ];
        }),
      );
      for (const [index, row] of packet.rows.entries()) {
        const native = await import(
          `data:text/javascript;base64,${Buffer.from(row.code).toString("base64")}#${name}-${index}`
        );
        assert.equal(typeof native.render, "function");
        assert.deepEqual(Array.from(native.staticRenderFns), []);
        assert.equal(row.originalAstIdentitiesPreserved, true);
        const reference = compiler.compileToFunctions(row.source);
        const actual = host(Vue, native);
        const expected = host(Vue, reference);
        const before = {
          native: vnode(actual.component._render(), actual.component),
          reference: vnode(expected.component._render(), expected.component),
          nativeEvents: actual.events.slice(),
          referenceEvents: expected.events.slice(),
        };
        results.push({
          host: name,
          index,
          source: row.source,
          code: row.code,
          phase: "first",
          ...before,
        });
        assert.deepEqual(before.native, before.reference, `${name}/${index}: whole VNode`);
        assert.deepEqual(
          before.nativeEvents,
          before.referenceEvents,
          `${name}/${index}: evaluation order`,
        );
        actual.values.a = 7;
        expected.values.a = 7;
        actual.events.length = 0;
        expected.events.length = 0;
        const after = {
          native: vnode(actual.component._render(), actual.component),
          reference: vnode(expected.component._render(), expected.component),
          nativeEvents: actual.events.slice(),
          referenceEvents: expected.events.slice(),
        };
        results.push({ host: name, index, phase: "updated", ...after });
        assert.deepEqual(after.native, after.reference, `${name}/${index}: updated whole VNode`);
        assert.deepEqual(
          after.nativeEvents,
          after.referenceEvents,
          `${name}/${index}: updated evaluation order`,
        );
        const actualHtml = await renderer.renderToString(host(Vue, native).component);
        const expectedHtml = await renderer.renderToString(host(Vue, reference).component);
        results.push({
          host: name,
          index,
          phase: "ssr",
          native: actualHtml,
          reference: expectedHtml,
        });
        assert.equal(actualHtml, expectedHtml, `${name}/${index}: whole SSR HTML`);
        assert.deepEqual(JSON.parse(row.map).sourcesContent, [row.source]);
      }
      results.push({
        host: name,
        identities,
        lockSha256: hash(readFileSync(join(directory, "package-lock.json"))),
      });
    },
  );
}
if (capture)
  writeFileSync(process.env.VIZE_VUE2_IR_RUNTIME_RESULTS, JSON.stringify(results, null, 2) + "\n");
