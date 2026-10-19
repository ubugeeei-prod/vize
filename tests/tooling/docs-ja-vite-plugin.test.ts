import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";
import { inboundFragments, missingInboundAnchors } from "./support/docs-inbound-anchors.ts";

const content = resolve(import.meta.dirname, "../../docs/content");
const path = "guide/vite-plugin.md";
const english = readFileSync(resolve(content, path), "utf8");
const japanese = readFileSync(resolve(content, "ja", path), "utf8");

void test("translation regeneration keeps the reviewed Japanese Vite guide", async () => {
  const retained = await collectReviewedTranslations([path], resolve(content, "ja"));
  assert.equal(retained.get(path), japanese);
});

void test("Japanese Vite guide documents the same integration options as English", () => {
  const optionKeys = (source: string) =>
    Array.from(source.matchAll(/^\| (`[^`]+`)\s*\|/gm), (match) => match[1]);
  assert.ok(optionKeys(english).length > 0);
  assert.deepEqual(optionKeys(japanese), optionKeys(english));
});

void test("Vite guide anchors that other pages link to exist", () => {
  for (const page of [path, `ja/${path}`]) {
    assert.ok(inboundFragments(page).size > 0, `${page} has inbound fragment links`);
    assert.deepEqual(missingInboundAnchors(page), [], page);
  }
});
