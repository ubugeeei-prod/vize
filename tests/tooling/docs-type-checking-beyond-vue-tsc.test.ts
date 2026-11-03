// The "Type Checking Beyond vue-tsc" page quotes its examples from the fixtures
// that the real-tsgo canon test pins (`recent_issues::beyond_vue_tsc`), so the
// page cannot drift from what Vize reports.
import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const fixtures = join(root, "tests/_fixtures/type-checking-beyond-vue-tsc");
const pages = [
  "docs/content/guide/type-checking-beyond-vue-tsc.md",
  "docs/content/ja/guide/type-checking-beyond-vue-tsc.md",
];
const CASE = /tests\/_fixtures\/type-checking-beyond-vue-tsc\/([a-z0-9-]+)/u;
const MARKER = /^<!-- (src\/[\w.-]+\.vue)( \(fixed\))? -->\n/u;
const VUE_TSC_ROW = /\(\d+,\d+\): error TS\d+:/u;

interface Block {
  lang: string;
  body: string;
}

const read = (path: string) => readFileSync(path, "utf8");
const page = (path: string) => read(join(root, path));
const blocks = (markdown: string): Block[] =>
  [...markdown.matchAll(/^```(\w*)\n([\s\S]*?)^```$/gmu)].map(([, lang = "", body = ""]) => ({
    lang,
    body,
  }));
const sections = (markdown: string) => markdown.split(/^### /mu).slice(1);
const cases = () =>
  readdirSync(fixtures, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
const sources = (dir: string) =>
  existsSync(join(dir, "src"))
    ? readdirSync(join(dir, "src")).map((file) => `src/${file.replace(/\.txt$/u, "")}`)
    : [];
const optional = (path: string) => (existsSync(path) ? read(path) : "");

for (const path of pages) {
  void test(`${path} quotes every pinned case byte for byte`, () => {
    const covered: string[] = [];
    for (const section of sections(page(path))) {
      const name = section.match(CASE)?.[1];
      if (!name) continue;
      covered.push(name);
      const dir = join(fixtures, name);
      assert.ok(existsSync(dir), `${path}: unknown case ${name}`);
      const shown = new Set<string>();
      let vize = 0;
      for (const { lang, body } of blocks(section)) {
        if (lang === "vue") {
          const marker = body.match(MARKER);
          assert.ok(marker, `${path} ${name}: a vue block must name its file`);
          const [, file, fixed] = marker;
          const fixture = join(dir, fixed ? "fixed" : "", `${file}.txt`);
          assert.equal(body.slice(marker[0].length), read(fixture), `${path} ${name} ${file}`);
          if (!fixed) shown.add(file);
        } else if (lang === "text" && body.includes("[TS")) {
          assert.ok(
            [read(join(dir, "expected.txt")), optional(join(dir, "fixed/expected.txt"))].includes(
              body,
            ),
            `${path} ${name}: a Vize diagnostic block must match expected.txt`,
          );
          vize += 1;
        } else if (lang === "text" && VUE_TSC_ROW.test(body)) {
          assert.ok(
            [read(join(dir, "vue-tsc.txt")), optional(join(dir, "fixed/vue-tsc.txt"))].includes(
              body,
            ),
            `${path} ${name}: a vue-tsc block must match vue-tsc.txt`,
          );
        }
      }
      assert.deepEqual([...shown].sort(), sources(dir).sort(), `${path} ${name}: authored files`);
      assert.ok(vize > 0, `${path} ${name}: shows no Vize diagnostic`);
    }
    assert.deepEqual(covered.sort(), cases(), `${path} must cover every pinned case once`);
  });

  void test(`${path} cites only existing fixtures`, () => {
    const links = [...page(path).matchAll(/tree\/main\/(tests\/_fixtures\/[^)\s]+)/gu)];
    assert.ok(links.length > 0);
    for (const [, fixture = ""] of links) {
      assert.ok(existsSync(join(root, fixture)), `${path} cites missing ${fixture}`);
    }
  });
}

void test("translations keep every code block of the English page", () => {
  const [english = "", ...translations] = pages;
  for (const translation of translations) {
    assert.deepEqual(blocks(page(translation)), blocks(page(english)), translation);
  }
});
