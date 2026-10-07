import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../..");
const corpus = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-original-path-scope-7903",
);
const engine = path.join(repository, "node_modules/oxlint/bin/oxlint");
const cli = path.join(packageDir, "dist/cli.mjs");
const plugin = path.join(packageDir, "dist/index.mjs");

void test("discovered JSON and inherited MTS preserve original ignores, overrides and whole core diagnostics", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-discovered-physical-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.mkdirSync(path.join(root, ".git"));
  fs.writeFileSync(path.join(root, ".gitignore"), "dist/\nsrc/Ignored.vue\n");
  const source = fs.readFileSync(path.join(corpus, "Panel.vue.txt"), "utf8");
  const scriptless = '<template>\n  <div v-html="html" />\n</template>\n';
  const files = [
    ["src/Panel.vue", source],
    ["src/Static.vue", scriptless],
    ["src/pages/About.vue", source],
    ["src/pages/generated/Generated.vue", source],
    ["src/Ignored.vue", source],
    ["src/CliIgnored.vue", source],
    ["vendor/Vendor.vue", source],
    ["dist/Built.vue", source],
  ];
  for (const [name, bytes] of files) {
    fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    fs.writeFileSync(path.join(root, name), bytes);
  }
  const layer = fs
    .readFileSync(path.join(corpus, "vue-layer.mts.txt"), "utf8")
    .replace('"PLUGIN_PATH"', JSON.stringify(plugin));
  const module = fs.readFileSync(path.join(corpus, "oxlint.config.mts.txt"), "utf8");
  fs.writeFileSync(path.join(root, "vue-layer.mts"), layer);
  fs.writeFileSync(path.join(root, "oxlint.config.mts"), module);
  const args = [
    "--format",
    "json",
    "--ignore-pattern",
    "src/CliIgnored.vue",
    "src",
    "dist",
    "vendor",
  ];
  const run = (entrypoint: string) => {
    const result = spawnSync(process.execPath, [entrypoint, ...args], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.equal(result.stderr, "");
    return JSON.parse(result.stdout);
  };
  const ordered = (rows: Array<{ filename: string; code: string }>) =>
    rows.sort((a, b) => a.filename.localeCompare(b.filename) || a.code.localeCompare(b.code));
  for (const kind of ["module", "json"]) {
    if (kind === "json") {
      fs.unlinkSync(path.join(root, "oxlint.config.mts"));
      fs.writeFileSync(
        path.join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [plugin],
          categories: { correctness: "off" },
          settings: { vize: { preset: "incremental", helpLevel: "none" } },
          ignorePatterns: ["vendor/**", "node_modules/**"],
          rules: { "no-debugger": "warn", "vize/vue/no-v-html": "warn" },
          overrides: [
            { files: ["src/pages/**/*.vue"], rules: { "vize/vue/no-v-html": "off" } },
            { files: ["src/pages/generated/**/*.vue"], rules: { "vize/vue/no-v-html": "warn" } },
          ],
        }),
      );
    }
    const before = fs.readdirSync(root).sort();
    const stock = run(engine);
    const transported = run(cli);
    assert.equal(stock.number_of_files, 4);
    assert.equal(transported.number_of_files, 4);
    const core = (report: typeof stock) =>
      ordered(report.diagnostics.filter((row: { code: string }) => !row.code.startsWith("vize(")));
    assert.deepEqual(
      core(transported),
      core(stock),
      `${kind}: preserve every core byte/range/field`,
    );
    assert.equal(
      core(stock).filter((row: { code: string }) => row.code.includes("no-debugger")).length,
      3,
    );
    const vize = ordered(
      transported.diagnostics.filter((row: { code: string }) => row.code.startsWith("vize(")),
    );
    assert.deepEqual(
      vize.map(
        (row: {
          filename: string;
          code: string;
          labels: Array<{ span: { line: number; column: number } }>;
        }) => ({
          filename: row.filename,
          code: row.code,
          positions: row.labels.map(({ span }) => [span.line, span.column]),
        }),
      ),
      ordered([
        { filename: "src/Panel.vue", code: "vize(vue/no-v-html)", positions: [[7, 8]] },
        { filename: "src/Static.vue", code: "vize(vue/no-v-html)", positions: [[2, 8]] },
        {
          filename: "src/pages/generated/Generated.vue",
          code: "vize(vue/no-v-html)",
          positions: [[7, 8]],
        },
      ]),
    );
    assert.deepEqual(fs.readdirSync(root).sort(), before);
    for (const [name, bytes] of files)
      assert.equal(fs.readFileSync(path.join(root, name), "utf8"), bytes);
    assert.equal(fs.readFileSync(path.join(root, "vue-layer.mts"), "utf8"), layer);
    if (kind === "module")
      assert.equal(fs.readFileSync(path.join(root, "oxlint.config.mts"), "utf8"), module);
  }
});
