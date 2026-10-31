import assert from "node:assert/strict";
import { asRecord, parseJson } from "../../json-value.ts";
import { originalBytes } from "./authority.ts";
import type { Context } from "./install.ts";
import { withFormattingSession } from "./session.ts";
import { applyTextEdits, formattingOptions } from "./sort.ts";

const position = (line: number, character: number) => ({ line, character });
const range = (line: number, start: number, end: number) => ({
  start: position(line, start),
  end: position(line, end),
});
const configuration = (preset = "vize", extra = "") =>
  Buffer.from(
    "export default { vue: { formatter: { preset: " + JSON.stringify(preset) + extra + " } } };\n",
  );
export function artOriginal(context: Pick<Context, "original">) {
  const prefix = "tests/_fixtures/differential/lsp/art-range-formatting-6098/";
  return {
    source: originalBytes(context.original, prefix + "App.vue.txt").toString("utf8"),
    config: originalBytes(context.original, prefix + "vite.config.mjs.txt"),
    case: asRecord(
      parseJson(originalBytes(context.original, prefix + "case.json").toString("utf8")),
    ),
    edits: parseJson(
      originalBytes(context.original, prefix + "response.expected.json").toString("utf8"),
    ),
  };
}
export const artProfileBodies = [
  ["vize", '\n  <Button data-a="a" data-z="z" />\n'],
  ["prettier", '\n  <Button data-z="z" data-a="a" />\n'],
  ["compact", '\n  <Button data-z="z" data-a="a" />\n'],
  ["documentation", '\n  <Button\n    data-z="z"\n    data-a="a"\n  />\n'],
] as const;

export async function runArt(context: Context): Promise<void> {
  const original = artOriginal(context);
  assert.equal(original.case.method, "textDocument/rangeFormatting");
  assert.deepEqual(original.case.initializationOptions, {
    editor: true,
    lint: false,
    typecheck: false,
    formatting: true,
  });
  const selection = range(1, 27, 28),
    content = range(1, 26, 58);
  assert.deepEqual(asRecord(original.case.request).params, {
    range: selection,
    options: formattingOptions,
  });
  const prefix = '<template><p>🌸 untouched</p></template>\n<!--🌸--><art title="one">';
  const body = '<Button data-z="z" data-a="a" />';
  const suffix = "</art>\n<script setup>const   value=1</script>\n";
  assert.equal(original.source, prefix + body + suffix);
  for (const [preset, text] of artProfileBodies) {
    const expected = [{ range: content, newText: text }];
    if (preset === "documentation") assert.deepEqual(expected, original.edits);
    await withFormattingSession(
      context,
      "art-profile-" + preset,
      original.source,
      preset === "documentation" ? original.config : configuration(preset),
      async (session) => {
        const params = {
          textDocument: { uri: session.uri },
          range: selection,
          options: formattingOptions,
        };
        const response = await session.request("textDocument/rangeFormatting", params, expected);
        const applied = applyTextEdits(original.source, response.result);
        assert.equal(applied, prefix + text + suffix);
        for (const version of [2, 3]) {
          session.change(applied, version);
          await session.request(
            "textDocument/rangeFormatting",
            { ...params, range: { start: position(2, 2), end: position(2, 3) } },
            [],
          );
        }
      },
    );
  }
  const first = '<art title="first"><Button   a="1" /></art>\n';
  const second = '<art title="second"><Other   b="2" /></art>\n';
  await withFormattingSession(
    context,
    "art-selected-second",
    first + second,
    configuration(),
    async (session) => {
      const expected = [{ range: range(1, 20, 37), newText: '\n  <Other b="2" />\n' }];
      const params = {
        textDocument: { uri: session.uri },
        range: range(1, 21, 22),
        options: formattingOptions,
      };
      const response = await session.request("textDocument/rangeFormatting", params, expected);
      const applied = applyTextEdits(first + second, response.result);
      assert.equal(applied, first + '<art title="second">\n  <Other b="2" />\n</art>\n');
      session.change(applied, 2);
      await session.request(
        "textDocument/rangeFormatting",
        { ...params, range: range(2, 2, 3) },
        [],
      );
    },
  );
  const crlf = "<art>\r\n<Button   />\r\n</art>\r\n";
  await withFormattingSession(
    context,
    "art-crlf",
    crlf,
    configuration("vize", ', endOfLine: "auto"'),
    async (session) => {
      const params = {
        textDocument: { uri: session.uri },
        range: range(1, 1, 2),
        options: formattingOptions,
      };
      const expected = [
        { range: { start: position(0, 5), end: position(2, 0) }, newText: "\r\n  <Button />\r\n" },
      ];
      const response = await session.request("textDocument/rangeFormatting", params, expected);
      const applied = applyTextEdits(crlf, response.result);
      assert.equal(applied, "<art>\r\n  <Button />\r\n</art>\r\n");
      session.change(applied, 2);
      await session.request("textDocument/rangeFormatting", params, []);
    },
  );
  await withFormattingSession(
    context,
    "art-unknown-custom-block",
    "<docs>  exact raw 🌸  </docs>\n<art><Button   /></art>\n",
    configuration(),
    async (session) => {
      await session.request(
        "textDocument/rangeFormatting",
        { textDocument: { uri: session.uri }, range: range(0, 7, 8), options: formattingOptions },
        [],
      );
    },
  );
  const onTypeSource = "<art>\n<Button />\n</art>\n";
  await withFormattingSession(
    context,
    "art-on-type",
    onTypeSource,
    configuration(),
    async (session) => {
      // The private source oracle has no trigger character; newline is this
      // declared public wire carrier, with its original position and edit intact.
      const params = {
        textDocument: { uri: session.uri },
        position: position(1, 2),
        ch: "\n",
        options: formattingOptions,
      };
      const expected = [{ range: range(1, 0, 0), newText: "  " }];
      const response = await session.request("textDocument/onTypeFormatting", params, expected);
      const applied = applyTextEdits(onTypeSource, response.result);
      assert.equal(applied, "<art>\n  <Button />\n</art>\n");
      session.change(applied, 2);
      await session.request("textDocument/onTypeFormatting", params, []);
      session.change('<art>\n<Button   a="1" />\n</art>\n', 3);
      await session.request("textDocument/onTypeFormatting", params, []);
    },
  );
  for (const [label, source, selected] of [
    ["broken-block", "<art><Button /></art>\n<template>", range(0, 6, 7)],
    ["invalid-range", "<art><Button   /></art>\n", { start: position(0, 6), end: position(99, 0) }],
  ] as const) {
    await withFormattingSession(
      context,
      "art-" + label,
      source,
      configuration(),
      async (session) => {
        await session.request(
          "textDocument/rangeFormatting",
          { textDocument: { uri: session.uri }, range: selected, options: formattingOptions },
          null,
        );
      },
    );
  }
  const scriptError = "<script setup>const value = ;</script>\n<art><Button /></art>\n";
  await withFormattingSession(
    context,
    "art-unrelated-script-error",
    scriptError,
    configuration(),
    async (session) => {
      const expected = [{ range: range(1, 5, 15), newText: "\n  <Button />\n" }];
      const response = await session.request(
        "textDocument/rangeFormatting",
        { textDocument: { uri: session.uri }, range: range(1, 6, 7), options: formattingOptions },
        expected,
      );
      assert.equal(
        applyTextEdits(scriptError, response.result),
        "<script setup>const value = ;</script>\n<art>\n  <Button />\n</art>\n",
      );
    },
  );
  context.record.json("art-qualification.json", {
    profiles: artProfileBodies.map(([preset]) => preset),
    wholeOriginalUtf16Edit: true,
    neighboringBytesAndFixedPoints: true,
    finiteOriginalEdgeControls: true,
    onTypeCarrier:
      "declared newline public trigger; private source oracle supplies only position/edit",
  });
}
