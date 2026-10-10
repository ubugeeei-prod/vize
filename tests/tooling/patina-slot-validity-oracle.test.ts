import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

interface Input {
  id: string;
  filename: string;
  source: string;
  sourceSha256: string;
}
interface ProviderPin {
  version: string;
  entrySha256: string;
}
interface Packet {
  filePath: string;
  [field: string]: unknown;
}
interface Capture {
  id: string;
  observations: Packet[][];
}
interface Projection {
  linter: { rules: Record<string, string>; ruleOptions: Record<string, unknown> };
  oracleRules: Record<string, string>;
}
interface Oracle {
  providers: Record<string, ProviderPin>;
  mandatoryInfrastructure: { name: string; processor: string; rules: Record<string, string> };
  records: {
    allowModifiers: boolean;
    configuredRules: Record<string, unknown>;
    cases: Capture[];
  }[];
}

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/slot-directive-validity-8142");
const read = (file: string) => JSON.parse(readFileSync(path.join(fixture, file), "utf8"));
const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

await test("slot-validity controls keep complete pinned official Vue-base 51-rule observations", async () => {
  const inputs: Input[] = read("cases.json").cases;
  const config: Projection = read("config.json");
  const golden: Oracle = read("independent.json");
  assert.equal(inputs.length, 47);
  assert.equal(Object.keys(config.linter.rules).length, 51);
  assert.equal(Object.keys(config.linter.ruleOptions).length, 3);
  assert.deepEqual(Object.keys(config.oracleRules).sort(), Object.keys(config.linter.rules).sort());
  assert.equal(new Set(Object.values(config.oracleRules)).size, 51);
  const manifest =
    process.env.VIZE_SLOT_VALIDITY_ORACLE_MANIFEST ??
    path.join(root, "tools/benchmarks/scripts/package.json");
  const req = createRequire(manifest);
  const artifact = path.join(root, "target/differential/slot-validity-oracle.json");
  mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence: {
    providers: Record<string, ProviderPin>;
    captures: unknown[];
    qualified: number;
  } = {
    providers: {},
    captures: [],
    qualified: 0,
  };
  const persist = () => writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  persist();
  try {
    for (const [name, pin] of Object.entries(golden.providers)) {
      const entry = req.resolve(name);
      const identity = {
        version: req(name + "/package.json").version,
        entrySha256: sha256(readFileSync(entry)),
      };
      evidence.providers[name] = identity;
      persist();
      assert.equal(identity.version, pin.version, name);
      assert.equal(identity.entrySha256, pin.entrySha256, name);
    }
    const { ESLint } = req("eslint");
    const vue = req("eslint-plugin-vue");
    const parser = req("vue-eslint-parser");
    const ts = req("@typescript-eslint/parser");
    const base = vue.configs["flat/base"].find(
      (item: { name: string }) => item.name === "vue/base/setup-for-vue",
    );
    assert.deepEqual(
      { name: base.name, processor: base.processor, rules: base.rules },
      golden.mandatoryInfrastructure,
    );
    const rules: Record<string, unknown> = Object.fromEntries(
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
    const casing = config.linter.ruleOptions["vue/component-name-in-template-casing"] as {
      casing: string;
    };
    rules["vue/component-name-in-template-casing"] = ["error", casing.casing];
    for (const record of golden.records) {
      const configuredRules = record.allowModifiers
        ? { ...rules, "vue/valid-v-slot": ["error", { allowModifiers: true }] }
        : { ...rules };
      assert.deepEqual(configuredRules, record.configuredRules);
      const eslint = new ESLint({
        cwd: root,
        overrideConfigFile: true,
        ignore: false,
        overrideConfig: [
          {
            files: ["**/*.vue"],
            plugins: { vue },
            processor: base.processor,
            languageOptions: {
              parser,
              parserOptions: { parser: ts, ecmaVersion: "latest", sourceType: "module" },
            },
            rules: { ...configuredRules, ...base.rules },
          },
        ],
      });
      for (const input of inputs) {
        assert.equal(sha256(input.source), input.sourceSha256, input.id);
        const expected = record.cases.find((item) => item.id === input.id);
        assert.ok(expected, input.id);
        const observations: Packet[][] = [];
        const capture = { id: input.id, allowModifiers: record.allowModifiers, observations };
        evidence.captures.push(capture);
        for (const repeat of [0, 1]) {
          const actualPath = path.join(fixture, input.filename);
          const raw: Packet[] = await eslint.lintText(input.source, { filePath: actualPath });
          observations.push(raw);
          persist();
          // Only the exact ephemeral recording path changes. All findings,
          // suppressions, suggestions, fixes, source and counts stay intact.
          const normalized = raw.map((packet) => ({
            ...packet,
            filePath: packet.filePath === actualPath ? input.filename : packet.filePath,
          }));
          assert.deepEqual(normalized, expected.observations[repeat], input.id);
        }
        assert.deepEqual(observations[0], observations[1], input.id);
        evidence.qualified += 1;
        persist();
      }
    }
    assert.equal(evidence.qualified, 94);
  } finally {
    persist();
  }
});

await test("slot source custody has different producer binaries and preserves all baseline bytes", () => {
  const custody = read("local-custody.json");
  assert.equal(custody.base, "26031a4fbb30a1e86511919bafc7035b08440ef3");
  assert.notEqual(custody.beforeBinarySha256, custody.afterBinarySha256);
  assert.equal(
    sha256(readFileSync(path.join(fixture, "before-observer.rs.txt"))),
    custody.driverSha256,
  );
  assert.equal(
    sha256(gunzipSync(readFileSync(path.join(fixture, "before-source-inventory.txt.gz")))),
    custody.sourceInventorySha256,
  );
  const corpus = read("cases.json");
  const before = read("source-before.json");
  const after = read("source-after.json");
  assert.equal(before.cases.length, 47);
  assert.equal(after.cases.length, 47);
  for (const input of corpus.cases) {
    const b = before.cases.find((item: Capture) => item.id === input.id);
    const a = after.cases.find((item: Capture) => item.id === input.id);
    assert.deepEqual(b.observations, [input.before, input.before], input.id);
    assert.deepEqual(a.observations, [input.after, input.after], input.id);
  }
});
