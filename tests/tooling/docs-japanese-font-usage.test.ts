import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { usesInstalledCjkFont } from "../../docs/scripts/japanese-font-usage.ts";

const packet = JSON.parse(
  readFileSync(new URL("./fixtures/docs-japanese-font-packet.json", import.meta.url), "utf8"),
) as { fonts: { familyName: string; glyphCount: number; isCustomFont: boolean }[] };

void test("the observed installed Japanese monospace glyphs satisfy the browser font gate", () => {
  assert.equal(usesInstalledCjkFont(packet.fonts), true);
  assert.equal(
    usesInstalledCjkFont([{ familyName: "Noto Sans CJK JP", glyphCount: 2 }]),
    true,
    "retain the existing proportional CJK acceptance",
  );
});

void test("Japanese font acceptance rejects missing, wrong, custom and unused fonts", () => {
  const mono = packet.fonts[0];
  assert.ok(mono);
  assert.equal(usesInstalledCjkFont([]), false);
  for (const familyName of [
    "Arial",
    "Noto Sans",
    "Noto Sans Mono",
    "Noto Sans Mono CJK",
    "Noto Sans Mono CJK SC",
    "Other Noto Sans Mono CJK JP",
  ])
    assert.equal(usesInstalledCjkFont([{ ...mono, familyName }]), false, familyName);
  assert.equal(usesInstalledCjkFont([{ ...mono, isCustomFont: true }]), false);
  assert.equal(usesInstalledCjkFont([{ familyName: mono.familyName, glyphCount: 2 }]), false);
  for (const glyphCount of [0, -1]) {
    assert.equal(usesInstalledCjkFont([{ ...mono, glyphCount }]), false);
    assert.equal(usesInstalledCjkFont([{ familyName: "Noto Sans CJK JP", glyphCount }]), false);
  }
  assert.equal(
    usesInstalledCjkFont([
      { ...mono, glyphCount: 0 },
      { familyName: "Arial", glyphCount: 2 },
    ]),
    false,
    "non-CJK glyphs cannot credit an unused CJK font",
  );
});
