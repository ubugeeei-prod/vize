const assert = require("node:assert/strict");

const pos = (document, offset) => {
  assert.ok(offset >= 0);
  const value = document.positionAt(offset);
  return { line: value.line, character: value.character };
};
const range = (document, offset, length) => ({
  start: pos(document, offset),
  end: pos(document, offset + length),
});
// An ordinary `<template>` is not an Art form (#10593): it is refused, not checked.
const plainArt = (document) =>
  document.languageId === "art-vue" && !document.getText().includes("<art");
function diagnostics(document, enabled, invalid) {
  const source = document.getText();
  if (plainArt(document))
    return [
      {
        range: range(document, source.indexOf("<template>"), "<template>".length),
        severity: 1,
        code: "invalid-art",
        source: "vize/types",
        message:
          "Art files need an `<art>` block with `<variant>` templates; this `<template>` is not type checked.",
      },
    ];
  const result = [];
  // Art's dedicated diagnostic pipeline has no filename-only Router warnings.
  if (!enabled && document.languageId !== "art-vue")
    for (const match of source.matchAll(/route\.params\.(page|tags|at|heading)\b/gu)) {
      result.push({
        range: range(document, match.index + "route.params.".length, match[1].length),
        severity: 2,
        code: "ecosystem/vue-router-route-param",
        source: "vize/ecosystem",
        message: `Route param \`${match[1]}\` is not defined by this page file. Available params: id`,
      });
    }
  if (invalid) {
    const start = source.lastIndexOf("toUpperCase");
    result.push({
      code: 2339,
      message: "Property 'toUpperCase' does not exist on type 'number'.",
      range: range(document, start, "toUpperCase".length),
      severity: 1,
      source: "vize/types",
    });
  }
  return result;
}

function publicDiagnostics(document, enabled, invalid) {
  return diagnostics(document, enabled, invalid).map((value) => ({
    code: value.code,
    message: value.message,
    range: [
      value.range.start.line,
      value.range.start.character,
      value.range.end.line,
      value.range.end.character,
    ],
    relatedInformation: undefined,
    severity: value.severity - 1,
    source: value.source,
    tags: undefined,
  }));
}

module.exports = { diagnostics, plainArt, pos, publicDiagnostics, range };
