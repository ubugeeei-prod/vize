import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { inboundFragments, missingInboundAnchors } from "./support/docs-inbound-anchors.ts";

const read = (path: string) =>
  readFileSync(new URL(`../../docs/content/${path}`, import.meta.url), "utf8");
const commands = (source: string) =>
  new Set(Array.from(source.matchAll(/^\| `([^`]+)`\s*\|/gm), (match) => match[1]));

test("Japanese CLI guide lists every command in the English command tables", () => {
  const japanese = commands(read("ja/guide/cli.md"));
  const english = commands(read("guide/cli.md"));
  assert.ok(english.size > 0);
  for (const command of english) assert.ok(japanese.has(command), `missing command: ${command}`);
});

test("CLI guide anchors that other pages link to exist", () => {
  for (const page of ["guide/cli.md", "ja/guide/cli.md"]) {
    assert.ok(inboundFragments(page).size > 0, `${page} has inbound fragment links`);
    assert.deepEqual(missingInboundAnchors(page), [], page);
  }
});
