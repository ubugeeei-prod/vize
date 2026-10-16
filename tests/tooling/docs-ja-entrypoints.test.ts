import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = resolve(import.meta.dirname, "../..");
const directory = resolve(root, "docs/content/ja");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const read = (file: string) => readFileSync(resolve(directory, file), "utf8");
const frontmatter = (source: string) => parse(source.split("---")[1] ?? "");
const fixture = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-ja-entrypoints.json"), "utf8"),
) as { documents: { file: string; codeSha256: string[]; headingIds: string[] }[] };

test("Japanese entrypoint review retains every complete example and original bookmark", () => {
  let examples = 0;
  let bookmarks = 0;
  for (const document of fixture.documents) {
    const source = read(document.file);
    const result = native.transform(source, {});
    assert.deepEqual(result.errors, [], document.file);
    assert.deepEqual(
      Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (match) =>
        createHash("sha256").update(match[1]!).digest("hex"),
      ),
      document.codeSha256,
      `${document.file}: complete original examples`,
    );
    const ids = Array.from((result.html as string).matchAll(/\bid="([^"]+)"/g), (m) => m[1]);
    for (const id of document.headingIds) {
      assert.equal(ids.filter((value) => value === id).length, 1, `${document.file}#${id}`);
      bookmarks += 1;
    }
    examples += document.codeSha256.length;
  }
  assert.equal(examples, 13);
  assert.equal(bookmarks, 27);
});

test("Japanese setup links to the existing Japanese editor and init guides", () => {
  const source = read("getting-started.md");
  for (const file of ["guide/vite-plus-editor.md", "guide/init.md"]) {
    assert.ok(source.includes(`](./${file})`), `${file}: Japanese reading path`);
    assert.ok(!source.includes(`](../${file})`), `${file}: no needless English detour`);
    assert.ok(read(file).includes("<!-- Reviewed translation;"), `${file}: reviewed destination`);
  }
  assert.ok(!source.includes("エディターを設定する（英語）"));
  assert.ok(!source.includes("プロジェクト設定（英語）"));
});

test("the landing page distinguishes shared tool configuration and the TypeScript project", async () => {
  const landing = frontmatter(read("index.md"));
  assert.ok(!landing.hero.tagline.includes("すべて Rust"));
  const setup = landing.features.find((feature: { link: string }) =>
    feature.link.endsWith("getting-started.md"),
  );
  assert.ok(setup.details.includes("vite.config.ts"));
  assert.ok(setup.details.includes("tsconfig.json"));
  await import("../../docs/theme/i18n/locales/ja.js");
  const locales = (
    globalThis as { __vizeDocsLocales?: Record<string, { labels: Record<string, string> }> }
  ).__vizeDocsLocales!;
  assert.equal(
    locales.ja!.labels["/getting-started"],
    frontmatter(read("getting-started.md")).title,
  );
  const configuration = read("guide/configuration.md");
  assert.ok(!configuration.includes("共有の全体 ignore"));
  assert.ok(!configuration.includes("ナビゲーションの診断"));
  assert.ok(configuration.includes("構文・型の診断やコードナビゲーション"));
});

test("regeneration retains all four complete reviewed Japanese entrypoints", async () => {
  const files = fixture.documents.map(({ file }) => file);
  const retained = await collectReviewedTranslations(files, directory);
  assert.equal(retained.size, files.length);
  for (const file of files) assert.equal(retained.get(file), read(file), file);
});
