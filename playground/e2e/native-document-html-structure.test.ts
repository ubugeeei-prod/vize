import { describe, expect, it } from "vite-plus/test";
import corpus from "../../davinci/vize_l1/tests/fixtures/document/html-structure.tsv?raw";

// The same original bytes and native ancestry census are checked by Rust.
// DOMParser supplies independent, actual browser HTML tree construction.
describe("native Document explicit HTML element ancestry", () => {
  const rows = corpus.split("\n").filter((row) => row.length > 0 && !row.startsWith("#"));
  it("keeps all twelve native law sources", () => {
    expect(rows).toHaveLength(12);
  });
  for (const row of rows) {
    const [name, source, expected] = row.split("\t");
    it(name ?? "invalid fixture", () => {
      expect(source).toBeDefined();
      expect(expected).toBeDefined();
      const document = new DOMParser().parseFromString(source ?? "", "text/html");
      expect(document.compatMode).toBe("CSS1Compat");
      const elements = Array.from(document.querySelectorAll("*"));
      const actual = elements.map((element) => {
        const parent = element.parentElement;
        const position = parent === null ? "-" : String(elements.indexOf(parent));
        expect(element.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
        return `${element.localName}:${position}`;
      });
      expect(actual.join(",")).toBe(expected);
    });
  }
});
