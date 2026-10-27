import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { LspWire } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import { range } from "./support/lsp/bind-style/expected.ts";

// The native CLI and editor must consume the same effective LinterConfig,
// including entry-local false values. Keep complete packets for both surfaces.
const code = "vue/comment-directive";
const rule = "vue/no-v-html";
const message = `Unused eslint-disable directive (no problems were reported from '${rule}').`;
const marker = `eslint-disable-next-line ${rule}`;
const source = (body: string) => `<template>\n${body}\n</template>\n`;
const fixtureRoot = path.join(root, "tests/_fixtures/differential/lsp/unused-disable-config-10930");
const unused = fs.readFileSync(path.join(fixtureRoot, "App.vue.txt"), "utf8");
const used = source(`😀 <!-- ${marker} -->\n<div v-html="'trusted'" />`);
const repaired = source("<div />");
const referenceBinary = process.env.VIZE_UNUSED_DISABLE_BASELINE_BIN;
assert.ok(
  !referenceBinary || process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1",
  "an external reference cannot satisfy requested source-built validation",
);
const binary =
  referenceBinary ??
  resolveVizeLaunchCommand(undefined, path.join(root, "target/ci/vize"), {
    required: true,
    repoRoot: root,
  })[0]!;
const output = path.join(root, "target/differential/lint-unused-disable-config");
const sha256 = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");

test("native unused-disable configuration preserves flag precedence and all template pragma forms", () => {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-unused-cli-")));
  const packets: unknown[] = [];
  const config = (linter: unknown, entries?: unknown[]) => {
    fs.writeFileSync(
      path.join(directory, "vite.config.mjs"),
      `export default ${JSON.stringify({ vue: { linter, entries } })};`,
    );
  };
  const run = (args: string[] = [], text = unused) => {
    fs.writeFileSync(path.join(directory, "App.vue"), text);
    const result = spawnSync(binary, ["lint", "App.vue", "--format", "json", ...args], {
      cwd: directory,
      encoding: "utf8",
      timeout: 30_000,
    });
    packets.push({
      args,
      source: text,
      config: fs.readFileSync(path.join(directory, "vite.config.mjs"), "utf8"),
      ...result,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return result;
  };
  try {
    config({ preset: "essential", reportUnusedDisableDirectives: true });
    const enabled = run();
    assert.equal(enabled.status, 1, enabled.stdout + enabled.stderr);
    const report = JSON.parse(enabled.stdout);
    assert.equal(report.length, 1);
    assert.equal(report[0].errorCount, 1);
    assert.equal(report[0].warningCount, 0);
    assert.equal(report[0].messages.length, 1);
    assert.equal(report[0].messages[0].ruleId, code);
    assert.equal(report[0].messages[0].message, `[vize:${code}] ${message}`);
    assert.equal(run(["--no-config", "--preset", "essential"]).status, 0);
    const flags = run([
      "--no-config",
      "--preset",
      "essential",
      "--report-unused-disable-directives",
    ]);
    assert.equal(flags.status, 1);
    assert.deepEqual(JSON.parse(flags.stdout), report);
    for (const setting of [false, undefined]) {
      config({ preset: "essential", reportUnusedDisableDirectives: setting });
      const disabled = run();
      assert.equal(disabled.status, 0, disabled.stdout + disabled.stderr);
      assert.deepEqual(JSON.parse(disabled.stdout)[0].messages, []);
      const explicit = run(["--report-unused-disable-directives"]);
      assert.equal(explicit.status, 1);
      assert.deepEqual(JSON.parse(explicit.stdout), report);
    }
    for (const linter of [{ rules: { "vue/no-v-html": "error" } }, { preset: "essential" }]) {
      config({ preset: "essential", reportUnusedDisableDirectives: true }, [
        { files: ["App.vue"], linter },
      ]);
      assert.deepEqual(JSON.parse(run().stdout), report, "omitted entry policy inherits true");
    }
    config({ preset: "essential", reportUnusedDisableDirectives: true }, [
      { files: ["App.vue"], linter: { reportUnusedDisableDirectives: false } },
    ]);
    assert.equal(run().status, 0);
    assert.equal(run(["--report-unused-disable-directives"]).status, 1);
    config({ preset: "essential", reportUnusedDisableDirectives: false }, [
      { files: ["App.vue"], linter: { reportUnusedDisableDirectives: true } },
    ]);
    assert.deepEqual(JSON.parse(run().stdout), report);
    config({ preset: "essential", reportUnusedDisableDirectives: true });
    for (const prefix of ["eslint", "oxlint"]) {
      for (const kind of ["disable-next-line", "disable-line", "disable"]) {
        const pragma = `${prefix}-${kind} ${rule}`;
        const body = (element: string) =>
          kind === "disable-line"
            ? `${element} <!-- ${pragma} -->`
            : `<!-- ${pragma} -->\n${element}\n<!-- ${prefix}-enable ${rule} -->`;
        const stale = run([], source(body("<div />")));
        assert.equal(stale.status, 1, stale.stdout + stale.stderr);
        assert.deepEqual(
          JSON.parse(stale.stdout)[0].messages.map((entry: { ruleId: string }) => entry.ruleId),
          [code],
        );
        assert.equal(run([], source(body("<div v-html=\"'trusted'\" />"))).status, 0);
      }
    }
    assert.equal(run([], used).status, 0);
    assert.equal(run([], repaired).status, 0);
    fs.writeFileSync(
      path.join(directory, "vite.config.mjs"),
      `export default {lint:{vue:{preset:'essential',reportUnusedDisableDirectives:true}}};`,
    );
    assert.deepEqual(
      JSON.parse(run().stdout),
      report,
      "lint.vue alias projects the same native setting",
    );
    config({ preset: "essential", reportUnusedDisableDirectives: true });
    for (const text of [
      "<!-- eslint-disable vue/no-v-html -->\n" + repaired,
      "<script setup>\n// eslint-disable-next-line script/no-with-defaults\nconst value=1;\n</script>\n" +
        repaired,
      repaired +
        "<style>\n/* eslint-disable-next-line css/no-important */\n.a {color:red}\n</style>\n",
      source("<!-- vize-disable-next-line vue/no-v-html -->\n<div />"),
      source("<!-- @vize:forget vue/no-v-html -->\n<div />"),
      source("<!-- eslint-enable vue/no-v-html -->\n<div />"),
      source("<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->\n<div />"),
    ]) {
      const configured = run([], text);
      const flag = run(
        ["--no-config", "--preset", "essential", "--report-unused-disable-directives"],
        text,
      );
      assert.equal(configured.status, 0, configured.stdout + configured.stderr);
      assert.equal(flag.status, 0, flag.stdout + flag.stderr);
      assert.deepEqual(
        JSON.parse(configured.stdout),
        JSON.parse(flag.stdout),
        "existing native pragma scope remains consistent",
      );
    }
    config({ ruleOptions: { [code]: { reportUnusedDisableDirectives: true } } });
    const unsupported = run();
    assert.equal(unsupported.status, 2);
    assert.match(
      unsupported.stderr,
      /vue\.linter\.ruleOptions: unknown option "vue\/comment-directive"/u,
    );
  } finally {
    fs.mkdirSync(output, { recursive: true });
    fs.writeFileSync(
      path.join(output, "cli.json"),
      JSON.stringify({ binary, binarySha256: sha256(binary), packets }, null, 2),
    );
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test(
  "editor publishes UTF16 unused pragma diagnostics after edits and configuration reload",
  { timeout: 90_000 },
  async () => {
    const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-unused-lsp-")));
    const file = path.join(directory, "App.vue");
    const configFile = path.join(directory, "vite.config.mjs");
    const uri = pathToFileURL(file).href;
    let wire: LspWire | undefined;
    const config = (report: boolean) =>
      fs.writeFileSync(
        configFile,
        `export default ${JSON.stringify({
          vue: {
            linter: { preset: "essential", reportUnusedDisableDirectives: report },
            languageServer: { lint: true, typecheck: false, editor: false, ecosystem: false },
          },
        })};`,
      );
    try {
      config(true);
      fs.writeFileSync(file, repaired);
      wire = new LspWire(binary, ["lsp", "--stdio"], directory);
      const initialized = await wire.request("initialize", {
        processId: null,
        rootUri: pathToFileURL(directory).href,
        workspaceFolders: [{ uri: pathToFileURL(directory).href, name: "unused-disable" }],
        capabilities: { textDocument: { publishDiagnostics: { versionSupport: true } } },
      });
      assert.equal(initialized.error, undefined);
      wire.notify("initialized", {});
      wire.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: repaired },
      });
      const publication = async (version: number, expected: unknown[]) => {
        const packet = await wire!.waitFor(
          (packet) =>
            packet.method === "textDocument/publishDiagnostics" &&
            packet.params?.uri === uri &&
            packet.params?.version === version,
        );
        assert.deepEqual(packet, {
          jsonrpc: "2.0",
          method: "textDocument/publishDiagnostics",
          params: { uri, version, diagnostics: expected },
        });
      };
      const diagnostic = {
        range: range(unused, marker),
        severity: 1,
        code,
        codeDescription: { href: "https://eslint.vuejs.org/rules/comment-directive.html" },
        source: "vize/lint",
        message,
      };
      assert.deepEqual(
        JSON.parse(fs.readFileSync(path.join(fixtureRoot, "case.json"), "utf8")).initialDiagnostics,
        [diagnostic],
      );
      const change = (version: number, text: string) =>
        wire!.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        });
      await publication(1, []);
      change(2, unused);
      await publication(2, [diagnostic]);
      change(3, used);
      await publication(3, []);
      change(4, unused);
      await publication(4, [diagnostic]);
      config(false);
      wire.notify("workspace/didChangeWatchedFiles", {
        changes: [{ uri: pathToFileURL(configFile).href, type: 2 }],
      });
      change(5, unused);
      await publication(5, []);
      config(true);
      wire.notify("workspace/didChangeWatchedFiles", {
        changes: [{ uri: pathToFileURL(configFile).href, type: 2 }],
      });
      change(6, unused);
      await publication(6, [diagnostic]);
      change(7, repaired);
      await publication(7, []);
      await wire.finish();
    } finally {
      if (wire) await wire.stop().catch(() => undefined);
      fs.mkdirSync(output, { recursive: true });
      fs.writeFileSync(
        path.join(output, "lsp.json"),
        JSON.stringify(
          {
            binary,
            binarySha256: sha256(binary),
            source: { unused, used, repaired },
            observation: wire?.observation() ?? null,
          },
          null,
          2,
        ),
      );
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

const pklAvailable = spawnSync("pkl", ["--version"], { encoding: "utf8" }).status === 0;
test(
  "rendered Pkl linter entries preserve omitted inheritance and explicit false",
  { skip: !pklAvailable },
  () => {
    const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-unused-pkl-")));
    const packets: unknown[] = [];
    try {
      fs.writeFileSync(path.join(directory, "App.vue"), unused);
      for (const schema of ["LinterConfig.pkl", "vize.pkl"]) {
        for (const setting of [undefined, false, true]) {
          const input = `import ${JSON.stringify(path.join(root, "npm/cli/pkl", schema))} as Schema
entry = new Schema.LinterConfig {
  preset = "essential"
${setting === undefined ? "" : `  reportUnusedDisableDirectives = ${setting}\n`}}
`;
          const pklFile = path.join(directory, "entry.pkl");
          fs.writeFileSync(pklFile, input);
          const rendered = spawnSync("pkl", ["eval", "-f", "json", pklFile], {
            encoding: "utf8",
            timeout: 30_000,
          });
          assert.equal(rendered.status, 0, rendered.stdout + rendered.stderr);
          const raw = JSON.parse(rendered.stdout);
          assert.equal(raw.entry.reportUnusedDisableDirectives ?? undefined, setting);
          // Public config normalization drops Pkl's nullable defaults before passing
          // the resulting document to native tools.
          const entry = JSON.parse(
            JSON.stringify(raw.entry, (_key, value: unknown) =>
              value === null ? undefined : value,
            ),
          );
          const config = {
            vue: {
              linter: { preset: "essential", reportUnusedDisableDirectives: true },
              entries: [{ files: ["App.vue"], linter: entry }],
            },
          };
          fs.writeFileSync(path.join(directory, "config.json"), JSON.stringify(config));
          const result = spawnSync(
            binary,
            ["lint", "App.vue", "--format", "json", "--config", "config.json"],
            { cwd: directory, encoding: "utf8", timeout: 30_000 },
          );
          packets.push({ schema, setting, input, rendered, config, result });
          assert.equal(result.error, undefined);
          assert.equal(result.status, setting === false ? 0 : 1, result.stdout + result.stderr);
          assert.deepEqual(
            JSON.parse(result.stdout)[0].messages.map(
              (message: { ruleId: string }) => message.ruleId,
            ),
            setting === false ? [] : [code],
          );
        }
      }
    } finally {
      fs.mkdirSync(output, { recursive: true });
      fs.writeFileSync(path.join(output, "pkl.json"), JSON.stringify({ binary, packets }, null, 2));
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
