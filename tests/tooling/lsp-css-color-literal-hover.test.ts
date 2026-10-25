import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

function markdown(forms: readonly string[]) {
  return (
    "**CSS color**\n\n_Concrete authored color_\n\n**Example**\n\n```css\n.example {\n" +
    forms.map((form) => `  color: ${form};\n`).join("") +
    "}\n```\n\n**Docs**\n\n[MDN CSS colors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors)"
  );
}
const blue = markdown(["#336699", "rgb(51, 102, 153)", "hsl(210 50% 40%)"]);
const alphaBlue = markdown(["#33669980", "rgba(51, 102, 153, 0.5)", "hsl(210 50% 40% / 0.5)"]);
const red = markdown(["#ff0000", "rgb(255, 0, 0)", "hsl(0 100% 50%)"]);

function position(source: string, offset: number) {
  const prefix = source.slice(0, offset).split("\n");
  return { line: prefix.length - 1, character: prefix.at(-1)!.length };
}
function expected(source: string, literal: string, value = blue) {
  const start = source.indexOf(literal);
  assert.notEqual(start, -1);
  return {
    contents: { kind: "markdown", value },
    range: { start: position(source, start), end: position(source, start + literal.length) },
  };
}

async function withSession(
  label: string,
  enabled: boolean,
  run: (session: LspSession, uri: string, observations: unknown[]) => Promise<void>,
) {
  const output = path.join(testOutputRoot, "css-color-literal-hover");
  fs.mkdirSync(output, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(output, `${label}-`));
  fs.writeFileSync(
    path.join(workspace, "vite.config.mjs"),
    `export default {vue:{languageServer:{hover:${enabled},lint:false,typecheck:false}}};\n`,
  );
  const session = new LspSession({
    repoRoot: root,
    binary: process.env.VIZE_LSP_BIN ?? path.join(root, "target/ci/vize"),
  });
  const observations: unknown[] = [];
  try {
    observations.push({
      initialize: await session.initialize(workspace, {
        editor: true,
        hover: enabled,
        typecheck: false,
        lint: false,
      }),
    });
    await run(session, pathToFileURL(path.join(workspace, "日本🧭.vue")).href, observations);
  } finally {
    await session.shutdown();
    await session.waitForClosed();
    fs.writeFileSync(
      path.join(output, `${label}.json`),
      JSON.stringify(observations, null, 2) + "\n",
    );
    fs.writeFileSync(path.join(output, `${label}.stderr.txt`), session.stderrText);
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}
async function open(session: LspSession, uri: string, source: string, version = 1) {
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version, text: source },
  });
  assert.deepEqual(
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    ),
    { uri, version, diagnostics: [] },
  );
}
async function hover(
  session: LspSession,
  uri: string,
  source: string,
  offset: number,
  observations: unknown[],
  wanted: unknown,
) {
  const params = { textDocument: { uri }, position: position(source, offset) };
  const started = performance.now();
  const result = await session.request("textDocument/hover", params);
  const elapsedMs = performance.now() - started;
  observations.push({ source, params, result, elapsedMs });
  assert.deepEqual(result, wanted, JSON.stringify({ source, params }));
}

await test("whole concrete color hovers preserve all source profiles, UTF-16 spans and strict endpoints", async () => {
  await withSession("profiles", true, async (session, uri, observations) => {
    for (const ending of ["\n", "\r\n"])
      for (const lang of ["", "css", "scss", "less", "postcss", "sass", "stylus", "styl"]) {
        const indented = ["sass", "stylus", "styl"].includes(lang);
        const attribute = lang ? ` lang="${lang}"` : "";
        const tokens = ["#336699", "rgb(51 102 153 / 50%)", "hsl(210 50% 40%)"];
        const body = indented
          ? `.日本🧭${ending}${tokens.map((token) => `  color: ${token}`).join(ending)}${ending}`
          : `/*日本🧭*/ .x {${tokens.map((token) => ` color: ${token};`).join("")} }`;
        const source = `<template><div /></template>${ending}<style${attribute}>${body}</style>`;
        await open(session, uri, source);
        for (const token of tokens) {
          const start = source.indexOf(token),
            end = start + token.length;
          const wanted = expected(source, token, token.startsWith("rgb") ? alphaBlue : blue);
          for (const offset of [start, start + 2, end - 1])
            await hover(session, uri, source, offset, observations, wanted);
          for (const offset of [start - 1, end])
            await hover(session, uri, source, offset, observations, null);
        }
        session.notify("textDocument/didClose", { textDocument: { uri } });
      }
  });
});

await test("selectors, opaque values, malformed functions and unsupported profiles retain whole refusals", async () => {
  const cases = [
    ["css", "#336699 { color:red; }", "#336699"],
    ["css", "rgb(51 102 153) { color:red; }", "rgb"],
    ["css", ".outer { color:#336699:hover { color:blue; } }", "#336699"],
    ["css", ".x { content:'#336699'; }", "#336699"],
    ["css", ".x { /*#336699*/ color:red; }", "#336699"],
    ["css", ".x { background:url(#336699); }", "#336699"],
    ["css", ".x { color:rgb(1 invalid 3); }", "rgb"],
    ["css", ".x { color:rgb(51 /* ```css **text** */ 102 153); }", "rgb"],
    ["css", ".x { color:rgb(1 var(--value) 3); }", "--value"],
    ["css", ".x { color:#336699z; }", "#336699"],
    ["scss", ".x { color:#{'#336699'}; }", "#336699"],
    ["less", ".x { color:@{'#336699'}; }", "#336699"],
    ["unknown", ".x { color:#336699; }", "#336699"],
  ];
  await withSession("refusals", true, async (session, uri, observations) => {
    for (const ending of ["\n", "\r\n"])
      for (const [lang, body, token] of cases) {
        const source = `<template><div /></template>${ending}<style lang="${lang}">/*日本🧭*/ ${body}</style>`;
        await open(session, uri, source);
        await hover(session, uri, source, source.indexOf(token) + 1, observations, null);
        session.notify("textDocument/didClose", { textDocument: { uri } });
      }
  });
});

await test("edited, closed, same-version reopened and disabled hovers never reuse an old literal", async () => {
  const original = "<style>/*日本🧭*/ .x { color:#336699; }</style>";
  const changed = "<style>/*日本🧭*/ .x { color:#ff0000; }</style>";
  await withSession("lifecycle", true, async (session, uri, observations) => {
    await open(session, uri, original);
    await hover(
      session,
      uri,
      original,
      original.indexOf("#336699"),
      observations,
      expected(original, "#336699"),
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri, version: 2 },
      contentChanges: [{ text: changed }],
    });
    assert.deepEqual(
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
      ),
      { uri, version: 2, diagnostics: [] },
    );
    await hover(
      session,
      uri,
      changed,
      changed.indexOf("#ff0000"),
      observations,
      expected(changed, "#ff0000", red),
    );
    session.notify("textDocument/didClose", { textDocument: { uri } });
    await hover(session, uri, original, original.indexOf("#336699"), observations, null);
    await open(session, uri, original);
    await hover(
      session,
      uri,
      original,
      original.indexOf("#336699"),
      observations,
      expected(original, "#336699"),
    );
  });
  await withSession("disabled", false, async (session, uri, observations) => {
    await open(session, uri, original);
    await hover(session, uri, original, original.indexOf("#336699"), observations, null);
  });
});
