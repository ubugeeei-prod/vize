import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { posix, resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";
import { renderedMoreGuideRoutes } from "../../docs/scripts/more-guides-coverage.ts";
import { uiLibRegistryOptions } from "../../npm/ui/scripts/build-source-registry.ts";
import { buildLibRegistry, sha256Hex } from "../../npm/ui/scripts/source-registry-bundle/bundle.ts";
import { collectExamples } from "../../npm/ui/scripts/source-registry-bundle/examples.ts";
import { createPackageLibRegistryInput } from "../../npm/ui/scripts/source-registry-bundle/write.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (path: string) => readFileSync(resolve(root, path), "utf8");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const documents = ["docs/content/guide/lib-pull.md", "docs/content/ja/guide/lib-pull.md"];
const section =
  /<!-- First-copy reader example: start -->\n[\s\S]+?<!-- First-copy reader example: end -->\n/g;
const fences = (source: string) =>
  [...source.matchAll(/^```[^\n]*\n([\s\S]*?)^```$/gm)].map((match) => match[1]);

await test("first-copy guides render without errors and their in-page links resolve", () => {
  for (const path of documents) {
    const source = read(path);
    const route = path.replace(/^docs\/content/, "").replace(/\.md$/, "");
    assert.ok(renderedMoreGuideRoutes.includes(route), `${route}: rendered in the browser checks`);
    assert.equal([...source.matchAll(section)].length, 1, `${path}: one first-copy example`);
    const rendered = native.transform(source, {});
    assert.deepEqual(rendered.errors, []);
    const ids = [...rendered.html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
    for (const [, fragment] of rendered.html.matchAll(/\bhref="#([^"]+)"/g)) {
      assert.equal(
        ids.filter((value) => value === decodeURIComponent(fragment)).length,
        1,
        `${path}: #${fragment}`,
      );
    }
  }
});

await test("the copied App example resolves to actual registry bytes and retains peer refusal", () => {
  const input = createPackageLibRegistryInput(uiLibRegistryOptions());
  const bundle = buildLibRegistry(input);
  const rating = bundle.manifest.items.find((item) => item.name === "rating");
  assert.ok(rating);
  const example = rating.examples.find((item) => item.path.endsWith("/rating-basic.vue"));
  assert.ok(example);
  assert.deepEqual(
    bundle.files.get(example.path),
    readFileSync(resolve(input.sourceRoot, example.path)),
  );
  const closure = new Set(
    [
      rating,
      ...rating.registryDependencies.map((name) => {
        const dependency = bundle.manifest.items.find((item) => item.name === name);
        assert.ok(dependency);
        return dependency;
      }),
    ].flatMap((item) => item.files.map((file) => file.path)),
  );
  const allowed = new Set([
    ...Object.keys(input.peerDependencies),
    ...Object.keys(input.dependencies),
  ]);
  const collect = (files: ReadonlySet<string>, packages: ReadonlySet<string>) =>
    collectExamples(
      input.sourceRoot,
      "rating",
      ["rating"],
      rating.entry,
      files,
      packages,
      sha256Hex,
    );
  assert.deepEqual(
    collect(closure, allowed).examples.find((item) => item.path === example.path),
    example,
  );
  assert.throws(() => collect(closure, new Set()), /imports undeclared package "vue"/);
  assert.throws(() => collect(new Set(), allowed), /outside the item's registry closure/);
  const copied = posix.join(bundle.manifest.defaultTargetDirectory, example.path);
  for (const path of documents) {
    const addition = read(path).match(section)?.[0];
    assert.ok(addition);
    const [commands, app] = fences(addition);
    assert.ok(commands && app);
    for (const line of commands.trim().split("\n"))
      assert.match(line, /^vpx vize lib (?:info|pull) rating\b/, `${path}: ${line}`);
    const target = app.match(/import RatingExample from "([^"]+)"/)?.[1];
    assert.ok(target);
    assert.equal(posix.normalize(posix.join("src", target)), copied, `${path}: import target`);
    assert.ok(addition.includes(`\`${copied}\``), `${path}: documented copy destination`);
  }
});

await test("translation regeneration retains the reviewed first-copy reading flow", async () => {
  const reviewed = await collectReviewedTranslations(
    ["guide/lib-pull.md"],
    resolve(root, "docs/content/ja"),
  );
  assert.equal(reviewed.get("guide/lib-pull.md"), read("docs/content/ja/guide/lib-pull.md"));
});
