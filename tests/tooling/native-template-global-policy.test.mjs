import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";

const proof = process.env.VIZE_TEMPLATE_GLOBAL_RUNTIME_PROOF;
const selected = Boolean(proof) || process.env.VIZE_TEMPLATE_GLOBAL_RUNTIME_REQUIRED === "1";

await test(
  "template globals match the actual Vue compiler and runtime in both setup modes",
  { skip: !selected },
  async () => {
    assert.ok(proof, "required execution retains every complete packet");
    assert.ok(process.env.VIZE_TEMPLATE_GLOBAL_ORACLE_PACKAGE);
    assert.ok(process.env.VIZE_TEMPLATE_GLOBAL_EXPECTED_TOKEN);
    assert.equal(
      process.env.NAPI_RS_NATIVE_LIBRARY_PATH,
      undefined,
      "public baseline cannot load a source override",
    );
    mkdirSync(proof, { recursive: true });
    const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
    const { Window } = await import(fromUi.resolve("happy-dom"));
    const window = new Window();
    for (const name of [
      "window",
      "document",
      "Document",
      "Node",
      "Text",
      "Comment",
      "Element",
      "HTMLElement",
      "SVGElement",
      "Event",
      "MouseEvent",
      "ShadowRoot",
    ])
      globalThis[name] = name === "window" ? window : window[name];
    // Runtime-dom must first load after the real DOM globals have been registered.
    const { stockSfc, stableProvider } = await import("./support/template-global-stock.mjs");
    const { loadWholeComponent, executeWholeComponent } =
      await import("./support/template-global-runtime.mjs");
    const require = createRequire(import.meta.url);
    const candidate = require("../../npm/native/index.js");
    const before = stableProvider("@vizejs/native");
    const { getBindingIdentity } = require("../../npm/native/native-binding-identity.js");
    const identities = {
      candidate: getBindingIdentity(candidate),
      before: getBindingIdentity(before),
    };
    assert.equal(identities.candidate.kind, "native");
    assert.equal(identities.before.kind, "native");
    assert.equal(candidate.getNativeBuildToken(), process.env.VIZE_TEMPLATE_GLOBAL_EXPECTED_TOKEN);
    assert.equal(identities.candidate.token, candidate.getNativeBuildToken());
    assert.equal(identities.before.token, before.getNativeBuildToken());
    assert.notEqual(identities.candidate.token, identities.before.token);
    assert.equal(
      JSON.parse(
        readFileSync(
          join(
            dirname(process.env.VIZE_TEMPLATE_GLOBAL_ORACLE_PACKAGE),
            "node_modules/vize/package.json",
          ),
          "utf8",
        ),
      ).version,
      "0.600.0",
    );
    const nativeBinary = new URL(
      "../../npm/native/vize-vitrine.linux-x64-gnu.node",
      import.meta.url,
    );
    const receipt = {
      source: process.env.VIZE_TEMPLATE_GLOBAL_SOURCE_SHA,
      identities,
      candidateBinary: {
        filename: nativeBinary.pathname,
        bytes: readFileSync(nativeBinary).length,
        sha256: createHash("sha256").update(readFileSync(nativeBinary)).digest("hex"),
      },
      oracleLock: JSON.parse(
        readFileSync(
          join(dirname(process.env.VIZE_TEMPLATE_GLOBAL_ORACLE_PACKAGE), "package-lock.json"),
          "utf8",
        ),
      ),
    };
    writeFileSync(
      join(proof, "source-and-providers.json"),
      JSON.stringify(receipt, null, 2) + "\n",
    );

    const script = `const emit = defineEmits<{ press: [] }>();`;
    const component = (setup, template) =>
      `<script setup lang="ts">\n${script}\n${setup}\n</script>\n<template>${template}</template>`;
    const cases = [
      {
        id: "focus",
        source: component(
          "",
          `<button @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
        ),
        missing: true,
        beforeEmits: 1,
      },
      {
        id: "explicit",
        source: component(
          "",
          `<button @click="(event: MouseEvent) => { if (globalThis.document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
        ),
        missing: true,
        beforeEmits: 1,
      },
      {
        id: "allowed",
        source: component(
          "",
          `<button @click="(event: MouseEvent) => { if (Math.max(1, 2) !== 2 || Intl.NumberFormat('en-US').format(2) !== '2') return; emit('press'); }">press</button>`,
        ),
        emits: 1,
      },
      {
        id: "callback",
        source: component(
          "",
          `<button @click="(document: MouseEvent) => { if (!document.currentTarget) return; emit('press'); }">press</button>`,
        ),
        emits: 1,
      },
      {
        id: "setup",
        source: component(
          "const document = { activeElement: null };",
          `<button @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
        ),
        emits: 0,
      },
      {
        id: "loop",
        source: component(
          "",
          `<div><button v-for="document in [{ activeElement: null }]" @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button></div>`,
        ),
        emits: 0,
      },
      {
        id: "render-unknown",
        source: component("", `<p>{{ document.activeElement }}{{ Math.max(1, 2) }}</p>`),
        missing: true,
        render: true,
      },
      {
        id: "render-setup",
        source: component(
          "const document = { activeElement: 'setup' };",
          `<p>{{ document.activeElement }}{{ Math.max(1, 2) }}</p>`,
        ),
        emits: 0,
        render: true,
        html: "<p>setup2</p>",
      },
    ];
    const rows = [];
    const persist = () =>
      writeFileSync(join(proof, "whole-observations.json"), JSON.stringify(rows, null, 2) + "\n");
    try {
      for (const input of cases)
        for (const target of ["dom", "vapor", "ssr"])
          for (const inlineTemplate of [false, true]) {
            const filename = `/template-global-policy/${input.id}-${target}-${inlineTemplate}.vue`;
            const options = {
              filename,
              inlineTemplate,
              sourceMap: true,
              isTs: false,
              isProd: inlineTemplate,
              ...(target === "ssr" ? { ssr: true } : target === "vapor" ? { vapor: true } : {}),
            };
            const row = {
              id: input.id,
              source: input.source,
              target,
              inlineTemplate,
              options,
              producers: {},
            };
            rows.push(row);
            persist();
            for (const [name, producer] of [
              ["before", before],
              ["candidate", candidate],
              ["stock", null],
            ]) {
              const packet =
                name === "stock"
                  ? await stockSfc(input.source, filename, target, inlineTemplate)
                  : producer.compileSfc(input.source, options);
              row.producers[name] = { packet };
              persist();
              if (name !== "stock")
                assert.deepEqual(
                  packet.errors,
                  [],
                  `${input.id}/${target}/${inlineTemplate}/${name} compiles`,
                );
              try {
                const loaded = await loadWholeComponent(packet, target);
                row.producers[name].resolvedWholeCode = loaded.resolvedCode;
                row.producers[name].runtime = await executeWholeComponent(
                  loaded,
                  target,
                  !input.render && target !== "ssr",
                );
              } catch (error) {
                row.producers[name].runtime = {
                  emitted: 0,
                  errors: [{ name: error.name, message: error.message, stack: error.stack }],
                  phase: "module-or-runtime",
                };
                persist();
                if (name !== "before") throw error;
              }
              persist();
            }
            const actual = row.producers.candidate.runtime;
            const expected = row.producers.stock.runtime;
            if (input.missing && (input.render || target !== "ssr")) {
              for (const [owner, result] of [
                ["candidate", actual],
                ["stock", expected],
              ]) {
                assert.ok(
                  result.errors.some((error) => error.name === "TypeError"),
                  `${input.id}/${target}/${inlineTemplate}/${owner} observes the missing component property`,
                );
                assert.equal(result.emitted, 0);
              }
            } else {
              assert.deepEqual(
                actual.errors,
                [],
                `${input.id}/${target}/${inlineTemplate} native runtime`,
              );
              assert.deepEqual(
                expected.errors,
                [],
                `${input.id}/${target}/${inlineTemplate} Vue runtime`,
              );
              assert.equal(actual.emitted, target === "ssr" ? 0 : (input.emits ?? 0));
              assert.equal(actual.emitted, expected.emitted);
              if (input.html) {
                assert.equal(actual.html, input.html);
                assert.equal(expected.html, input.html);
              }
            }
            if (target === "dom" && input.beforeEmits !== undefined) {
              assert.deepEqual(
                row.producers.before.runtime.errors,
                [],
                "public 0.600 reproduces the host-global behavior",
              );
              assert.equal(row.producers.before.runtime.emitted, input.beforeEmits);
            }
          }
      assert.equal(rows.length, 48);
      assert.ok(
        rows.every(
          (row) =>
            Object.values(row.producers).length === 3 &&
            Object.values(row.producers).every((value) => value.runtime),
        ),
      );
    } finally {
      persist();
      await window.happyDOM.close();
    }
  },
);
