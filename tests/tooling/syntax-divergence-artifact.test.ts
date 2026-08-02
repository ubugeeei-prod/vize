import assert from "node:assert/strict";
import { test } from "node:test";

import { shikiVueOracleProvenance } from "./support/shiki-vue-oracle.ts";
import {
  createDivergenceArtifact,
  renderDivergenceMarkdown,
  validateDivergenceArtifact,
} from "./support/syntax-divergence-artifact.ts";
import { textmateDependencyVersions } from "./support/textmate-deps.ts";

test("divergence artifacts are deterministic and reject malformed records", () => {
  const input = validInput();
  const first = createDivergenceArtifact(input);
  const second = createDivergenceArtifact(input);
  assert.deepEqual(first, second);

  const malformed = structuredClone(first);
  malformed.summary.fileCount = 2;
  assert.throws(
    () => validateDivergenceArtifact(malformed),
    /syntax divergence artifact summary mismatch/,
  );
  const malformedRecord = structuredClone(first);
  malformedRecord.projects[0].falsePositives = [
    {
      category: "mystery",
      endColumn: 2,
      file: "src/App.vue",
      kind: "false-positive",
      line: 1,
      startColumn: 1,
    },
  ];
  assert.throws(
    () => validateDivergenceArtifact(malformedRecord),
    /unknown artifact semantic category mystery/,
  );
});

test("divergence markdown reports the summary and digest", () => {
  const artifact = createDivergenceArtifact(validInput());
  const markdown = renderDivergenceMarkdown(artifact);
  assert.match(markdown, new RegExp(`Commit: ${artifact.commitSha}`));
  assert.match(markdown, /Projects: 1/);
  assert.match(markdown, /Vue files: 1/);
  assert.match(markdown, new RegExp(`Digest: ${artifact.sha256}`));
});

test("divergence artifacts reject malformed provenance and topology", () => {
  const input = validInput();
  const cases = [
    {
      label: "missing grammars",
      value: { ...input, grammars: null },
      expected: /artifact grammars/,
    },
    {
      label: "half-null shard",
      value: { ...input, shard: { count: null, index: 0 } },
      expected: /unsharded artifact index must be null/,
    },
    {
      label: "negative shard count",
      value: { ...input, shard: { count: -1, index: 999 } },
      expected: /artifact shard count must be a positive integer/,
    },
    {
      label: "duplicate project",
      value: { ...input, projects: [...input.projects, structuredClone(input.projects[0])] },
      expected: /duplicate artifact project fixture/,
    },
    {
      label: "unexpected ledger field",
      value: { ...input, ledger: { ...input.ledger, unexpected: true } },
      expected: /unexpected artifact ledger fields/,
    },
    {
      label: "oracle provenance drift",
      value: {
        ...input,
        grammars: {
          ...input.grammars,
          oracle: { ...input.grammars.oracle, grammarClosureSha256: "0".repeat(64) },
        },
      },
      expected: /oracle grammarClosureSha256 provenance drifted/,
    },
  ];
  for (const { label, value, expected } of cases) {
    assert.throws(() => createDivergenceArtifact(value), expected, label);
  }
});

function validInput() {
  return {
    commitSha: "a".repeat(40),
    grammars: grammarEvidence(),
    ledger: {
      path: "tests/_fixtures/syntax-highlighter-documented-divergences.json",
      sha256: "b".repeat(64),
    },
    projects: [
      {
        id: "fixture",
        revision: "c".repeat(40),
        fileCount: 1,
        inputSha256: "d".repeat(64),
        semanticSpanCount: 3,
        comparisonSha256: "e".repeat(64),
        falsePositives: [],
        falseNegatives: [],
        documentedDivergences: [],
      },
    ],
    shard: { count: 11, index: 1 },
  };
}

function grammarEvidence() {
  const provenance = shikiVueOracleProvenance;
  return {
    oracle: {
      configuredGrammarSha256: "f".repeat(64),
      dependencyVersions: textmateDependencyVersions,
      grammarClosureSha256: provenance.grammarClosureSha256,
      licenseSha256: provenance.licenseSha256,
      module: provenance.module,
      moduleSha256: provenance.moduleSha256,
      package: provenance.package,
      requestedScopes: ["text.html.vue"],
      rootScope: "text.html.vue",
      unresolvedScopeSentinels: [],
      version: provenance.version,
    },
    vize: [
      {
        configuredGrammarSha256: "1".repeat(64),
        dependencyVersions: textmateDependencyVersions,
        requestedScopes: ["source.vue"],
        rootScope: "source.vue",
      },
    ],
  };
}
