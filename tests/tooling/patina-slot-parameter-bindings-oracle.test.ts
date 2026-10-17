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
interface Diagnostic {
  start: number;
  end: number;
  message: string;
  help: string;
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

interface SourceTransition {
  path: string;
  originalSha256: string;
  currentSha256: string;
  originalBase64: string;
  edits: { kind: string; offset: number; before: string; after: string }[];
}
const transitionBytes = fs.readFileSync(
  path.join(fixture, "canonical-config-source-transitions.json"),
);
assert.equal(
  sha256(transitionBytes),
  "2c5a8b6b9c34e2e14ae53bae247a747600ad0b92a53f0da9605f107c8c116be2",
);
const transitions: { schema: string; records: SourceTransition[] } = JSON.parse(
  transitionBytes.toString(),
);
assert.equal(transitions.schema, "vize.slot-options-canonical-config-source-transitions-v1");
assert.deepEqual(
  transitions.records.map(({ path: file }) => file),
  [
    "tests/tooling/support/n8n-slot-options-acceptance.ts",
    "tests/tooling/support/n8n-installed-slot-options-acceptance.ts",
  ],
);

// The old manifest remains authority for the whole original source. A finite
// reviewed import/config-carrier transition binds its active counterpart;
// none of its vectors, invocations or complete output qualifications change.
function preservedSource(
  transition: SourceTransition,
  current: Buffer,
  originalDigest: string,
): Buffer {
  const original = Buffer.from(transition.originalBase64, "base64");
  assert.equal(transition.originalSha256, originalDigest);
  assert.equal(sha256(original), originalDigest);
  assert.equal(sha256(current), transition.currentSha256);
  assert.deepEqual(
    transition.edits.map(({ kind }) => kind),
    ["canonical-helper-import", "canonical-config-carrier"],
  );
  const [helper, carrier] = transition.edits;
  assert.equal(helper.before, "");
  assert.equal(
    helper.after,
    'import { nativeConfigToVueOptions, vueConfigFixtureSource } from "./vue-config-fixture.ts";\n',
  );
  const text = original.toString("utf8");
  assert.deepEqual(Buffer.from(text), original);
  const start = text.indexOf("const configSource =");
  const end = text.indexOf("const custody:");
  assert.ok(start > 0 && end > start);
  assert.equal(carrier.offset, Buffer.byteLength(text.slice(0, start)));
  assert.equal(carrier.before, text.slice(start, end));
  assert.ok(helper.offset < carrier.offset);
  let composed = original;
  for (const edit of transition.edits.toReversed()) {
    assert.ok(Number.isSafeInteger(edit.offset) && edit.offset >= 0);
    const removed = Buffer.from(edit.before);
    assert.deepEqual(composed.subarray(edit.offset, edit.offset + removed.length), removed);
    composed = Buffer.concat([
      composed.subarray(0, edit.offset),
      Buffer.from(edit.after),
      composed.subarray(edit.offset + removed.length),
    ]);
  }
  assert.deepEqual(composed, current, "active helper must equal the complete reviewed counterpart");
  return original;
}

await test("parameter vectors keep authored coordinates, reference differences and original bytes", () => {
  const controls = read("controls.json");
  const consumers = read("consumers.json");
  const golden = read("independent.json");
  for (const [file, digest] of Object.entries(controls.preservedOriginalInputs)) {
    const current = fs.readFileSync(path.join(root, file));
    const transition = transitions.records.find((record) => record.path === file);
    const original = transition ? preservedSource(transition, current, digest as string) : current;
    assert.equal(sha256(original), digest, file);
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
      // Modifier findings cover the modifiers; every other finding the directive.
      const position = (byte: number, scalar: boolean) =>
        at(Buffer.from(input.source).subarray(0, byte).toString().length, scalar);
      for (const diagnostic of expected.diagnostics) {
        if (diagnostic.message.includes("modifiers")) {
          assert.ok(diagnostic.start > byteSpan.start && diagnostic.end <= byteSpan.end);
        } else {
          assert.equal(diagnostic.start, byteSpan.start);
          assert.equal(diagnostic.end, byteSpan.end);
        }
      }
      const cliMessages = expected.diagnostics.map((d: Diagnostic) => ({
        ruleId: "vue/valid-v-slot",
        ruleDocsPath: "docs/content/rules/vue.md",
        severity: 2,
        message: "[vize:vue/valid-v-slot] " + d.message,
        line: position(d.start, true).line + 1,
        column: position(d.start, true).character + 1,
        endLine: position(d.end, true).line + 1,
        endColumn: position(d.end, true).character + 1,
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
        expected.diagnostics.map((d: Diagnostic) => ({
          range: { start: position(d.start, false), end: position(d.end, false) },
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

await test("canonical config source counterparts reject lost edits, changed originals and unrelated live drift", () => {
  const controls = read("controls.json");
  for (const transition of transitions.records) {
    const live = fs.readFileSync(path.join(root, transition.path));
    const expected = controls.preservedOriginalInputs[transition.path];
    for (const mutate of [
      (row: SourceTransition) => {
        row.originalBase64 = Buffer.from("foreign original").toString("base64");
      },
      (row: SourceTransition) => {
        row.originalSha256 = "0".repeat(64);
      },
      (row: SourceTransition) => {
        row.currentSha256 = "0".repeat(64);
      },
      (row: SourceTransition) => {
        row.edits.pop();
      },
      (row: SourceTransition) => {
        row.edits.reverse();
      },
      (row: SourceTransition) => {
        row.edits[1].offset++;
      },
      (row: SourceTransition) => {
        row.edits[1].before += "foreign";
      },
      (row: SourceTransition) => {
        row.edits[1].after += "\n// foreign\n";
      },
    ]) {
      const changed = structuredClone(transition);
      mutate(changed);
      assert.throws(() => preservedSource(changed, live, expected));
    }
    assert.throws(() =>
      preservedSource(transition, Buffer.concat([live, Buffer.from("\nforeign")]), expected),
    );
    const foreign = Buffer.concat([live, Buffer.from("\nforeign")]);
    const rehashed = structuredClone(transition);
    rehashed.currentSha256 = sha256(foreign);
    assert.throws(
      () => preservedSource(rehashed, foreign, expected),
      "a new hash cannot authorize unrelated source edits",
    );
  }
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
