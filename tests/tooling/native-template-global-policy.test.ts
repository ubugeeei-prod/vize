import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import {
  asArray,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
} from "../../tools/support/json-value.ts";
import type { Target, WholePacket } from "./support/template-global-stock.ts";
import type { RuntimeResult } from "./support/template-global-runtime.ts";
import type { SfcCompileOptionsNapi } from "../../npm/native/index.js";
type Native = typeof import("../../npm/native/index.js");
type ProducerName = "before" | "candidate" | "stock";
type ProducerResult = { packet: WholePacket; runtime?: RuntimeResult; resolvedWholeCode?: string };
type Row = {
  id: string;
  source: string;
  target: Target;
  inlineTemplate: boolean;
  options: SfcCompileOptionsNapi;
  authoredModules?: Readonly<Record<string, string>>;
  producers: Partial<Record<ProducerName, ProducerResult>>;
};
import { templateGlobalCases as cases } from "./support/template-global-inputs.ts";

const proof = process.env.VIZE_TEMPLATE_GLOBAL_RUNTIME_PROOF;
const selected = Boolean(proof) || process.env.VIZE_TEMPLATE_GLOBAL_RUNTIME_REQUIRED === "1";

await test(
  "Vue 3.5.41 template globals hold across native targets and both setup modes",
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
      Reflect.set(globalThis, name, name === "window" ? window : Reflect.get(window, name));
    // Runtime-dom must first load after the real DOM globals have been registered.
    const { stockSfc, stableProvider } = await import("./support/template-global-stock.ts");
    const { loadWholeComponent, executeWholeComponent } =
      await import("./support/template-global-runtime.ts");
    const require = createRequire(import.meta.url);
    const candidate: Native = require("../../npm/native/index.js");
    const before: Native = stableProvider("@vizejs/native");
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
      policy: {
        authority: "Vue 3.5.41 DOM/SFC",
        nativeTargets: "same explicit compiler policy",
        vaporReference: "independent RC9 observation; not native parity",
      },
      publicBinary: {
        filename: stableProvider.resolve("@vizejs/native-linux-x64-gnu"),
        sha256: createHash("sha256")
          .update(readFileSync(stableProvider.resolve("@vizejs/native-linux-x64-gnu")))
          .digest("hex"),
      },
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

    const vaporOracle = readJsonRecord(
      new URL(
        "../_fixtures/differential/compiler/template-global-policy/stock-vapor-rc9-runtime.json",
        import.meta.url,
      ),
    );
    assert.equal(vaporOracle.compiler, "3.6.0-rc.9");
    assert.equal(vaporOracle.plugin, "6.0.7");
    const originalVaporObservations = asArray(vaporOracle.observations).map((value) => {
      const record = asRecord(value);
      assert.ok(typeof record.inline === "boolean");
      return { ...record, id: asString(record.id), inline: record.inline };
    });
    assert.equal(originalVaporObservations.length, 16);
    assert.equal(
      new Set(originalVaporObservations.map((value) => value.id + "/" + String(value.inline))).size,
      16,
    );
    const importOracle = readJsonRecord(
      new URL(
        "../_fixtures/differential/compiler/template-global-policy/stock-vapor-rc9-import-runtime.json",
        import.meta.url,
      ),
    );
    assert.equal(importOracle.compiler, "3.6.0-rc.9");
    assert.equal(importOracle.plugin, "6.0.7");
    const importObservations = asArray(importOracle.observations).map((value) => {
      const record = asRecord(value);
      assert.ok(typeof record.inline === "boolean");
      return { ...record, id: asString(record.id), inline: record.inline };
    });
    assert.equal(importObservations.length, 2);
    const rawImport = gunzipSync(
      Buffer.from(
        readFileSync(
          new URL(
            "../_fixtures/differential/compiler/template-global-policy/stock-import-reference.json.gz.b64.txt",
            import.meta.url,
          ),
          "utf8",
        ).trim(),
        "base64",
      ),
    );
    assert.equal(
      createHash("sha256").update(rawImport).digest("hex"),
      asString(asRecord(importOracle.archive).rawSha256),
    );
    assert.equal(asArray(asRecord(parseJson(rawImport.toString("utf8"))).rows).length, 6);
    const vaporObservations = [...originalVaporObservations, ...importObservations];
    assert.equal(
      new Set(vaporObservations.map((value) => value.id + "/" + String(value.inline))).size,
      18,
    );
    const rows: Row[] = [];
    const persist = () =>
      writeFileSync(join(proof, "whole-observations.json"), JSON.stringify(rows, null, 2) + "\n");
    try {
      for (const input of cases)
        for (const target of ["dom", "vapor", "ssr"] as const)
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
            const row: Row = {
              id: input.id,
              source: input.source,
              target,
              inlineTemplate,
              options,
              authoredModules: input.authoredModules,
              producers: {},
            };
            rows.push(row);
            persist();
            const producers: [ProducerName, Native | null][] = [
              ["before", before],
              ["candidate", candidate],
              ["stock", null],
            ];
            for (const [name, producer] of producers) {
              const packet =
                name === "stock"
                  ? await stockSfc(input.source, filename, target, inlineTemplate)
                  : producer!.compileSfc(input.source, options);
              const observation: ProducerResult = { packet };
              row.producers[name] = observation;
              persist();
              if (name !== "stock")
                assert.deepEqual(
                  packet.errors,
                  [],
                  `${input.id}/${target}/${inlineTemplate}/${name} compiles`,
                );
              try {
                const loaded = await loadWholeComponent(packet, target, input.authoredModules);
                observation.resolvedWholeCode = loaded.resolvedCode;
                observation.runtime = await executeWholeComponent(
                  loaded,
                  target,
                  !input.render && target !== "ssr",
                );
              } catch (error) {
                assert.ok(error instanceof Error);
                observation.runtime = {
                  html: null,
                  warnings: [],
                  emitted: 0,
                  errors: [
                    { name: error.name, message: error.message, stack: error.stack, info: null },
                  ],
                  phase: "module-or-runtime",
                };
                persist();
                if (name !== "before") throw error;
              }
              persist();
            }
            const actual = row.producers.candidate?.runtime;
            assert.ok(actual);
            const expected = row.producers.stock?.runtime;
            assert.ok(expected);
            if (input.missing && (input.render || target !== "ssr")) {
              const results: [string, RuntimeResult][] =
                target === "vapor"
                  ? [["candidate", actual]]
                  : [
                      ["candidate", actual],
                      ["stock", expected],
                    ];
              for (const [owner, result] of results) {
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
              if (target !== "vapor")
                assert.deepEqual(
                  expected.errors,
                  [],
                  `${input.id}/${target}/${inlineTemplate} Vue runtime`,
                );
              assert.equal(actual.emitted, target === "ssr" ? 0 : (input.emits ?? 0));
              if (target !== "vapor") assert.equal(actual.emitted, expected.emitted);
              if (input.html) {
                assert.equal(actual.html, input.html);
                if (target !== "vapor") assert.equal(expected.html, input.html);
              }
            }
            if (target === "vapor") {
              // This independently freezes what the actual RC9 compiler/runtime does.
              // Native Vapor is checked above against our explicit 3.5.41 policy.
              const observation = vaporObservations.find(
                (value) => value.id === input.id && value.inline === inlineTemplate,
              );
              assert.ok(observation);
              assert.deepEqual(
                {
                  id: input.id,
                  inline: inlineTemplate,
                  html: expected.html,
                  emitted: expected.emitted,
                  errors: expected.errors.map(({ name, message }) => ({ name, message })),
                },
                observation,
                "the upstream RC9 divergence remains an independent observation",
              );
            }
            if (target === "dom" && input.beforeEmits !== undefined) {
              assert.deepEqual(
                row.producers.before!.runtime!.errors,
                [],
                "public 0.600 reproduces the host-global behavior",
              );
              assert.equal(row.producers.before!.runtime!.emitted, input.beforeEmits);
            }
          }
      assert.equal(rows.length, 54);
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
