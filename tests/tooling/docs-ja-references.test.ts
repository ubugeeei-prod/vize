import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const content = resolve(root, "docs/content");
const paths = ["guide/compiler-configuration-reference.md", "guide/experimentals-reference.md"];
const references = paths.flatMap((path) => [join(content, path), join(content, "ja", path)]);
const ids = (file: string) =>
  new Set(
    Array.from(
      native.transform(readFileSync(file, "utf8"), {}).html.matchAll(/\bid="([^"]+)"/g),
      (match: RegExpMatchArray) => match[1],
    ),
  );
const pages = (directory: string): string[] =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return entry.name === "generated" ? [] : pages(path);
    return entry.name.endsWith(".md") ? [path] : [];
  });

void test("configuration references render and every fragment link into them resolves", () => {
  for (const file of references)
    assert.deepEqual(native.transform(readFileSync(file, "utf8"), {}).errors, [], file);
  for (const file of [join(content, "guide"), join(content, "ja/guide")].flatMap(pages)) {
    const source = readFileSync(file, "utf8")
      .replace(/^```[\s\S]*?^```/gm, "")
      .replace(/`[^`\n]*`/g, "");
    for (const [, target, fragment] of source.matchAll(/\]\(([^)\s#:]*)#([^)\s]+)\)/g)) {
      const destination = target ? resolve(dirname(file), target) : file;
      if (!references.includes(destination)) continue;
      assert.ok(ids(destination).has(fragment), `${file.slice(root.length + 1)}: #${fragment}`);
    }
  }
});

void test("translation regeneration retains both reviewed Japanese references", async () => {
  const retained = await collectReviewedTranslations(paths, join(content, "ja"));
  for (const path of paths) {
    const reviewed = readFileSync(join(content, "ja", path), "utf8");
    assert.equal(retained.get(path), reviewed, path);
    assert.ok(reviewed.includes(`<!-- Reviewed translation; source: ${path}`), path);
  }
});
