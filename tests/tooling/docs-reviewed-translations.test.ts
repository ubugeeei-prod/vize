import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { readFileSync } from "node:fs";
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
