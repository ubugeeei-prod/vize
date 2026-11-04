import assert from "node:assert/strict";
import fs from "node:fs";

const oldReference = "docs/content/rules/musea-and-css.md";
const currentPages = {
  "css/no-display-none": "css-no-display-none",
  "css/no-id-selectors": "css-no-id-selectors",
  "css/no-utility-classes": "css-no-utility-classes",
} as const;

function currentReference(ruleId: unknown): string {
  assert.ok(
    typeof ruleId === "string" && Object.hasOwn(currentPages, ruleId),
    "finite historical CSS rule",
  );
  const page = currentPages[ruleId as keyof typeof currentPages];
  for (const locale of ["", "ja/"]) {
    assert.ok(
      fs.existsSync(
        new URL(`../../../docs/content/${locale}rules/reference/${page}.md`, import.meta.url),
      ),
    );
  }
  return `docs/content/rules/reference/${page}.md`;
}

export function currentCssPacket(expected: unknown): unknown[] {
  assert.ok(Array.isArray(expected));
  return expected.map((row: { messages: unknown[] }) => ({
    ...row,
    messages: row.messages.map((entry) => {
      assert.ok(entry !== null && typeof entry === "object");
      const message = entry as Record<string, unknown>;
      const reference = currentReference(message.ruleId);
      assert.equal(message.ruleDocsPath, oldReference, "frozen CSS documentation URL");
      return { ...message, ruleDocsPath: reference };
    }),
  }));
}

function currentPlain(expected: string): string {
  let reference: string | null = null;
  return expected
    .split(/(?<=\n)/u)
    .map((line) => {
      for (const rule of Object.keys(currentPages)) {
        if (line.includes(` ${rule} `)) reference = currentReference(rule);
      }
      if (!line.startsWith("    Reference: ")) return line;
      assert.ok(reference, "preceding authored CSS finding");
      assert.equal(line.trimEnd(), `    Reference: ${oldReference}`);
      const current = reference;
      reference = null;
      return `    Reference: ${current}${line.endsWith("\n") ? "\n" : ""}`;
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

export function currentCssReport<T extends { json: unknown; plain: string }>(expected: T): T {
  return {
    ...expected,
    json: currentCssPacket(expected.json),
    plain: currentPlain(expected.plain),
  };
}
