import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const references = "type/no-reactivity-loss";

function location(source, offset) {
  const before = source.slice(0, offset);
  const lines = before.split("\n");
  return { line: lines.length, column: [...lines.at(-1)].length + 1 };
}

function message(source, target, text, help, start = source.indexOf(target)) {
  assert.ok(start >= 0, target);
  assert.equal(source.slice(start, start + target.length), target);
  const from = location(source, start);
  const to = location(source, start + target.length);
  return {
    ruleId: references,
    ruleDocsPath: "docs/content/rules/type-and-script.md",
    severity: 1,
    message: `[vize:${references}] ${text}`,
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
    help,
  };
}

function original(directory, filename) {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, directory, "source.json"), "utf8"));
  assert.equal(manifest.author.login, "ubugeeei");
  assert.equal(manifest.author.id, 71201308);
  const input = manifest.inputs.find((entry) => entry.path === filename);
  assert.ok(input, filename);
  const bytes = fs.readFileSync(path.join(root, directory, input.file));
  assert.equal(bytes.length, input.bytes);
  assert.equal(sha256(bytes), input.sha256);
  return { manifest, input, source: bytes.toString("utf8") };
}

function capture(command, args, directory, env) {
  const result = spawnSync(command, args, {
    cwd: directory,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  return {
    command,
    args,
    status: result.status,
    signal: result.signal,
    error: result.error ? { name: result.error.name, message: result.error.message } : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

test("source-built lint preserves complete original P0 findings with native TypeScript", () => {
  const artifact = path.join(root, "target/differential/lint-p0-originals.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.p0-originals", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-p0-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { receipt, expected: build };
    persist();
    validateBuildReceipt(receipt, build);
    const extension = process.platform === "win32" ? ".exe" : "";
    const provider = path.join(
      root,
      "node_modules/@typescript",
      `typescript-${process.platform}-${process.arch}`,
    );
    const native = path.join(provider, "lib", `tsc${extension}`);
    const nativePackage = JSON.parse(fs.readFileSync(path.join(provider, "package.json"), "utf8"));
    assert.equal(nativePackage.version, "7.0.2");
    const env = { ...process.env, CORSA_BIN: native };
    delete env.VIZE_TEST_DISABLE_TSGO;
    const version = capture(native, ["--version"], directory, env);
    evidence.native = {
      package: nativePackage.name,
      version: nativePackage.version,
      binarySha256: sha256(fs.readFileSync(native)),
      probe: version,
    };
    persist();
    assert.equal(version.error, null, JSON.stringify(version));
    assert.equal(version.status, 0, JSON.stringify(version));
    assert.match(version.stdout.trim(), /^Version 7\.0\.2$/);
    const cli = path.join(root, build.binaryPath);
    const cliVersion = capture(cli, ["--version"], directory, env);
    evidence.cliVersion = cliVersion;
    persist();
    assert.equal(cliVersion.error, null, JSON.stringify(cliVersion));
    assert.equal(cliVersion.status, 0, JSON.stringify(cliVersion));
    assert.equal(cliVersion.stdout.trim(), build.cliVersion);
    const modules = path.join(root, "tests/node_modules");
    const vue = JSON.parse(fs.readFileSync(path.join(modules, "vue/package.json"), "utf8"));
    evidence.vue = {
      version: vue.version,
      typesSha256: sha256(fs.readFileSync(path.join(modules, "vue/dist/vue.d.ts"))),
    };
    fs.symlinkSync(
      modules,
      path.join(directory, "node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
    const tsconfig = {
      compilerOptions: {
        strict: true,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
      },
      include: ["*.vue", "*.ts"],
    };
    fs.writeFileSync(path.join(directory, "tsconfig.json"), JSON.stringify(tsconfig));
    evidence.tsconfig = tsconfig;
    const patina = "crates/vize_patina/tests/fixtures";
    const croquis = "crates/vize_croquis/tests/fixtures";
    const cases = [
      {
        issue: 7913,
        rule: "script/require-function-return-type",
        original: original(`${patina}/issue-7913`, "RunButton.vue"),
        messages: [],
      },
      {
        issue: 7913,
        rule: "script/require-function-return-type",
        original: original(`${patina}/issue-7913`, "callbacks.ts"),
        messages: [],
      },
      {
        issue: 7911,
        rule: "vue/no-deprecated-filter",
        original: original(`${patina}/issue-7911`, "TextField.vue"),
        messages: [],
      },
      {
        issue: 7914,
        rule: references,
        original: original(`${croquis}/issue-7914`, "LazyInput.vue"),
      },
    ];
    cases[3].messages = [
      message(
        cases[3].original.source,
        "props.modelValue",
        "Assigning 'props.modelValue' to 'lastEmitted' stores a plain snapshot",
        "Use toRef(source, 'key'), toRefs(source), or access the property on the reactive object.",
      ),
    ];
    const controlSource = fs.readFileSync(
      path.join(root, croquis, "issue-7914/RightHandSide.vue.fixture"),
      "utf8",
    );
    cases.push({
      issue: 7914,
      rule: references,
      control: {
        authority: "Authored JavaScript RHS evaluation-order regression from source peer review",
        input: {
          path: "RightHandSide.vue",
          bytes: Buffer.byteLength(controlSource),
          sha256: sha256(controlSource),
        },
        source: controlSource,
      },
      messages: [
        message(
          controlSource,
          "props.modelValue",
          "Assigning 'props.modelValue' to 'last' stores a plain snapshot",
          "Use toRef(source, 'key'), toRefs(source), or access the property on the reactive object.",
        ),
        message(
          controlSource,
          "last",
          "Passing 'last' to 'useFeature' cuts the reactive graph from 'props.modelValue'",
          "Pass Ref<T> or ComputedRef<T> instead, for example toRef(source, 'key') or computed(() => value).",
          controlSource.indexOf("useFeature(last)") + "useFeature(".length,
        ),
      ],
    });
    for (const entry of cases) {
      const { input, source } = entry.original ?? entry.control;
      fs.writeFileSync(path.join(directory, input.path), source);
      const config = {
        linter: {
          preset: "incremental",
          rules: { [entry.rule]: "warn" },
          ...(entry.rule === references ? { strictReactivity: true } : {}),
        },
        typeChecker: { corsaPath: native },
      };
      fs.writeFileSync(path.join(directory, "vize.config.json"), JSON.stringify(config));
      const modes = entry.rule === references ? [[], ["--type-aware"]] : [[]];
      for (const mode of modes) {
        const run = capture(
          cli,
          [
            "lint",
            input.path,
            "--format",
            "json",
            "--locale",
            "en",
            "--help-level",
            "full",
            ...mode,
          ],
          directory,
          env,
        );
        const expected = [
          {
            file: input.path,
            messages: entry.messages,
            errorCount: 0,
            warningCount: entry.messages.length,
          },
        ];
        const observation = {
          issue: entry.issue,
          original: entry.original,
          control: entry.control,
          config,
          mode,
          run,
          expected,
        };
        evidence.runs.push(observation);
        persist();
        assert.equal(run.error, null, JSON.stringify(run));
        assert.equal(run.signal, null, JSON.stringify(run));
        assert.equal(run.status, 0, JSON.stringify(run));
        observation.actual = JSON.parse(run.stdout);
        persist();
        assert.deepEqual(observation.actual, expected);
      }
      fs.unlinkSync(path.join(directory, input.path));
    }
    console.log(
      "VIZE_LINT_P0_ORIGINALS",
      JSON.stringify({
        sourceRevision: build.sourceRevision,
        binarySha256: build.binarySha256,
        native: nativePackage.version,
        runs: evidence.runs.length,
      }),
    );
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
