import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { LspDiagnostic, LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixture = path.join(root, "tests/_fixtures/differential/lsp/code-action-diagnostics");
const reference = JSON.parse(fs.readFileSync(path.join(fixture, "references.json"), "utf8"));
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function currentCliExpectation(expected: unknown) {
  assert.ok(Array.isArray(expected));
  return expected.map((row: { messages: unknown[] }) => ({
    ...row,
    messages: row.messages.map((entry) => {
      assert.ok(entry !== null && typeof entry === "object");
      const message = entry as Record<string, unknown>;
      const accessibility = message.ruleId === "a11y/alt-text";
      if (!accessibility)
        assert.equal(message.ruleId, "vue/html-self-closing", "authored code-action rule");
      assert.equal(
        message.ruleDocsPath,
        accessibility ? "docs/content/rules/accessibility.md" : "docs/content/rules/vue.md",
        "frozen code-action documentation",
      );
      const page = accessibility ? "a11y-alt-text" : "vue-html-self-closing";
      for (const locale of ["", "ja/"])
        assert.ok(
          fs.existsSync(path.join(root, `docs/content/${locale}rules/reference/${page}.md`)),
        );
      return { ...message, ruleDocsPath: `docs/content/rules/reference/${page}.md` };
    }),
  }));
}

void test("current code-action CLI expectations preserve the complete historical witness", () => {
  const frozen = fs.readFileSync(path.join(fixture, "references.json"));
  assert.equal(frozen.length, 2643);
  assert.equal(hash(frozen), "28013efd08e0386a6a34ac94f718e29ec1d7d4196f0d8b63a7dd93bc703d13d0");
  const saved = JSON.stringify(reference);
  const current = currentCliExpectation(reference.cli);
  assert.equal(JSON.stringify(reference), saved);
  assert.equal(
    JSON.stringify(current)
      .replaceAll(
        "docs/content/rules/reference/a11y-alt-text.md",
        "docs/content/rules/accessibility.md",
      )
      .replaceAll(
        "docs/content/rules/reference/vue-html-self-closing.md",
        "docs/content/rules/vue.md",
      ),
    JSON.stringify(reference.cli),
  );
  for (const row of current)
    for (const message of row.messages)
      assert.ok(
        [
          "docs/content/rules/reference/a11y-alt-text.md",
          "docs/content/rules/reference/vue-html-self-closing.md",
        ].includes(message.ruleDocsPath),
      );
  for (const ruleDocsPath of [
    undefined,
    "docs/content/rules/foreign.md",
    "docs/content/rules/reference/vue-html-self-closing.md",
  ]) {
    assert.throws(
      () =>
        currentCliExpectation([{ messages: [{ ruleId: "vue/html-self-closing", ruleDocsPath }] }]),
      /frozen code-action documentation/u,
    );
  }
  assert.throws(
    () =>
      currentCliExpectation([
        { messages: [{ ruleId: "vue/foreign", ruleDocsPath: "docs/content/rules/vue.md" }] },
      ]),
    /authored code-action rule/u,
  );
});

test("source LSP selects each same-range original diagnostic and preserves valid fixes", async () => {
  assert.equal(reference.schema, "vize.lsp.code-action-diagnostics.regression");
  for (const row of reference.originalFiles) {
    const bytes = fs.readFileSync(path.join(fixture, row.path));
    assert.equal(bytes.length, row.bytes);
    assert.equal(hash(bytes), row.sha256);
  }
  const source = fs.readFileSync(path.join(fixture, "App.vue.txt"), "utf8");
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const sourceBuild = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
  validateBuildReceipt(sourceBuild, build);
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-actions-8000-")));
  const file = path.join(workspace, "src/App.vue");
  const uri = pathToFileURL(file).href;
  const oldBinary = process.env.VIZE_LSP_BIN;
  const oldRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  const rows: Array<Record<string, unknown>> = [];
  const output = path.join(root, "target/differential/lsp-code-action-diagnostics.json");
  const record = {
    sourceBuild,
    reference,
    original: source,
    rows,
    status: "PENDING",
    error: null as string | null,
    stderr: "",
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const save = () => fs.writeFileSync(output, `${JSON.stringify(record, null, 2)}\n`);
  let session: LspSession | undefined;
  try {
    fs.mkdirSync(path.dirname(file));
    fs.writeFileSync(file, source);
    writeVueConfigFixture(
      workspace,
      fs.readFileSync(path.join(fixture, "vize.config.json"), "utf8"),
    );
    const argv = ["lint", "src/App.vue", "--format", "json"];
    const cli = spawnSync(binary, argv, { cwd: workspace });
    rows.push({
      method: "CLI",
      argv,
      status: cli.status,
      signal: cli.signal,
      error: cli.error?.message ?? null,
      stdoutBytes: cli.stdout === null ? null : [...cli.stdout],
      stderrBytes: cli.stderr === null ? null : [...cli.stderr],
    });
    save();
    assert.equal(cli.error, undefined);
    assert.equal(cli.signal, null);
    assert.equal(cli.status, 0);
    assert.deepEqual(cli.stderr, Buffer.alloc(0));
    assert.deepEqual(
      cli.stdout,
      Buffer.from(JSON.stringify(currentCliExpectation(reference.cli), null, 2)),
    );
    assert.equal(fs.readFileSync(file, "utf8"), source);

    process.env.VIZE_LSP_BIN = binary;
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
    session = new LspSession();
    const initialized = await session.initialize(workspace, {
      editor: true,
      lint: true,
      codeActions: true,
      typecheck: false,
    });
    rows.push({ method: "initialize", result: initialized });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await publication(uri, 1, reference.diagnostics);
    const diagnostics: LspDiagnostic[] = reference.diagnostics;
    const range: LspRange = reference.diagnostics[0].range;
    const [alt, style] = diagnostics;
    const expected = [
      {
        title: "Fix: Use separate start and end tags",
        kind: "quickfix",
        diagnostics: [style],
        edit: {
          changes: {
            [uri]: [
              {
                range: {
                  start: { line: 1, character: 22 },
                  end: { line: 1, character: 24 },
                },
                newText: "",
              },
            ],
          },
        },
        isPreferred: true,
      },
      suppression(alt, 1),
      suppression(style, 1),
    ];
    assert.deepEqual(await request(uri, range, [alt]), [expected[1]]);
    assert.deepEqual(await request(uri, range, [style]), [expected[0], expected[2]]);
    assert.deepEqual(await request(uri, range, diagnostics), expected);
    assert.deepEqual(await request(uri, range, [...diagnostics].reverse()), expected);
    assert.deepEqual(await request(uri, range, [...diagnostics, ...diagnostics]), expected);
    for (const rejected of [
      { ...alt, code: "vue/no-multi-spaces" },
      { ...alt, source: "vize/types", code: 2322 },
      { ...alt, source: "foreign/linter" },
      { ...alt, code: undefined },
      { ...alt, source: undefined },
      { ...alt, code: 2322 },
      { ...alt, range: { start: { line: 1, character: 3 }, end: range.end } },
    ])
      assert.equal(await request(uri, range, [rejected]), null);
    const cursorOnly = expected.map(({ diagnostics: _diagnostics, ...action }) => action);
    assert.deepEqual(await request(uri, range, []), cursorOnly);
    assert.equal(await request(uri, range, diagnostics, ["source"]), null);
    assert.equal(await request(uri, range, diagnostics, []), null);
    assert.equal(
      await request(
        uri,
        { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
        diagnostics,
      ),
      null,
    );
    assert.equal(
      await request(pathToFileURL(path.join(workspace, "Other.vue")).href, range, diagnostics),
      null,
    );

    const insertedStyle = source.replace(
      "  <img",
      "  <!-- @vize:forget vue/html-self-closing -->\n  <img",
    );
    const shiftedAlt = shifted(alt);
    const shiftedStyle = shifted(style);
    await change(2, insertedStyle, [shiftedAlt]);
    assert.equal(await request(uri, range, diagnostics), null);
    assert.deepEqual(await request(uri, shiftedAlt.range as LspRange, [shiftedAlt]), [
      suppression(shiftedAlt, 2),
    ]);
    await change(3, source, diagnostics);
    assert.deepEqual(await request(uri, range, diagnostics), expected);
    const insertedAlt = source.replace("  <img", "  <!-- @vize:forget a11y/alt-text -->\n  <img");
    await change(4, insertedAlt, [shiftedStyle]);
    assert.deepEqual(await request(uri, shiftedStyle.range as LspRange, [shiftedStyle]), [
      {
        ...expected[0],
        diagnostics: [shiftedStyle],
        edit: {
          changes: {
            [uri]: [
              {
                range: {
                  start: { line: 2, character: 22 },
                  end: { line: 2, character: 24 },
                },
                newText: "",
              },
            ],
          },
        },
      },
      suppression(shiftedStyle, 2),
    ]);
    await change(5, reference.fixedSource, []);
    assert.equal(await request(uri, range, diagnostics), null);
    await change(6, source, diagnostics);
    session.notify("textDocument/didClose", { textDocument: { uri } });
    await publication(uri, undefined, []);
    assert.equal(await request(uri, range, diagnostics), null);

    const fixSource = '<template><div  id="x">ok</div></template>';
    const fixUri = pathToFileURL(path.join(workspace, "src/Fix.vue")).href;
    const fixRange = { start: { line: 0, character: 14 }, end: { line: 0, character: 16 } };
    const fixDiagnostic = {
      range: fixRange,
      severity: 2,
      code: "vue/no-multi-spaces",
      codeDescription: { href: "https://eslint.vuejs.org/rules/no-multi-spaces.html" },
      source: "vize/lint",
      message: "Multiple consecutive spaces",
    };
    const componentNameDiagnostic = {
      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
      severity: 1,
      code: "vue/multi-word-component-names",
      codeDescription: {
        href: "https://eslint.vuejs.org/rules/multi-word-component-names.html",
      },
      source: "vize/lint",
      message:
        'Component name "Fix" should be multi-word to avoid conflicts with HTML elements\n\n' +
        'Help: Rename the component to use multiple words (e.g., "TodoItem" instead of "Item")',
    };
    session.notify("textDocument/didOpen", {
      textDocument: { uri: fixUri, languageId: "vue", version: 1, text: fixSource },
    });
    await publication(fixUri, 1, [componentNameDiagnostic, fixDiagnostic]);
    const fix = {
      title: "Fix: Replace multiple spaces with single space",
      kind: "quickfix",
      diagnostics: [fixDiagnostic],
      edit: { changes: { [fixUri]: [{ range: fixRange, newText: " " }] } },
      isPreferred: true,
    };
    const suppress = {
      title: "Suppress with @vize:forget (vue/no-multi-spaces)",
      kind: "quickfix",
      diagnostics: [fixDiagnostic],
      edit: {
        changes: {
          [fixUri]: [
            {
              range: { start: { line: 0, character: 10 }, end: { line: 0, character: 10 } },
              newText: "<!-- @vize:forget vue/no-multi-spaces -->\n",
            },
          ],
        },
      },
      isPreferred: false,
    };
    assert.deepEqual(await request(fixUri, fixRange, [fixDiagnostic]), [fix, suppress]);
    session.notify("textDocument/didChange", {
      textDocument: { uri: fixUri, version: 2 },
      contentChanges: [{ range: fixRange, text: " " }],
    });
    await publication(fixUri, 2, [componentNameDiagnostic]);
    assert.equal(await request(fixUri, fixRange, [fixDiagnostic]), null);
    assert.equal(
      fs.readFileSync(file, "utf8"),
      source,
      "buffer edits never rewrite the authored file",
    );
    assert.equal(
      rows.length,
      36,
      "one CLI, initialize, 25 requests and nine complete publications",
    );
    record.status = "PASS";
  } catch (error) {
    record.status = "FAIL";
    record.error = String(error);
    throw error;
  } finally {
    try {
      await session?.shutdown();
    } catch (error) {
      record.status = "FAIL";
      record.error ??= String(error);
    } finally {
      record.stderr = session?.stderrText ?? "";
      save();
      if (oldBinary === undefined) delete process.env.VIZE_LSP_BIN;
      else process.env.VIZE_LSP_BIN = oldBinary;
      if (oldRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
      else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = oldRequired;
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  }

  assert.equal(record.status, "PASS", record.error ?? "complete session must finish successfully");

  async function publication(target: string, version: number | undefined, diagnostics: unknown[]) {
    assert.ok(session);
    const actual = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, target) && params.version === version,
    )) as PublishDiagnosticsParams;
    rows.push({ method: "textDocument/publishDiagnostics", result: actual });
    save();
    assert.deepEqual(actual, {
      uri: target,
      ...(version === undefined ? {} : { version }),
      diagnostics,
    });
  }
  async function request(
    target: string,
    range: LspRange,
    diagnostics: LspDiagnostic[],
    only = ["quickfix"],
  ) {
    assert.ok(session);
    const params = { textDocument: { uri: target }, range, context: { diagnostics, only } };
    const row: Record<string, unknown> = { method: "textDocument/codeAction", params };
    rows.push(row);
    try {
      row.result = await session.request("textDocument/codeAction", params);
      return row.result;
    } catch (error) {
      row.error = String(error);
      throw error;
    } finally {
      save();
    }
  }
  async function change(version: number, text: string, diagnostics: unknown[]) {
    assert.ok(session);
    session.notify("textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: [{ text }],
    });
    await publication(uri, version, diagnostics);
  }
  function shifted(diagnostic: LspDiagnostic) {
    const range = diagnostic.range as LspRange;
    return {
      ...diagnostic,
      range: {
        start: { ...range.start, line: range.start.line + 1 },
        end: { ...range.end, line: range.end.line + 1 },
      },
    };
  }
  function suppression(diagnostic: LspDiagnostic, line: number) {
    const position = { line, character: 0 };
    return {
      title: `Suppress with @vize:forget (${String(diagnostic.code)})`,
      kind: "quickfix",
      diagnostics: [diagnostic],
      edit: {
        changes: {
          [uri]: [
            {
              range: { start: position, end: position },
              newText: `  <!-- @vize:forget ${String(diagnostic.code)} -->\n`,
            },
          ],
        },
      },
      isPreferred: false,
    };
  }
});
