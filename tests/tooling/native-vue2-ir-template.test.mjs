import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";

import { vnode } from "./native-vue2-ir-vnode.mjs";

const capture = process.env.VIZE_VUE2_IR_CAPTURE;
const required = process.env.VIZE_VUE2_IR_REQUIRED === "1";
if (required) assert.ok(capture, "the required native capture is missing");
const packet = capture ? JSON.parse(readFileSync(capture, "utf8")) : null;
const results = [];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

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
      assert.equal(packet.rows.length, 19);
      const directory = join(process.env.VIZE_VUE2_IR_ORACLES_ROOT, name);
      const require = createRequire(join(directory, "package.json"));
      const Vue = require("vue");
      const compiler = require("vue-template-compiler");
      const renderer = require("vue-server-renderer").createRenderer();
      assert.equal(Vue.version, version);
      assert.equal(require("vue-template-compiler/package.json").version, version);
      assert.equal(require("vue-server-renderer/package.json").version, version);
      function inventory(directory) {
        return readdirSync(directory, { withFileTypes: true })
          .sort((a, b) => a.name.localeCompare(b.name))
          .flatMap((entry) => {
            if (entry.name === "node_modules") return [];
            const path = join(directory, entry.name);
            if (entry.isDirectory()) return inventory(path);
            assert.equal(entry.isFile(), true);
            const bytes = readFileSync(path);
            return [{ path, bytes: bytes.length, sha256: hash(bytes) }];
          });
      }
      const identities = Object.fromEntries(
        ["vue", "vue-template-compiler", "vue-server-renderer"].map((dependency) => {
          const path = require.resolve(`${dependency}/package.json`);
          return [
            dependency,
            { path, files: inventory(dirname(path)), version: require(path).version },
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
