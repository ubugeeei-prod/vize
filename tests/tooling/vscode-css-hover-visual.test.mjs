import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertProviderPicker,
  assertRenderedCss,
  fixtureSource,
  hoverCases,
  workspaceSettings,
} from "../../editors/vscode/test/css-hover-visual-contract.mjs";
import {
  currentHoverPackets,
  currentDocumentColors,
  currentColorPresentations,
} from "../../editors/vscode/test/css-hover-current-packets.mjs";

await test("registered renderer differential input pins both complete current hovers", () => {
  const fixture = new URL(
    "../_fixtures/differential/lsp/css-hover-stylesheet-examples-10572/",
    import.meta.url,
  );
  assert.equal(fs.readFileSync(new URL("Colors.vue.txt", fixture), "utf8"), fixtureSource);
  assert.deepEqual(
    JSON.parse(fs.readFileSync(new URL("responses.expected.json", fixture), "utf8")),
    [
      { position: { line: 6, character: 5 }, result: currentHoverPackets.property },
      { position: { line: 3, character: 10 }, result: currentHoverPackets.named },
    ],
  );
});

await test("current whole packets author only valid stylesheet example bytes and keep color coordinates", () => {
  assert.ok(
    currentHoverPackets.named.contents.value.includes("```css\n.example {\n  color: red;\n}\n```"),
  );
  assert.ok(
    currentHoverPackets.named.contents.value.includes(
      "**Property syntax**\n\n```css\n<color>\n```",
    ),
  );
  assert.ok(
    currentHoverPackets.property.contents.value.includes(
      "```css\n.example {\n  display: block;\n}\n```",
    ),
  );
  assert.deepEqual(
    currentDocumentColors.map((entry) => entry.range),
    [
      [3, 9, 3, 12],
      [4, 20, 4, 27],
      [5, 16, 5, 39],
    ],
  );
  assert.deepEqual(
    currentColorPresentations.map((entry) => entry.label),
    ["#336699", "rgb(51, 102, 153)", "hsl(210 50% 40%)"],
  );
});

await test("CSS capture disables built-in default color fallback and keeps authored literal controls", () => {
  const settings = workspaceSettings("/actual/source-built/vize");
  assert.equal(settings["editor.defaultColorDecorators"], "never");
  assert.equal(settings["editor.colorDecorators"], true);
  for (const fixture of hoverCases) {
    assert.equal(fixtureSource.split(fixture.needle).length, 2, "request anchors must be unique");
  }
  assert.deepEqual(
    hoverCases.filter((fixture) => fixture.baselineNull).map((fixture) => fixture.id),
    ["hex-baseline", "rgba-baseline"],
  );
});

await test("render acceptance rejects fallback greys and requires property/value/punctuation in one CSS fence", () => {
  const snapshot = {
    hoverText: "background-color",
    links: [{ href: "https://developer.mozilla.org/docs/Web/CSS/background-color" }],
    tokens: [{ text: "color", color: "rgb(255, 255, 255)" }],
    codeBlocks: [
      {
        text: "<color>",
        tokens: [
          { text: "<color", color: "rgb(204, 204, 204)" },
          { text: ">", color: "rgb(212, 212, 212)" },
        ],
      },
    ],
  };
  assert.throws(() => assertRenderedCss(snapshot), /property\/value\/punctuation/);
  assertRenderedCss({
    ...snapshot,
    links: [
      {
        href: "vscode-file://workbench",
        attributeHref: "",
        dataHref: "https://developer.mozilla.org/docs/Web/CSS/background-color",
      },
    ],
    tokens: [...snapshot.tokens, { text: "red", color: "rgb(206, 145, 120)" }],
    codeBlocks: [
      {
        text: ".sample { color: red; }",
        tokens: [
          { text: "color", color: "rgb(156, 220, 254)" },
          { text: "red", color: "rgb(206, 145, 120)" },
          { text: ": ", color: "rgb(212, 212, 212)" },
        ],
      },
    ],
  });
});

await test("picker acceptance rejects standalone/default picker and wrong actual RGBA", () => {
  const snapshot = {
    decorations: [{ backgroundColor: "rgb(51, 102, 153)" }],
    pickerOriginalColor: "rgb(51, 102, 153)",
    pickerPickedColor: "rgb(51, 102, 153)",
    pickerPresentation: "#336699",
    standalonePicker: false,
  };
  assertProviderPicker(snapshot);
  assert.throws(
    () => assertProviderPicker({ ...snapshot, standalonePicker: true }),
    /provider-backed/,
  );
  assert.throws(() =>
    assertProviderPicker({ ...snapshot, pickerOriginalColor: "rgba(51, 102, 153, 0.5)" }),
  );
});

await test("visual mechanism observes real Electron DOM and never uses standalone picker or HTML mocks", () => {
  const capture = fs.readFileSync(
    new URL("../../editors/vscode/test/css-hover-visual-capture.mjs", import.meta.url),
    "utf8",
  );
  const suite = fs.readFileSync(
    new URL("../../editors/vscode/test/suite/css-hover-visual.cjs", import.meta.url),
    "utf8",
  );
  assert.match(capture, /connectOverCDP/);
  assert.match(capture, /DOMSnapshot.captureSnapshot/);
  assert.match(capture, /offline: true/);
  assert.doesNotMatch(capture, /setContent|insertAdjacentHTML|innerHTML\s*=/);
  assert.doesNotMatch(
    suite,
    /showOrFocusStandaloneColorPicker|registerHoverProvider|registerColorProvider/,
  );
  assert.match(suite, /vscode.executeDocumentColorProvider/);
});
