import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";
import { LspWire } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";
import { diagnostics } from "./support/lsp/bind-style/expected.ts";

const source =
  '<script setup>\nconst html="<b>hello</b>";\n</script>\n<template>\n<p v-html="html" />\n</template>\n';
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const evidenceDirectory = path.join(root, "target/differential/vue-config-cli-lsp");
const fluentModule = pathToFileURL(path.join(root, "npm/builder/vite/src/vite-plus.ts")).href;
const upstreamModule = import.meta.resolve("vite-plus");

test("plain and public fluent Vue settings preserve whole CLI and editor results", async () => {
  // Both PR and protected tooling restore this same-run receipted producer.
  const [binary] = resolveVizeLaunchCommand(undefined, path.join(root, "target/ci/vize"), {
    required: true,
    repoRoot: root,
  });
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-vue-config-")));
  const app = path.join(directory, "App.vue");
  const uri = pathToFileURL(app).href;
  const packets: unknown[] = [];
  const retiredPackets: unknown[] = [];
  fs.mkdirSync(evidenceDirectory, { recursive: true });
  try {
    fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}');
    const retiredConfig = JSON.stringify({ linter: { rules: { "vue/no-v-html": "off" } } });
    const retiredPath = path.join(directory, "vize.config.json");
    fs.writeFileSync(retiredPath, retiredConfig);
    fs.writeFileSync(app, source);
    for (const args of [
      ["lint", "App.vue", "--format", "json"],
      ["lint", "App.vue", "--format", "json", "--config", "vize.config.json"],
    ]) {
      const result = spawnSync(binary, args, {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
      });
      retiredPackets.push({
        config: retiredConfig,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout,
        stderr: result.stderr,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 2, result.stdout + result.stderr);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /vize\.config\.\* is no longer supported/);
      assert.match(result.stderr, /vue: in vite\.config\.\*/);
    }
    fs.rmSync(retiredPath);
    for (const active of [true, false]) {
      const settings = {
        formatter: { singleQuote: true, tabWidth: 4 },
        linter: { preset: "essential", rules: { "vue/no-v-html": active ? "warn" : "off" } },
        languageServer: { editor: false, typecheck: false, lint: true, formatting: true },
      };
      const answers: { lint: unknown; diagnostics: unknown; formatted: string; edits: unknown }[] =
        [];
      for (const selected of ["plain", "fluent"]) {
        const configFile = path.join(directory, "vite.config.mjs");
        const configSource = (vue: unknown) =>
          selected === "plain"
            ? `export default {vue:${JSON.stringify(vue)}};`
            : `import {defineConfig} from ${JSON.stringify(fluentModule)};
import {defineConfig as defineVpConfig} from ${JSON.stringify(upstreamModule)};
export default defineConfig(defineVpConfig(async () => ({
  vue: {formatter: {singleQuote: false}, linter: {rules: {"vue/no-v-html": "error"}}}
}))).withVue(${JSON.stringify(vue)});`;
        const config = configSource(settings);
        fs.writeFileSync(configFile, config);
        fs.writeFileSync(app, source);
        const commands: unknown[] = [];
        let scopedObservation: unknown = null;
        const run = (args: string[]) => {
          const result = spawnSync(binary, args, {
            cwd: directory,
            encoding: "utf8",
            timeout: 30_000,
          });
          commands.push({
            args,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdout: result.stdout,
            stderr: result.stderr,
          });
          assert.equal(result.error, undefined);
          assert.equal(result.signal, null);
          assert.equal(result.status, 0, result.stdout + result.stderr);
          return result;
        };
        let wire: LspWire | undefined;
        try {
          const lint = JSON.parse(run(["lint", "App.vue", "--format", "json"]).stdout);
          assert.equal(lint.length, 1);
          assert.equal(lint[0].warningCount, active ? 1 : 0);
          assert.equal(lint[0].errorCount, 0);
          assert.deepEqual(
            lint[0].messages.map((message: { ruleId: string }) => message.ruleId),
            active ? ["vue/no-v-html"] : [],
          );
          if (active) {
            const scoped = {
              ...settings,
              entries: [{ files: ["App.vue"], linter: { rules: { "vue/no-v-html": "off" } } }],
            };
            const scopedConfig = configSource(scoped);
            const otherApp = path.join(directory, "OtherView.vue");
            fs.writeFileSync(otherApp, source);
            fs.writeFileSync(configFile, scopedConfig);
            const selectedPacket = JSON.parse(run(["lint", "App.vue", "--format", "json"]).stdout);
            const outsidePacket = JSON.parse(
              run(["lint", "OtherView.vue", "--format", "json"]).stdout,
            );
            scopedObservation = { scopedConfig, selectedPacket, outsidePacket };
            assert.equal(selectedPacket.length, 1);
            assert.equal(outsidePacket.length, 1);
            assert.equal(selectedPacket[0].warningCount, 0);
            assert.deepEqual(selectedPacket[0].messages, []);
            assert.equal(outsidePacket[0].warningCount, 1);
            assert.deepEqual(outsidePacket[0].messages, lint[0].messages);
            fs.writeFileSync(configFile, config);
            fs.rmSync(otherApp);
          }
          run(["fmt", "App.vue", "--write"]);
          const formatted = fs.readFileSync(app, "utf8");
          assert.match(formatted, /const html = '<b>hello<\/b>';/);
          fs.writeFileSync(app, source);
          wire = new LspWire(binary, ["lsp", "--stdio"], directory);
          const initialized = await wire.request("initialize", {
            processId: null,
            rootUri: pathToFileURL(directory).href,
            workspaceFolders: [{ uri: pathToFileURL(directory).href, name: "vue-config" }],
            capabilities: { textDocument: { publishDiagnostics: { versionSupport: true } } },
          });
          assert.equal(initialized.error, undefined);
          wire.notify("initialized", {});
          wire.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version: 1, text: source },
          });
          const publication = await wire.waitFor(
            (message) =>
              message.method === "textDocument/publishDiagnostics" &&
              message.params?.uri === uri &&
              message.params?.version === 1,
          );
          assert.deepEqual(publication, {
            jsonrpc: "2.0",
            method: "textDocument/publishDiagnostics",
            params: { uri, version: 1, diagnostics: active ? diagnostics(source) : [] },
          });
          const response = await wire.request("textDocument/formatting", {
            textDocument: { uri },
            options: { tabSize: 4, insertSpaces: true },
          });
          assert.equal(response.error, undefined);
          assert.deepEqual(response.result, [
            {
              range: { start: { line: 0, character: 0 }, end: { line: 6, character: 0 } },
              newText: formatted,
            },
          ]);
          answers.push({
            lint,
            diagnostics: publication.params,
            formatted,
            edits: response.result,
          });
          await wire.finish();
          assert.equal(fs.readFileSync(configFile, "utf8"), config);
          assert.equal(fs.readFileSync(app, "utf8"), source);
        } finally {
          if (wire) await wire.stop().catch(() => undefined);
          packets.push({
            active,
            selected,
            config,
            source,
            commands,
            scopedObservation,
            wire: wire?.observation() ?? null,
            allDecodedMessages: wire?.messages ?? [],
            joinedThroughClose: wire ? wire.exitStatus !== null || wire.signal !== null : false,
          });
        }
      }
      assert.deepEqual(answers[1], answers[0]);
    }
  } finally {
    fs.writeFileSync(
      path.join(evidenceDirectory, "complete-observations.json"),
      JSON.stringify(
        {
          sourceSha: process.env.SOURCE_SHA ?? null,
          binary,
          binarySha256: hash(fs.readFileSync(binary)),
          buildReceipt: JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8")),
          source,
          fluentApi: {
            kind: "public-source-entry",
            url: fluentModule,
            sha256: hash(fs.readFileSync(fileURLToPath(fluentModule))),
          },
          upstreamApi: {
            url: upstreamModule,
            sha256: hash(fs.readFileSync(fileURLToPath(upstreamModule))),
          },
          retiredPackets,
          packets,
        },
        null,
        2,
      ) + "\n",
    );
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
