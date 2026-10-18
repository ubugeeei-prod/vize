import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { preparePublication } from "../../tools/benchmarks/scripts/publish-snapshot.mjs";

const root = resolve(import.meta.dirname, "../..");
const read = (file: string) => readFileSync(resolve(root, file), "utf8");
const pages = ["", "ja/", "zh-CN/", "pt-BR/", "fr/"].map(
  (prefix) => `docs/content/${prefix}architecture/performance.md`,
);

void test("all five performance pages use canonical Markdown snapshot links from the real publisher", async () => {
  const outputs = await preparePublication(
    JSON.parse(read("tools/benchmarks/results/tool-benchmark-latest.json")),
    read,
  );
  for (const file of pages) {
    const source = outputs.get(file);
    assert.equal(source, read(file), `${file}: regeneration retains the exact corrected page`);
    const links = [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)]
      .map((match) => match[1])
      .filter((target) => target.includes("performance-blacksmith"));
    assert.deepEqual(links, Array(3).fill("./performance-blacksmith.md"), file);
    assert.ok(
      existsSync(resolve(root, dirname(file), links[0])),
      `${file}: its locale snapshot source exists`,
    );
    const originalUrl = new URL(
      "./performance-blacksmith",
      "https://example.test/ja/architecture/performance/index.html",
    );
    assert.equal(
      originalUrl.pathname,
      "/ja/architecture/performance/performance-blacksmith",
      "the original directory-relative link addresses the wrong route",
    );
  }
});
