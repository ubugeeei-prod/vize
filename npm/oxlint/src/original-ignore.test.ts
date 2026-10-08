import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../..");
const corpus = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-original-ignore-7903",
);
const engine =
  process.env.VIZE_OXLINT_TEST_ENTRYPOINT ??
  path.join(repository, "node_modules/oxlint/bin/oxlint");
const cli = path.join(packageDir, "dist/cli.mjs");
const environment = { ...process.env };
delete environment.GITHUB_ACTIONS;
const capturePath =
  process.env.VIZE_OXLINT_ORIGINAL_IGNORE_CAPTURE ??
  path.join(repository, "target/oxlint-original-ignore-7903.json");
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

void test("literal original ignore reproduction preserves VCS, config and CLI exclusions", (t) => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-original-ignore-"));
  t.after(() => fs.rmSync(workspace, { recursive: true, force: true }));
  const root = path.join(workspace, "project");
  const temporary = path.join(workspace, "transport");
  fs.mkdirSync(root);
  fs.mkdirSync(temporary);
  const tree = (directory = workspace): string[] =>
    fs
      .readdirSync(directory, { withFileTypes: true })
      .flatMap((entry) => {
        const file = path.join(directory, entry.name);
        const relative = path.relative(workspace, file);
        if (entry.isSymbolicLink()) return [`link:${relative}:${fs.readlinkSync(file)}`];
        return entry.isDirectory()
          ? [`directory:${relative}`, ...tree(file)]
          : [`file:${relative}`];
      })
      .sort();
  const source = fs.readFileSync(path.join(corpus, "AppPanel.vue.txt"), "utf8");
  const config = fs.readFileSync(path.join(corpus, ".oxlintrc.json"), "utf8");
  const vcs = fs.readFileSync(path.join(corpus, ".gitignore"), "utf8");
  const pins = JSON.parse(fs.readFileSync(path.join(corpus, "source.json"), "utf8")) as {
    files: Array<{ file: string; bytes: number; sha256: string }>;
  };
  for (const pin of pins.files) {
    const bytes = fs.readFileSync(path.join(corpus, pin.file));
    assert.equal(bytes.length, pin.bytes, pin.file);
    assert.equal(sha256(bytes), pin.sha256, pin.file);
  }
  const capture: {
    schema: string;
    complete: boolean;
    pins: unknown;
    setup: unknown;
    observations: unknown[];
  } = {
    schema: "vize.oxlint.original-ignore-7903.v1",
    complete: false,
    pins,
    setup: null,
    observations: [],
  };
  const persist = () => {
    fs.mkdirSync(path.dirname(capturePath), { recursive: true });
    fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
  };
  const initialized = spawnSync("git", ["init", "-q"], {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
  });
  capture.setup = {
    command: "git",
    args: ["init", "-q"],
    cwd: root,
    status: initialized.status,
    signal: initialized.signal,
    error: initialized.error?.message ?? null,
    stdout: initialized.stdout,
    stderr: initialized.stderr,
  };
  persist();
  assert.equal(initialized.error, undefined);
  assert.equal(initialized.signal, null);
  assert.equal(initialized.status, 0, initialized.stdout + initialized.stderr);
  assert.equal(initialized.stdout, "");
  assert.equal(initialized.stderr, "");
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.symlinkSync(packageDir, path.join(root, "node_modules/oxlint-plugin-vize"));
  fs.writeFileSync(path.join(root, ".oxlintrc.json"), config);
  fs.writeFileSync(path.join(root, ".gitignore"), vcs);
  const names = [
    "src/AppPanel.vue",
    "dist/BuiltPanel.vue",
    "vendor/VendorPanel.vue",
    "src/SkippedPanel.vue",
  ];
  for (const name of names) {
    fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    fs.writeFileSync(path.join(root, name), source);
  }
  const custody = () =>
    Object.fromEntries(
      [...names, ".oxlintrc.json", ".gitignore"].map((name) => [
        name,
        fs.readFileSync(path.join(root, name), "utf8"),
      ]),
    );
  const before = custody();
  const directory = tree();
  const save = (row: unknown) => {
    capture.observations.push(row);
    persist();
  };
  const run = (entrypoint: string, args: string[]) => {
    const prior = custody();
    const priorTree = tree();
    const engineSha256 = sha256(fs.readFileSync(engine));
    const result = spawnSync(process.execPath, [entrypoint, ...args], {
      cwd: root,
      env: { ...environment, TMPDIR: temporary },
      encoding: "utf8",
      timeout: 30_000,
    });
    save({
      kind: "process",
      entrypoint,
      args,
      engineSha256,
      before: prior,
      treeBefore: priorTree,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout,
      stderr: result.stderr,
    });
    // Persist the entire raw process before reading possibly damaged inputs.
    const after = custody();
    const afterTree = tree();
    save({ kind: "custody", entrypoint, args, after, treeAfter: afterTree });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.equal(result.stderr, "");
    assert.deepEqual(after, before);
    assert.deepEqual(afterTree, directory);
    return result.stdout;
  };
  const originalArgs = ["--ignore-pattern", "src/SkippedPanel.vue", "."];
  const expected = JSON.parse(fs.readFileSync(path.join(corpus, "expected-wrapper.json"), "utf8"));
  for (let query = 0; query < 3; query++) {
    for (const entrypoint of [engine, cli]) {
      const original = run(entrypoint, originalArgs);
      assert.match(original, /src\/AppPanel\.vue/u);
      assert.doesNotMatch(original, /BuiltPanel|VendorPanel|SkippedPanel/u);
    }
    const stock = JSON.parse(run(engine, ["-f", "json", ...originalArgs]));
    const wrapper = JSON.parse(run(cli, ["-f", "json", ...originalArgs]));
    assert.equal(stock.number_of_files, 1);
    assert.equal(wrapper.number_of_files, 1);
    const isVize = (packet: { code?: string }) => packet.code?.startsWith("vize(");
    assert.deepEqual(
      wrapper.diagnostics.filter((packet: { code?: string }) => !isVize(packet)),
      stock.diagnostics.filter((packet: { code?: string }) => !isVize(packet)),
    );
    assert.deepEqual(wrapper.diagnostics.filter(isVize), expected);
    assert.deepEqual(
      stock.diagnostics.filter(isVize).map((packet: { filename: string }) => packet.filename),
      ["src/AppPanel.vue"],
    );
    for (const report of [stock, wrapper]) {
      assert.deepEqual(Object.keys(report).sort(), [
        "diagnostics",
        "number_of_files",
        "number_of_rules",
        "start_time",
        "threads_count",
      ]);
      assert.ok(Number.isSafeInteger(report.number_of_rules) && report.number_of_rules > 0);
      assert.ok(Number.isSafeInteger(report.threads_count) && report.threads_count >= 0);
      assert.ok(Number.isFinite(report.start_time) && report.start_time >= 0);
    }
  }
  capture.complete = true;
  save({
    terminal: "12 complete literal default/JSON stock/wrapper observations passed",
    sourceSha256: sha256(source),
    configSha256: sha256(config),
    before,
    after: custody(),
  });
});
