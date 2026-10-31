import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
  type JsonRecord,
} from "../../json-value.ts";
import { originalBytes } from "./authority.ts";
import { cli, config, resetAuthoredCase, type Context } from "./install.ts";
import { withFormattingSession, type FormattingSession } from "./session.ts";

export function applyTextEdits(source: string, edits: unknown): string {
  const offset = (position: unknown) => {
    const point = asRecord(position),
      line = asNumber(point.line),
      character = asNumber(point.character);
    assert(
      Number.isSafeInteger(line) && line >= 0 && Number.isSafeInteger(character) && character >= 0,
    );
    const lines = source.split("\n"),
      text = lines[line];
    assert(
      text !== undefined && character <= (text.endsWith("\r") ? text.length - 1 : text.length),
    );
    const result =
      lines.slice(0, line).reduce((sum, value) => sum + value.length + 1, 0) + character;
    const before = source.charCodeAt(result - 1),
      after = source.charCodeAt(result);
    assert(
      !(before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff),
      "UTF-16 range splits a surrogate pair",
    );
    return result;
  };
  const changes = asArray(edits)
    .map((value) => {
      const row = asRecord(value),
        range = asRecord(row.range);
      const start = offset(range.start),
        end = offset(range.end);
      assert(start <= end);
      return { start, end, text: asString(row.newText) };
    })
    .sort((left, right) => right.start - left.start);
  let result = source,
    previous = source.length;
  for (const change of changes) {
    assert(change.end <= previous, "overlapping edits");
    result = result.slice(0, change.start) + change.text + result.slice(change.end);
    previous = change.start;
  }
  return result;
}
export const formattingOptions = { tabSize: 2, insertSpaces: true } as const;
const whole = (uri: string) => ({ textDocument: { uri }, options: formattingOptions });
// The auxiliary Rust handler controls use FormattingOptions::default().
export const defaultFormattingOptions = { tabSize: 0, insertSpaces: false } as const;
const auxiliary = (uri: string) => ({ textDocument: { uri }, options: defaultFormattingOptions });
const selection = { start: { line: 1, character: 0 }, end: { line: 3, character: 0 } };
const onType = (uri: string, line = 2, character = 1) => ({
  textDocument: { uri },
  position: { line, character },
  ch: "}",
  options: defaultFormattingOptions,
});
async function emptyResponses(session: FormattingSession) {
  await session.request("textDocument/formatting", auxiliary(session.uri), []);
  await session.request(
    "textDocument/rangeFormatting",
    { ...auxiliary(session.uri), range: selection },
    [],
  );
  await session.request("textDocument/onTypeFormatting", onType(session.uri), []);
}
export function sortOriginal(context: Pick<Context, "original">) {
  const prefix = "tests/_fixtures/differential/lsp/editor-import-sorting-6098/";
  return {
    source: originalBytes(context.original, prefix + "App.vue.txt").toString("utf8"),
    config: originalBytes(context.original, prefix + "vite.config.mjs.txt"),
    case: asRecord(
      parseJson(originalBytes(context.original, prefix + "case.json").toString("utf8")),
    ),
    edits: parseJson(
      originalBytes(context.original, prefix + "response.expected.json").toString("utf8"),
    ),
    sorted: originalBytes(
      context.original,
      "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue",
    ).toString("utf8"),
  };
}

export async function runSort(context: Context): Promise<void> {
  const original = sortOriginal(context);
  assert.equal(original.case.method, "textDocument/formatting");
  assert.deepEqual(original.case.initializationOptions, {
    editor: true,
    lint: false,
    typecheck: false,
    formatting: true,
  });
  assert.deepEqual(original.case.options, formattingOptions);
  assert.equal(
    original.config.toString("utf8"),
    "export default { vue: { formatter: { sortImports: {} } } };\n",
  );
  assert.equal(applyTextEdits(original.source, original.edits), original.sorted);
  runSortCli(context, original.source, original.sorted);
  await withFormattingSession(
    context,
    "sort-document",
    original.source,
    original.config,
    async (session) => {
      const result = await session.request(
        "textDocument/formatting",
        whole(session.uri),
        original.edits,
      );
      assert.equal(applyTextEdits(original.source, result.result), original.sorted);
      for (const version of [2, 3]) {
        session.change(original.sorted, version);
        await emptyResponses(session);
      }
    },
  );
  const rangeEdits = [
    {
      range: { start: { line: 0, character: 24 }, end: { line: 9, character: 0 } },
      newText: original.sorted.slice(24, original.sorted.indexOf("</script>")),
    },
  ];
  await withFormattingSession(
    context,
    "sort-range",
    original.source,
    original.config,
    async (session) => {
      const result = await session.request(
        "textDocument/rangeFormatting",
        { ...auxiliary(session.uri), range: selection },
        rangeEdits,
      );
      assert.equal(applyTextEdits(original.source, result.result), original.sorted);
    },
  );
  for (const [label, configuration] of [
    ["default", Buffer.from("export default {};\n")],
    ["disabled", Buffer.from("export default { vue: { formatter: { sortImports: false } } };\n")],
  ] as const) {
    await withFormattingSession(
      context,
      "sort-" + label,
      original.source,
      configuration,
      emptyResponses,
    );
  }
  for (const [label, source, expected, line] of [
    [
      "sorted-indent",
      original.sorted.replace("import { computed }", "  import { computed }"),
      original.sorted,
      1,
    ],
    [
      "unsorted-indent",
      original.source.replace("const route =", "  const route ="),
      original.source,
      7,
    ],
  ] as const) {
    const configuration =
      label === "unsorted-indent"
        ? Buffer.from(
            'export default { vue: { formatter: {"sortImports":{"groups":["external",{"newlinesBetween":true},"sibling"]}} } };',
          )
        : original.config;
    await withFormattingSession(
      context,
      "sort-" + label,
      source,
      configuration,
      async (session) => {
        const edits = [
          { range: { start: { line, character: 0 }, end: { line, character: 2 } }, newText: "" },
        ];
        const result = await session.request(
          "textDocument/onTypeFormatting",
          onType(session.uri, line, 3),
          edits,
        );
        assert.equal(applyTextEdits(source, result.result), expected);
        session.change(expected, 2);
        await session.request("textDocument/onTypeFormatting", onType(session.uri, line, 3), []);
      },
    );
  }
  const invalid = Buffer.from(
    'export default { vue: { formatter: { sortImports: { groups: ["external", {"newlinesBetween":true}] } } } };\n',
  );
  await withFormattingSession(
    context,
    "sort-invalid",
    original.source,
    invalid,
    async (session) => {
      await session.request("textDocument/formatting", auxiliary(session.uri), null);
      await session.request(
        "textDocument/rangeFormatting",
        { ...auxiliary(session.uri), range: selection },
        null,
      );
      await session.request("textDocument/onTypeFormatting", onType(session.uri), null);
    },
  );
  await withFormattingSession(
    context,
    "sort-repaired",
    original.source,
    Buffer.from("export default { vue: { formatter: { sortImports: false } } };\n"),
    emptyResponses,
  );
  context.record.json("sort-qualification.json", {
    wholeOriginalOutput: true,
    documentRangeOnType: true,
    invalidAndRepaired: true,
    warmWatchedConfigRecovery: "not claimed; separate original policy sessions",
  });
}

function runSortCli(context: Context, source: string, expected: string) {
  const file = join(context.root, "UserCard.vue"),
    configPath = join(context.root, "vite.config.mjs");
  writeFileSync(file, source);
  const bytes = config(context.root, { sortImports: {} });
  const tracked = ["UserCard.vue", "vite.config.mjs"];
  cli(context, "sort-cli-no-config", ["--no-config", "--check", "UserCard.vue"], 0, tracked);
  cli(context, "sort-cli-dirty", ["--check", "UserCard.vue"], 1, tracked);
  assert.equal(readFileSync(file, "utf8"), source);
  cli(context, "sort-cli-write", ["--write", "UserCard.vue"], 0, tracked);
  assert.equal(readFileSync(file, "utf8"), expected);
  cli(context, "sort-cli-fixed-point", ["--check", "UserCard.vue"], 0, tracked);
  assert.deepEqual(readFileSync(configPath), bytes);
  writeFileSync(file, source);
  config(context.root, { sortImports: false });
  cli(context, "sort-cli-disabled", ["--write", "UserCard.vue"], 0, tracked);
  assert.equal(readFileSync(file, "utf8"), source);
  const script = (text: string) => text.slice(text.indexOf("\n") + 1, text.indexOf("</script>"));
  writeFileSync(join(context.root, "imports.ts"), script(source));
  config(context.root, { sortImports: {} });
  cli(context, "sort-cli-standalone", ["--write", "imports.ts"], 0, [
    "imports.ts",
    "vite.config.mjs",
  ]);
  assert.equal(readFileSync(join(context.root, "imports.ts"), "utf8"), script(expected));
  const invalid: JsonRecord[] = [
    { partitionByNewline: true },
    { groups: {} },
    { newlineBetween: false },
    { customGroups: [{ groupName: "local", selectorTypo: "type" }] },
    { groups: [{ newlinesBetween: true }, "external"] },
    { groups: ["external", { newlinesBetween: true }] },
    { groups: ["external", { newlinesBetween: true }, { newlinesBetween: false }, "sibling"] },
  ];
  for (const [index, policy] of invalid.entries()) {
    writeFileSync(file, source);
    const before = config(context.root, { sortImports: policy });
    cli(
      context,
      "sort-cli-refusal-" + index,
      ["--config", "vite.config.mjs", "--write", "UserCard.vue"],
      2,
      tracked,
    );
    assert.equal(readFileSync(file, "utf8"), source);
    assert.deepEqual(readFileSync(configPath), before);
    cli(
      context,
      "sort-cli-refusal-bypass-" + index,
      ["--no-config", "--check", "UserCard.vue"],
      0,
      tracked,
    );
  }
  writeFileSync(join(context.root, "config.json"), "{");
  cli(
    context,
    "sort-cli-malformed-json",
    ["--config", "config.json", "--write", "UserCard.vue"],
    2,
    ["UserCard.vue", "config.json"],
  );
  assert.equal(readFileSync(file, "utf8"), source);
  cli(context, "sort-cli-malformed-bypass", ["--no-config", "--check", "UserCard.vue"], 0, tracked);
  resetAuthoredCase(context, "sort-cli-single-evaluation");
  mkdirSync(join(context.root, "nested"));
  for (const name of ["UserCard.vue", "Ignored.vue"])
    writeFileSync(join(context.root, "nested", name), source);
  writeFileSync(join(context.root, "eval-count.txt"), "0");
  const testSource = originalBytes(
    context.original,
    "crates/vize/tests/fmt_import_sorting_cli.rs",
  ).toString("utf8");
  const effectful = /r#"(\nimport \{ readFileSync, writeFileSync \}[\s\S]*?)"#,\n/u.exec(
    testSource,
  );
  assert(effectful && effectful[1]);
  writeFileSync(configPath, effectful[1]);
  cli(
    context,
    "sort-cli-single-evaluation",
    ["--config", "vite.config.mjs", "--write", "*.vue"],
    0,
    ["nested/UserCard.vue", "nested/Ignored.vue", "eval-count.txt", "vite.config.mjs"],
  );
  assert.equal(readFileSync(join(context.root, "nested/UserCard.vue"), "utf8"), expected);
  assert.equal(readFileSync(join(context.root, "nested/Ignored.vue"), "utf8"), source);
  assert.equal(readFileSync(join(context.root, "eval-count.txt"), "utf8"), "1");
}
