import assert from "node:assert/strict";
import fs from "node:fs";

const oldReference = "docs/content/rules/musea-and-css.md";
const currentPages = {
  "css/no-display-none": "css-no-display-none",
  "css/no-id-selectors": "css-no-id-selectors",
  "css/no-utility-classes": "css-no-utility-classes",
} as const;

type GeometryProjection = "none" | "display-utf16" | "utility-utf16";

function currentUtf16Geometry(
  message: Record<string, unknown>,
  projection: GeometryProjection,
): Record<string, unknown> {
  if (
    projection === "display-utf16" &&
    message.ruleId === "css/no-display-none" &&
    message.line === 3 &&
    message.column === 22 &&
    message.endLine === 3 &&
    message.endColumn === 29
  ) {
    return { ...message, column: 23, endColumn: 30 };
  }
  if (
    projection === "utility-utf16" &&
    message.ruleId === "css/no-utility-classes" &&
    message.line === 2 &&
    message.column === 29 &&
    message.endLine === 2 &&
    message.endColumn === 34
  ) {
    return { ...message, column: 30, endColumn: 35 };
  }
  return message;
}

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

export function currentCssPacket(
  expected: unknown,
  geometry: GeometryProjection = "none",
): unknown[] {
  assert.ok(Array.isArray(expected));
  return expected.map((row: { messages: unknown[] }) => ({
    ...row,
    messages: row.messages.map((entry) => {
      assert.ok(entry !== null && typeof entry === "object");
      const message = entry as Record<string, unknown>;
      const reference = currentReference(message.ruleId);
      assert.equal(message.ruleDocsPath, oldReference, "frozen CSS documentation URL");
      return { ...currentUtf16Geometry(message, geometry), ruleDocsPath: reference };
    }),
  }));
}

export function currentCssPlain(expected: string, geometry: GeometryProjection = "none"): string {
  let reference: string | null = null;
  const geometryAdjusted =
    geometry === "utility-utf16"
      ? expected.replace(
          "UnicodeMultipleStyles.vue:2:29 warning css/no-utility-classes",
          "UnicodeMultipleStyles.vue:2:30 warning css/no-utility-classes",
        )
      : expected;
  return geometryAdjusted
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
  readonly id?: string;
  readonly source?: string;
  readonly expectedCli: unknown;
  readonly expectedPlain?: string;
}

// These are expectation-only clones. Frozen corpus readers and actual packets
// retain their original bytes and full comparisons.
export function currentCssCases<T extends CssCase>(cases: readonly T[]): T[] {
  return cases.map((row) => {
    const geometry =
      row.id === "utf8-crlf" && row.source?.includes('.日本[data-note="😀"]')
        ? "display-utf16"
        : "none";
    return {
      ...row,
      expectedCli: currentCssPacket(row.expectedCli, geometry),
      ...(row.expectedPlain === undefined
        ? {}
        : { expectedPlain: currentCssPlain(row.expectedPlain, geometry) }),
    };
  });
}

export function currentCssReport<T extends { json: unknown; plain: string }>(expected: T): T {
  return {
    ...expected,
    json: currentCssPacket(expected.json, "utility-utf16"),
    plain: currentCssPlain(expected.plain, "utility-utf16"),
  };
}
