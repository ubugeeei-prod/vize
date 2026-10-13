import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { currentEditorialWitness } from "./config-editorial-succession.ts";

export interface NonRuleWitness {
  case: string;
  route: string;
  kind: "paragraph" | "list" | "heading" | "image";
  before: string;
  after: string;
  originalAlias: string | null;
}
const fixture = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "../../tests/_fixtures/docs-ja-nonrule-editorial.json"),
    "utf8",
  ),
) as { originalSourceSha: string; cases: NonRuleWitness[] };

export function nonRuleText(source: string) {
  return source
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^#{1,6}\s+|^-\s+/gm, "")
    .replace(/<\/?code>/g, "")
    .replace(/&#(\d+);/g, (_, value: string) => String.fromCodePoint(Number(value)))
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&")
    .replace(/\s+/g, " ")
    .trim();
}

export function witnessBlocks(witness: NonRuleWitness, version: "before" | "after") {
  const source = witness[version];
  if (witness.kind === "image") {
    const image = source.match(/^!\[([^\]]+)\]\(([^)]+)\)$/);
    assert(image, `${witness.case}: complete image source`);
    return [image[1], image[2]];
  }
  const blocks =
    witness.kind === "paragraph"
      ? source.split(/\n\n/)
      : witness.kind === "list"
        ? source.split(/\n(?=- )/)
        : [source];
  return blocks.map(nonRuleText);
}

export function assertNonRuleWitness(actual: string[], witness: NonRuleWitness) {
  assert.deepEqual(
    actual,
    witnessBlocks(witness, "after"),
    `${witness.case}: complete rendered block`,
  );
  assert.notDeepEqual(
    actual,
    witnessBlocks(witness, "before"),
    `${witness.case}: original mistranslation`,
  );
}

/** Read complete original elements without editing text, styles, or source DOM. */
export async function readNonRuleWitnesses(page: Page, route: string, version: "before" | "after") {
  const cases = fixture.cases.filter((entry) => entry.route === route).map(currentEditorialWitness);
  const expected = cases.map((entry) => ({
    witness: entry,
    blocks: witnessBlocks(entry, version),
  }));
  const provenance = cases.some((entry) => entry.case === "N31")
    ? page.locator(".content details.benchmark-provenance")
    : null;
  let opened = false;
  if (provenance) {
    assert.equal(await provenance.count(), 1, `${route}: one original provenance disclosure`);
    if ((await provenance.getAttribute("open")) === null) {
      await provenance.locator(":scope > summary").click();
      opened = true;
      assert.notEqual(
        await provenance.getAttribute("open"),
        null,
        `${route}: native disclosure opens`,
      );
    }
  }
  try {
    const actual = await page.evaluate((expected) => {
      const content = document.querySelector(".content");
      if (!content) throw new Error("Authored Japanese requires original content authority");
      const normalized = (text: string | null) => text?.replace(/\s+/g, " ").trim() ?? "";
      return expected.map(({ witness, blocks }) => {
        if (witness.kind === "image") {
          const images = [...content.querySelectorAll("img")].filter(
            (image) => image.alt === blocks[0],
          );
          if (images.length !== 1 || !images[0].checkVisibility() || images[0].naturalWidth <= 0)
            throw new Error(`${witness.case}: original image not rendered`);
          return [images[0].alt, new URL(images[0].src).pathname];
        }
        if (witness.kind === "list") {
          const candidates = [...content.querySelectorAll("ul")].filter(
            (list) =>
              normalized(list.querySelector(":scope > li")?.textContent ?? null) === blocks[0],
          );
          if (candidates.length !== 1)
            throw new Error(`${witness.case}: authored complete list missing or ambiguous`);
          const items = [...candidates[0].querySelectorAll(":scope > li")];
          if (items.length !== blocks.length || items.some((node) => !node.checkVisibility()))
            throw new Error(`${witness.case}: original whole list is not rendered`);
          return items.map((node) => normalized(node.textContent));
        }
        const selector = witness.kind === "heading" ? "h2,h3" : "p";
        const candidates = [...content.querySelectorAll(selector)].filter(
          (node) => normalized(node.textContent) === blocks[0],
        );
        if (candidates.length !== 1)
          throw new Error(`${witness.case}: complete authored first block is missing or ambiguous`);
        let node: Element | null = candidates[0];
        return blocks.map(() => {
          if (!node || !node.checkVisibility() || node.getBoundingClientRect().height <= 0)
            throw new Error(`${witness.case}: complete block is not rendered`);
          if (witness.kind === "paragraph" && node.tagName !== "P")
            throw new Error(`${witness.case}: authored paragraph flow was interrupted`);
          const text = normalized(node.textContent);
          node = node.nextElementSibling;
          return text;
        });
      });
    }, expected);
    assert.equal(actual.length, cases.length, `${route}: every authored original case`);
    return cases.map((witness, index) => ({ witness, actual: actual[index] }));
  } finally {
    if (opened && provenance) {
      await provenance.locator(":scope > summary").click();
      assert.equal(
        await provenance.getAttribute("open"),
        null,
        `${route}: native disclosure restores`,
      );
    }
  }
}

export async function verifyJapaneseNonRule(page: Page, route: string) {
  if (!fixture.cases.some((entry) => entry.route === route)) return null;
  const blocks = await readNonRuleWitnesses(page, route, "after");
  for (const { witness, actual } of blocks) {
    assertNonRuleWitness(actual, witness);
    if (witness.originalAlias)
      assert.equal(
        await page.locator(`[id="${witness.originalAlias}"]`).count(),
        1,
        `${witness.case}: original native bookmark`,
      );
  }
  return {
    originalSourceSha: fixture.originalSourceSha,
    witnesses: blocks.map(({ witness, actual }) => ({
      case: witness.case,
      kind: witness.kind,
      wholeTextSha256: createHash("sha256").update(JSON.stringify(actual)).digest("hex"),
    })),
  };
}
