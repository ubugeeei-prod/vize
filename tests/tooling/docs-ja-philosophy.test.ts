import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const pages = ["docs/content/philosophy.md", "docs/content/ja/philosophy.md"];
const read = (file: string): string => readFileSync(resolve(root, file), "utf8");
const ids = (file: string) =>
  new Set(
    Array.from(
      native.transform(readFileSync(file, "utf8"), {}).html.matchAll(/\bid="([^"]+)"/g),
      (match: RegExpMatchArray) => match[1],
    ),
  );

void test("philosophy pages render and their fragment links reach existing anchors", () => {
  for (const page of pages) {
    assert.deepEqual(native.transform(read(page), {}).errors, [], page);
    const file = resolve(root, page);
    const source = read(page)
      .replace(/^```[\s\S]*?^```/gm, "")
      .replace(/`[^`\n]*`/g, "");
    for (const [, target, fragment] of source.matchAll(/\]\(([^)\s#:]*)#([^)\s]+)\)/g)) {
      const destination = target ? resolve(dirname(file), target) : file;
      assert.ok(ids(destination).has(fragment), `${page}: ${target}#${fragment}`);
    }
  }
});

void test("the Japanese art-name table names the same crates as English", () => {
  const names = (source: string) =>
    source
      .split("\n")
      .filter((line) => line.startsWith("| **"))
      .map((line) => line.split("|")[1].trim());
  const [english, japanese] = pages.map((page) => names(read(page)));
  assert.ok(english.length > 0);
  assert.deepEqual(japanese, english);
});

void test("translation regeneration retains the reviewed Japanese philosophy", async () => {
  const retained = await collectReviewedTranslations(
    ["philosophy.md"],
    resolve(root, "docs/content/ja"),
  );
  const reviewed = read("docs/content/ja/philosophy.md");
  assert.equal(retained.get("philosophy.md"), reviewed);
  assert.ok(reviewed.includes("<!-- Reviewed translation; source: philosophy.md"));
});
