import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { verifyRenderedRulePackets } from "../../docs/scripts/rule-render-assertions.ts";

const route = "/ja/rules/accessibility-structure";
const expected = [
  "a11y/heading-has-content",
  "a11y/heading-levels",
  "a11y/iframe-has-title",
  "a11y/img-alt",
  "a11y/interactive-supports-focus",
  "a11y/label-has-for",
  "a11y/landmark-roles",
  "a11y/media-has-caption",
].toSorted();
const read = (file: string) => readFileSync(join(repoRoot, "docs/content", file), "utf8");
const fence = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
const blocks = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);
const prose = (text: string) =>
  text
    .replace(/^---\n[\s\S]*?\n---\n/, "")
    .replace(fence, "")
    .split(/\n\s*\n/)
    .map((block) => block.trim())
    .filter(
      (block) =>
        block &&
        !/^#{1,6} /.test(block) &&
        !/^<span id="[^"]+"><\/span>$/.test(block) &&
        !["**設定（Vite+）**", "**悪い**", "**良い**"].includes(block) &&
        !/^\[(?:悪い例|悪い)\]\(#[^)]+\) · \[(?:良い例|良い)\]\(#[^)]+\)$/.test(block),
    )
    .map((block) =>
      block
        .replace(/`([^`]+)`/g, "$1")
        .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
        .replace(/\s+/g, " ")
        .trim(),
    );
const urls = (text: string) =>
  [...text.replace(fence, "").matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);

await test("Japanese structure preserves eight complete original references and localized next steps", () => {
  const source = read("generated/rules/ja/accessibility-structure.md");
  const index = read("ja/rules/accessibility-structure.md");
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(headings.map((match) => match[1]).toSorted(), expected);
  assert.equal(blocks(source).length, 32);
  for (const [position, heading] of headings.entries()) {
    const id = heading[1].replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`ja/rules/reference/${id}.md`);
    assert.equal(blocks(reference).length, 4);
    assert.deepEqual(blocks(packet), blocks(reference), `${heading[1]}: whole copied source`);
    assert.deepEqual(
      blocks(packet),
      blocks(read(`rules/reference/${id}.md`)),
      `${heading[1]}: original English code parity`,
    );
    assert.deepEqual(prose(packet), prose(reference), `${heading[1]}: every authored paragraph`);
    for (const line of reference.split("\n")) {
      if (/^(既定の重大度|プリセット|自動修正|適用範囲|オプション):/.test(line))
        assert.ok(packet.includes(line), `${heading[1]}: complete original metadata`);
    }
    assert.match(packet, /既定の重大度: `warning`/);
    assert.match(packet, /自動修正: なし。修正内容を確認してください/);
    if (heading[1] === "a11y/img-alt") assert.match(packet, /プリセット: _none_/);
    assert.deepEqual(
      urls(packet),
      urls(reference).map((target) =>
        target === "#悪い"
          ? `#${id}-bad`
          : target === "#良い"
            ? `#${id}-good`
            : target === "../all.md"
              ? "all.md"
              : target,
      ),
      `${heading[1]}: every original reader link`,
    );
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  assert.match(
    index,
    /<span id="accessibility-structure"><\/span>\n\n# ページ構造のアクセシビリティ/,
  );
  assert.equal(source.split('<span id="accessibility-structure"></span>').length, 2);
  assert.equal([...source.matchAll(/^# ページ構造のアクセシビリティ$/gm)].length, 1);
  for (const target of ["./reference/a11y-heading-levels.md", "./accessibility.md"]) {
    assert.ok(index.includes(`](${target})`));
    assert.match(read(`ja/rules/${target.slice(2)}`), /^# .+/m);
  }
  assert.match(index, /`vp run lint`/);
});

await test("Japanese structure enrolls both native viewports and refuses source-only DOM success", async () => {
  assert.ok(defaultNavigationRenderRoutes.includes(route));
  assert.deepEqual(
    createRenderJobs([route]).map((job) => job.device),
    ["desktop", "mobile"],
  );
  const nativeBoundary = new Error("Actual native DOM execution is required");
  let dispatched: { id: string }[] | undefined;
  const page = {
    evaluate: async (_read: unknown, packet: { id: string }[]) => {
      dispatched = packet;
      throw nativeBoundary;
    },
  } as unknown as Parameters<typeof verifyRenderedRulePackets>[0];
  await assert.rejects(verifyRenderedRulePackets(page, route), (error) => error === nativeBoundary);
  assert.deepEqual(
    dispatched?.map(({ id }) => id).toSorted(),
    expected.map((id) => id.replaceAll("/", "-")),
  );
});
