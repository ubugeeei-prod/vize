/// <reference lib="dom" />

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

type NamespacePacket = "html" | "svg" | "mathml";
type AttributePacket = readonly [name: string, value: string];
type ElementPacket = readonly [
  name: string,
  namespace: NamespacePacket,
  parent: number | null,
  attributes: readonly AttributePacket[],
];
type ChildPacket = readonly [type: number, value: string];
type NodePacket = readonly [name: string, children: readonly ChildPacket[]];
interface FixtureCase {
  readonly name: string;
  readonly source: string;
  readonly elements: readonly ElementPacket[];
  readonly nodes: readonly NodePacket[];
}

const root = new URL("../../", import.meta.url);
const fixture = new URL("davinci/vize_l1/tests/fixtures/document/html-phrasing.json", root);
const cases = JSON.parse(readFileSync(fixture, "utf8")) as readonly FixtureCase[];
const fromTests = createRequire(new URL("tests/package.json", root));
const { chromium } = fromTests("@playwright/test") as typeof import("@playwright/test");

function namespace(uri: string | null): NamespacePacket {
  if (uri === "http://www.w3.org/2000/svg") return "svg";
  if (uri === "http://www.w3.org/1998/Math/MathML") return "mathml";
  assert.equal(uri, "http://www.w3.org/1999/xhtml");
  return "html";
}

void test("the complete phrasing packet matches Chromium", async () => {
  assert.equal(cases.length, 8);
  const browser = await chromium.launch();
  const page = await browser.newPage();
  try {
    for (const fixtureCase of cases) {
      const actual = await page.evaluate((source) => {
        const document = new DOMParser().parseFromString(source, "text/html");
        const all = [...document.querySelectorAll("*")];
        return {
          elements: all.map((element) => [
            element.localName,
            element.namespaceURI,
            element.parentElement ? all.indexOf(element.parentElement) : null,
            [...element.attributes].map(({ name, value }) => [name, value]),
          ]),
          nodes: all
            .filter((element) => !["html", "head"].includes(element.localName))
            .map((element) => [
              element.localName,
              [...element.childNodes].map((node) => [
                node.nodeType,
                node.nodeType === Node.ELEMENT_NODE ? (node as Element).localName : node.nodeValue,
              ]),
            ]),
        };
      }, fixtureCase.source);
      const actualElements = actual.elements.map(([name, uri, parent, attributes]) => [
        name,
        namespace(uri as string),
        parent,
        attributes,
      ]);
      assert.deepEqual(actualElements, fixtureCase.elements, fixtureCase.name);
      assert.deepEqual(actual.nodes, fixtureCase.nodes, fixtureCase.name);
    }
  } finally {
    await page.close();
    await browser.close();
  }
});
