import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { verifyRenderedRulePackets } from "../../docs/scripts/rule-render-assertions.ts";

const route = "/ja/rules/accessibility-structure";
const generated = "docs/content/generated/rules/ja/accessibility-structure.md";
const read = (file: string) => readFileSync(join(repoRoot, file), "utf8");
const blocks = (text: string) =>
  [...text.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)^```[^\n]*$/gm)].map((match) => [
    match[1],
    match[2],
  ]);
const headings = [...read(generated).matchAll(/^### `([^`]+)`$/gm)];
const rules = headings.map((match) => match[1]);

await test("Japanese structure packets match their rule references and rule metadata", () => {
  const source = read(generated);
  const index = read("docs/content/ja/rules/accessibility-structure.md");
  assert.ok(rules.length > 0);
  const allRows = read("docs/content/rules/all.md").split("\n");
  for (const [position, heading] of headings.entries()) {
    const name = heading[1];
    const id = name.replaceAll("/", "-");
    const packet = source.slice(heading.index, headings[position + 1]?.index);
    const reference = read(`docs/content/ja/rules/reference/${id}.md`);
    assert.deepEqual(blocks(packet), blocks(reference), name);
    assert.deepEqual(blocks(packet), blocks(read(`docs/content/rules/reference/${id}.md`)), name);
    const implementation = reference.match(/\/blob\/main\/([^#]+)#L\d+/)?.[1];
    assert.ok(implementation, name);
    const metadata = read(implementation).match(
      /static META: RuleMeta = RuleMeta \{([\s\S]*?)\n\};/,
    )?.[1];
    assert.ok(metadata, name);
    assert.ok(metadata.includes(`name: "${name}"`));
    const severity = metadata.match(/default_severity: Severity::(\w+)/)?.[1]?.toLowerCase();
    assert.ok(packet.includes(`既定の重大度: \`${severity}\``), name);
    assert.equal(
      packet.includes("自動修正: 対応している診断で利用できます"),
      metadata.includes("fixable: true"),
      name,
    );
    const row = allRows.find((line) => line.startsWith(`| [\`${name}\`](`));
    assert.ok(row, name);
    const cells = row.split(" | ");
    const column = cells.findIndex((cell) => /^`(?:error|warning)`$/.test(cell));
    assert.equal(packet.match(/^プリセット: (.+)$/m)?.[1].trim(), cells[column + 1], name);
    for (const kind of ["bad", "good"]) {
      assert.equal(packet.split(`<span id="${id}-${kind}"></span>`).length, 2);
      assert.ok(source.includes(`](#${id}-${kind})`));
      assert.ok(index.includes(`](https://vizejs.dev${route}.html#${id}-${kind})`));
    }
  }
  for (const text of [index, source])
    assert.equal(text.split('<span id="accessibility-structure"></span>').length, 2);
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
    rules.map((id) => id.replaceAll("/", "-")).toSorted(),
  );
});
