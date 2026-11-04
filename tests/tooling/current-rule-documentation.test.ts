import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { asArray, asRecord, asString } from "../../tools/support/json-value.ts";
import {
  currentRuleDocumentationMessage,
  currentRuleDocumentationPacket,
} from "./support/current-rule-documentation.ts";

const histories = [
  ["a11y/interactive-supports-focus", "accessibility"],
  ["a11y/no-aria-hidden-on-focusable", "accessibility"],
  ["css/no-display-none", "musea-and-css"],
  ["script/no-import-compiler-macros", "type-and-script"],
  ["script/no-required-prop-with-default", "type-and-script"],
  ["script/no-reserved-keys", "type-and-script"],
  ["script/no-reserved-props", "type-and-script"],
  ["script/return-in-emits-validator", "type-and-script"],
  ["script/valid-define-emits", "type-and-script"],
  ["vue/no-deprecated-scope-attribute", "vue"],
  ["vue/no-deprecated-slot-scope-attribute", "vue"],
  ["vue/no-deprecated-v-bind-sync", "vue"],
  ["vue/no-deprecated-v-on-native-modifier", "vue"],
  ["vue/v-bind-style", "vue"],
  ["vue/v-on-style", "vue"],
] as const;
const message = (ruleId: string, category: string) => ({
  ruleId,
  ruleDocsPath: `docs/content/rules/${category}.md`,
  severity: 2,
  message: "complete authored message",
  help: "complete authored help",
  line: 2,
  column: 3,
  endLine: 2,
  endColumn: 9,
});

void test("only the fifteen known documentation fields change, preserving complete packets and frozen inputs", () => {
  for (const [rule, category] of histories) {
    const expected = [
      { file: "Probe.vue", messages: [message(rule, category)], errorCount: 1, warningCount: 0 },
    ];
    const before = structuredClone(expected);
    const projected = currentRuleDocumentationPacket(expected);
    const target = `docs/content/rules/reference/${rule.replace("/", "-")}.md`;
    assert.equal(asArray(projected[0]!.messages).length, 1);
    assert.equal(asRecord(asArray(projected[0]!.messages)[0]).ruleDocsPath, target);
    const inverse = structuredClone(projected);
    asRecord(asArray(inverse[0]!.messages)[0]).ruleDocsPath = before[0]!.messages[0]!.ruleDocsPath;
    assert.deepEqual(inverse, before);
    assert.deepEqual(expected, before, "never mutate the original");
    for (const locale of ["", "ja/"])
      assert.match(
        fs.readFileSync(
          new URL(
            `../../${target.replace("docs/content/", `docs/content/${locale}`)}`,
            import.meta.url,
          ),
          "utf8",
        ),
        new RegExp(`^title: "${rule}"$`, "mu"),
      );
  }
});

void test("complete stored CSS, listbox and runtime-prop packets keep every non-URL field", () => {
  let packets = 0;
  const visit = (value: unknown): void => {
    if (Array.isArray(value)) {
      if (
        value.length &&
        value.every((row) => row && typeof row === "object" && "file" in row && "messages" in row)
      ) {
        const before = structuredClone(value);
        const projected = currentRuleDocumentationPacket(value);
        for (let index = 0; index < projected.length; index++) {
          const original = asRecord(before[index]);
          const messages = asArray(projected[index]!.messages);
          const originalMessages = asArray(original.messages);
          assert.equal(messages.length, originalMessages.length);
          for (let offset = 0; offset < messages.length; offset++)
            asRecord(messages[offset]).ruleDocsPath = asRecord(
              originalMessages[offset],
            ).ruleDocsPath;
        }
        assert.deepEqual(projected, before);
        assert.deepEqual(value, before);
        packets++;
      }
      value.forEach(visit);
    } else if (value && typeof value === "object") Object.values(value).forEach(visit);
  };
  for (const path of [
    "crates/vize_patina/tests/fixtures/issue-7976-compound/cases.json",
    "crates/vize_patina/tests/fixtures/issue-7976-universal/cases.json",
    "tests/_fixtures/differential/linter/a11y-listbox-focus-10925/cases.json",
    "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/scoped-route-contract.json",
    "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/default-route-controls.json",
  ])
    visit(
      JSON.parse(fs.readFileSync(new URL(`../../${path}`, import.meta.url), "utf8")) as unknown,
    );
  assert.equal(packets, 535, "retain every stored whole packet in the five frozen corpora");
});

void test("unknown rules, old/current URLs, severity, packet fields and counter changes are refused", () => {
  const original = message("script/no-reserved-keys", "type-and-script");
  for (const patch of [
    { ruleId: "script/future-rule" },
    { ruleDocsPath: "docs/content/rules/vue.md" },
    { ruleDocsPath: "docs/content/rules/reference/script-no-reserved-keys.md" },
    { ruleDocsPath: "https://example.invalid/rule" },
    { severity: 0 },
  ])
    assert.throws(() => currentRuleDocumentationMessage({ ...original, ...patch }));
  const packet = [{ file: "Probe.vue", messages: [original], errorCount: 1, warningCount: 0 }];
  for (const patch of [
    { errorCount: 0 },
    { warningCount: 1 },
    { errorCount: "1" },
    { foreignCounter: 0 },
    { file: 1 },
    { messages: null },
  ])
    assert.throws(() => currentRuleDocumentationPacket([{ ...packet[0], ...patch }]));
  assert.deepEqual(
    currentRuleDocumentationPacket([
      { file: "Clean.vue", messages: [], errorCount: 0, warningCount: 0 },
    ]),
    [{ file: "Clean.vue", messages: [], errorCount: 0, warningCount: 0 }],
  );
  assert.equal(asString(original.ruleDocsPath), "docs/content/rules/type-and-script.md");
});
