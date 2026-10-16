import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";
import { projection } from "./support/n8n-cli-config-inputs.mjs";
import { loadAuthoredCases } from "./support/n8n-cli-config-oracle.mjs";
import {
  oracleLayers,
  recordCorpusPackets,
  assertCompletePackets,
  semanticFindings,
  compareSemanticFindings,
  physicalOffset,
  assertQualifiedSummary,
} from "./support/n8n-whole-corpus-core.mjs";

await test("all51 aliases retain three options, registered-only casing and literal six scopes", () => {
  const editor = oracleLayers("packages/frontend/editor-ui", "scoped");
  assert.equal(Object.keys(editor.layers[0].rules).length, 51);
  assert.deepEqual(editor.layers[0].rules["vue/attribute-hyphenation"], ["warn", "always"]);
  assert.deepEqual(editor.layers[0].rules["vue/component-name-in-template-casing"], [
    "error",
    "PascalCase",
    { registeredComponentsOnly: true },
  ]);
  assert.deepEqual(editor.layers[0].rules["vue/block-order"], [
    "error",
    { order: ["script", "template", "style"] },
  ]);
  assert.equal(editor.layers.slice(1).flatMap(({ files }) => files).length, 5);
  for (const layer of editor.layers.slice(1)) {
    assert.deepEqual(layer.rules, { "vue/no-multiple-template-root": "off" });
    assert.ok(layer.files.every((file) => !file.includes("*")));
  }
  const design = oracleLayers("packages/frontend/@n8n/design-system", "scoped");
  assert.deepEqual(design.layers[1], {
    files: ["src/components/N8nDatatable/Datatable.vue"],
    rules: { "vue/require-v-for-key": "off" },
  });
  assert.equal(oracleLayers("packages/frontend/editor-ui", "baseline").layers.length, 1);
});

await test("whole packet sidecar preserves suppressions, fix metadata and every unknown field", () => {
  const raw = [
    {
      filePath: "/owned/work/Input.vue",
      messages: [{ ruleId: "vue/valid-v-on", fix: { range: [1, 2], text: "x" }, foreign: 7 }],
      suppressedMessages: [
        { ruleId: "vue/no-v-html", suppressions: [{ kind: "directive", justification: "owned" }] },
      ],
      foreignEnvelope: { retained: true },
    },
  ];
  const recorded = recordCorpusPackets(raw, "/owned/work");
  assert.deepEqual(recorded.serializedRawPackets, raw);
  assert.deepEqual(recorded.packets, [{ ...raw[0], filePath: "Input.vue" }]);
  assert.equal(raw[0].filePath, "/owned/work/Input.vue");
});

await test("completeness refuses missing clean inputs, duplicates, hidden findings and fatal errors", () => {
  const clean = {
    filePath: "Clean.vue",
    messages: [],
    suppressedMessages: [],
    errorCount: 0,
    warningCount: 0,
    fatalErrorCount: 0,
  };
  assertCompletePackets([clean], ["Clean.vue"], "eslint");
  assert.throws(() => assertCompletePackets([], ["Clean.vue"], "eslint"));
  assert.throws(() => assertCompletePackets([clean, clean], ["Clean.vue"], "eslint"));
  assert.throws(() =>
    assertCompletePackets([{ ...clean, errorCount: 1 }], ["Clean.vue"], "eslint"),
  );
  assert.throws(() =>
    assertCompletePackets([{ ...clean, fatalErrorCount: 1 }], ["Clean.vue"], "eslint"),
  );
});

await test("comparison retains full physical ranges, severities, unknown identities and multiplicity", () => {
  const finding = {
    file: "owned/Input.vue",
    rule: "vue/valid-v-on",
    severity: 2,
    startByte: 4,
    endByte: 19,
  };
  assert.deepEqual(compareSemanticFindings([finding], [finding]), {
    matched: 1,
    nativeOnly: [],
    oracleOnly: [],
  });
  for (const change of [
    { startByte: 6 },
    { endByte: 20 },
    { endByte: null },
    { severity: 1 },
    { rule: "foreign/rule" },
  ]) {
    const compared = compareSemanticFindings([{ ...finding, ...change }], [finding]);
    assert.equal(compared.matched, 0);
    assert.equal(compared.nativeOnly.length, 1);
    assert.equal(compared.oracleOnly.length, 1);
  }
  assert.equal(compareSemanticFindings([finding, finding], [finding]).nativeOnly.length, 1);
  assert.equal(compareSemanticFindings([finding], [finding, finding]).oracleOnly.length, 1);
  const raw = [
    {
      filePath: "owned/Input.vue",
      messages: [{ ruleId: "foreign/rule", severity: 2, line: 1, column: 1 }],
    },
  ];
  assert.equal(semanticFindings(raw, "eslint", "owned", () => "owned")[0].rule, "foreign/rule");
});

await test("native scalar and Vue ESLint UTF16 coordinates map LF-based locations to original bytes", () => {
  const source = "😀x\r\nβy\nx\ry\u2028z";
  assert.equal(physicalOffset(source, 1, 2, "native"), 4);
  assert.equal(physicalOffset(source, 1, 3, "eslint"), 4);
  assert.equal(physicalOffset(source, 2, 2, "native"), Buffer.byteLength("😀x\r\nβ"));
  assert.equal(physicalOffset(source, 2, 2, "eslint"), Buffer.byteLength("😀x\r\nβ"));
  assert.equal(physicalOffset(source, 3, 3, "native"), physicalOffset(source, 3, 3, "eslint"));
  assert.equal(physicalOffset(source, 3, 5, "native"), physicalOffset(source, 3, 5, "eslint"));
  assert.throws(() => physicalOffset(source, 4, 1, "eslint"));
  assert.throws(() => physicalOffset(source, 1, 2, "eslint"), /UTF-8 boundary/);
  assert.throws(() => physicalOffset(source, 99, 1, "native"));
  assert.throws(() => physicalOffset(source, 1, 99, "native"));
  assert.equal(physicalOffset(source, undefined, undefined, "eslint"), null);
});

await test("whole repeated pinned template/script packets independently establish Vue coordinate units", () => {
  const fixture = JSON.parse(
    fs.readFileSync(
      new URL("../_fixtures/differential/lint/n8n-whole-corpus-coordinates.json", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(fixture.schema, "vize.n8n.whole-corpus-coordinate-authority");
  assert.equal(fixture.version, 1);
  assert.deepEqual(fixture.providers, loadAuthoredCases().oracleCapture.providers);
  assert.equal(fixture.cases.length, 10);
  assert.equal(Object.keys(fixture.configuration.effectiveRules).length, 53);
  let messages = 0;
  for (const sourceCase of fixture.cases) {
    assert.equal(
      createHash("sha256").update(sourceCase.source).digest("hex"),
      sourceCase.sourceSha256,
    );
    assert.equal(
      Buffer.from(sourceCase.source)
        .subarray(sourceCase.target.startByte, sourceCase.target.endByte)
        .toString("utf8"),
      sourceCase.target.token,
    );
    assert.equal(sourceCase.observations.length, 2);
    assert.deepEqual(sourceCase.observations[0], sourceCase.observations[1]);
    for (const observation of sourceCase.observations) {
      assert.equal(observation.serializedRawPackets.length, 1);
      assert.equal(observation.packets.length, 1);
      const packet = observation.packets[0];
      assert.deepEqual(packet, {
        ...observation.serializedRawPackets[0],
        filePath: packet.filePath,
      });
      messages += packet.messages.length;
      const targets = packet.messages.filter(({ ruleId }) => ruleId === sourceCase.target.ruleId);
      assert.equal(targets.length, 1);
      const [target] = targets;
      assert.equal(
        physicalOffset(sourceCase.source, target.line, target.column, "eslint"),
        sourceCase.target.startByte,
      );
      assert.equal(
        physicalOffset(sourceCase.source, target.endLine, target.endColumn, "eslint"),
        sourceCase.target.endByte,
      );
    }
  }
  assert.equal(messages, 40, "complete foreign findings remain in every repeated provider packet");
});

await test("gate refuses stale schemas, incomplete observations and hidden comparison differences", () => {
  const summary = {
    schema: "vize.n8n.whole-corpus-observer",
    version: 1,
    captureComplete: true,
    observedInventory: { originalFiles: 1369, scriptlessFiles: 19, packageRoots: 9 },
    observations: { native: 36, eslint: 36, nativeFilePackets: 5476, eslintFilePackets: 5476 },
    modes: ["baseline", "scoped"],
    repeats: 2,
    failures: [],
    canonicalDifferenceCount: 0,
    comparisons: Object.keys(projection.packageVueFileCounts).flatMap((packageRoot) =>
      ["baseline", "scoped"].map((mode) => ({ packageRoot, mode, nativeOnly: [], oracleOnly: [] })),
    ),
  };
  assertQualifiedSummary(summary);
  for (const mutation of [
    { version: 2 },
    { captureComplete: false },
    { failures: [{ retained: true }] },
    { observations: { ...summary.observations, eslintFilePackets: 5475 } },
    {
      comparisons: [
        { ...summary.comparisons[0], nativeOnly: [{ rule: "foreign" }] },
        ...summary.comparisons.slice(1),
      ],
    },
    { comparisons: [summary.comparisons[1], ...summary.comparisons.slice(1)] },
    { canonicalDifferenceCount: 1 },
  ])
    assert.throws(() => assertQualifiedSummary({ ...summary, ...mutation }));
});
