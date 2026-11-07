import { expect, it } from "vite-plus/test";
import { codeToThemedTokenLines } from "./codeHighlightingEngine";

it("colors script keywords as independent tokens without changing source bytes", async () => {
  const source = "import { ref } from 'vue'\n  const count: number = ref(0)\n";
  const lines = await codeToThemedTokenLines(source, "typescript");

  expect(lines.map((line) => line.map((token) => token.content).join(""))).toEqual([
    "import { ref } from 'vue'",
    "  const count: number = ref(0)",
  ]);
  expect(lines[0]?.[0]).toEqual({
    content: "import",
    darkColor: "#F2B8A0",
    lightColor: "#7A2F15",
  });
  expect(lines[1]?.[0]).toEqual({ content: "  ", darkColor: undefined, lightColor: undefined });
  expect(lines[1]?.find((token) => token.content === "number")).toEqual({
    content: "number",
    darkColor: "#C4C8FF",
    lightColor: "#3D4B80",
  });
});

it("preserves multiline comments, strings, and HTML while assigning stable colors", async () => {
  const script = "/* start\nstill comment */ const value = `雪😀`";
  const scriptLines = await codeToThemedTokenLines(script, "javascript");
  expect(scriptLines.map((line) => line.map((token) => token.content).join(""))).toEqual(
    script.split("\n"),
  );
  expect(scriptLines[1]?.[0]?.darkColor).toBe("#A39B8F");

  const html = '<section aria-label="雪">value</section>';
  const htmlLines = await codeToThemedTokenLines(html, "html");
  expect(htmlLines[0]?.map((token) => token.content).join("")).toBe(html);
  expect(htmlLines[0]?.find((token) => token.content === "section")?.darkColor).toBe("#8ECDF0");
  expect(htmlLines[0]?.find((token) => token.content === "aria-label")?.darkColor).toBe("#D8B4FE");
});
