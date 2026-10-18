import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  collectReviewedTranslations,
  isReviewedTranslation,
} from "../../docs/scripts/i18n/reviewed.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { updatePerformance } from "../../tools/benchmarks/scripts/published-snapshot-sections.mjs";
import { currentEditorialWitness } from "../../docs/scripts/config-editorial-succession.ts";

const root = resolve(import.meta.dirname, "../..");
const path = "architecture/performance.md";
const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
const digest = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const benchmarks = /<!-- benchmark:(\w+):start -->[\s\S]*?<!-- benchmark:\1:end -->/g;
const completeBlocks = (source: string) =>
  [...source.matchAll(benchmarks)].map((match) => match[0]);
const restoreOriginalSnapshotLinks = (source: string) =>
  source.replaceAll("](./performance-blacksmith.md)", "](./performance-blacksmith)");

void test("Japanese performance review survives both translation and benchmark regeneration", async () => {
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "the complete authored article is a generation input");
  assert.equal(
    isReviewedTranslation(
      japanese.replace("<!-- Reviewed translation;", "<!-- Generated translation;"),
    ),
    false,
    "the original machine translation was not protected from regeneration",
  );
  const data = JSON.parse(
    readFileSync(resolve(root, "tools/benchmarks/results/tool-benchmark-latest.json"), "utf8"),
  );
  const regenerated = updatePerformance(japanese, data, "ja");
  assert.equal(
    regenerated.replace(benchmarks, ""),
    japanese.replace(benchmarks, ""),
    "benchmark publication changes only its three marked blocks and preserves the complete authored context",
  );
  assert.equal(completeBlocks(japanese).length, 3);
  const { cases } = JSON.parse(
    readFileSync(resolve(root, "tests/_fixtures/docs-ja-nonrule-editorial.json"), "utf8"),
  ) as { cases: { case: string; after: string }[] };
  const originalBackend = cases.find((entry) => entry.case === "N31");
  assert.ok(originalBackend, "the original complete backend paragraph remains frozen");
  const reviewedBackend = currentEditorialWitness(originalBackend);
  assert.equal(japanese.split(reviewedBackend.after).length, 2);
  const originalProse = japanese.replace(reviewedBackend.after, originalBackend.after);
  assert.equal(
    digest(completeBlocks(originalProse).map(restoreOriginalSnapshotLinks)),
    "22cbc8705fdfe99e3d7901fa24759fa0125b98ee4453e140022d1168ba51a25f",
    "all original complete blocks after reversing the declared snapshot links and whole backend paragraph",
  );
});

void test("the wording review preserves all original examples, APIs, measurements and destinations", () => {
  const fences = [...japanese.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map(
    (match) => match[0],
  );
  assert.equal(fences.length, 2);
  assert.equal(digest(fences), "b7651c851b115ac08898782a27b68321ff6e8a762050c44dd51ed50ba40a7708");
  const contracts = [...japanese.matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((match) => match[1]);
  assert.equal(contracts.length, 89);
  assert.equal(
    digest(contracts),
    "97283c47a54584a4ad5a795179975e1e660148e38a9766a830172c2cec7016a4",
  );
  const links = [...japanese.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.equal(links.filter((link) => link === "./performance-blacksmith.md").length, 3);
  const originalLinks = links.map((link) =>
    link === "./performance-blacksmith.md" ? "./performance-blacksmith" : link,
  );
  assert.equal(links.length, 15);
  assert.equal(
    digest(originalLinks),
    "3d648a3c9c68363e33c469f80dbf321549123a53681bf7ff29b21b5e2f0c6a28",
  );
  const measurements = japanese
    .split("\n")
    .filter((row) => row.startsWith("|") && /\d/.test(row))
    .map((row) => row.match(/\d+(?:\.\d+)?/g));
  assert.equal(measurements.length, 36);
  assert.equal(
    digest(measurements),
    "bb85002a41712326c9b47cb8cc2fbffd3c3c9f39f70aea1a0223d3e5231afbc8",
  );
  for (const bookmark of [
    "ネイティブ-マルチスレッド",
    "パフォーマンスのためのアーキテクチャの選択",
    "アリーナの割り当て",
    "シングルパス-トークナイザー",
    "実測-アリーナと式の作業",
    "タイプ認識型-lint-プロファイル",
    "ベンチマーク-フォーマッタ-グリフと-prettier",
    "タイプチェッカープロファイル",
    "診断を多用する-e2e-フィクスチャ",
  ])
    assert.equal(
      japanese.split(`<span id="${bookmark}"></span>`).length,
      2,
      `${bookmark}: retain exactly one original bookmark`,
    );
  assert.equal(
    defaultNavigationRenderRoutes.filter((route) => route === "/ja/architecture/performance")
      .length,
    1,
    "the complete article receives the existing desktop/mobile render checks",
  );
});
