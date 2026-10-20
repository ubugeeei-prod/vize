import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { inlineVuePluginFixtureOptions } from "./support/vue-config-fixture.ts";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const directory = new URL(
  "../_fixtures/differential/compiler/nuxt-compiler-defaults/",
  import.meta.url,
);
const corpus = JSON.parse(fs.readFileSync(new URL("corpus.json", directory), "utf8"));
const bytes = (name) => fs.readFileSync(new URL(name, directory));

await test("#7959 retains all five complete reporter code blocks and explicit derivative custody", () => {
  for (const [name, pin] of Object.entries(corpus.files)) {
    assert.equal(bytes(name).length, pin.bytes);
    assert.equal(createHash("sha256").update(bytes(name)).digest("hex"), pin.sha256);
  }
  const blocks = [
    ...bytes("original-issue.md.txt")
      .toString()
      .matchAll(/```(?:ts|json|vue|js)\n([\s\S]*?)```/g),
  ].map((match) => match[1]);
  assert.equal(blocks.length, 5);
  assert.deepEqual(
    [
      "nuxt.config.ts.txt",
      "vize.config.json.txt",
      "TightRow.vue.txt",
      "app.vue.txt",
      "check.mjs.txt",
    ].map((name) => bytes(name).toString()),
    blocks,
  );
  assert.equal(bytes("LooseRow.vue.txt").toString(), blocks[2].replaceAll("tight-", "loose-"));
  assert.equal(
    corpus.reporter.coauthor,
    "Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  );
  assert.deepEqual(
    corpus.cohorts.map((cohort) => [cohort.nuxt, cohort.vue]),
    [
      ["3.19.3", "3.5.43"],
      ["4.5.2", "3.5.43"],
    ],
  );
});

await test("authored complete render matrix distinguishes inherited defaults from authored precedence", () => {
  assert.deepEqual(
    corpus.scenarios.map((scenario) => [scenario.id, scenario.expected]),
    [
      ["original-no-forward", ["condense", "preserve", "preserve"]],
      ["original-forward", ["condense", "preserve", "preserve"]],
      ["unconfigured-nuxt-default", ["preserve", "preserve", "preserve"]],
      ["unconfigured-core-default", ["condense", "condense", "condense"]],
      ["top-level-overrides-default", ["condense", "condense", "condense"]],
      ["explicit-module-overrides-entry", ["preserve", "preserve", "preserve"]],
      ["explicit-template-overrides-entry", ["preserve", "preserve", "preserve"]],
      ["project-overrides-condensed-nuxt", ["condense", "preserve", "preserve"]],
    ],
  );
  for (const scenario of corpus.scenarios) {
    const row = (name, space) =>
      `<p class="${name}-row"><span>${name}-alpha</span>${space}<span>${name}-beta</span></p>`;
    const html = `<main>${row("tight", scenario.expected[0] === "preserve" ? " " : "")}${row("loose", scenario.expected[1] === "preserve" ? " " : "")}</main>`;
    assert.equal(scenario.html, html);
    assert.notEqual(scenario.html, html.replace("tight-beta", "omitted"));
  }
  assert.equal(corpus.referenceAuthority.limits.fullNuxtGenerate, false);
  assert.equal(corpus.referenceAuthority.limits.nativeMigrationCredit, false);
});

await test("the current Nuxt probe carries every authored scoped compiler policy through inline plugin config", async () => {
  const original = bytes("vize.config.json.txt");
  const frozen = Buffer.from(original);
  for (const scenario of corpus.scenarios) {
    const authored = structuredClone(scenario);
    const document = scenario.config === "original" ? original : scenario.config;
    const options = inlineVuePluginFixtureOptions(scenario.module, document);
    assert.deepEqual(options, {
      ...scenario.module,
      ...(document === null
        ? {}
        : { config: document === original ? JSON.parse(original) : document }),
    });
    assert.deepEqual(scenario, authored);
  }
  const originalOptions = inlineVuePluginFixtureOptions({}, original);
  assert.deepEqual(originalOptions.config, {
    compiler: { whitespace: "preserve" },
    entries: [
      {
        files: ["app/components/tight/**", "node_modules/**"],
        compiler: { whitespace: "condense" },
      },
    ],
  });
  // This fixture must use the existing per-file JavaScript compiler API. The
  // native canonical schema must continue to reject unsupported compiler scopes.
  await assert.rejects(
    projectViteConfigSource({ vue: originalOptions.config }),
    /vue\.entries\[0\]: unknown option "compiler"/u,
  );
  assert.deepEqual(original, frozen);
  assert.deepEqual(bytes("vize.config.json.txt"), frozen);
  const activeProbe = fs.readFileSync(
    new URL("../../tools/support/compat/nuxt/compiler-defaults-probe.ts", import.meta.url),
    "utf8",
  );
  assert.ok(activeProbe.includes("inlineVuePluginFixtureOptions(scenario.module, shared)"));
  assert.ok(activeProbe.includes("vize: { compiler, lint: false, checker: false, musea: false }"));
  assert.ok(!activeProbe.includes('const configFile = path.join(project, "vize.config.json")'));
});
