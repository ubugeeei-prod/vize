import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = fileURLToPath(
  new URL("../_fixtures/differential/lint/overwritten-child-content-8285/", import.meta.url),
);
const req = createRequire(new URL("../../tools/benchmarks/scripts/package.json", import.meta.url));
const read = (...parts) => JSON.parse(readFileSync(join(root, ...parts), "utf8"));
const ids = read("controls.json");
const config = read("config.json");
const identity = read("independent", "providers.json");

function authoredFilenameOnly(results, id) {
  assert.equal(results.length, 1, id);
  assert.equal(results[0].filePath, join(root, `${id}.vue`), id);
  return results.map((result) => ({ ...result, filePath: `${id}.vue` }));
}

test("all36 whole official Vue-base packets reproduce twice with all51 explicit rules", async () => {
  assert.equal(ids.length, 36);
  assert.equal(new Set(ids).size, ids.length);
  assert.equal(Object.keys(config.linter.rules).length, 51);
  const providers = Object.fromEntries(
    Object.keys(identity.providers).map((name) => [
      name,
      {
        version: req(`${name}/package.json`).version,
        sha256: createHash("sha256")
          .update(readFileSync(req.resolve(name)))
          .digest("hex"),
      },
    ]),
  );
  assert.deepEqual(providers, identity.providers);
  const { ESLint } = req("eslint");
  const vue = req("eslint-plugin-vue");
  const rules = Object.fromEntries(
    Object.entries(config.oracleRules).map(([native, oracle]) => [
      oracle,
      [
        "error",
        ...(config.linter.ruleOptions[native] === undefined
          ? []
          : [config.linter.ruleOptions[native]]),
      ],
    ]),
  );
  rules["vue/component-name-in-template-casing"] = [
    "error",
    config.linter.ruleOptions["vue/component-name-in-template-casing"].casing,
  ];
  assert.deepEqual(rules, identity.rules);
  const eslint = new ESLint({
    cwd: root,
    overrideConfigFile: true,
    ignore: false,
    overrideConfig: [
      ...vue.configs["flat/base"],
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: req("vue-eslint-parser"),
          parserOptions: {
            parser: req("@typescript-eslint/parser"),
            ecmaVersion: "latest",
            sourceType: "module",
          },
        },
        plugins: { vue },
        rules,
      },
    ],
  });
  const packets = [];
  for (const id of ids) {
    const control = read("controls", `${id}.json`);
    assert.equal(control.id, id);
    const observations = [];
    for (let repeat = 0; repeat < 2; repeat++) {
      try {
        observations.push({
          ok: true,
          result: await eslint.lintText(control.source, { filePath: `${id}.vue` }),
        });
      } catch (error) {
        observations.push({
          ok: false,
          error: { name: error.name, message: error.message, stack: error.stack },
        });
      }
    }
    packets.push({ id, observations });
  }
  const artifact = fileURLToPath(
    new URL("../../target/tooling/overwritten-child-content-8285/", import.meta.url),
  );
  mkdirSync(artifact, { recursive: true });
  writeFileSync(
    join(artifact, "whole-provider.json"),
    `${JSON.stringify({ providers, rules, packets }, null, 2)}\n`,
  );
  for (const { id, observations } of packets) {
    for (const observation of observations) {
      assert.equal(observation.ok, true, JSON.stringify(observation));
      assert.deepEqual(
        authoredFilenameOnly(observation.result, id),
        read("independent", `${id}.json`).result,
        id,
      );
    }
    assert.deepEqual(observations[0], observations[1], id);
  }
});

test("the whole historical delta introduces only12 owned comment findings", () => {
  const changed = [];
  for (const id of ids) {
    const before = read("source-before", `${id}.json`);
    const after = read("native", `${id}.json`);
    if (JSON.stringify(before) === JSON.stringify(after)) continue;
    changed.push(id);
    assert.match(id, /-(?:comment|empty-comment|comment-whitespace)-/);
    assert.equal(
      before.api.diagnostics.some((d) => d.rule === "vue/no-child-content"),
      false,
    );
    const expectedDelta = structuredClone(after);
    expectedDelta.api.diagnostics = expectedDelta.api.diagnostics.filter(
      (d) => d.rule !== "vue/no-child-content",
    );
    expectedDelta.api.errorCount--;
    expectedDelta.packet[0].messages = expectedDelta.packet[0].messages.filter(
      (m) => m.ruleId !== "vue/no-child-content",
    );
    expectedDelta.packet[0].errorCount--;
    assert.deepEqual(expectedDelta, before, id);
  }
  assert.equal(changed.length, 12);
});
