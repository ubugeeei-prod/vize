// Explicit current packets for #10572. Historical baseline packets/artifacts
// remain unchanged; these are authored expectations for the example-byte delta.
export const namedMarkdown = [
  "**red**",
  "_CSS value for color_",
  "#ff0000",
  "**Example**",
  "```css\n.example {\n  color: red;\n}\n```",
  "**Property syntax**",
  "```css\n<color>\n```",
  "**Docs**",
  "[MDN CSS named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color)",
  "**Docs**",
  "[MDN Reference](https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/color)",
].join("\n\n");

export const propertyMarkdown = [
  "**display**",
  "_CSS property_",
  "In combination with 'float' and 'position', determines the type of box or boxes that are generated for an element.",
  "**Syntax**",
  "```css\n[ <display-outside> || <display-inside> ] | <display-listitem> | <display-internal> | <display-box> | <display-legacy>\n```",
  "**Example**",
  "```css\n.example {\n  display: block;\n}\n```",
  "**Baseline (catalog snapshot)**",
  "Widely available; interoperable since 2015-07-29; widely available since 2018-01-29",
  "**Docs**",
  "[MDN Reference](https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/display)",
].join("\n\n");

export const currentHoverPackets = {
  property: {
    contents: { kind: "markdown", value: propertyMarkdown },
    range: { start: { line: 6, character: 2 }, end: { line: 6, character: 9 } },
  },
  named: {
    contents: { kind: "markdown", value: namedMarkdown },
    range: { start: { line: 3, character: 9 }, end: { line: 3, character: 12 } },
  },
};

export const currentDocumentColors = [
  { range: [3, 9, 3, 12], color: { red: 1, green: 0, blue: 0, alpha: 1 }, source: "red" },
  {
    range: [4, 20, 4, 27],
    color: {
      red: 0.20000000298023224,
      green: 0.4000000059604645,
      blue: 0.6000000238418579,
      alpha: 1,
    },
    source: "#336699",
  },
  {
    range: [5, 16, 5, 39],
    color: {
      red: 0.20000000298023224,
      green: 0.4000000059604645,
      blue: 0.6000000238418579,
      alpha: 0.5,
    },
    source: "rgba(51, 102, 153, 0.5)",
  },
];

export const currentColorPresentations = [
  { label: "#336699" },
  { label: "rgb(51, 102, 153)" },
  { label: "hsl(210 50% 40%)" },
];
