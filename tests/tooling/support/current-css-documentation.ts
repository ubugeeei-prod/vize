import assert from "node:assert/strict";
import fs from "node:fs";

const oldReference = "docs/content/rules/musea-and-css.md";
const currentReference = "docs/content/rules/reference/css-no-display-none.md";

export function currentCssPacket(expected: unknown): unknown[] {
  assert.ok(Array.isArray(expected));
  for (const locale of ["", "ja/"]) {
    assert.ok(
      fs.existsSync(
        new URL(
          `../../../docs/content/${locale}rules/reference/css-no-display-none.md`,
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
      assert.equal(message.ruleId, "css/no-display-none", "finite historical CSS rule");
      assert.equal(message.ruleDocsPath, oldReference, "frozen CSS documentation URL");
      return { ...message, ruleDocsPath: currentReference };
    }),
  }));
}

function currentPlain(expected: string): string {
  let owned = false;
  return expected
    .split(/(?<=\n)/u)
    .map((line) => {
      if (line.includes(" css/no-display-none ")) owned = true;
      if (!line.startsWith("    Reference: ")) return line;
      assert.ok(owned, "preceding authored CSS finding");
      assert.equal(line.trimEnd(), `    Reference: ${oldReference}`);
      owned = false;
      return `    Reference: ${currentReference}${line.endsWith("\n") ? "\n" : ""}`;
    })
    .join("");
}

interface CssCase {
  readonly expectedCli: unknown;
  readonly expectedPlain?: string;
}

// These are expectation-only clones. Frozen corpus readers and actual packets
// retain their original bytes and full comparisons.
export function currentCssCases<T extends CssCase>(cases: readonly T[]): T[] {
  return cases.map((row) => ({
    ...row,
    expectedCli: currentCssPacket(row.expectedCli),
    ...(row.expectedPlain === undefined ? {} : { expectedPlain: currentPlain(row.expectedPlain) }),
  }));
}
