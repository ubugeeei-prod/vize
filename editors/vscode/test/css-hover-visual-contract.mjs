import assert from "node:assert/strict";

export const vscodeVersion = "1.107.1";
export const baselineRevision = "2934b35b18393c188b3da5f8e50595299589fb8c";
export const fixtureSource = `<template><div class="sample">Color custody</div></template>
<style scoped>
.sample {
  color: red;
  background-color: #336699;
  border-color: rgba(51, 102, 153, 0.5);
  display: block;
}
</style>
`;

export const hoverCases = [
  { id: "property", needle: "display", offset: 3, markdown: "CSS property" },
  { id: "named", needle: "red;", offset: 1, markdown: "CSS value for color" },
  { id: "hex-baseline", needle: "#336699", offset: 3, baselineNull: true },
  { id: "rgba-baseline", needle: "rgba(51, 102, 153, 0.5)", offset: 12, baselineNull: true },
];

export function workspaceSettings(serverPath) {
  return {
    "vize.serverPath": serverPath,
    "vize.enable": true,
    "vize.typecheck.enable": false,
    "vize.lint.enable": false,
    "vize.trace.server": "verbose",
    "editor.colorDecorators": true,
    // Forbid the built-in default color provider, including its fallback.
    "editor.defaultColorDecorators": "never",
    "editor.colorDecoratorsActivatedOn": "hover",
    "editor.hover.delay": 100,
    "editor.hover.sticky": true,
    "editor.fontSize": 16,
    "editor.minimap.enabled": false,
    "workbench.colorTheme": "Default Dark Modern",
    "window.zoomLevel": 0,
    "window.restoreWindows": "none",
    "workbench.startupEditor": "none",
    "security.workspace.trust.enabled": false,
    "telemetry.telemetryLevel": "off",
    "update.mode": "none",
    "extensions.autoUpdate": false,
    "extensions.autoCheckUpdates": false,
  };
}

export function assertRenderedCss(snapshot) {
  assert.ok(snapshot.hoverText.length > 0, "real editor hover must be visible");
  assert.ok(
    snapshot.links.some((link) => (link.dataHref ?? link.href).includes("developer.mozilla.org/")),
  );
  assert.ok(
    snapshot.tokens.some((token) => token.text.trim()),
    "CSS fence must be tokenized",
  );
  const meaningful = snapshot.codeBlocks.some((block) => {
    const declaration = block.text.match(/([\w-]+)\s*:\s*([\w#-]+)\s*;/);
    if (!declaration) return false;
    const property = block.tokens.find((token) => token.text.trim() === declaration[1]);
    const value = block.tokens.find((token) => token.text.trim() === declaration[2]);
    const punctuation = block.tokens.find((token) => /[:;]/.test(token.text));
    return (
      property &&
      value &&
      punctuation &&
      new Set([property.color, value.color, punctuation.color]).size === 3
    );
  });
  assert.ok(
    meaningful,
    `CSS fence must distinguish property/value/punctuation tokens: ${JSON.stringify(snapshot.codeBlocks)}`,
  );
}

export function assertProviderPicker(snapshot) {
  assert.ok(snapshot.decorations.some((item) => item.backgroundColor === "rgb(51, 102, 153)"));
  assert.equal(snapshot.pickerOriginalColor, "rgb(51, 102, 153)");
  assert.equal(snapshot.pickerPickedColor, "rgb(51, 102, 153)");
  assert.equal(snapshot.pickerPresentation.toLowerCase(), "#336699");
  assert.equal(snapshot.standalonePicker, false, "must use the provider-backed hover picker");
}
