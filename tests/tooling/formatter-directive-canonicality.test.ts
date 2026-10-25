import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = JSON.parse(
  fs.readFileSync(
    path.join(root, "crates/vize_glyph/tests/fixtures/preset-layout-10609/directives.json"),
    "utf8",
  ),
) as { id: string; input: string; expected: string; printWidth: number }[];

void test("source CLI directive layouts preserve parsed expressions and rendered values", async (t) => {
  const identity = expectedBuildIdentity(root);
  validateBuildReceipt(
    JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
    ),
    identity,
  );
  const binary = path.join(root, identity.binaryPath);
  const require = createRequire(path.join(root, "npm/ui/package.json"));
  const Vue = require("vue");
  const vueRequire = createRequire(require.resolve("vue"));
  const { compile } = vueRequire("@vue/compiler-dom");
  const { parse } = require("vue/compiler-sfc");
  const { renderToString } = require("vue/server-renderer");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-directive-canonicality-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));

  async function render(source: string, state: Record<string, unknown>) {
    const parsed = parse(source);
    assert.deepEqual(parsed.errors, []);
    const diagnostics: unknown[] = [];
    const compiled = compile(parsed.descriptor.template.content, {
      mode: "function",
      onError: (error: unknown) => diagnostics.push(error),
    });
    assert.deepEqual(diagnostics, []);
    const render = compileFunction(compiled.code, ["Vue"])(Vue);
    render._rc = true;
    const app = Vue.createSSRApp({
      data: () => state,
      render,
      components: { MyChip: { render: () => Vue.h("span") } },
    });
    const warnings: string[] = [];
    app.config.warnHandler = (warning: string) => warnings.push(warning);
    const html = await renderToString(app);
    assert.deepEqual(warnings, []);
    return html;
  }

  for (const fixture of fixtures) {
    for (const [ending, newline] of [
      ["lf", "\n"],
      ["crlf", "\r\n"],
      ["cr", "\r"],
    ]) {
      const source = fixture.input.replaceAll("\n", newline);
      const expected = fixture.expected.replaceAll("\n", newline);
      const file = path.join(directory, "App.vue");
      fs.writeFileSync(file, source);
      for (const mode of ["--write", "--check", "--write"]) {
        const result = spawnSync(
          binary,
          [
            "fmt",
            mode,
            "--no-config",
            "--preset",
            "prettier",
            "--print-width",
            String(fixture.printWidth),
            "--trailing-comma",
            "none",
            "--arrow-parens",
            "avoid",
            "--end-of-line",
            ending,
            file,
          ],
          { encoding: "utf8", timeout: 60_000 },
        );
        assert.ifError(result.error);
        assert.equal(result.status, 0, `${fixture.id}: ${result.stderr}`);
        assert.equal(fs.readFileSync(file, "utf8"), expected, fixture.id);
      }
      for (const unread of [0, 2]) {
        for (const visible of [false, true]) {
          const state = {
            unread,
            visible,
            isSelected: visible,
            isEnabled: visible,
            hasPermissionForTheSelectedItemInTheCurrentView: true,
            props: {
              selectedIdentifier: visible ? "selected" : "other",
              totalPoint: 103,
              label: "状態 & <value>",
            },
            SELECTED_IDENTIFIER_CONSTANT: "selected",
          };
          assert.equal(await render(expected, state), await render(source, state), fixture.id);
        }
      }
    }
  }

  for (const raw of ["\n", "\r\n", "\r"]) {
    const literal = `String.raw\`first${raw}  exact \${unread} line${raw}last\``;
    const printedLiteral = "String.raw`first\r  exact ${unread} line\rlast`";
    const source = `<template>\r  <div :class="{ [${literal}]: visible, active: visible }" />\r</template>\r`;
    const expected = `<template>\r  <div\r    :class="{\r      [${printedLiteral}]: visible,\r      active: visible\r    }"\r  />\r</template>\r`;
    const file = path.join(directory, "Raw.vue");
    fs.writeFileSync(file, source);
    let canonical: string | undefined;
    for (const mode of ["--write", "--check", "--write"]) {
      const result = spawnSync(
        binary,
        [
          "fmt",
          mode,
          "--no-config",
          "--preset",
          "prettier",
          "--print-width",
          "80",
          "--trailing-comma",
          "none",
          "--end-of-line",
          "cr",
          file,
        ],
        { encoding: "utf8", timeout: 60_000 },
      );
      assert.ifError(result.error);
      assert.equal(result.status, 0, result.stderr);
      const actual = fs.readFileSync(file, "utf8");
      assert.equal(actual, expected, "configured CR layout retains literal spaces");
      if (canonical !== undefined) assert.equal(actual, canonical);
      canonical = actual;
    }
    for (const unread of [0, 2]) {
      const authoredValue = compileFunction(`return (${literal})`, ["unread"])(unread);
      const printedValue = compileFunction(`return (${printedLiteral})`, ["unread"])(unread);
      assert.equal(authoredValue, `first\n  exact ${unread} line\nlast`);
      assert.equal(printedValue, authoredValue, "String.raw normalizes CR, CRLF and LF equally");
      for (const visible of [false, true]) {
        const state = { unread, visible };
        assert.equal(await render(canonical!, state), await render(source, state));
      }
    }
  }
});

void test("fluent formatter presets retain default order and explicit sorting overrides", (t) => {
  const identity = expectedBuildIdentity(root);
  validateBuildReceipt(
    JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
    ),
    identity,
  );
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preset-transport-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const input =
    '<template>\n  <div title="kept" id="main">{{ id }}</div>\n</template>\n\n<script setup lang="ts">\nconst id = "main";\n</script>\n';
  const file = path.join(directory, "App.vue");
  const helper = pathToFileURL(path.join(root, "npm/builder/vite/src/vite-plus.ts")).href;
  for (const sorting of [undefined, false, true]) {
    const formatter = {
      preset: "prettier",
      printWidth: 120,
      ...(sorting === undefined ? {} : { sortAttributes: sorting, sortBlocks: sorting }),
    };
    fs.writeFileSync(
      path.join(directory, "vite.config.mts"),
      `import { defineConfig } from ${JSON.stringify(helper)};\nexport default defineConfig({ fmt: { vue: false } }).withVue({ formatter: ${JSON.stringify(formatter)} });\n`,
    );
    fs.writeFileSync(file, input);
    const result = spawnSync(path.join(root, identity.binaryPath), ["fmt", "--write", file], {
      cwd: directory,
      encoding: "utf8",
      timeout: 60_000,
    });
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    const expected =
      sorting === true
        ? '<script setup lang="ts">\nconst id = "main";\n</script>\n\n<template>\n  <div id="main" title="kept">{{ id }}</div>\n</template>\n'
        : input;
    assert.equal(fs.readFileSync(file, "utf8"), expected);
  }
});
