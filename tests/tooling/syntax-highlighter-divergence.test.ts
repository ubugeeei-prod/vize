import assert from "node:assert/strict";
import { test } from "node:test";

import {
  applyDocumentedDivergences,
  compareSemanticSpans,
  validateLedger,
} from "./support/syntax-semantic-comparison.ts";
import {
  canonicalJson,
  semanticNormalization,
  tokenizeSemanticSource,
  type SemanticSpan,
} from "./support/syntax-semantic-divergence.ts";
import { createSyntaxAuditDeadline } from "./support/syntax-audit-deadline.ts";

const shared: SemanticSpan = {
  categories: ["tag"],
  line: 1,
  startColumn: 1,
  endColumn: 4,
};
const vizeOnly: SemanticSpan = {
  categories: ["comment"],
  line: 1,
  startColumn: 5,
  endColumn: 8,
};
const oracleOnly: SemanticSpan = {
  categories: ["invalid"],
  line: 1,
  startColumn: 5,
  endColumn: 8,
};

test("semantic comparison records exact shared, false-positive, and false-negative spans", () => {
  const comparison = compareSemanticSpans(
    "src/App.vue",
    "abc def",
    [shared, vizeOnly],
    [shared, oracleOnly],
  );
  assert.deepEqual(comparison.shared, [
    {
      category: "tag",
      file: "src/App.vue",
      line: 1,
      startColumn: 1,
      endColumn: 4,
    },
  ]);
  assert.deepEqual(comparison.falsePositives, [
    {
      category: "comment",
      file: "src/App.vue",
      kind: "false-positive",
      line: 1,
      startColumn: 5,
      endColumn: 8,
    },
  ]);
  assert.deepEqual(comparison.falseNegatives, [
    {
      category: "invalid",
      file: "src/App.vue",
      kind: "false-negative",
      line: 1,
      startColumn: 5,
      endColumn: 8,
    },
  ]);
  assert.match(comparison.sha256, /^[0-9a-f]{64}$/);
  assert.equal(
    compareSemanticSpans("src/App.vue", "abc def", [shared, vizeOnly], [shared, oracleOnly]).sha256,
    comparison.sha256,
  );
});

test("published normalization evidence is immutable canonical JSON", () => {
  assert.ok(Object.isFrozen(semanticNormalization));
  assert.equal(semanticNormalization.version, 3);
  assert.ok(Object.isFrozen(semanticNormalization.categories));
  assert.ok(Object.isFrozen(semanticNormalization.omittedCategories));
  assert.ok(Object.isFrozen(semanticNormalization.ignoredScopeFamilies));
  assert.equal(canonicalJson({ omitted: undefined, retained: 1 }), '{"retained":1}');
  assert.throws(() => canonicalJson(undefined), /value is not JSON-serializable/);
});

test("normalization ignores structural nesting but detects changed semantic scopes", () => {
  const structuralA = tokenizeSemanticSource(
    grammar(["source.vue", "meta.tag.vue", "punctuation.definition.tag.vue"]),
    "x",
    "source.vue",
    "structural-a.vue",
  );
  const structuralB = tokenizeSemanticSource(
    grammar(["text.html.vue", "text.html.derivative", "meta.element.vue"]),
    "x",
    "text.html.vue",
    "structural-b.vue",
  );
  assert.deepEqual(structuralA.semanticSpans, structuralB.semanticSpans);

  const tag = tokenizeSemanticSource(
    grammar(["source.vue", "entity.name.tag.html.vue"]),
    "x",
    "source.vue",
    "tag.vue",
  );
  const attribute = tokenizeSemanticSource(
    grammar(["text.html.vue", "entity.other.attribute-name.html.vue"]),
    "x",
    "text.html.vue",
    "attribute.vue",
  );
  const changed = compareSemanticSpans(
    "src/Changed.vue",
    "x",
    tag.semanticSpans,
    attribute.semanticSpans,
  );
  assert.equal(changed.falsePositives[0].category, "tag");
  assert.equal(changed.falseNegatives[0].category, "attribute");

  const knownTextMateScopes = tokenizeSemanticSource(
    grammar([
      "source.vue",
      "new.expr.ts",
      "entity.other.inherited-class.ts",
      "support.constant.property-value.css",
      "support.variable.property.ts",
      "support.other.variable.less",
      "entity.name.label.ts",
      "entity.name.section.markdown",
      "entity.name.constant.counter-name.less",
      "entity.other.keyframe-offset.percentage.css",
      "entity.other.counter-name.less",
      "attribute_value",
      "attribute_value2",
      "entity.other.attribute-selector.sass",
      "html",
      "inline.pug",
      "name.generic.filter.pug",
    ]),
    "x",
    "source.vue",
    "known-textmate-scopes.vue",
  );
  assert.deepEqual(knownTextMateScopes.semanticSpans[0].categories, []);

  const vueShell = tokenizeSemanticSource(
    grammar([
      "source.vue",
      "punctuation.attribute-shorthand.bind.html.vue",
      "keyword.control.conditional.vue",
    ]),
    "x",
    "source.vue",
    "vue-shell.vue",
  );
  assert.deepEqual(vueShell.semanticSpans[0].categories, ["attribute"]);

  const pugInlineTag = tokenizeSemanticSource(
    grammar(["source.vue", "tag.inline.pug"]),
    "x",
    "source.vue",
    "pug-inline-tag.vue",
  );
  assert.deepEqual(pugInlineTag.semanticSpans[0].categories, ["tag"]);
});

test("semantic tokenizer fails closed on unknown scopes, malformed spans, and timeouts", () => {
  assert.throws(
    () =>
      tokenizeSemanticSource(
        grammar(["source.vue", "mystery.semantic.vue"]),
        "x",
        "source.vue",
        "unknown.vue",
      ),
    /unknown semantic scope mystery\.semantic\.vue/,
  );
  assert.throws(
    () =>
      tokenizeSemanticSource(
        {
          tokenizeLine() {
            return {
              ruleStack: null,
              tokens: [{ startIndex: 1, endIndex: 2, scopes: ["source.vue"] }],
            };
          },
        },
        "x",
        "source.vue",
        "gap.vue",
      ),
    /gap\.vue:1:0/,
  );
  assert.throws(
    () =>
      tokenizeSemanticSource(
        {
          tokenizeLine() {
            return {
              ruleStack: null,
              stoppedEarly: true,
              tokens: [{ startIndex: 0, endIndex: 1, scopes: ["source.vue"] }],
            };
          },
        },
        "x",
        "source.vue",
        "slow.vue",
      ),
    /exceeded 1000ms/,
  );
  const timestamps = [0, 0, 0, 2];
  const deadline = createSyntaxAuditDeadline(
    { SYNTAX_HIGHLIGHTER_ORACLE_TIMEOUT_MS: "1" },
    () => timestamps.shift() ?? 2,
  );
  assert.throws(
    () =>
      tokenizeSemanticSource(
        grammar(["source.vue", "entity.name.tag.html.vue"]),
        "x",
        "source.vue",
        "deadline.vue",
        deadline,
      ),
    /syntax oracle shard exceeded 1ms/,
  );
});

test("intentional divergence ledger is exact and stale entries fail closed", () => {
  const comparison = compareSemanticSpans(
    "src/App.vue",
    "abc def",
    [shared, vizeOnly],
    [shared, oracleOnly],
  );
  const entry = {
    project: "fixture",
    file: "src/App.vue",
    kind: "false-positive" as const,
    category: "comment",
    line: 1,
    startColumn: 5,
    endColumn: 8,
    issue: 3677,
    reason: "Art Vue deliberately assigns a Vize-only semantic role at this exact authored span.",
  };
  const ledger = ledgerWith(entry);
  assert.deepEqual(validateLedger(ledger), [entry]);
  const classified = applyDocumentedDivergences("fixture", comparison, ledger);
  assert.equal(classified.falsePositives.length, 0);
  assert.equal(classified.falseNegatives.length, 1);
  assert.deepEqual(classified.documented, [entry]);
  assert.throws(
    () => applyDocumentedDivergences("fixture", comparison, ledgerWith({ ...entry, line: 2 })),
    /stale syntax divergence ledger entry/,
  );
  assert.throws(
    () => validateLedger({ schema: "wrong", version: 1, entries: [] }),
    /unexpected syntax divergence ledger schema/,
  );
  assert.throws(
    () => validateLedger(ledger, new Set(["another-fixture"])),
    /unknown syntax divergence ledger project fixture/,
  );
});

function grammar(scopes: string[]) {
  return {
    tokenizeLine(line: string) {
      return {
        ruleStack: null,
        tokens: [{ startIndex: 0, endIndex: Math.max(line.length, 1), scopes }],
      };
    },
  };
}

function ledgerWith(entry: object) {
  return { schema: "vize.fixtureSyntaxDivergenceLedger", version: 1, entries: [entry] };
}
