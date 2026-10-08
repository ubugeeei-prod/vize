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
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const fixture = path.join(root, "tests/_fixtures/differential/linter/ssr-script-setup-7982");
const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "corpus.json"), "utf8"));
const read = (file: string) => fs.readFileSync(path.join(fixture, file), "utf8");
const rule = "ssr/no-browser-globals-in-ssr";
const message = (name: string) => `'${name}' is a browser-only global and is not available in SSR`;
const help =
  "Move browser-only code to client lifecycle hooks like onMounted() or use <ClientOnly>";
const original = read("WidthLabel.vue.txt");
const spans = [
  ["window", 2, 15, 21],
  ["navigator", 3, 14, 23],
  ["document", 7, 29, 49],
] as const;
const jsonMessages = spans.map(([name, line, column, endColumn]) => ({
  ruleId: rule,
  ruleDocsPath: "docs/content/rules/ssr.md",
  severity: 1,
  message: `[vize:${rule}] ${message(name)}`,
  line,
  column,
  endLine: line,
  endColumn,
}));
const diagnostics = spans.map(([name, line, column, endColumn]) => ({
  range: {
    start: { line: line - 1, character: column - 1 },
    end: { line: line - 1, character: endColumn - 1 },
  },
  severity: 2,
  code: rule,
  codeDescription: { href: `https://eslint.vuejs.org/rules/${rule}.html` },
  source: "vize/lint",
  message: `${message(name)}\n\nHelp: ${help}`,
}));

await test("SSR issue original, documented Bad/Good and configuration retain complete authored bytes", () => {
  assert.equal(corpus.schema, "vize.ssr.script-setup-7982");
  for (const [file, expected] of Object.entries(corpus.files) as Array<
    [string, { bytes: number; sha256: string }]
  >) {
    const bytes = fs.readFileSync(path.join(fixture, file));
    assert.equal(bytes.length, expected.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), expected.sha256);
  }
  assert.equal(
    original,
    '<script setup lang="ts">\nconst width = window.innerWidth;\nconst lang = navigator.language;\n</script>\n\n<template>\n  <p>{{ width }} {{ lang }} {{ document.title }}</p>\n</template>\n',
  );
  assert.deepEqual(corpus.expectedOriginalBrowserGlobals, ["window", "navigator", "document"]);
  assert.equal(
    read("DocsBad.vue.txt"),
    '<script setup lang="ts">\nconst width = window.innerWidth;\n</script>\n',
  );
  assert.equal(
    read("DocsGood.vue.txt"),
    '<script setup lang="ts">\nconst width = ref(0);\n\nonMounted(() => {\n  width.value = window.innerWidth;\n});\n</script>\n',
  );
});

await test(
  "source CLI and complete Nuxt/editor publications report original setup reads and unsaved repair",
  {
    skip: process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1",
  },
  async () => {
    const build = expectedBuildIdentity(root),
      binary = path.join(root, build.binaryPath);
    const sourceBuild = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
    validateBuildReceipt(sourceBuild, build);
    const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-ssr-7982-")));
    const file = path.join(workspace, "WidthLabel.vue"),
      uri = pathToFileURL(file).href;
    const output = path.join(root, "target/differential/lint-ssr-script-setup-7982.json");
    const rows: Array<Record<string, unknown>> = [];
    const record = {
      sourceBuild,
      corpus,
      original,
      rows,
      status: "PENDING",
      error: null as string | null,
    };
    fs.mkdirSync(path.dirname(output), { recursive: true });
    const save = () => fs.writeFileSync(output, JSON.stringify(record, null, 2) + "\n");
    const oldBinary = process.env.VIZE_LSP_BIN,
      oldRequired = process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
    let session: LspSession | undefined;
    try {
      fs.writeFileSync(file, original);
      fs.copyFileSync(
        path.join(fixture, "vize.config.json"),
        path.join(workspace, "vize.config.json"),
      );
      for (const preset of [[], ["--no-config", "--preset", "nuxt"]]) {
        const expected = [
          { file: "WidthLabel.vue", messages: jsonMessages, errorCount: 0, warningCount: 3 },
        ];
        const json = invoke([
          "lint",
          "WidthLabel.vue",
          ...preset,
          "--format",
          "json",
          "--help-level",
          "none",
        ]);
        assert.deepEqual(json, Buffer.from(JSON.stringify(expected, null, 2)));
        const plain = invoke([
          "lint",
          "WidthLabel.vue",
          ...preset,
          "-f",
          "plain",
          "--help-level",
          "none",
        ]);
        assert.deepEqual(
          plain,
          Buffer.from(
            "Patina lint report: 3 warnings in 1 file\n\nWidthLabel.vue\n" +
              spans
                .map(
                  ([name, line, column]) =>
                    `  WidthLabel.vue:${line}:${column} warning ${rule} ${message(name)}\n    Reference: docs/content/rules/ssr.md\n`,
                )
                .join(""),
          ),
        );
      }
      for (const [file, expectedNames] of [
        ["DocsBad.vue.txt", ["window"]],
        ["DocsGood.vue.txt", []],
      ] as const) {
        fs.writeFileSync(path.join(workspace, "DocsExample.vue"), read(file));
        const json = JSON.parse(
          invoke([
            "lint",
            "DocsExample.vue",
            "--format",
            "json",
            "--help-level",
            "none",
          ]).toString(),
        );
        const expectedMessages = expectedNames.map((name) => ({
          ...jsonMessages[0],
          message: `[vize:${rule}] ${message(name)}`,
        }));
        assert.deepEqual(json, [
          {
            file: "DocsExample.vue",
            messages: expectedMessages,
            errorCount: 0,
            warningCount: expectedNames.length,
          },
        ]);
      }
      process.env.VIZE_LSP_BIN = binary;
      process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";
      for (const preset of ["incremental", "nuxt"]) {
        const config = JSON.parse(read("vize.config.json"));
        config.linter.preset = preset;
        fs.writeFileSync(path.join(workspace, "vize.config.json"), JSON.stringify(config));
        session = new LspSession();
        rows.push({
          method: "initialize",
          preset,
          result: await session.initialize(workspace, {
            editor: true,
            lint: true,
            typecheck: false,
            ecosystem: false,
          }),
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: original },
        });
        await publication(1, diagnostics);
        const repaired = original
          .replace("window.innerWidth", "0")
          .replace("navigator.language", "'en'")
          .replace("document.title", "lang");
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: repaired }],
        });
        await publication(2, []);
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 3 },
          contentChanges: [{ text: original }],
        });
        await publication(3, diagnostics);
        assert.equal(fs.readFileSync(file, "utf8"), original);
        await session.shutdown();
        session = undefined;
      }
      record.status = "PASS";
    } catch (error) {
      record.status = "FAIL";
      record.error = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      try {
        if (session) await session.shutdown();
      } finally {
        save();
        if (oldBinary === undefined) delete process.env.VIZE_LSP_BIN;
        else process.env.VIZE_LSP_BIN = oldBinary;
        if (oldRequired === undefined) delete process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
        else process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = oldRequired;
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }

    function invoke(argv: string[]) {
      const result = spawnSync(binary, argv, {
        cwd: workspace,
        timeout: 30_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      rows.push({
        method: "CLI",
        argv,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdoutBase64: result.stdout?.toString("base64") ?? null,
        stderrBase64: result.stderr?.toString("base64") ?? null,
      });
      save();
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0);
      assert.deepEqual(result.stderr, Buffer.alloc(0));
      return result.stdout;
    }
    async function publication(version: number, expected: typeof diagnostics) {
      assert(session);
      const actual = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === version,
        30_000,
      )) as PublishDiagnosticsParams;
      rows.push({ method: "textDocument/publishDiagnostics", params: actual });
      save();
      assert.deepEqual(actual, { uri, version, diagnostics: expected });
    }
  },
);
