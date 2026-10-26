import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = JSON.parse(
  fs.readFileSync(
    path.join(root, "crates/vize_glyph/tests/fixtures/preset-layout-10609/text.json"),
    "utf8",
  ),
) as { id: string; input: string; expected?: string; printWidth: number }[];

void test("source CLI text layouts preserve parsed markup and rendered whitespace", async (t) => {
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
      components: Object.fromEntries(
        ["MyChip", "MyButton", "MyIcon"].map((name) => [
          name,
          {
            render(this: { $slots: { default?: () => unknown } }) {
              return Vue.h("span", this.$slots.default?.());
            },
          },
        ]),
      ),
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
    ]) {
      const source = fixture.input.replaceAll("\n", newline);
      let expected = fixture.expected?.replaceAll("\n", newline);
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
        const actual = fs.readFileSync(file, "utf8");
        if (expected === undefined) expected = actual;
        assert.equal(actual, expected, fixture.id);
      }
      for (const unread of [0, 2]) {
        for (const visible of [false, true]) {
          const state = {
            unread,
            count: unread,
            label: "状態 & <value>",
            openNav: () => {},
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
          assert.equal(await render(expected!, state), await render(source, state), fixture.id);
        }
      }
    }
  }

  const empty = path.join(directory, "Empty.vue");
  fs.writeFileSync(empty, "");
  for (const preset of [[], ["--preset", "prettier"]]) {
    for (const mode of ["--check", "--write", "--check"]) {
      const result = spawnSync(binary, ["fmt", mode, "--no-config", ...preset, empty], {
        encoding: "utf8",
        timeout: 60_000,
      });
      assert.ifError(result.error);
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(fs.readFileSync(empty), Buffer.alloc(0));
    }
  }
});
