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
