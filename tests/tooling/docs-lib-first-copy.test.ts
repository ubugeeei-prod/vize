import assert from "node:assert/strict";
import { originalConfigDocument } from "./support/docs-config-archive.ts";
import { currentConfigExample } from "./support/docs-config-successor.ts";
import { createHash } from "node:crypto";
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
const baseline = JSON.parse(read("tests/tooling/fixtures/docs-lib-first-copy.json")) as {
  base: string;
  documents: { path: string; sha256: string; headings: string[] }[];
};
const sha = (source: string) => createHash("sha256").update(source).digest("hex");
const section =
  /<!-- First-copy reader example: start -->\n[\s\S]+?<!-- First-copy reader example: end -->\n\n/g;
const fences = (source: string) =>
  [...source.matchAll(/^```[^\n]*\n([\s\S]*?)^```$/gm)].map((match) => match[1]);

await test("first-copy guides preserve every original byte and every native bookmark", () => {
  assert.equal(baseline.base, "479402e2bd77a1c7b3a4025293b884965d6e2b46");
  for (const document of baseline.documents) {
    const source = read(document.path);
    const route = document.path.replace(/^docs\/content/, "").replace(/\.md$/, "");
    assert.ok(renderedMoreGuideRoutes.includes(route), `${route}: actual native reading campaign`);
    assert.equal([...source.matchAll(section)].length, 1);
    const restored = source.replace(section, "");
    const archived = originalConfigDocument(document.path);
    assert.equal(sha(archived), document.sha256, `${document.path}: complete original document`);
    assert.deepEqual(
      fences(restored),
      fences(archived).map(currentConfigExample),
      `${document.path}: complete current recipes retain the original registry and CLI payloads`,
    );
    assert.equal(fences(restored).length, 8);
    assert.equal(fences(source).length, 10);
    const rendered = native.transform(source, {});
    assert.deepEqual(rendered.errors, []);
    const ids = [...rendered.html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
    for (const id of document.headings)
      assert.equal(ids.filter((value) => value === id).length, 1, id);
    for (const [, fragment] of rendered.html.matchAll(/\bhref="#([^"]+)"/g)) {
      assert.equal(
        ids.filter((value) => value === decodeURIComponent(fragment)).length,
        1,
        fragment,
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
  assert.equal(example.path, "families/form/rating/examples/rating-basic.vue");
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
  const vuePeer = input.peerDependencies.vue;
  assert.ok(vuePeer);
  assert.ok(vuePeer.includes("^3.5.0"));
  for (const document of baseline.documents) {
    const source = read(document.path);
    const addition = source.match(section)?.[0];
    assert.ok(addition);
    const [commands, app] = fences(addition);
    assert.ok(app);
    assert.equal(
      commands,
      "vpx vize lib info rating\nvpx vize lib pull rating --with-examples --dry-run\nvpx vize lib pull rating --with-examples\n",
    );
    const target = app.match(/import RatingExample from "([^"]+)"/)?.[1];
    assert.ok(target);
    assert.equal(
      posix.normalize(posix.join("src", target)),
      posix.join(bundle.manifest.defaultTargetDirectory, example.path),
    );
    assert.match(app, /<RatingExample \/>/);
    assert.match(addition, /Vue 3\.5/);
    assert.match(addition, /`allowImportingTsExtensions`/);
    assert.match(
      addition,
      /does not install dependencies or check their version compatibility|依存のインストールやバージョンの互換性の確認は行いません/,
    );
  }
});

await test("translation regeneration retains the complete authored first-copy reading flow", async () => {
  const reviewed = await collectReviewedTranslations(
    ["guide/lib-pull.md"],
    resolve(root, "docs/content/ja"),
  );
  assert.equal(reviewed.get("guide/lib-pull.md"), read("docs/content/ja/guide/lib-pull.md"));
});
