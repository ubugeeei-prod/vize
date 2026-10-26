import assert from "node:assert/strict";
import { currentHoverPackets } from "./css-hover-current-packets.mjs";
import { assertRenderedCss } from "./css-hover-visual-contract.mjs";

// Authored current expectations for #10556. Historical null observations stay
// byte-exact in historical-2934; this live contract accepts only the new packet.
export const literalMarkdown = [
  "**CSS color**",
  "_Concrete authored color_",
  "**Example**",
  "```css\n.example {\n  color: #336699;\n  color: rgb(51, 102, 153);\n  color: hsl(210 50% 40%);\n}\n```",
  "**Docs**",
  "[MDN CSS colors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors)",
].join("\n\n");
const rgbaMarkdown = [
  "**CSS color**",
  "_Concrete authored color_",
  "**Example**",
  "```css\n.example {\n  color: #33669980;\n  color: rgba(51, 102, 153, 0.5);\n  color: hsl(210 50% 40% / 0.5);\n}\n```",
  "**Docs**",
  "[MDN CSS colors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors)",
].join("\n\n");
export const literalCurrentHoverPackets = {
  ...currentHoverPackets,
  literal: {
    contents: { kind: "markdown", value: literalMarkdown },
    range: { start: { line: 4, character: 20 }, end: { line: 4, character: 27 } },
  },
  "rgba-current": {
    contents: { kind: "markdown", value: rgbaMarkdown },
    range: { start: { line: 5, character: 16 }, end: { line: 5, character: 39 } },
  },
  "literal-before": null,
  "literal-end": null,
};
export const literalHoverCases = [
  { id: "property", needle: "display", offset: 3, markdown: "CSS property", render: true },
  { id: "named", needle: "red;", offset: 1, markdown: "CSS value for color", render: true },
  {
    id: "literal",
    needle: "#336699",
    offset: 3,
    markdown: "Concrete authored color",
    render: true,
  },
  {
    id: "rgba-current",
    needle: "rgba(51, 102, 153, 0.5)",
    offset: 12,
    markdown: "Concrete authored color",
  },
  { id: "literal-before", needle: "#336699", offset: -1 },
  { id: "literal-end", needle: "#336699", offset: 7 },
];
export const literalVisualPhases = [
  { id: "property", caption: "display" },
  { id: "named", caption: "CSS value for color" },
  { id: "literal", caption: "Concrete authored color" },
  { id: "picker" },
];
// Pinned VS Code tokenized-source DOM represents line breaks with elements;
// textContent retains the complete token text without those line separators.
export const literalExampleDomText =
  ".example {  color: #336699;  color: rgb(51, 102, 153);  color: hsl(210 50% 40%);}";
export function assertLiteralRenderedCss(snapshot) {
  assert.ok(
    snapshot.hoverText.includes("Concrete authored color"),
    "literal provider Markdown must be visible",
  );
  assertRenderedCss(snapshot);
  assert.ok(
    snapshot.links.some(
      (link) =>
        (link.dataHref ?? link.href) ===
        "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors",
    ),
  );
  assert.ok(
    snapshot.codeBlocks.some((block) => block.text === literalExampleDomText),
    "one real CSS fence must retain all three normalized declarations",
  );
}
