import assert from "node:assert/strict";
import { test } from "node:test";
import {
  fixtureSource,
  hoverCases as historicalCases,
} from "../../editors/vscode/test/css-hover-visual-contract.mjs";
import {
  assertLiteralRenderedCss,
  literalCurrentHoverPackets,
  literalExampleDomText,
  literalHoverCases,
  literalVisualPhases,
} from "../../editors/vscode/test/css-hover-literal-current-packets.mjs";

await test("live literal packets are complete positives at authored ranges; only strict endpoints are null", () => {
  assert.deepEqual(literalCurrentHoverPackets.literal.range, {
    start: { line: 4, character: 20 },
    end: { line: 4, character: 27 },
  });
  assert.deepEqual(literalCurrentHoverPackets["rgba-current"].range, {
    start: { line: 5, character: 16 },
    end: { line: 5, character: 39 },
  });
  assert.equal(fixtureSource.split("\n")[4].slice(20, 27), "#336699");
  assert.deepEqual(
    literalHoverCases
      .filter((row) => literalCurrentHoverPackets[row.id] === null)
      .map((row) => [row.id, row.offset]),
    [
      ["literal-before", -1],
      ["literal-end", 7],
    ],
  );
  assert.deepEqual(
    historicalCases.filter((row) => row.baselineNull).map((row) => row.id),
    ["hex-baseline", "rgba-baseline"],
  );
  assert.deepEqual(
    literalVisualPhases.map((row) => row.id),
    ["property", "named", "literal", "picker"],
  );
});

const tokens = [
  { text: "color", color: "rgb(156, 220, 254)" },
  { text: ": ", color: "rgb(204, 204, 204)" },
  { text: "#336699", color: "rgb(206, 145, 120)" },
  { text: ";", color: "rgb(204, 204, 204)" },
];
const rendered = {
  hoverText: "CSS color Concrete authored color Example",
  links: [
    {
      href: "vscode-file://workbench",
      dataHref: "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors",
    },
  ],
  tokens,
  codeBlocks: [{ text: literalExampleDomText, tokens }],
};
await test("picker-only, stale named hover, incomplete normalized examples and grey fallback cannot prove literal hover", () => {
  assertLiteralRenderedCss(rendered);
  assert.throws(
    () =>
      assertLiteralRenderedCss({ ...rendered, hoverText: "#336699", tokens: [], codeBlocks: [] }),
    /literal provider Markdown/,
  );
  assert.throws(
    () => assertLiteralRenderedCss({ ...rendered, hoverText: "CSS value for color" }),
    /literal provider Markdown/,
  );
  assert.throws(
    () =>
      assertLiteralRenderedCss({
        ...rendered,
        codeBlocks: [{ text: ".example { color: #336699; }", tokens }],
      }),
    /all three normalized/,
  );
  const grey = tokens.map((row) => ({ ...row, color: "rgb(204, 204, 204)" }));
  assert.throws(
    () =>
      assertLiteralRenderedCss({
        ...rendered,
        tokens: grey,
        codeBlocks: [{ text: literalExampleDomText, tokens: grey }],
      }),
    /property\/value\/punctuation/,
  );
});
