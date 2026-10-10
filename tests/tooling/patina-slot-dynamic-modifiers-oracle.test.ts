import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

interface Packet {
  filePath: string;
  [key: string]: unknown;
}
interface Provider {
  version: string;
  entrySha256: string;
}
interface Input {
  id: string;
  source: string;
  sourceSha256: string;
}
const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/slot-dynamic-modifiers-8142");
const read = (file: string) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

await test("both slot modifier policies preserve complete pinned official Vue packets", async () => {
  const inputs: Input[] = read("controls.json").cases;
  const golden: {
    providers: Record<string, Provider>;
    mandatoryInfrastructure: unknown;
    records: { id: string; allowModifiers: boolean; observations: Packet[][] }[];
  } = read("independent.json");
  assert.equal(inputs.length, 12);
  assert.equal(golden.records.length, 24);
  const req = createRequire(
    process.env.VIZE_SLOT_VALIDITY_ORACLE_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const artifact = path.join(root, "target/differential/slot-dynamic-modifiers-oracle.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    providers: {} as Record<string, Provider>,
    captures: [] as unknown[],
    qualified: 0,
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  persist();
  try {
    for (const [name, expected] of Object.entries(golden.providers)) {
      const actual = {
        version: req(name + "/package.json").version,
        entrySha256: sha256(fs.readFileSync(req.resolve(name))),
      };
      evidence.providers[name] = actual;
      persist();
      assert.deepEqual(actual, expected, name);
    }
    const { ESLint } = req("eslint");
    const vue = req("eslint-plugin-vue");
    const base = vue.configs["flat/base"].find(
      (config: { name: string }) => config.name === "vue/base/setup-for-vue",
    );
    assert.deepEqual(
      { name: base.name, processor: base.processor, rules: base.rules },
      golden.mandatoryInfrastructure,
    );
    for (const allowModifiers of [false, true]) {
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
              parser: req("vue-eslint-parser"),
              parserOptions: {
                parser: req("@typescript-eslint/parser"),
                ecmaVersion: "latest",
                sourceType: "module",
              },
            },
            rules: { "vue/valid-v-slot": ["error", { allowModifiers }], ...base.rules },
          },
        ],
      });
      for (const input of inputs) {
        assert.equal(sha256(input.source), input.sourceSha256, input.id);
        const expected = golden.records.find(
          (record) => record.id === input.id && record.allowModifiers === allowModifiers,
        );
        assert.ok(expected, input.id);
        const observations: Packet[][] = [];
        evidence.captures.push({ id: input.id, allowModifiers, observations });
        for (const repeat of [0, 1]) {
          const filename = input.id + ".vue";
          const actualPath = path.join(fixture, filename);
          const raw: Packet[] = await eslint.lintText(input.source, { filePath: actualPath });
          observations.push(raw);
          persist();
          const recorded = raw.map((packet) => ({
            ...packet,
            filePath: packet.filePath === actualPath ? filename : packet.filePath,
          }));
          assert.deepEqual(recorded, expected.observations[repeat], input.id);
        }
        assert.deepEqual(observations[0], observations[1], input.id);
        evidence.qualified++;
        persist();
      }
    }
    assert.equal(evidence.qualified, 24);
  } finally {
    persist();
  }
});
