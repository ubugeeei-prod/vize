import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  collectReviewedTranslations,
  isReviewedTranslation,
} from "../../docs/scripts/i18n/reviewed.ts";

void test("translation regeneration retains complete authored and reviewed inputs only", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "vize-reviewed-translations-"));
  const authored = "---\ntitle: 設定\n---\n\n人が書いた本文。\n```ts\nconst x = 1;\n```\n";
  const reviewed = "<!-- Reviewed translation; source: guide/reviewed.md -->\n\n自然な日本語。\n";
  const machine = "<!-- Generated translation; source: guide/machine.md -->\n\n古い訳。\n";
  try {
    await mkdir(resolve(directory, "guide"));
    await Promise.all([
      writeFile(resolve(directory, "guide/authored.md"), authored),
      writeFile(resolve(directory, "guide/reviewed.md"), reviewed),
      writeFile(resolve(directory, "guide/machine.md"), machine),
      writeFile(resolve(directory, "guide/removed.md"), authored),
    ]);
    const retained = await collectReviewedTranslations(
      ["guide/authored.md", "guide/reviewed.md", "guide/machine.md", "guide/new.md"],
      directory,
    );
    assert.deepEqual(
      [...retained],
      [
        ["guide/authored.md", authored],
        ["guide/reviewed.md", reviewed],
      ],
    );
    await rm(directory, { recursive: true });
    assert.equal(
      retained.get("guide/reviewed.md"),
      reviewed,
      "directory replacement retains review",
    );
    assert.equal(isReviewedTranslation(machine), false);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

void test("Japanese stability retains every current public package, crate and API contract", () => {
  const root = resolve(import.meta.dirname, "../..");
  const english = readFileSync(resolve(root, "docs/content/stability.md"), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja/stability.md"), "utf8");
  const packageNames = (source: string) =>
    [...source.matchAll(/`(@vizejs\/[^`]+|oxlint-plugin-vize|vize)`/g)]
      .map((match) => match[1])
      .toSorted();
  assert.deepEqual(packageNames(japanese), packageNames(english));
  const rows = (source: string) =>
    source
      .split("<!-- rust-crate-support:start -->")[1]
      .split("<!-- rust-crate-support:end -->")[0]
      .split("\n")
      .filter((line) => line.startsWith("| `vize_"))
      .map((line) =>
        line
          .split("|")
          .slice(1, -1)
          .map((cell) => cell.trim()),
      );
  const original = rows(english);
  const reviewed = rows(japanese);
  assert.deepEqual(
    reviewed.map((row) => [row[0], row[3]]),
    original.map((row) => [row[0], row[3]]),
    "every published crate and public entrypoint remains associated",
  );
  const tiers: Record<string, string> = {
    "Alpha-supported": "アルファ版対応",
    "Compatibility preview": "互換性プレビュー",
    Experimental: "実験的",
    Incubating: "開発初期",
  };
  for (const [index, row] of original.entries()) {
    assert.equal(reviewed[index][1], tiers[row[1]], row[0]);
    if (row[4].startsWith("One minor"))
      assert.equal(reviewed[index][4], "`#[deprecated]` を付けて 1 minor release の間保持", row[0]);
    else assert.ok(reviewed[index][4].startsWith("最低期間の保証なし"), row[0]);
  }
  assert.doesNotMatch(japanese, /未成年|木箱|木枠|貨物|ノード フロア/);
});

void test("Japanese contribution and credits preserve complete recipes and attribution on regeneration", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const paths = ["contributing.md", "credits.md"];
  const retained = await collectReviewedTranslations(paths, resolve(root, "docs/content/ja"));
  assert.equal(retained.size, paths.length);
  for (const path of paths) {
    const english = readFileSync(resolve(root, "docs/content", path), "utf8");
    const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
    assert.equal(retained.get(path), japanese, `${path}: retain the entire reviewed input`);
    const fences = (source: string) =>
      [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map((match) => match[0]);
    assert.deepEqual(fences(japanese), fences(english), `${path}: complete runnable recipes`);
    const links = (source: string) =>
      [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
    assert.deepEqual(
      links(japanese),
      links(english),
      `${path}: ordered attribution and support links`,
    );
  }
  const contribution = retained.get("contributing.md")!;
  assert.equal(contribution.split('<span id="言語プロセッサの規律変更"></span>').length, 2);
  assert.match(contribution, /Conventional Commits/);
  assert.match(contribution, /`Co-authored-by`/);
  assert.match(contribution, /`ID\+LOGIN@users\.noreply\.github\.com`/);
});

void test("Japanese Content Mapper review survives regeneration with every whole original recipe", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "guide/content-mapper.md";
  const english = readFileSync(resolve(root, "docs/content", path), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains the complete authored review");
  const fences = (source: string) =>
    [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map((match) => match[0]);
  assert.deepEqual(fences(japanese), fences(english), "all five full runnable examples remain");
  const digest = createHash("sha256")
    .update(JSON.stringify(fences(japanese)))
    .digest("hex");
  assert.equal(digest, "8940b963ebe24b35e139a63098e85a348de702114d24b7d243b4497cf01ffcea");
  const links = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.deepEqual(
    links(japanese),
    links(english),
    "all current setup and historical destinations agree",
  );
});

void test("Japanese rule options retain the complete authored review and original runnable contracts", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "rules/options.md";
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "regeneration keeps the entire authored input");
  assert.equal(
    isReviewedTranslation(
      japanese.replace("<!-- Reviewed translation;", "<!-- Generated translation;"),
    ),
    false,
    "the previous generated input has no authored regeneration authority",
  );
  const fencePattern = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
  const fences = [...japanese.matchAll(fencePattern)].map((match) => match[0]);
  const currentInlineTokens = [
    ...japanese.replace(fencePattern, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g),
  ].map((match) => match[1]);
  assert.equal(currentInlineTokens.length, 135);
  const inlineTokens = [...currentInlineTokens];
  assert.deepEqual(
    inlineTokens.splice(19, 3),
    ["vue/valid-v-on", "{ modifiers?: string[] }", "[]"],
    "only the approved custom modifier row succeeds the frozen 132-token contract",
  );
  const digest = (value: string[]) =>
    createHash("sha256").update(JSON.stringify(value)).digest("hex");
  const propsCorrections = new Map<number, string>([
    [36, "ac7cbb7acba123a3444f59de631d61af27236fc70cf257ba9ff79064e463b87c"],
    [39, "b7088e7786b595f49df377ff71e83e7248b6ffb22d0b72225e7f64f04c1c57bc"],
  ]);
  const originalFences = fences.map((recipe, index) => {
    const corrected = propsCorrections.get(index);
    if (!corrected) return recipe;
    assert.equal(
      createHash("sha256").update(recipe).digest("hex"),
      corrected,
      "only the exact reviewed destructure correction succeeds the original recipe",
    );
    return recipe.replace(
      '"script/define-props-destructuring": { destructure:',
      '"script/define-props-destructuring": { mode:',
    );
  });
  assert.equal(fences.length, 43);
  assert.equal(
    digest(originalFences),
    "7af601c217a94df10b23bd1e10860b4329c2356b90deb9910cb113300490bf43",
  );
  assert.equal(inlineTokens[26], '{ destructure?: "only-when-assigned" \\| "always" \\| "never" }');
  const originalTokens = [...inlineTokens];
  originalTokens[26] = '{ mode?: "only-when-assigned" \\| "always" \\| "never" }';
  assert.equal(inlineTokens.length, 132);
  assert.equal(
    digest(originalTokens),
    "90b695added36c831b6af36e61fc2b31a0f7cd58f914ba16974bb3d8a8de30b1",
  );
  assert.deepEqual(
    [...japanese.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]),
    ["./options-type.md", "../guide/lint-execution.md", "./reference/vue-valid-v-on.md"],
    "retain the original type-aware destination and the authored rule-selection next step",
  );
  assert.ok(readFileSync(resolve(root, "docs/content/ja/guide/lint-execution.md"), "utf8"));
  assert.equal(japanese.split('<span id="scoped-entries"></span>').length, 2);
});

void test("Japanese script options survive regeneration with the complete current recipes and APIs", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "rules/options-script.md";
  const english = readFileSync(resolve(root, "docs/content", path), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains the entire authored article");
  assert.equal(
    isReviewedTranslation(
      japanese.replace("<!-- Reviewed translation;", "<!-- Generated translation;"),
    ),
    false,
    "the previous generated body has no authored authority",
  );
  const pattern = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
  const recipes = (source: string) => [...source.matchAll(pattern)].map((item) => item[0]);
  assert.equal(recipes(japanese).length, 13);
  assert.deepEqual(recipes(japanese), recipes(english), "all thirteen current whole recipes agree");
  const tokens = (source: string) =>
    [...source.replace(pattern, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((item) => item[1]);
  assert.equal(tokens(japanese).length, 16);
  assert.deepEqual(
    tokens(japanese),
    tokens(english),
    "every ordered inline API and default remains",
  );
  const links = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((item) => item[1]);
  assert.deepEqual(
    links(japanese),
    links(english),
    "original option reference destination remains",
  );
  assert.equal(japanese.split('<span id="script-rule-options"></span>').length, 2);
});

void test("Japanese init survives regeneration with the original setup and recovery contracts", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "guide/init.md";
  const english = readFileSync(resolve(root, "docs/content", path), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "retain the complete authored Japanese article");
  const fences = (source: string) =>
    [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map((match) => match[0]);
  const digest = (value: unknown) =>
    createHash("sha256").update(JSON.stringify(value)).digest("hex");
  assert.deepEqual(
    fences(japanese),
    fences(english),
    "all nine whole original recipes and outputs",
  );
  assert.equal(fences(japanese).length, 9);
  assert.equal(
    digest(fences(japanese)),
    "b8cac95696ac0a6f2ef2291370af86a70d3b1cc3211a976b8c823cc2ef18e182",
  );
  const inline = (source: string) =>
    [
      ...source
        .replace(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm, "")
        .matchAll(/(?<!`)`([^`\n]+)`(?!`)/g),
    ].map((match) => match[1]);
  const tokens = inline(japanese);
  assert.deepEqual(tokens, inline(english), "all authored CLI options and inline contracts agree");
  assert.equal(tokens.length, 120);
  for (const [index, addition] of [
    [95, "vpx vize init --yes --dry-run"],
    [67, "--vite"],
    [2, "package.json"],
  ] as const) {
    assert.equal(tokens[index], addition, "only the declared contextual additions");
    tokens.splice(index, 1);
  }
  assert.equal(
    digest(tokens),
    "9b9d215f3cf78c2adc62534c2caf8b15118717308d3471a889e5b79ad5eb5314",
    "all 117 original inline occurrences remain in order",
  );
  const links = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.deepEqual(links(japanese), links(english), "all original recovery and next-step URLs");
  assert.deepEqual(links(japanese), [
    "./configuration.md",
    "./oxlint.md",
    "#when-init-refuses-to-edit",
    "../integrations/vscode.md",
    "./static-analysis.md",
    "./oxlint.md",
    "./configuration.md",
    "./cli.md",
  ]);
  for (const bookmark of [
    "project-setup-with-vize-init",
    "what-it-detects",
    "which-lint-file-it-writes",
    "non-interactive-use",
    "running-it-twice",
    "when-init-refuses-to-edit",
    "editor-support",
    "relationship-to-vize-setup",
    "next-steps",
  ]) {
    assert.equal(
      japanese.split(`<span id="${bookmark}"></span>`).length,
      2,
      `${bookmark}: preserve the English bookmark`,
    );
  }
});

void test("Japanese Vue options retain authored contracts on regeneration", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "rules/options-vue.md";
  const english = readFileSync(resolve(root, "docs/content", path), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese);
  const pattern = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
  const contracts = (source: string) => [
    [...source.matchAll(pattern)].map((match) => match[0]),
    [...source.replace(pattern, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((match) => match[1]),
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]),
  ];
  const prose = english.replace(pattern, "").replaceAll("`", "");
  const bookmarks = [...prose.matchAll(/^#{1,2} (.+)$/gm)].map((match) =>
    match[1].replaceAll(/[ /]/g, "-").toLowerCase(),
  );
  assert.deepEqual(contracts(japanese), contracts(english));
  const original = [contracts(english), bookmarks];
  assert.equal(
    createHash("sha256").update(JSON.stringify(original)).digest("hex"),
    "4967998d7e0fc247c3959fec10853ca9798263226e94c40b9a41d2eee42b1660",
  );
  assert.ok(readFileSync(resolve(root, "docs/content/ja/rules/options.md"), "utf8"));
  for (const bookmark of bookmarks) {
    assert.equal(japanese.split(`<span id="${bookmark}"></span>`).length, 2);
    assert.match(japanese, new RegExp(`<span id="${bookmark}"></span>\\n\\n#{1,2} .+`));
  }
  assert.match(english, /`never` forbids self-closing tags/);
  assert.match(japanese, /`never` は自己終了タグを禁止します/);
});
