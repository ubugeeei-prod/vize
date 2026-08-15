import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  ELK_E2E_OPTIMIZE_DEPS,
  patchElkViteOptimizeDeps,
} from "../_helpers/app-fixture-runtime.ts";

const ELK_OPTIMIZE_DEPS_UNRELATED_ENTRIES = ["string-length", "workbox-expiration"] as const;

test("elk setup pre-bundles every explore route lazy dependency that is absent", (t) => {
  const configPath = writeElkOptimizeDepsFixture(t, ELK_OPTIMIZE_DEPS_UNRELATED_ENTRIES);

  patchElkViteOptimizeDeps(configPath);

  const patched = fs.readFileSync(configPath, "utf8");
  assert.deepEqual(extractOptimizeDepsInclude(patched), [
    ...ELK_OPTIMIZE_DEPS_UNRELATED_ENTRIES,
    ...ELK_E2E_OPTIMIZE_DEPS,
  ]);

  patchElkViteOptimizeDeps(configPath);
  assert.equal(fs.readFileSync(configPath, "utf8"), patched);
});

test("elk setup pre-bundles explore route lazy dependencies without duplicating existing entries", (t) => {
  const configPath = writeElkOptimizeDepsFixture(t, [
    "punycode/",
    ...ELK_OPTIMIZE_DEPS_UNRELATED_ENTRIES,
  ]);

  patchElkViteOptimizeDeps(configPath);

  const patched = fs.readFileSync(configPath, "utf8");
  const includeDeps = extractOptimizeDepsInclude(patched);
  for (const dep of ELK_E2E_OPTIMIZE_DEPS) {
    assert.equal(countOccurrences(includeDeps, dep), 1);
  }
  assert.deepEqual(includeDeps, [
    "punycode/",
    ...ELK_OPTIMIZE_DEPS_UNRELATED_ENTRIES,
    "virtua/vue",
  ]);

  patchElkViteOptimizeDeps(configPath);
  assert.equal(fs.readFileSync(configPath, "utf8"), patched);
});

function writeElkOptimizeDepsFixture(
  t: { after: (fn: () => void) => void },
  includeEntries: readonly string[],
): string {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-elk-optimize-deps-"));
  t.after(() => fs.rmSync(tempDir, { recursive: true, force: true }));

  const configPath = path.join(tempDir, "nuxt.config.ts");
  fs.writeFileSync(
    configPath,
    [
      "export default defineNuxtConfig({",
      "  unrelated: \"'virtua/vue' should not satisfy optimizeDeps\",",
      "  vite: {",
      "    optimizeDeps: {",
      "      include: [",
      // The first entry is double quoted and the last omits its separator so the
      // patch has to cope with either source style.
      ...includeEntries.map((entry, index) => {
        const quoted = index === 0 ? `"${entry}"` : `'${entry}'`;
        return `        ${quoted}${index === includeEntries.length - 1 ? "" : ","}`;
      }),
      "      ],",
      "    },",
      "  },",
      "})",
      "",
    ].join("\n"),
  );

  return configPath;
}

function extractOptimizeDepsInclude(config: string): string[] {
  const lines = extractOptimizeDepsIncludeLines(config);
  // Every entry but the last needs its separator, otherwise the patched config
  // is not a parseable array literal.
  for (const line of lines.slice(0, -1)) {
    assert.ok(line.trimEnd().endsWith(","), `missing separator after ${line.trim()}`);
  }
  return parseOptimizeDepsIncludeLines(lines);
}

function parseOptimizeDepsIncludeLines(lines: readonly string[]): string[] {
  return Array.from(lines, (line) => {
    const match = line.match(/^\s*['"]([^'"]+)['"],?\s*$/);
    assert.ok(match);
    return match[1]!;
  });
}

function extractOptimizeDepsIncludeLines(config: string): string[] {
  const includeAnchor = "    optimizeDeps: {\n      include: [\n";
  const includeStart = config.indexOf(includeAnchor);
  assert.notEqual(includeStart, -1);

  const includeBodyStart = includeStart + includeAnchor.length;
  const includeBodyEnd = config.indexOf("\n      ],", includeBodyStart);
  assert.notEqual(includeBodyEnd, -1);

  const includeBody = config.slice(includeBodyStart, includeBodyEnd);
  return includeBody.split("\n").filter((line) => line.trim().length > 0);
}

function countOccurrences(items: readonly string[], item: string): number {
  return items.filter((value) => value === item).length;
}
