import { describe, expect, it } from "vite-plus/test";
import corpus from "../../davinci/vize_l1/tests/fixtures/document/html-body-nodes.json";

// Rust consumes these same whole original sources and body child-node arrays.
// The last three characterize genuine browser insertion that native body nodes
// explicitly refuse, while keeping the existing element ancestry available.
describe("native Document original HTML body nodes", () => {
  it("retains twelve admitted and three explicitly refused whole sources", () => {
    expect(corpus).toHaveLength(15);
    expect(corpus.filter((case_) => case_.bodyNodeRefusal)).toHaveLength(3);
  });
  for (const { name, source, nodes } of corpus) {
    it(name, () => {
      const document = new DOMParser().parseFromString(source, "text/html");
      expect(document.compatMode).toBe("CSS1Compat");
      const elements = [document.body, ...document.body.querySelectorAll("*")];
      const actual = elements.map((element) => {
        expect(element.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
        return [
          element.localName,
          Array.from(element.childNodes, (node) => {
            expect([1, 3, 8]).toContain(node.nodeType);
            if (node.nodeType === 1) {
              const child = node as Element;
              expect(child.parentElement).toBe(element);
              expect(child.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
              return [node.nodeType, child.localName];
            }
            expect(node.parentNode).toBe(element);
            return [node.nodeType, (node as CharacterData).data];
          }),
        ];
      });
      expect(actual).toEqual(nodes);
    });
  }
});
