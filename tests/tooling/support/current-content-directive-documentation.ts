import assert from "node:assert/strict";
import fs from "node:fs";

// Preserve the authored #8018 corpus; project its one rule only for current CLI reports.
export function currentContentDirectivePacket(expected: unknown): unknown[] {
  assert.ok(Array.isArray(expected));
  for (const locale of ["", "ja/"]) {
    assert.ok(
      fs.existsSync(
        new URL(
          `../../../docs/content/${locale}rules/reference/html-no-empty-palpable-content.md`,
          import.meta.url,
        ),
      ),
    );
  }
  return expected.map((row: { messages: unknown[] }) => ({
    ...row,
    messages: row.messages.map((entry) => {
      assert.ok(entry !== null && typeof entry === "object");
      const message = entry as Record<string, unknown>;
      assert.equal(
        message.ruleId,
        "html/no-empty-palpable-content",
        "authored content-directive rule",
      );
      assert.equal(
        message.ruleDocsPath,
        "docs/content/rules/html.md",
        "frozen content-directive documentation",
      );
      return {
        ...message,
        ruleDocsPath: "docs/content/rules/reference/html-no-empty-palpable-content.md",
      };
    }),
  }));
}
