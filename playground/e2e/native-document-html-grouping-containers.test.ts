import { describe, expect, it } from "vite-plus/test";
import corpus from "../../davinci/vize_l1/tests/fixtures/document/html-grouping-body-containers.json";

describe("native Document grouping HTML body containers", () => {
  it("retains eight complete sources and whole ancestry, attribute and node packets", () => {
    expect(corpus).toHaveLength(8);
  });
  for (const { name, source, elements: expectedElements, nodes: expectedNodes } of corpus) {
    it(name, () => {
      const document = new DOMParser().parseFromString(source, "text/html");
      expect(document.compatMode).toBe("CSS1Compat");
      const elements = [...document.querySelectorAll("*")];
      expect(
        elements.map((element) => {
          expect(element.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
          const parent = element.parentElement;
          return [
            element.localName,
            parent ? elements.indexOf(parent) : null,
            [...element.attributes].map(({ name, value }) => [name, value]),
          ];
        }),
      ).toEqual(expectedElements);
      expect(
        elements
          .filter(({ localName }) => localName !== "html" && localName !== "head")
          .map((element) => [
            element.localName,
            [...element.childNodes].map((node) => {
              expect(node.parentNode).toBe(element);
              expect([1, 3, 8]).toContain(node.nodeType);
              return [
                node.nodeType,
                node.nodeType === 1 ? (node as Element).localName : (node as CharacterData).data,
              ];
            }),
          ]),
      ).toEqual(expectedNodes);
    });
  }
});
