import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

const root = new URL("../../", import.meta.url);
const fixture = new URL("davinci/vize_l1/tests/fixtures/document/html-special-text.json", root);
const cases = JSON.parse(readFileSync(fixture, "utf8"));
const fromTests = createRequire(new URL("tests/package.json", root));
const { chromium } = fromTests("@playwright/test");

void test("the complete special-text packet matches Chromium", async () => {
  assert.equal(cases.length, 8);
  const browser = await chromium.launch();
  const page = await browser.newPage();
  try {
    for (const fixtureCase of cases) {
      const actual = await page.evaluate((source) => {
        const document = new DOMParser().parseFromString(source, "text/html");
        const all = [...document.querySelectorAll("*")];
        const elements = all.map((element) => [
          element.localName,
          element.parentElement ? all.indexOf(element.parentElement) : null,
          [...element.attributes].map(({ name, value }) => [name, value]),
        ]);
        const nodes = all
          .filter((element) => !["html", "head"].includes(element.localName))
          .map((element) => [
            element.localName,
            [...element.childNodes].map((node) => [
              node.nodeType,
              node.nodeType === Node.ELEMENT_NODE ? node.localName : node.nodeValue,
            ]),
          ]);
        return { elements, nodes };
      }, fixtureCase.source);
      assert.deepEqual(actual.elements, fixtureCase.elements, fixtureCase.name);
      assert.deepEqual(actual.nodes, fixtureCase.nodes, fixtureCase.name);
    }
  } finally {
    await page.close();
    await browser.close();
  }
});
