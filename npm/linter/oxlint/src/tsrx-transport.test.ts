import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { formatPatinaMessage } from "./format.ts";
import type { PatinaDiagnostic } from "./model.js";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../../..");
const engine =
  process.env.VIZE_OXLINT_TEST_ENTRYPOINT ??
  path.join(repository, "node_modules/oxlint/bin/oxlint");
const plugin = path.join(packageDir, "dist/index.mjs");
const cli = path.join(packageDir, "dist/cli.mjs");
const native = createRequire(import.meta.url)(path.join(repository, "npm/native/index.js"));
const enabledRules = ["a11y/alt-text", "vue/no-multi-spaces"];
const source = `export function Avatar() @{
  // é🙂 retains the original byte and UTF-16 coordinate domains.
  <figure  class="avatar">é🙂<img src="inline" />
    <img src="one" />
  </figure>
}
export function Footer() @{
  <img src="two" />
}
`;
const malformed = "export function Broken() @{\n  <div>hello</span>\n}\n";
const capture: unknown[] = [];
const save = (row: unknown) => {
  capture.push(row);
  const file = process.env.VIZE_OXLINT_TSRX_CAPTURE;
  if (file) fs.writeFileSync(file, JSON.stringify(capture, null, 2) + "\n");
};

type Packet = {
  code?: string;
  filename: string;
  severity: string;
  message: string;
  labels: unknown[];
};
function expected(
  filename: string,
  text: string,
  diagnostics: PatinaDiagnostic[],
  helpLevel: "full" | "short" | "none",
) {
  return diagnostics.map((finding) => {
    const prefix = Buffer.from(text).subarray(0, finding.location.start.offset).toString("utf8");
    const lines = prefix.split(/\r\n|\r|\n/u);
    return {
      message: formatPatinaMessage(finding, {
        hasMappedLocation: true,
        blockLabel: "tsrx",
        helpLevel,
      }),
      code: `vize(${finding.rule})`,
      severity: finding.rule === "parser/tsrx" ? "error" : "warning",
      filename,
      labels: [
        {
          span: {
            offset: finding.location.start.offset,
            length: finding.location.end.offset - finding.location.start.offset,
            line: lines.length,
            column: finding.location.start.column,
          },
        },
      ],
    };
  });
}

void test("installed hosts lint original TSRX through authenticated carriers and keep whole normal packets", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tsrx-transport-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.mkdirSync(path.join(root, ".git"));
  fs.mkdirSync(path.join(root, "src"));
  const files = {
    "src/Avatar.tsrx": source,
    "src/Broken.tsrx": malformed,
    "src/Clean.tsrx": 'export function Clean() @{\n  <img src="x" alt="safe" />\n}\n',
    "src/Ignored.tsrx": source,
    "src/Custom.tsrx": source,
    "src/Cli.tsrx": source,
    "src/Config.tsrx": source,
    "src/Original.vue": '<script setup>debugger;</script>\n<template><img src="x" /></template>\n',
    "src/Original.jsx": 'const element = <img src="x" />;\n',
    "src/Original.tsx": 'const element: unknown = <img src="x" />;\n',
    "src/Original.ts": "debugger;\n",
  };
  for (const [file, bytes] of Object.entries(files)) fs.writeFileSync(path.join(root, file), bytes);
  fs.writeFileSync(path.join(root, ".gitignore"), "src/Ignored.tsrx\nnode_modules/\n");
  fs.writeFileSync(path.join(root, ".eslintignore"), "src/Custom.tsrx\n");
  const configFile = path.join(root, ".oxlintrc.json");
  const base = {
    jsPlugins: [plugin],
    settings: { vize: { preset: "incremental", helpLevel: "full" } },
    ignorePatterns: ["src/Config.tsrx"],
    rules: {
      "no-debugger": "error",
      "vize/a11y/alt-text": "warn",
      "vize/vue/no-multi-spaces": "warn",
    },
  };
  const invoke = (
    entry: string,
    args = ["-f", "json", "--threads", "1", "--ignore-pattern", "src/Cli.tsrx", "src"],
  ) => {
    const result = spawnSync(process.execPath, [entry, ...args], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    save({ entry, args, config: fs.readFileSync(configFile, "utf8"), ...result });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return result;
  };
  for (const helpLevel of ["full", "short", "none"] as const) {
    const config = JSON.stringify({
      ...base,
      settings: { vize: { preset: "incremental", helpLevel } },
    });
    fs.writeFileSync(configFile, config);
    const stock = invoke(engine),
      wrapper = invoke(cli);
    assert.equal(wrapper.status, 1, wrapper.stderr);
    assert.equal(wrapper.stderr, "");
    const before = JSON.parse(stock.stdout),
      after = JSON.parse(wrapper.stdout);
    const foreign = (report: { diagnostics: Packet[] }) =>
      report.diagnostics.filter((packet) => !packet.code?.startsWith("vize("));
    assert.deepEqual(
      foreign(after),
      foreign(before),
      "whole normal foreign/core packets, multiplicity and order remain exact",
    );
    for (const file of ["src/Original.jsx", "src/Original.tsx"])
      assert.deepEqual(
        after.diagnostics.filter((packet: Packet) => packet.filename === file),
        before.diagnostics.filter((packet: Packet) => packet.filename === file),
        "original JSX/TSX plugin packets remain exact",
      );
    for (const file of [
      "src/Avatar.tsrx",
      "src/Broken.tsrx",
      "src/Clean.tsrx",
      "src/Original.vue",
    ]) {
      const text = files[file as keyof typeof files];
      const direct = native.lintPatinaSfc(text, {
        filename: path.join(root, file),
        preset: "incremental",
        helpLevel,
        enabledRules,
      });
      save({ kind: "native-oracle", file, direct });
      assert.deepEqual(
        after.diagnostics.filter(
          (packet: Packet) => packet.filename === file && packet.code?.startsWith("vize("),
        ),
        expected(file, text, direct.diagnostics, helpLevel),
      );
    }
    assert.equal(
      after.diagnostics.filter((packet: Packet) => packet.code === "vize(parser/tsrx)").length,
      1,
    );
    assert.equal(after.number_of_files, before.number_of_files + 3);
    assert.equal(
      after.number_of_rules,
      before.number_of_rules + 1,
      "only the operational parser rule is added to the authored rule count",
    );
    assert.equal(
      after.diagnostics.some((packet: Packet) =>
        /Ignored|Custom|Cli|Config/u.test(packet.filename),
      ),
      false,
    );
    assert.equal(fs.readFileSync(configFile, "utf8"), config);
  }
  fs.writeFileSync(configFile, JSON.stringify(base));
  for (const target of ["src/Avatar.tsrx", "src/Clean.tsrx"]) {
    const result = invoke(cli, ["-f", "json", "--threads", "1", target]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const report = JSON.parse(result.stdout);
    assert.equal(report.number_of_files, 1);
    assert.equal(
      report.diagnostics.some((packet: Packet) => packet.filename !== target),
      false,
    );
  }
  fs.writeFileSync(
    configFile,
    JSON.stringify({ ...base, rules: { ...base.rules, "vize/a11y/alt-text": "error" } }),
  );
  const markupError = invoke(cli, ["-f", "json", "--threads", "1", "src/Avatar.tsrx"]);
  assert.equal(markupError.status, 1, markupError.stderr);
  const nativeAvatar = native.lintPatinaSfc(source, {
    filename: path.join(root, "src/Avatar.tsrx"),
    preset: "incremental",
    helpLevel: "full",
    enabledRules,
  });
  assert.deepEqual(
    JSON.parse(markupError.stdout).diagnostics,
    expected("src/Avatar.tsrx", source, nativeAvatar.diagnostics, "full").map((packet) => ({
      ...packet,
      severity: packet.code === "vize(a11y/alt-text)" ? "error" : packet.severity,
    })),
  );
  for (const parserSeverity of ["off", "warn", "error"]) {
    fs.writeFileSync(
      configFile,
      JSON.stringify({ ...base, rules: { ...base.rules, "vize/parser/tsrx": parserSeverity } }),
    );
    const result = invoke(cli, ["--quiet", "-f", "json", "--threads", "1", "src/Broken.tsrx"]);
    assert.equal(result.status, 1, result.stderr);
    const direct = native.lintPatinaSfc(malformed, {
      filename: path.join(root, "src/Broken.tsrx"),
      preset: "incremental",
      enabledRules,
    });
    assert.deepEqual(
      JSON.parse(result.stdout).diagnostics,
      expected("src/Broken.tsrx", malformed, direct.diagnostics, "full"),
      "quiet and authored parser severities cannot hide operational syntax failures",
    );
  }
  fs.writeFileSync(configFile, JSON.stringify(base));
  for (const format of ["unix", "stylish", "default"]) {
    const result = invoke(cli, ["-f", format, "--threads", "1", "src/Broken.tsrx"]);
    assert.equal(result.status, 1, result.stderr);
    assert.equal(result.stderr, "");
    assert.match(result.stdout, /parser\/tsrx/u);
    assert.match(result.stdout, /Broken\.tsrx/u);
    assert.doesNotMatch(result.stdout, /oxlint-vize-|\.tsrx\.vue/u);
  }
  for (const value of [
    { ...base, overrides: [{ files: ["**/*.tsrx"], rules: { "vize/a11y/alt-text": "off" } }] },
    { ...base, extends: [] },
  ]) {
    fs.writeFileSync(configFile, JSON.stringify(value));
    const stock = invoke(engine),
      wrapper = invoke(cli);
    assert.deepEqual(JSON.parse(wrapper.stdout).diagnostics, JSON.parse(stock.stdout).diagnostics);
    assert.equal(wrapper.status, 1);
    assert.match(wrapper.stderr, /ConfigOverrides|ConfigInheritance/u);
    assert.match(wrapper.stderr, /vize lint/u);
  }
  fs.writeFileSync(configFile, JSON.stringify(base));
  for (const flags of [["--fix"], ["-W", "all"]]) {
    const result = invoke(cli, [...flags, "-f", "json", "src/Avatar.tsrx"]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unavailable/u);
  }
  fs.mkdirSync(path.join(root, "custom-ignore"));
  fs.writeFileSync(path.join(root, "custom-ignore/.ignore"), "");
  const externalIgnore = invoke(cli, [
    "--ignore-path",
    "custom-ignore/.ignore",
    "-f",
    "json",
    "src/Avatar.tsrx",
  ]);
  assert.equal(externalIgnore.status, 1);
  assert.match(externalIgnore.stderr, /CustomIgnoreFilename/u);
  assert.match(externalIgnore.stderr, /vize lint/u);
  fs.rmSync(path.join(root, "custom-ignore"), { recursive: true });
  for (const mutation of ["source", "ignore", "membership"] as const) {
    const module = path.join(root, `mutate-${mutation}.mjs`);
    const changed =
      mutation === "source"
        ? path.join(root, "src/Avatar.tsrx")
        : mutation === "ignore"
          ? path.join(root, ".gitignore")
          : path.join(root, "src/Added.tsrx");
    const changedBytes =
      mutation === "source"
        ? source.replace("one", "new")
        : mutation === "ignore"
          ? "src/Avatar.tsrx\nnode_modules/\n"
          : source;
    fs.writeFileSync(
      module,
      `import fs from 'node:fs';export default {meta:{name:'mutator'},rules:{change:{meta:{type:'problem'},create(context){return {Program(){if(context.physicalFilename.endsWith('Original.ts'))fs.writeFileSync(${JSON.stringify(changed)},${JSON.stringify(changedBytes)});}};}}}};`,
    );
    fs.writeFileSync(
      configFile,
      JSON.stringify({
        ...base,
        jsPlugins: [plugin, module],
        rules: { ...base.rules, "mutator/change": "warn" },
      }),
    );
    const result = invoke(cli);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /changed during host execution/u);
    assert.equal(
      JSON.parse(result.stdout).diagnostics.some((packet: Packet) =>
        packet.filename.endsWith(".tsrx"),
      ),
      false,
    );
    fs.writeFileSync(path.join(root, "src/Avatar.tsrx"), source);
    fs.writeFileSync(path.join(root, ".gitignore"), "src/Ignored.tsrx\nnode_modules/\n");
    fs.rmSync(path.join(root, "src/Added.tsrx"), { force: true });
    fs.unlinkSync(module);
  }
  for (const [file, bytes] of Object.entries(files))
    assert.equal(fs.readFileSync(path.join(root, file), "utf8"), bytes);
  assert.deepEqual(fs.readdirSync(root).sort(), [
    ".eslintignore",
    ".git",
    ".gitignore",
    ".oxlintrc.json",
    "node_modules",
    "src",
  ]);
  save({
    terminal:
      "complete native/host packets, parser errors, help, locations, ignore selection, refusal and unchanged normal-file controls passed",
  });
});

void test(
  "representative TSRX batches retain complete reports and record CPU and peak RSS",
  { skip: process.platform !== "linux" },
  (t) => {
    assert.ok(fs.existsSync("/usr/bin/time"), "resource qualification requires GNU time");
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tsrx-resources-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.mkdirSync(path.join(root, ".git"));
    fs.mkdirSync(path.join(root, "src"));
    fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
    fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
    const config = JSON.stringify({
      jsPlugins: [plugin],
      settings: { vize: { preset: "incremental" } },
      rules: { "vize/a11y/alt-text": "warn", "vize/vue/no-multi-spaces": "warn" },
    });
    fs.writeFileSync(path.join(root, ".oxlintrc.json"), config);
    const text = 'export function Badge() @{\n  <img src="avatar" alt="safe" />\n}\n';
    for (const count of [100, 1000]) {
      for (let index = 0; index < count; index++)
        fs.writeFileSync(path.join(root, "src", `${index}.tsrx`), text);
      for (let sample = 0; sample < 3; sample++) {
        const usage = path.join(root, "usage.txt");
        const args = [
          "-f",
          "%U %S %M %e",
          "-o",
          usage,
          process.execPath,
          cli,
          "-f",
          "json",
          "--threads",
          "1",
          "src",
        ];
        const result = spawnSync("/usr/bin/time", args, {
          cwd: root,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(result.status, 0, result.stderr);
        assert.equal(result.stderr, "");
        const report = JSON.parse(result.stdout);
        assert.equal(report.number_of_files, count);
        assert.deepEqual(report.diagnostics, []);
        const [userSeconds, systemSeconds, maxRssKiB, elapsedSeconds] = fs
          .readFileSync(usage, "utf8")
          .trim()
          .split(/\s+/u)
          .map(Number);
        assert.ok(userSeconds > 0 && systemSeconds >= 0 && maxRssKiB > 0 && elapsedSeconds > 0);
        save({
          kind: "tsrx-process-resources",
          count,
          sample,
          userSeconds,
          systemSeconds,
          maxRssKiB,
          elapsedSeconds,
          args,
          config,
          ...result,
        });
      }
    }
  },
);
