import assert from "node:assert/strict";

// Mirrors `HOST_TEST_LSP_REQUEST_COMMAND` in `src/host-test-core.ts`; the
// tooling tests assert both stay in sync.
export const HOST_TEST_LSP_REQUEST_COMMAND = "vize.test.executeLspRequest";

export function assertRealHostTemplateBindingHover(actual) {
  const markdown = hoverToMarkdown(actual);

  assert.match(markdown, /^```typescript\n/);
  assert.ok(
    markdown.includes('const label: "hello from vize"'),
    `raw LSP hover must report the backend type of the binding: ${JSON.stringify(markdown)}`,
  );
  assert.doesNotMatch(
    markdown,
    /TypeScript quick info|Template binding from script|Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>/,
  );

  return markdown;
}

function hoverToMarkdown(hover) {
  assert.ok(
    hover?.contents,
    `real host raw LSP hover must include contents: ${JSON.stringify(hover)}`,
  );

  return markupToMarkdown(hover.contents);
}

function markupToMarkdown(markup) {
  if (typeof markup === "string") {
    return markup;
  }
  if (Array.isArray(markup)) {
    return markup.map(markupToMarkdown).join("\n");
  }
  if (typeof markup?.value === "string") {
    if (typeof markup.language === "string") {
      return `\`\`\`${markup.language}\n${markup.value}\n\`\`\``;
    }
    return markup.value;
  }

  assert.fail(`unsupported real host hover contents: ${JSON.stringify(markup)}`);
}
