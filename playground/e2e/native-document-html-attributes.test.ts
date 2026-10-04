import { describe, expect, it } from "vite-plus/test";
import corpus from "../../davinci/vize_l1/tests/fixtures/document/html-attributes.json";

// These same complete authored sources and expected effective attribute arrays
// execute in Rust. DOMParser independently constructs actual Chromium values.
describe("native Document static HTML attributes", () => {
  it("retains all ten complete original attribute sources", () => {
    expect(corpus).toHaveLength(10);
  });
  for (const { name, source, attributes } of corpus) {
    it(name, () => {
      const document = new DOMParser().parseFromString(source, "text/html");
      expect(document.compatMode).toBe("CSS1Compat");
      const actual = Array.from(document.querySelectorAll("*"), (element) => {
        expect(element.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
        return Array.from(element.attributes, (attribute) => {
          expect(attribute.namespaceURI).toBeNull();
          return [attribute.localName, attribute.value];
        });
      });
      expect(actual).toEqual(attributes);
    });
  }
});
