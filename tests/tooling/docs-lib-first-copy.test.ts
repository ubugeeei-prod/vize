import assert from "node:assert/strict";
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
// Japanese prose was reviewed in #9522 (64f070766c3915698bd076574c7ba03dc07d7d98).
// Its original eight code fences remain byte-identical to baseline.base.
const reviewedJapaneseProse = "b565f137d2c726716ea9a1372eb6bde119b852581b432aef6d418ca66ff4dd03";
const originalFenceHashes: Record<string, string> = {
  "docs/content/guide/lib-pull.md":
    "5837f9440664eec71cdb91d249f2335f811270fb6f98d79c0c4f371181ebf06f",
  "docs/content/ja/guide/lib-pull.md":
    "2d84439ab44279ef0fe1ef06c35c01c1e0682bc9c2f6d0ecfcf6aa715940e207",
};

await test("first-copy guides preserve reviewed prose, all original code and every native bookmark", () => {
  assert.equal(baseline.base, "479402e2bd77a1c7b3a4025293b884965d6e2b46");
  for (const document of baseline.documents) {
    const source = read(document.path);
    const route = document.path.replace(/^docs\/content/, "").replace(/\.md$/, "");
    assert.ok(renderedMoreGuideRoutes.includes(route), `${route}: actual native reading campaign`);
    assert.equal([...source.matchAll(section)].length, 1);
    const restored = source.replace(section, "");
    if (document.path === "docs/content/ja/guide/lib-pull.md")
      assert.equal(
        document.sha256,
        "67cd464bdb5a871c97a49ade58d0475bb924ec10457e05efbc553d00877246bd",
      );
    const expectedProse = document.path.startsWith("docs/content/ja/")
      ? reviewedJapaneseProse
      : document.sha256;
    assert.equal(sha(restored), expectedProse, document.path);
    assert.equal(sha(JSON.stringify(fences(restored))), originalFenceHashes[document.path]);
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
