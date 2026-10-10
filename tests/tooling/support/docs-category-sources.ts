import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { CATALOGUE_SOURCES } from "../../../docs/scripts/materialize-content.ts";

/** Compare the actual reader documents with complete independent reference authorities. */
export function verifyCompleteCategorySources(root: string) {
  const read = (file: string) => readFileSync(resolve(root, "docs/content", file), "utf8");
  const sources = (text: string) =>
    [...text.matchAll(/```(\w+)[^\n]*\n([\s\S]*?)\n```/g)].map((match) => [match[1], match[2]]);
  const references = new Map(
    ["reference", "project"].flatMap((section) =>
      readdirSync(resolve(root, `docs/content/rules/${section}`)).map((file) => {
        const source = read(`rules/${section}/${file}`);
        const id = source.match(/^# `([^`]+)`/m)?.[1];
        assert(id);
        return [id, source] as const;
      }),
    ),
  );
  assert.equal(references.size, 317);
  const counts: Record<string, number> = {
    all: 317,
    vue: 104,
    "cross-file": 66,
    "type-and-script": 70,
    html: 9,
    accessibility: 31,
    ssr: 2,
    "petite-vue": 3,
    vapor: 7,
    ecosystem: 13,
    "musea-and-css": 16,
  };
  const singleFile = new Set<string>();
  for (const file of CATALOGUE_SOURCES) {
    const [, locale, page] = file.match(/^generated\/rules\/([^/]+)\/([^/]+)\.md$/)!;
    const catalogue = read(file);
    const headings = [...catalogue.matchAll(/^### `([^`]+)`$/gm)];
    if (page in counts)
      assert.equal(headings.length, counts[page], `${file}: current complete coverage`);
    assert.equal(
      new Set(headings.map((match) => match[1])).size,
      headings.length,
      `${file}: unique rule packets`,
    );
    for (let index = 0; index < headings.length; index += 1) {
      const [, id] = headings[index];
      const section = catalogue.slice(headings[index].index, headings[index + 1]?.index);
      const reference = references.get(id);
      assert(reference, `${file}: current source identity ${id}`);
      assert.deepEqual(
        sources(section),
        sources(reference),
        `${file} ${id}: whole copied source including shared files and graphs`,
      );
      const slug = id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
      for (const kind of ["bad", "good"]) {
        assert.equal(
          section.split(`<span id="${slug}-${kind}"></span>`).length,
          2,
          `${file} ${id}: unique ${kind} anchor`,
        );
        assert(catalogue.includes(`](#${slug}-${kind})`), `${file} ${id}: same-page ${kind} link`);
      }
      if (
        locale === "en" &&
        !page.startsWith("vue-") &&
        !page.startsWith("accessibility-") &&
        page !== "cross-file" &&
        page !== "all"
      )
        singleFile.add(id);
    }
    assert.doesNotMatch(catalogue, /\]\(\.\/all\.md#/, `${file}: examples stay on the reader page`);
    assert.match(
      catalogue,
      /```(?:vue|ts|html) annotate="(?:remove|add):/,
      `${file}: native annotation metadata`,
    );
    if (page === "cross-file" || page === "all") {
      assert.match(catalogue, /`no-source-async-fact`/);
      assert.match(catalogue, /`illustrative-source-pair`/);
      assert.match(catalogue, /2\.7/);
    }
    if (page === "cross-file") {
      for (const token of [
        "vize lint --cross-file",
        "vize:croquis/cf/*",
        "croquis/cf/*",
        "lint.vize.rules",
        "vize:",
      ])
        assert(catalogue.includes(`\`${token}\``), `${file}: complete public CLI guidance`);
      const statusRows = [...catalogue.matchAll(/^\| \[`([^`]+)`\].*\| ([^|]+) \|$/gm)];
      assert.equal(statusRows.length, 66, `${file}: every project has an overview support status`);
      const statusCounts = new Map<string, number>();
      for (const row of statusRows) statusCounts.set(row[2], (statusCounts.get(row[2]) ?? 0) + 1);
      assert.equal(statusCounts.get("CLI"), 19, `${file}: actual CLI boundary`);
      assert.deepEqual(
        [...statusCounts.values()].sort((a, b) => a - b),
        [6, 16, 19, 25],
      );
    }
  }
  assert.equal(
    singleFile.size,
    251,
    "all source rules are reachable through their own complete category pages",
  );
}
