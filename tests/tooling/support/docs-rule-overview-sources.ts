import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

/** Independent reference inventory and actual owning-page targets define the overview contract. */
export function verifyRuleOverviewSource(root: string, locale: string) {
  const read = (path: string) => readFileSync(resolve(root, "docs/content", path), "utf8");
  const prefix = locale === "en" ? "" : `${locale}/`;
  const authority = locale === "ja" ? "ja/" : "";
  const references = new Map(
    ["reference", "project"].flatMap((directory) =>
      readdirSync(resolve(root, `docs/content/${authority}rules/${directory}`)).map((file) => {
        const source = read(`${authority}rules/${directory}/${file}`);
        const id = source.match(/^# `([^`]+)`$/m)?.[1];
        assert(id);
        return [id, { directory, source, file }] as const;
      }),
    ),
  );
  assert.equal(references.size, 332);
  const source = read(`generated/rules/${locale}/all.md`);
  const headings = [...source.matchAll(/^### `([^`]+)`$/gm)];
  assert.deepEqual(
    headings.map((match) => match[1]).sort(),
    [...references.keys()].sort(),
    `${locale}: all 332 independent source identities`,
  );
  assert.doesNotMatch(source, /^```/m, `${locale}: All Rules is an overview without full packets`);
  const anchors = [...source.matchAll(/<span id="([^"\n]+)"><\/span>/g)].map((match) => match[1]);
  assert.equal(new Set(anchors).size, anchors.length, `${locale}: unique compatibility aliases`);
  for (let index = 0; index < headings.length; index += 1) {
    const [, id] = headings[index];
    const section = source.slice(headings[index].index, headings[index + 1]?.index);
    const reference = references.get(id)!;
    const slug = id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    const summary = section.match(/^### `[^`]+`\n\n([^\n]+)/)?.[1];
    assert(summary, `${locale} ${id}: retained purpose`);
    if (locale === "en" || locale === "ja")
      assert.equal(summary, reference.source.match(/^# `[^`]+`\n\n([^\n]+)/m)?.[1]);
    for (const kind of ["bad", "good"]) {
      assert.equal(
        section.split(`<span id="${slug}-${kind}"></span>`).length,
        2,
        `${locale} ${id}: retained unique ${kind} alias`,
      );
      const links = [...section.matchAll(/\]\(\.\/([^/#]+)\.md#([^)]*)\)/g)].filter(
        (match) => match[2] === `${slug}-${kind}`,
      );
      assert.equal(links.length, 1, `${locale} ${id}: one owning-group ${kind} destination`);
      assert.notEqual(links[0][1], "all", "examples belong to the group page");
      const target = read(`generated/rules/${locale}/${links[0][1]}.md`);
      assert(target.includes(`### \`${id}\``), `${locale} ${id}: real owning group heading`);
      assert(target.includes(`<span id="${slug}-${kind}"></span>`));
    }
    const detailPrefix = locale === "en" || locale === "ja" ? "./" : "/rules/";
    assert(
      section.includes(`](${detailPrefix}${reference.directory}/${reference.file})`),
      `${locale} ${id}: complete reference remains reachable`,
    );
  }
  const targets = new Set([
    ...anchors,
    ...headings.map((match) => match[1].replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase()),
  ]);
  for (const [, anchor] of source.matchAll(/\]\(#([^)]*)\)/g))
    assert(targets.has(anchor), `${prefix}: missing existing local target ${anchor}`);
}
