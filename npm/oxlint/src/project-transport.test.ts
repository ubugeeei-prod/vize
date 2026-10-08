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
const engine =
  process.env.VIZE_OXLINT_TEST_ENTRYPOINT ??
  path.join(repository, "node_modules/oxlint/bin/oxlint");
const cli = path.join(packageDir, "dist/cli.mjs");
const plugin = path.join(packageDir, "dist/index.mjs");
const capturePath =
  process.env.VIZE_OXLINT_PROJECT_CAPTURE ??
  path.join(repository, "target/oxlint-original-project-checks-7903.json");
const capture: {
  engine: string;
  engineVersion: string;
  qualified: { imports: boolean; typeAware: boolean };
  observations: unknown[];
} = {
  engine,
  engineVersion: JSON.parse(
    fs.readFileSync(path.join(path.dirname(engine), "../package.json"), "utf8"),
  ).version,
  qualified: { imports: false, typeAware: false },
  observations: [],
};
function save(observation: unknown): void {
  capture.observations.push(observation);
  fs.mkdirSync(path.dirname(capturePath), { recursive: true });
  fs.writeFileSync(capturePath, `${JSON.stringify(capture, null, 2)}\n`);
}

void test("import and custom script rules retain whole stock diagnostics on original paths", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-original-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.mkdirSync(path.join(root, "src/pages"), { recursive: true });
  fs.mkdirSync(path.join(root, ".git"));
  fs.writeFileSync(path.join(root, ".gitignore"), "src/Ignored.vue\n");
  const source = fs.readFileSync(path.join(corpus, "Panel-project.vue.txt"), "utf8");
  const files = [
    ["src/Panel.vue", source],
    ["src/pages/About.vue", source.replace("./Panel.vue", "../Panel.vue")],
    ["src/Static.vue", '<template>\n  <div v-html="html" />\n</template>\n'],
    ["src/Ignored.vue", source],
    ["src/helper.ts", "export const helper = 1;\n"],
  ];
  for (const [file, bytes] of files) fs.writeFileSync(path.join(root, file), bytes);
  fs.writeFileSync(
    path.join(root, "source-custody.mjs"),
    fs.readFileSync(path.join(corpus, "source-custody.mjs.txt")),
  );
  const config = {
    extends: [
      {
        plugins: ["import"],
        jsPlugins: [plugin, path.join(root, "source-custody.mjs")],
        categories: { correctness: "off" },
        rules: {
          "no-debugger": "warn",
          "import/no-self-import": "warn",
          "source-custody/original-path": ["warn", { marker: "configured-options" }],
          "vize/vue/no-v-html": "warn",
        },
      },
    ],
    settings: { vize: { preset: "incremental", helpLevel: "none" } },
    ignorePatterns: ["node_modules/**"],
    overrides: [{ files: ["src/pages/**/*.vue"], rules: { "vize/vue/no-v-html": "off" } }],
  };
  const configFile = path.join(root, "oxlint.config.mts");
  const writeConfig = () =>
    fs.writeFileSync(configFile, `export default ${JSON.stringify(config)};\n`);
  writeConfig();
  const before = fs.readdirSync(root).sort();
  const run = (entrypoint: string, args = ["-f", "json", "src"], status = 0) => {
    const result = spawnSync(process.execPath, [entrypoint, ...args], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    });
    save({
      phase: entrypoint === cli ? "wrapper" : "stock",
      args,
      config: fs.readFileSync(configFile, "utf8"),
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout,
      stderr: result.stderr,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, status, result.stdout + result.stderr);
    if (status === 0) assert.equal(result.stderr, "");
    return result;
  };
  const ordered = (rows: Array<{ filename: string; code: string }>) =>
    rows.sort((a, b) => a.filename.localeCompare(b.filename) || a.code.localeCompare(b.code));
  const stock = JSON.parse(run(engine).stdout);
  const wrapper = JSON.parse(run(cli).stdout);
  const core = (report: typeof stock) =>
    ordered(report.diagnostics.filter((row: { code: string }) => !row.code.startsWith("vize(")));
  assert.deepEqual(
    core(wrapper),
    core(stock),
    "complete native/custom diagnostics retain filename, severity, option messages and source spans",
  );
  assert.equal(
    core(stock).filter((row: { code: string }) => row.code.includes("no-self-import")).length,
    1,
    "the real import resolver must identify the authored self import",
  );
  assert.equal(
    core(stock).filter((row: { code: string }) => row.code.includes("original-path")).length,
    3,
  );
  assert.equal(wrapper.number_of_files, stock.number_of_files);
  const vize = ordered(
    wrapper.diagnostics.filter((row: { code: string }) => row.code.startsWith("vize(")),
  );
  assert.deepEqual(
    vize.map(
      (row: { filename: string; labels: Array<{ span: { line: number; column: number } }> }) => ({
        filename: row.filename,
        positions: row.labels.map(({ span }) => [span.line, span.column]),
      }),
    ),
    [
      { filename: "src/Panel.vue", positions: [[8, 8]] },
      { filename: "src/Static.vue", positions: [[2, 8]] },
    ],
  );
  assert.equal(wrapper.number_of_rules, stock.number_of_rules);
  assert.equal(wrapper.threads_count, stock.threads_count);
  assert.ok(Number.isFinite(wrapper.start_time) && wrapper.start_time > 0);
  // The original checks pass, but an error found only by Vize must fail the union.
  config.extends[0].rules["vize/vue/no-v-html"] = "error";
  writeConfig();
  const error = JSON.parse(run(cli, ["-f", "json", "src/Static.vue"], 1).stdout);
  assert.equal(error.diagnostics.length, 1);
  assert.equal(error.diagnostics[0].severity, "error");
  for (const format of ["default", "unix", "stylish"]) {
    const text = run(cli, ["-f", format, "src/Panel.vue"], 1).stdout;
    assert.match(text, /Original script owner:/u);
    assert.match(text, /v-html/u);
    assert.doesNotMatch(text, /oxlint-vize-original-|oxlint-vize-[^\s/]+\/inside/u);
  }
  const fix = run(cli, ["--fix", "src"], 1);
  assert.match(fix.stderr, /remove CLI severity\/limit overrides and fix flags/u);
  for (const [file, bytes] of files)
    assert.equal(fs.readFileSync(path.join(root, file), "utf8"), bytes);
  assert.deepEqual(fs.readdirSync(root).sort(), before);
  capture.qualified.imports = true;
  save({
    terminal: "whole import/custom/core and scriptless/report/exit/fix custody controls passed",
  });
});

void test(
  "real type-aware TS diagnostics retain the complete original program result",
  {
    skip:
      process.env.VIZE_REQUIRE_REAL_OXLINT_TYPE_AWARE !== "1"
        ? "the default engine lane does not install oxlint-tsgolint"
        : false,
  },
  (t) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-types-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
    fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
    const tsgolint = path.resolve(path.dirname(engine), "../..", "oxlint-tsgolint");
    fs.symlinkSync(tsgolint, path.join(root, "node_modules/oxlint-tsgolint"));
    fs.mkdirSync(path.join(root, "node_modules/.bin"));
    fs.symlinkSync(
      path.join(tsgolint, "bin/tsgolint.js"),
      path.join(root, "node_modules/.bin/tsgolint"),
    );
    fs.mkdirSync(path.join(root, "src"));
    const ts = 'export {};\nPromise.resolve("missing-await");\n';
    const vue = '<template>\n  <div v-html="html" />\n</template>\n';
    fs.writeFileSync(path.join(root, "src/input.ts"), ts);
    fs.writeFileSync(path.join(root, "src/Static.vue"), vue);
    fs.writeFileSync(
      path.join(root, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { target: "ES2022", strict: true, types: [] },
        include: ["src/**/*.ts"],
      }),
    );
    const config = {
      plugins: ["typescript"],
      jsPlugins: [plugin],
      categories: { correctness: "off" },
      options: { typeAware: true },
      settings: { vize: { preset: "incremental", helpLevel: "none" } },
      rules: { "typescript/no-floating-promises": "error", "vize/vue/no-v-html": "warn" },
    };
    fs.writeFileSync(
      path.join(root, "oxlint.config.mts"),
      `export default ${JSON.stringify(config)};\n`,
    );
    const before = fs.readdirSync(root).sort();
    const run = (entrypoint: string) => {
      const result = spawnSync(process.execPath, [entrypoint, "-f", "json", "src"], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      });
      save({
        phase: entrypoint === cli ? "type-aware-wrapper" : "type-aware-stock",
        args: ["-f", "json", "src"],
        config: fs.readFileSync(path.join(root, "oxlint.config.mts"), "utf8"),
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout,
        stderr: result.stderr,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 1, result.stdout + result.stderr);
      assert.equal(result.stderr, "");
      assert.match(result.stdout, /^\s*\{/u, result.stdout + result.stderr);
      return JSON.parse(result.stdout);
    };
    const stock = run(engine);
    const wrapper = run(cli);
    const core = (report: typeof stock) =>
      report.diagnostics.filter((row: { code: string }) => !row.code.startsWith("vize("));
    assert.equal(core(stock).length, 1);
    assert.match(core(stock)[0].code, /no-floating-promises/u);
    assert.deepEqual(core(wrapper), core(stock));
    assert.equal(wrapper.number_of_files, stock.number_of_files);
    assert.equal(wrapper.number_of_rules, stock.number_of_rules);
    assert.equal(wrapper.threads_count, stock.threads_count);
    assert.deepEqual(
      wrapper.diagnostics
        .filter((row: { code: string }) => row.code.startsWith("vize("))
        .map(
          (row: {
            filename: string;
            labels: Array<{ span: { line: number; column: number } }>;
          }) => ({
            filename: row.filename,
            positions: row.labels.map(({ span }) => [span.line, span.column]),
          }),
        ),
      [{ filename: "src/Static.vue", positions: [[2, 8]] }],
    );
    assert.equal(fs.readFileSync(path.join(root, "src/input.ts"), "utf8"), ts);
    assert.equal(fs.readFileSync(path.join(root, "src/Static.vue"), "utf8"), vue);
    assert.deepEqual(fs.readdirSync(root).sort(), before);
    capture.qualified.typeAware = true;
    save({ terminal: "whole actual type-aware TS program and scriptless Vize controls passed" });
  },
);
