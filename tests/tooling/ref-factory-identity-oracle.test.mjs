import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/ref-factory-identity-8275");
const read = (file) => JSON.parse(readFileSync(path.join(fixture, file), "utf8"));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

// Recording normalizes only the provider's ephemeral physical filename. Every
// message, source, suggestion, fix, suppressed finding, and count stays intact.
function recordedPaths(results, id) {
  return results.map((result) => ({ ...result, filePath: `oracle/${id}.vue` }));
}

await test("Vue ref identity controls retain the complete independent 51-rule packets", async () => {
  const config = read("config.json");
  const golden = read("independent.json");
  assert.equal(Object.keys(config.linter.rules).length, 51);
  assert.deepEqual(Object.keys(config.oracleRules).sort(), Object.keys(config.linter.rules).sort());
  assert.equal(new Set(Object.values(config.oracleRules)).size, 51);
  assert.equal(golden.cases.length, 37);
  const manifest =
    process.env.VIZE_REF_ORACLE_MANIFEST ??
    path.join(root, "tools/benchmarks/scripts/package.json");
  const require = createRequire(manifest);
  const { ESLint } = require("eslint");
  const vue = require("eslint-plugin-vue");
  const parser = require("vue-eslint-parser");
  const ts = require("@typescript-eslint/parser");
  const providers = Object.fromEntries(
    Object.entries(golden.providers).map(([name, pin]) => {
      const entry = require.resolve(name);
      const identity = {
        entry,
        version: require(`${name}/package.json`).version,
        sha256: sha256(readFileSync(entry)),
      };
      assert.equal(identity.version, pin.version, name);
      assert.equal(identity.sha256, pin.sha256, name);
      return [name, identity];
    }),
  );
  const rules = Object.fromEntries(
    Object.entries(config.oracleRules).map(([source, oracle]) => {
      const option = config.linter.ruleOptions[source];
      return [oracle, ["error", ...(option === undefined ? [] : [option])]];
    }),
  );
  rules["vue/component-name-in-template-casing"] = [
    "error",
    config.linter.ruleOptions["vue/component-name-in-template-casing"].casing,
  ];
  assert.deepEqual(rules, golden.rules);
  const eslint = new ESLint({
    cwd: root,
    overrideConfigFile: true,
    ignore: false,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser,
          parserOptions: { parser: ts, ecmaVersion: "latest", sourceType: "module" },
        },
        plugins: { vue },
        rules,
      },
    ],
  });
  const artifact = path.join(root, "target/differential/ref-factory-identity-oracle.json");
  mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.ref-factory-identity.independent-execution",
    issue: 8275,
    node: process.version,
    providers,
    rules,
    qualified: 0,
    cases: [],
  };
  const persist = () => writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  persist();
  try {
    for (const item of golden.cases) {
      const observations = [];
      const recorded = {
        id: item.id,
        source: item.source,
        inputSha256: sha256(item.source),
        observations,
      };
      evidence.cases.push(recorded);
      for (let repeat = 0; repeat < 2; repeat++) {
        const packet = await eslint.lintText(item.source, { filePath: `oracle/${item.id}.vue` });
        observations.push(packet);
        persist();
        assert.deepEqual(recordedPaths(packet, item.id), item.observations[repeat], item.id);
      }
      assert.deepEqual(observations[0], observations[1], `repeat ${item.id}`);
      evidence.qualified++;
      persist();
    }
    assert.equal(evidence.qualified, 37);
  } finally {
    persist();
  }
});
