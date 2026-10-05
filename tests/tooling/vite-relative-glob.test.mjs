import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

const directory = new URL(
  "../_fixtures/differential/compiler/vite-relative-glob/",
  import.meta.url,
);
const read = (name) => fs.readFileSync(new URL(name, directory));
const corpus = JSON.parse(read("corpus.json"));
await test("#7936 preserves all nine original blocks and declared standalone/Nuxt controls", () => {
  for (const [name, pin] of Object.entries(corpus.files)) {
    assert.equal(read(name).length, pin.bytes);
    assert.equal(createHash("sha256").update(read(name)).digest("hex"), pin.sha256);
  }
  const blocks = [
    ...read("original-issue.md.txt")
      .toString()
      .matchAll(/```[^\n]*\n([\s\S]*?)```/g),
  ].map((match) => match[1]);
  assert.equal(blocks.length, 9);
  assert.deepEqual(
    [
      "layout.txt",
      "package.json.txt",
      "vite.config.ts.txt",
      "vite.src-root.config.ts.txt",
      "index.html.txt",
      "main.ts.txt",
      "Plain.vue.txt",
      "Alpha.vue.txt",
      "reproduction.sh.txt",
    ].map((name) => read(name).toString()),
    blocks,
  );
  assert.equal(
    read("src-index.html.txt").toString(),
    blocks[4].replace('src="/src/main.ts"', 'src="./main.ts"'),
  );
  assert.equal(corpus.reporter.githubId, 71201308);
  assert.equal(
    corpus.reporter.coauthor,
    "Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  );
  const packageJson = fs.readFileSync(
    new URL(
      "../../tools/support/compat/nuxt/fixtures/vite-relative-glob/package.json",
      import.meta.url,
    ),
  );
  assert.deepEqual(packageJson, read("package.json.txt"));
  const lock = JSON.parse(
    fs.readFileSync(
      new URL(
        "../../tools/support/compat/nuxt/fixtures/vite-relative-glob/package-lock.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  for (const [name, version] of Object.entries(JSON.parse(packageJson).devDependencies))
    assert.equal(lock.packages[`node_modules/${name}`].version, version);
});

await test("whole fixed references retain relative spelling, successful lookup, raw contents and no-match behavior", () => {
  assert.deepEqual(corpus.roots, ["project", "src"]);
  assert.deepEqual(corpus.scenarios, ["plain", "lookup", "options"]);
  assert.deepEqual(corpus.expected.plain, {
    innerHtml: "<p>../../../fixtures/Alpha.vue</p>",
    ssrHtml: "<p>../../../fixtures/Alpha.vue</p>",
  });
  assert.deepEqual(corpus.expected.lookup, { innerHtml: "<p>alpha</p>", ssrHtml: "<p>alpha</p>" });
  const escaped = read("Alpha.vue.txt")
    .toString()
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  const whole = `<main><p>../../../control-fixtures/First.vue</p><p>../../../fixtures/Alpha.vue</p><p>./Alpha.vue</p><p>(none)</p><pre>${escaped}</pre></main>`;
  assert.deepEqual(corpus.expected.options, { innerHtml: whole, ssrHtml: whole });
  assert.notEqual(whole, whole.replace("../../../fixtures/Alpha.vue", "/fixtures/Alpha.vue"));
  assert.notEqual(whole, whole.replace("<p>../../../fixtures/Alpha.vue</p>", "<p>(none)</p>"));
});
