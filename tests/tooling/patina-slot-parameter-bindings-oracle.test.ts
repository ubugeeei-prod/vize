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
const fixture = path.join(root, "tests/_fixtures/differential/lint/slot-parameter-bindings-8142");
const read = (file: string) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

await test("parameter vectors keep authored coordinates, reference differences and original bytes", () => {
  const controls = read("controls.json");
  const consumers = read("consumers.json");
  const golden = read("independent.json");
  for (const [file, digest] of Object.entries(controls.preservedOriginalInputs)) {
    assert.equal(sha256(fs.readFileSync(path.join(root, file))), digest, file);
  }
  const original = JSON.parse(
    fs.readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/lint/slot-dynamic-modifiers-8142/independent.json",
      ),
      "utf8",
    ),
  );
  assert.deepEqual(golden.providers, original.providers);
  assert.deepEqual(golden.mandatoryInfrastructure, original.mandatoryInfrastructure);
  assert.equal(controls.cases.length, 24);
  assert.equal(consumers.records.length, 24);
  assert.deepEqual(consumers.counts, { cliCalls: 147, wholeLspPackets: 98, total: 245 });
  const differences = new Set<string>();
  for (const input of controls.cases) {
    const directive = input.authored.directive;
    const start = input.source.indexOf(directive);
    assert.ok(start >= 0, input.id);
    assert.equal(input.source.lastIndexOf(directive), start, input.id);
    const end = start + directive.length;
    const byteSpan = {
      start: Buffer.byteLength(input.source.slice(0, start)),
      end: Buffer.byteLength(input.source.slice(0, end)),
    };
    assert.deepEqual(input.authored.byteSpan, byteSpan, input.id);
    const at = (index: number, scalar: boolean) => {
      const lines = input.source.slice(0, index).split("\n");
      const tail = lines.at(-1)!;
      return { line: lines.length - 1, character: scalar ? Array.from(tail).length : tail.length };
    };
    assert.deepEqual(input.authored.scalarRange, { start: at(start, true), end: at(end, true) });
    assert.deepEqual(input.authored.utf16Range, { start: at(start, false), end: at(end, false) });
    const consumer = consumers.records.find((record: Input) => record.id === input.id);
    assert.ok(consumer, input.id);
    assert.equal(consumer.sourceSha256, input.sourceSha256, input.id);
    for (const allowModifiers of [false, true]) {
      const key = String(allowModifiers);
      const expected = input.expectedByOption[key];
      assert.equal(expected.filename, "ParameterSlot.vue");
      assert.equal(expected.error_count, expected.diagnostics.length);
      assert.equal(expected.warning_count, 0);
      for (const diagnostic of expected.diagnostics) {
        assert.equal(diagnostic.start, byteSpan.start);
        assert.equal(diagnostic.end, byteSpan.end);
      }
      const scalar = input.authored.scalarRange;
      const cliMessages = expected.diagnostics.map((d: { message: string }) => ({
        ruleId: "vue/valid-v-slot",
        ruleDocsPath: "docs/content/rules/vue.md",
        severity: 2,
        message: "[vize:vue/valid-v-slot] " + d.message,
        line: scalar.start.line + 1,
        column: scalar.start.character + 1,
        endLine: scalar.end.line + 1,
        endColumn: scalar.end.character + 1,
      }));
      assert.deepEqual(consumer.options[key].cli, [
        {
          file: input.id + ".vue",
          messages: cliMessages,
          errorCount: cliMessages.length,
          warningCount: 0,
        },
      ]);
      assert.deepEqual(
        consumer.options[key].lspDiagnostics,
        expected.diagnostics.map((d: { message: string; help: string }) => ({
          range: input.authored.utf16Range,
          severity: 1,
          code: "vue/valid-v-slot",
          codeDescription: { href: "https://eslint.vuejs.org/rules/valid-v-slot.html" },
          source: "vize/lint",
          message: d.message + "\n\nHelp: " + d.help,
        })),
      );
      const declared = expected.diagnostics.map((d: { message: string }) =>
        d.message.startsWith("Dynamic slot name")
          ? "disallowArgumentUseSlotParams"
          : "disallowAnyModifier",
      );
      const official = golden.records.find(
        (record: { id: string; allowModifiers: boolean }) =>
          record.id === input.id && record.allowModifiers === allowModifiers,
      );
      assert.ok(official, input.id);
      const observed = official.observations[0].flatMap(
        (packet: { messages: { messageId: string }[] }) =>
          packet.messages.map((message) => message.messageId),
      );
      if (JSON.stringify(declared) !== JSON.stringify(observed)) differences.add(input.id);
    }
  }
  assert.deepEqual(
    [...differences].sort(),
    controls.referenceDifferences.map((d: Input) => d.id).sort(),
  );
});

await test("whole parameter goals preserve both policies and complete pinned official Vue packets", async () => {
  const inputs: Input[] = read("controls.json").cases;
  const golden: {
    providers: Record<string, Provider>;
    mandatoryInfrastructure: unknown;
    records: { id: string; allowModifiers: boolean; observations: Packet[][] }[];
  } = read("independent.json");
  assert.equal(inputs.length, 24);
  assert.equal(golden.records.length, 48);
  const req = createRequire(
    process.env.VIZE_SLOT_VALIDITY_ORACLE_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const artifact = path.join(root, "target/differential/slot-parameter-bindings-oracle.json");
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
    assert.equal(evidence.qualified, 48);
  } finally {
    persist();
  }
});
