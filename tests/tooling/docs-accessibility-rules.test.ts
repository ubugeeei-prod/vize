import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { verifyRenderedRulePackets } from "../../docs/scripts/rule-render-assertions.ts";

const accessibilityRules = [
  "a11y/alt-text",
  "a11y/anchor-has-content",
  "a11y/anchor-is-valid",
  "a11y/aria-props",
  "a11y/aria-role",
  "a11y/aria-unsupported-elements",
  "a11y/click-events-have-key-events",
  "a11y/form-control-has-label",
  "a11y/heading-has-content",
  "a11y/heading-levels",
  "a11y/iframe-has-title",
  "a11y/img-alt",
  "a11y/interactive-supports-focus",
  "a11y/label-has-for",
  "a11y/landmark-roles",
  "a11y/media-has-caption",
  "a11y/mouse-events-have-key-events",
  "a11y/no-access-key",
  "a11y/no-aria-hidden-on-focusable",
  "a11y/no-autofocus",
  "a11y/no-distracting-elements",
  "a11y/no-i-for-icon",
  "a11y/no-redundant-roles",
  "a11y/no-refer-to-non-existent-id",
  "a11y/no-role-presentation-on-focusable",
  "a11y/no-static-element-interactions",
  "a11y/placeholder-label-option",
  "a11y/role-has-required-aria-props",
  "a11y/tabindex-no-positive",
  "a11y/use-list",
  "vue/use-unique-element-ids",
];

const vueSources = (text: string) =>
  [...text.matchAll(/```vue[^\n]*\n([\s\S]*?)\n```/g)].map((match) => match[1]);

for (const locale of ["en", "ja", "fr", "pt-BR", "zh-CN"]) {
  await test(`${locale} accessibility navigation targets complete same-page examples`, () => {
    const overview = fs.readFileSync(
      path.join(repoRoot, `docs/content/generated/rules/${locale}/accessibility.md`),
      "utf8",
    );
    const links = [...overview.matchAll(/^\| \[`([^`]+)`\]\(#([^)]*)\)/gm)];
    const headings = [...overview.matchAll(/^### `([^`]+)`$/gm)];
    const expected = [...accessibilityRules].sort((a, b) => a.localeCompare(b));
    assert.deepEqual(
      links.map((row) => row[1]).sort((a, b) => a.localeCompare(b)),
      expected,
    );
    assert.deepEqual(
      headings.map((row) => row[1]).sort((a, b) => a.localeCompare(b)),
      expected,
    );
    assert.doesNotMatch(overview, /\]\(\.\/all\.md#/);
    for (const [_, ruleId, slug] of links) {
      const index = headings.findIndex((heading) => heading[1] === ruleId);
      assert.notEqual(index, -1, `${locale} ${ruleId}: local packet`);
      const packet = overview.slice(headings[index].index, headings[index + 1]?.index);
      const reference = fs.readFileSync(
        path.join(
          repoRoot,
          `docs/content/${locale === "ja" ? "ja/" : ""}rules/reference/${slug}.md`,
        ),
        "utf8",
      );
      assert.ok(reference.includes(`# \`${ruleId}\``));
      for (const kind of ["bad", "good"]) {
        assert.ok(
          overview.includes(`](#${slug}-${kind})`),
          `${locale} ${ruleId}: local ${kind} link`,
        );
        assert.equal(
          packet.split(`<span id="${slug}-${kind}"></span>`).length,
          2,
          `${locale} ${ruleId}: unique ${kind} target`,
        );
      }
      const sources = vueSources(packet);
      assert.ok(sources.length >= 2, `${locale} ${ruleId}: complete Bad/Good Vue witnesses`);
      assert.deepEqual(
        sources,
        vueSources(reference),
        `${locale} ${ruleId}: exact whole copied source`,
      );
      assert.ok(packet.includes(`"${ruleId}":`), `${locale} ${ruleId}: inline rule configuration`);
      const labels =
        locale === "ja"
          ? ["既定の重大度:", "プリセット:", "オプション:", "**悪い**", "**良い**"]
          : locale === "en"
            ? ["Default severity:", "Presets:", "Options:", "**Bad**", "**Good**"]
            : [];
      for (const label of labels)
        assert.ok(packet.includes(label), `${locale} ${ruleId}: ${label}`);
    }
  });
}

await test("Japanese core retains complete references, localized next steps and native packet enrollment", async () => {
  const route = "/ja/rules/accessibility-core";
  const read = (file: string) => fs.readFileSync(path.join(repoRoot, "docs/content", file), "utf8");
  const source = read("generated/rules/ja/accessibility-core.md");
  const index = read("ja/rules/accessibility-core.md");
  const expected = accessibilityRules.slice(0, 8).toSorted();
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(headings.map((match) => match[1]).toSorted(), expected);
  const blocks = (text: string) =>
    [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
      match[1],
      match[2],
    ]);
  assert.equal(blocks(source).length, 32);
  for (const [position, heading] of headings.entries()) {
    const id = heading[1].replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    assert.deepEqual(blocks(packet), blocks(read(`ja/rules/reference/${id}.md`)), heading[1]);
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
    }
  }
  assert.match(index, /<span id="accessibility-core"><\/span>\n\n# アクセシビリティの基本/);
  assert.equal(source.split('<span id="accessibility-core"></span>').length, 2);
  for (const target of ["./reference/a11y-form-control-has-label.md", "./accessibility.md"]) {
    assert.ok(index.includes(`](${target})`));
    assert.match(read(`ja/rules/${target.slice(2)}`), /^# .+/m);
  }
  assert.match(index, /`vp run lint`/);
  assert.deepEqual(
    defaultNavigationRenderRoutes.filter((item) => item.startsWith("/ja/rules/accessibility-")),
    [
      route,
      "/ja/rules/accessibility-integrity",
      "/ja/rules/accessibility-interactions",
      "/ja/rules/accessibility-structure",
    ],
  );
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

await test("Japanese integrity retains all seven original packets and enrolls its actual native route", async () => {
  const route = "/ja/rules/accessibility-integrity";
  const read = (file: string) => fs.readFileSync(path.join(repoRoot, "docs/content", file), "utf8");
  const source = read("generated/rules/ja/accessibility-integrity.md");
  const index = read("ja/rules/accessibility-integrity.md");
  const expected = [
    "a11y/no-role-presentation-on-focusable",
    "a11y/no-static-element-interactions",
    "a11y/placeholder-label-option",
    "a11y/role-has-required-aria-props",
    "a11y/tabindex-no-positive",
    "a11y/use-list",
    "vue/use-unique-element-ids",
  ].toSorted();
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(headings.map((match) => match[1]).toSorted(), expected);
  const blocks = (text: string) =>
    [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
      match[1],
      match[2],
    ]);
  assert.equal(blocks(source).length, 28);
  for (const [position, heading] of headings.entries()) {
    const id = heading[1].replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`ja/rules/reference/${id}.md`);
    assert.deepEqual(blocks(packet), blocks(reference), heading[1]);
    for (const line of reference.split("\n")) {
      if (/^(既定の重大度|プリセット|自動修正|適用範囲|オプション):/.test(line))
        assert.ok(packet.includes(line), `${heading[1]}: complete original metadata`);
    }
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
    }
  }
  assert.match(
    index,
    /<span id="accessibility-integrity"><\/span>\n\n# アクセシビリティ情報の整合性/,
  );
  assert.equal(source.split('<span id="accessibility-integrity"></span>').length, 2);
  for (const target of ["./reference/a11y-role-has-required-aria-props.md", "./accessibility.md"]) {
    assert.ok(index.includes(`](${target})`));
    assert.match(read(`ja/rules/${target.slice(2)}`), /^# .+/m);
  }
  assert.match(index, /`vp run lint`/);
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

await test("Japanese interactions retain complete source packets and require native execution", async () => {
  const route = "/ja/rules/accessibility-interactions";
  const read = (file: string) => fs.readFileSync(path.join(repoRoot, "docs/content", file), "utf8");
  const source = read("generated/rules/ja/accessibility-interactions.md");
  const index = read("ja/rules/accessibility-interactions.md");
  const expected = [
    "a11y/mouse-events-have-key-events",
    "a11y/no-access-key",
    "a11y/no-aria-hidden-on-focusable",
    "a11y/no-autofocus",
    "a11y/no-distracting-elements",
    "a11y/no-i-for-icon",
    "a11y/no-redundant-roles",
    "a11y/no-refer-to-non-existent-id",
  ].toSorted();
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(headings.map((match) => match[1]).toSorted(), expected);
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
  assert.equal(blocks(source).length, 32);
  for (const [position, heading] of headings.entries()) {
    const id = heading[1].replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`ja/rules/reference/${id}.md`);
    assert.deepEqual(blocks(packet), blocks(reference), heading[1]);
    assert.deepEqual(prose(packet), prose(reference), `${heading[1]}: every authored paragraph`);
    for (const line of reference.split("\n")) {
      if (/^(既定の重大度|プリセット|自動修正|適用範囲|オプション):/.test(line))
        assert.ok(packet.includes(line), `${heading[1]}: complete original metadata`);
    }
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
    );
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
    }
  }
  assert.match(
    index,
    /<span id="accessibility-interactions"><\/span>\n\n# 操作とフォーカスのアクセシビリティ/,
  );
  assert.equal(source.split('<span id="accessibility-interactions"></span>').length, 2);
  for (const target of ["./reference/a11y-no-aria-hidden-on-focusable.md", "./accessibility.md"]) {
    assert.ok(index.includes(`](${target})`));
    assert.match(read(`ja/rules/${target.slice(2)}`), /^# .+/m);
  }
  assert.match(index, /支援技術から隠す要素/);
  assert.match(index, /`vp run lint`/);
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
