import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
} from "../../tools/support/json-value.ts";

interface Host {
  verify(
    source: string,
    config: unknown[],
    options: { filename: string; allowInlineConfig: false },
  ): unknown[];
  verifyAndFix(
    source: string,
    config: unknown[],
    options: { filename: string; allowInlineConfig: false },
  ): unknown;
}
const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = join(root, "tests/_fixtures/differential/linter/slot-typeof-presence-10822");
const rule = "vue/require-slots-as-functions";
const hash = (value: Buffer | string) => createHash("sha256").update(value).digest("hex");
const caseBytes = readFileSync(join(directory, "cases.json"));
assert.equal(hash(caseBytes), "e5846f88426847c1eed349f1914b103aea7a58e5f534b998d2675c876f853554");
const corpus = asRecord(parseJson(caseBytes.toString()));
const rows = asArray(corpus.cases).map((value) => {
  const row = asRecord(value);
  return { id: asString(row.id), script: asString(row.script), spans: asArray(row.officialSpans) };
});
assert.equal(rows.length, 9);
const originals = [
  {
    id: "original-field-body",
    file: "FieldBody.vue.txt",
    sha256: "549dd10935821983608154160721b6a600b8e4f74b74a60902872df81e934598",
  },
].map((row) => {
  const bytes = readFileSync(join(directory, row.file));
  assert.equal(hash(bytes), row.sha256);
  return { ...row, source: bytes.toString(), sourceSha256: hash(bytes), spans: [] };
});
const provenanceBytes = readFileSync(join(directory, "source-provenance.json"));
assert.equal(
  hash(provenanceBytes),
  "bfdfd9e669dbe68d671fecffc8d68ae48e13b96bb5d884c66a95092804ff7d83",
);
const provenance = asRecord(parseJson(provenanceBytes.toString()));

const successorBytes = readFileSync(join(directory, "semantic-successors.json"));
assert.equal(
  hash(successorBytes),
  "0d831b09099288dda67162ec4986637f6214d3f599ca203bfc663041263dcba3",
);
const successors = asRecord(parseJson(successorBytes.toString()));
const conditionalDirectory = join(
  root,
  "tests/_fixtures/differential/linter/slot-conditional-arms-10822",
);
const conditionalBytes = readFileSync(join(conditionalDirectory, "cases.json"));
assert.equal(
  hash(conditionalBytes),
  "36e47c4efcd91e10a53a86f7a5b6ce08485f95107e130e1cae97d2ddfec52e70",
);
const conditionalRows = asArray(asRecord(parseJson(conditionalBytes.toString())).cases).map(
  (value) => {
    const row = asRecord(value);
    return {
      id: asString(row.id),
      script: asString(row.script),
      spans: asArray(row.officialSpans),
    };
  },
);
assert.equal(conditionalRows.length, 10);
const conditionalSuccessorBytes = readFileSync(
  join(conditionalDirectory, "semantic-successors.json"),
);
assert.equal(
  hash(conditionalSuccessorBytes),
  "7bb4eaa041d6d4a48545f4f8b97520d35af29308e3290c8c61d4c47f31082b3c",
);
const conditionalSuccessors = asRecord(parseJson(conditionalSuccessorBytes.toString()));
const conditionalProvenanceBytes = readFileSync(
  join(conditionalDirectory, "source-provenance.json"),
);
assert.equal(
  hash(conditionalProvenanceBytes),
  "6c422e3ee89efe8ee557b1c420519de86812c8f5f99496d93fc291ed2743ad50",
);
const conditionalProvenance = asRecord(parseJson(conditionalProvenanceBytes.toString()));
test("named Typeof-v1 successors preserve exact two old rows, SHA pins and whole before packets", () => {
  assert.equal(successors.schema, "vize.slot-typeof-semantic-successors.v1");
  assert.equal(successors.feature, "immediate-authenticated-typeof-operand-v1");
  assert.equal(successors.version, 1);
  const entries = asArray(successors.successors).map((value) => asRecord(value));
  assert.equal(entries.length, 2);
  for (const entry of entries) {
    const suite = asString(entry.predecessorCorpus);
    assert(["slot-existence-negation-10822", "slot-existence-conditions-10822"].includes(suite));
    const oldBytes = readFileSync(
      join(root, "tests/_fixtures/differential/linter", suite, "cases.json"),
    );
    assert.equal(hash(oldBytes), entry.predecessorCorpusSha256);
    const old = asArray(asRecord(parseJson(oldBytes.toString())).cases)
      .map((value) => asRecord(value))
      .find((row) => row.id === "typeof-outside-slice")!;
    assert.deepEqual(entry.historicalCase, old);
    const script = asString(old.script);
    assert.equal(hash(script), entry.inputSha256);
    assert.deepEqual(entry.currentNativeSpans, []);
    assert.equal(asArray(old.nativeSpans).length, 1);
    const before = asArray(asRecord(entry.historicalNativeCapture).observations).map((value) =>
      asRecord(value),
    );
    assert.deepEqual(
      before.map((row) => row.offset),
      [0, 17],
    );
    for (const row of before) {
      const start =
        Buffer.byteLength(script.slice(0, script.indexOf("this.$slots.default"))) +
        asNumber(row.offset);
      assert.equal(
        row.completeResultDebug,
        `ScriptLintResult {
    diagnostics: [
        LintDiagnostic {
            rule_name: "script/require-slots-as-functions",
            severity: Error,
            message: "\`$slots\` properties are functions in Vue 3 and must be called.",
            start: ${start},
            end: ${start + 19},
            help: None,
            labels: [],
            fix: None,
        },
    ],
    error_count: 1,
    warning_count: 0,
}`,
      );
    }
  }
  const current = readFileSync(
    join(root, "crates/vize_patina/src/rules/script/require_slots_as_functions.rs"),
    "utf8",
  );
  assert.equal(
    successors.beforeRuleSha256,
    "0673f14ce68018c1636dd398d0c36108af8f5a17a90140cb10ad3b6c503f2118",
  );
  assert.equal(
    conditionalSuccessors.beforeRuleSha256,
    "9804044031a7975b732af2e1853388313bad635b69b5fd8332d4e4c8f8e87819",
  );
  assert.equal(hash(current), "02b4c57bcfc7f931b1e5dff6051abbb27a192bce5e14676699da317b4199cae9");
  for (const ownedBoundary of [
    "fn slot_function_origin(&self, expression: &Expression<'_>) -> Option<Span>",
    "if self.vue2_render_depth > 0",
    "member_of(expression).and_then(slot_property_span)",
    "fn visit_member_expression(&mut self, member: &MemberExpression<'a>)",
  ]) {
    assert.equal(current.split(ownedBoundary).length, 2, ownedBoundary);
  }
  assert.ok(!current.includes("suppress_direct_slot_function"));
});

test("named ConditionalArms-v1 retains the exact old alternate and original CLI packets", () => {
  assert.equal(conditionalSuccessors.schema, "vize.slot-conditional-arm-semantic-successors.v1");
  assert.equal(conditionalSuccessors.version, 1);
  assert.equal(conditionalSuccessors.feature, "immediate-authenticated-conditional-arms-v1");
  const entry = asRecord(conditionalSuccessors.conditionCase);
  const oldBytes = readFileSync(
    join(root, "tests/_fixtures/differential/linter/slot-existence-conditions-10822/cases.json"),
  );
  assert.equal(hash(oldBytes), entry.predecessorCorpusSha256);
  const old = asArray(asRecord(parseJson(oldBytes.toString())).cases)
    .map((value) => asRecord(value))
    .find((row) => row.id === "alternate-value-outside-slice")!;
  assert.deepEqual(entry.historicalCase, old);
  const script = asString(old.script);
  assert.equal(hash(script), entry.inputSha256);
  assert.deepEqual(entry.currentNativeSpans, []);
  const before = asArray(entry.historicalNativeObservations).map((value) => asRecord(value));
  assert.deepEqual(
    before.map((row) => row.offset),
    [0, 17],
  );
  for (const row of before) {
    assert.equal(row.source, script);
    const start =
      Buffer.byteLength(script.slice(0, script.indexOf("this.$slots.default"))) +
      asNumber(row.offset);
    assert.equal(
      row.completeResultDebug,
      `ScriptLintResult {
    diagnostics: [
        LintDiagnostic {
            rule_name: "script/require-slots-as-functions",
            severity: Error,
            message: "\`$slots\` properties are functions in Vue 3 and must be called.",
            start: ${start},
            end: ${start + 19},
            help: None,
            labels: [],
            fix: None,
        },
    ],
    error_count: 1,
    warning_count: 0,
}`,
    );
  }
  const original = asRecord(conditionalSuccessors.originalCli);
  assert.equal(original.id, "original-field-body");
  const literal = asString(original.beforeProducerLine);
  assert.equal(hash(literal), original.beforeProducerLineSha256);
  assert.equal(literal.startsWith("vize-slot-typeof-cli "), true);
  const packet = asRecord(parseJson(literal.slice("vize-slot-typeof-cli ".length)));
  assert.deepEqual(Object.keys(packet).sort(), [
    "fileAfter",
    "id",
    "source",
    "status",
    "stderrBytes",
    "stdoutBytes",
  ]);
  const bytes = readFileSync(join(conditionalDirectory, "FieldBody.vue.txt"));
  assert.equal(hash(bytes), original.inputSha256);
  assert.equal(hash(bytes), "549dd10935821983608154160721b6a600b8e4f74b74a60902872df81e934598");
  assert.equal(packet.source, bytes.toString());
  assert.deepEqual(bytes, Buffer.from(asArray(packet.fileAfter).map((value) => asNumber(value))));
  assert.equal(packet.status, 1);
  assert.deepEqual(packet.stderrBytes, []);
  assert.deepEqual(
    parseJson(Buffer.from(asArray(packet.stdoutBytes).map((value) => asNumber(value))).toString()),
    original.beforeWholeJson,
  );
  assert.deepEqual(original.currentNativeSpans, []);
});

test("the pinned Vue rule retains complete immediate presence originals and boundaries", () => {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "reference execution is Actions-only");
  const require = createRequire(join(root, "npm/linter/oxlint/package.json"));
  const providers = Object.fromEntries(
    Object.entries({
      eslint: "10.13.0",
      "eslint-plugin-vue": "10.9.2",
      "vue-eslint-parser": "10.4.1",
      "@typescript-eslint/parser": "8.65.0",
    }).map(([name, version]) => {
      const path = realpathSync(require.resolve(name + "/package.json"));
      assert.equal(asRecord(parseJson(readFileSync(path, "utf8"))).version, version);
      return [name, { path, version }];
    }),
  );
  const pluginDirectory = dirname(require.resolve("eslint-plugin-vue/package.json"));
  const rulePath = join(pluginDirectory, "dist/rules/require-slots-as-functions.js");
  const helperPath = join(pluginDirectory, "dist/utils/index.js");
  assert.equal(
    hash(readFileSync(rulePath)),
    "94857af48a201bc454f37630d1176a373a60d8fd0f6dc9d22a758a3243d76575",
  );
  assert.equal(
    hash(readFileSync(helperPath)),
    "4727c10651f789ee6308b4311f3694dd302faae9e5b821ae4b31432c3b105925",
  );
  const Linter = asRecord(require("eslint")).Linter as new () => Host;
  const plugin: unknown = require("eslint-plugin-vue"),
    parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  const source = {
    head: process.env.GITHUB_SHA ?? null,
    node: process.version,
    providers,
    rule: { path: rulePath, sha256: hash(readFileSync(rulePath)) },
    helper: { path: helperPath, sha256: hash(readFileSync(helperPath)) },
    originalSources: originals.map(({ id, sourceSha256 }) => ({ id, sourceSha256 })),
    casesSha256: hash(caseBytes),
    historicalTypeofProvenance: provenance,
    conditionalProvenance,
    conditionalCasesSha256: hash(conditionalBytes),
    inlineConfig:
      "disabled for this isolated-rule host; original non-target directives remain literal input",
    currentRuleSha256: hash(
      readFileSync(join(root, "crates/vize_patina/src/rules/script/require_slots_as_functions.rs")),
    ),
    scope:
      "Historical matrix ESLint10.4.1 is distinct from current explicit-rule ESLint10.13.0. Current semantics include immediate Typeof and conditional arms only. Typeof provenance remains historical; nested/other-operator and existing native value-use differences remain explicit.",
  };
  console.log("vize-slot-typeof-reference-source " + JSON.stringify(source));
  const inputs = [
    ...originals,
    ...[...rows, ...conditionalRows].map((row) => ({
      ...row,
      source: `<script>\n${row.script}\n</script>\n`,
    })),
  ];
  const config = [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
      },
      plugins: { vue: plugin },
      rules: { [rule]: "error" },
    },
  ];
  const observations = inputs.map((input) => {
    const host = new Linter(),
      // Keep every source byte; run only the explicitly configured target rule.
      filename = { filename: "SlotPresence.vue", allowInlineConfig: false as const };
    const verify = host.verify(input.source, config, filename);
    const fixResult = host.verifyAndFix(input.source, config, filename);
    const output = asString(asRecord(fixResult).output);
    const requery = host.verify(output, config, filename);
    const secondFixResult = host.verifyAndFix(output, config, filename);
    const packet = {
      id: input.id,
      source: input.source,
      sourceSha256: hash(input.source),
      verify,
      fixResult,
      requery,
      secondFixResult,
    };
    console.log("vize-slot-typeof-reference-observation " + JSON.stringify(packet));
    return { input, packet };
  });
  for (const { input, packet } of observations) {
    const messages = input.spans.map((value) => {
      const span = asRecord(value),
        needle = asString(span.needle);
      assert.equal(input.source.split(needle).length, 2, input.id);
      const start = input.source.indexOf(needle) + asNumber(span.shift);
      const end = start + asNumber(span.length);
      const position = (offset: number) => {
        const prefix = input.source.slice(0, offset),
          lines = prefix.split("\n");
        return { line: lines.length, column: lines.at(-1)!.length + 1 };
      };
      const from = position(start),
        to = position(end);
      return {
        ruleId: rule,
        severity: 2,
        message: "Property in `$slots` should be used as function.",
        messageId: "unexpected",
        line: from.line,
        column: from.column,
        endLine: to.line,
        endColumn: to.column,
      };
    });
    const fixed = { fixed: false, messages, output: input.source };
    assert.deepEqual(packet.verify, messages, input.id);
    assert.deepEqual(packet.fixResult, fixed, input.id);
    assert.deepEqual(packet.requery, messages, input.id);
    assert.deepEqual(packet.secondFixResult, fixed, input.id);
  }
  console.log(
    "vize-slot-typeof-reference-complete " +
      JSON.stringify({
        head: process.env.GITHUB_SHA ?? null,
        cases: observations.length,
        complete: true,
        limits:
          "Whole original FieldBody plus direct-Typeof and direct-arm controls. No project-matrix/all24FP/public/native-history qualification claim.",
      }),
  );
});
