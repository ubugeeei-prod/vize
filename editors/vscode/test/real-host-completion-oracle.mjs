import assert from "node:assert/strict";

// The host smoke asks the packaged Vize LanguageClient for completions at the
// `{{ label }}` template expression. Keeping the oracle here lets both the real
// extension host and the tooling tests run the same assertions, so a broken
// command gate, registration, or request forwarding fails somewhere.
// Mirrors `HOST_TEST_COMPLETION_COMMAND` in `src/host-test-core.ts`; the tooling
// tests assert both stay in sync.
export const HOST_TEST_COMPLETION_COMMAND = "vize.test.executeCompletion";
export const REQUIRED_TEMPLATE_COMPLETION_LABELS = ["Child", "amount", "label"];
export const FORBIDDEN_TEMPLATE_COMPLETION_LABELS = [
  "Fake Vize Completion",
  "v-bind",
  "v-else",
  "v-else-if",
  "v-for",
  "v-html",
  "v-if",
  "v-model",
  "v-on",
  "v-once",
  "v-pre",
  "v-show",
  "v-slot",
  "v-text",
];

export function completionLabels(response) {
  const items = Array.isArray(response) ? response : (response?.items ?? []);
  return items.map((item) => (typeof item.label === "string" ? item.label : item.label.label));
}

export function assertRealHostCompletionLabels(response) {
  const labels = completionLabels(response);

  for (const required of REQUIRED_TEMPLATE_COMPLETION_LABELS) {
    assert.ok(
      labels.includes(required),
      `template completion must include script binding ${JSON.stringify(required)}: ${JSON.stringify(labels)}`,
    );
  }
  for (const forbidden of FORBIDDEN_TEMPLATE_COMPLETION_LABELS) {
    assert.equal(
      labels.includes(forbidden),
      false,
      `template expression completion must not surface ${JSON.stringify(forbidden)}: ${JSON.stringify(labels)}`,
    );
  }

  return labels;
}
