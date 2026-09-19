import assert from "node:assert/strict";
import { createRequire } from "node:module";
import process from "node:process";
import { build } from "vite-plus";
import { chromium } from "@playwright/test";
import { mountEventFixture } from "./davinci-event-browser.mjs";

const require = createRequire(import.meta.url);
const stable = require("@vue/compiler-dom");
const sfc = require("@vue/compiler-sfc");
const beta = createRequire(require.resolve("@vue/compiler-sfc"))("@vue/compiler-dom");
const versions = {
  stable: require("@vue/compiler-dom/package.json").version,
  beta: require("@vue/compiler-sfc/package.json").version,
};
assert.equal(require("@vue/runtime-dom/package.json").version, versions.stable);
assert.equal(require("vue/package.json").version, versions.beta);

async function runtimeUrl(entry) {
  const result = await build({
    configFile: false,
    logLevel: "silent",
    define: {
      "process.env.NODE_ENV": '"development"',
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
    build: { write: false, minify: false, lib: { entry, formats: ["es"] } },
  });
  const outputs = (Array.isArray(result) ? result : [result]).flatMap((item) => item.output);
  assert.equal(outputs.length, 1, "self-contained Vue runtime");
  return `data:text/javascript;base64,${Buffer.from(outputs[0].code).toString("base64")}`;
}

const runtimes = {
  stable: await runtimeUrl(require.resolve("@vue/runtime-dom/dist/runtime-dom.esm-bundler.js")),
  beta: await runtimeUrl(require.resolve("vue/dist/vue.runtime.esm-bundler.js")),
};
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const { fixtures } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(fixtures.length > 0, "Rust must supply executable scenarios");
const browser = await chromium.launch({ headless: true });
const failures = [];
let runs = 0;
let detectedMutations = 0;
const knownReferenceDifferences = [];
let nativeJavaScriptOracles = 0;
try {
  for (const fixture of fixtures) {
    const options = { mode: "module", prefixIdentifiers: true };
    const vapor = sfc.compileTemplate({
      id: "event-contract",
      filename: "Event.vue",
      source: fixture.source,
      vapor: true,
    });
    assert.deepEqual(vapor.errors, [], "official Vapor compilation");
    const engines = [
      {
        name: "Vue 3.5 VDOM",
        backend: "vdom",
        code: stable.compile(fixture.source, options).code,
        runtime: "stable",
        official: true,
      },
      {
        name: "Vue 3.6 VDOM",
        backend: "vdom",
        code: beta.compile(fixture.source, options).code,
        runtime: "beta",
        official: true,
      },
      {
        name: "Vue 3.6 Vapor",
        backend: "vapor",
        code: vapor.code,
        runtime: "beta",
        official: true,
      },
      ...fixture.compiled.map(({ backend, mode, code }) => ({
        name: `Vize ${backend}/${mode}`,
        backend,
        code,
        runtime: "beta",
      })),
    ];
    if (fixture.referenceExpected) {
      // Execute authored JavaScript without any compiler rewrite for every
      // claimed upstream difference. The fixture explicitly owns body wrapping.
      const body = `with (_ctx) { return (${fixture.nativeHandler}) }`;
      engines.push({
        name: "Native JavaScript",
        backend: "vdom",
        runtime: "beta",
        code: `import { h } from 'vue';
          const handler = Function('_ctx', ${JSON.stringify(body)});
          export function render(_ctx) {
            return h('button', { id: 'target', onClick: handler(_ctx) }, String(_ctx.count));
          }`,
      });
      nativeJavaScriptOracles++;
    }
    // Rust owns and compiles the valid mutant programs. Their mounted traces
    // must differ, regardless of generated whitespace or printer formatting.
    if (fixture.mutantCode) {
      const base = engines.find((engine) => engine.name === "Vize vapor/prefix");
      const code = fixture.mutantCode;
      assert.notEqual(code, base.code, "mutation must change the compiled artifact");
      engines.push({ ...base, name: `${base.name} mutation`, code, mutant: true });
    }
    for (const engine of engines) {
      const page = await browser.newPage();
      const pageErrors = [];
      page.on("pageerror", (error) => pageErrors.push(error.message));
      try {
        const actual = [
          await page.evaluate(mountEventFixture, {
            code: engine.code,
            runtimeUrl: runtimes[engine.runtime],
            backend: engine.backend,
            fixture,
          }),
        ];
        assert.deepEqual(actual[0], fixture.expected[engine.backend][0], "mount contract");
        for (const phase of ["click-a", "replace", "click-b", "unmount"]) {
          if (phase.startsWith("click-")) {
            await page.evaluate((phase) => window.eventFixture.step(phase), phase);
            await page.locator("#target").click();
          }
          actual.push(await page.evaluate((phase) => window.eventFixture.step(phase), phase));
        }
        assert.deepEqual(pageErrors, [], "unhandled browser errors");
        if (engine.mutant) {
          assert.notDeepEqual(
            actual,
            fixture.expected[engine.backend],
            "oracle accepted a broken handler",
          );
          detectedMutations++;
        } else if (engine.official && fixture.referenceExpected) {
          assert.deepEqual(
            actual,
            fixture.referenceExpected[engine.backend],
            "pinned upstream difference changed",
          );
          assert.notDeepEqual(
            actual,
            fixture.expected[engine.backend],
            "known difference is stale",
          );
          knownReferenceDifferences.push({
            fixture: fixture.name,
            engine: engine.name,
            code: engine.code,
            actual,
          });
          runs++;
        } else {
          assert.deepEqual(actual, fixture.expected[engine.backend]);
          runs++;
        }
      } catch (error) {
        failures.push({
          fixture: fixture.name,
          engine: engine.name,
          error: error.message,
          code: engine.code,
        });
      } finally {
        await page.close();
      }
    }
  }
} finally {
  await browser.close();
}
if (failures.length) {
  console.error(JSON.stringify(failures, null, 2));
  process.exitCode = 1;
} else {
  assert.equal(detectedMutations, 5, "all event contract mutations must execute");
  assert.equal(nativeJavaScriptOracles, 5, "each known difference has a native JavaScript oracle");
  assert.equal(
    knownReferenceDifferences.length,
    15,
    "track the five explicit scope differences in all three pinned compilers",
  );
  console.log(
    JSON.stringify({
      versions,
      scenarios: fixtures.length,
      mountedRuns: runs,
      detectedMutations,
      nativeJavaScriptOracles,
      knownReferenceDifferences,
    }),
  );
}
