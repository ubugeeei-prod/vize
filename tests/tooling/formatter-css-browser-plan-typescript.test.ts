import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { needsFormatterCssBrowser } from "../../tools/support/compat/github/formatter-css-browser-plan.ts";
import { toolingTestFiles } from "../../tools/support/compat/github/tooling-test-shards.ts";
import { captureWholeValue } from "./support/sfc-entry-validation-reference.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const helper = "tools/support/compat/github/formatter-css-browser-plan.ts";
const oldHelper = "tools/support/compat/github/formatter-css-browser-plan.mjs";
const fixture =
  "tests/tooling/fixtures/formatter-css-browser-plan-8840/formatter-css-browser-plan.mjs.txt";
const browser = "tests/tooling/formatter-css-rule-layout.test.mjs";
const originalSha = "1b06624431b9f0e1c659b5b1692adcb73cfec4e8cf6827231b452f4b2960d4e0";
const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const dependencies = ["./plan-tooling-tests.mjs", "./run-tooling-tests.mjs"] as const;
type Planner = (plan: unknown, available: unknown, shard: unknown, tier: unknown) => unknown;
type Args = [unknown, unknown, unknown, unknown];
type Input = { id: string; make: (trace: string[]) => Args };

function originalModule(directory: string) {
  const bytes = fs.readFileSync(path.join(root, fixture));
  assert.equal(bytes.length, 1050);
  assert.equal(sha(bytes), originalSha, "whole original 36d998e3 Git bytes");
  assert.equal(
    createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    "d4cc23fe9dc8f36de5ba3add1b6da2a0a1291171",
  );
  let source = bytes.toString("utf8");
  const addresses = dependencies.map((relative) => {
    const file = new URL(relative, pathToFileURL(path.join(root, helper)));
    assert(fs.lstatSync(file).isFile() && !fs.lstatSync(file).isSymbolicLink());
    const raw = fs.readFileSync(file);
    const needle = JSON.stringify(relative);
    assert.equal(source.split(needle).length - 1, 1);
    source = source.replace(needle, JSON.stringify(file.href));
    return { relative, address: file.href, sha256: sha(raw) };
  });
  const file = path.join(directory, "original.mjs");
  fs.writeFileSync(file, source, { flag: "wx" });
  return { file, originalSha256: sha(bytes), addressedSha256: sha(source), addresses };
}

function unchangedDependencies(source: ReturnType<typeof originalModule>) {
  for (const file of source.addresses) {
    assert.equal(sha(fs.readFileSync(new URL(file.address))), file.sha256);
  }
}

function nullTierFrame(file: string): string {
  const raw = fs.readFileSync(file, "utf8");
  const source = file.endsWith(".ts") ? stripTypeScriptTypes(raw, { mode: "strip" }) : raw;
  const matches = source
    .split("\n")
    .flatMap((text, index) => (text.includes(".tier !== tier") ? [{ text, index }] : []));
  assert.equal(matches.length, 1);
  const { text, index } = matches[0];
  const column = text.indexOf(".tier") + 1;
  return `${pathToFileURL(file).href}:${index + 1}\n${text}\n${" ".repeat(column)}^\n\n[TypeError: Cannot read properties of null (reading 'tier')]\n\nNode.js ${process.version}\n`;
}

function observe(fn: Planner, input: Input) {
  const trace: string[] = [];
  const args = input.make(trace);
  try {
    const value = fn(...args);
    return { id: input.id, state: "returned", trace: [...trace], whole: captureWholeValue(value) };
  } catch (error) {
    return { id: input.id, state: "threw", trace: [...trace], whole: captureWholeValue(error) };
  }
}

test("the live typed planner retains whole original outcomes and property evaluation order", async (t) => {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-plan-ts-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const source = originalModule(directory);
  const module: unknown = await import(pathToFileURL(source.file).href);
  assert(module !== null && typeof module === "object" && "needsFormatterCssBrowser" in module);
  assert(typeof module.needsFormatterCssBrowser === "function");
  const before = module.needsFormatterCssBrowser as Planner;
  const available = ["tests/tooling/other.test.mjs", browser];
  const inputs: Input[] = [];
  for (const tier of ["pr", "merge"]) {
    for (const tests of [[], [browser], [...available]]) {
      for (const shard of [undefined, "", "1/1", "0/1", "2/1", "bad", null, 1]) {
        inputs.push({
          id: JSON.stringify({ tier, tests, shard }),
          make: () => [{ version: 1, tier, tests: [...tests] }, [...available], shard, tier],
        });
      }
    }
  }
  for (const [index, plan] of [
    null,
    undefined,
    false,
    0,
    "pr",
    [],
    {},
    { tier: "pr" },
    { tier: "pr", version: 2, tests: [] },
  ].entries()) {
    for (const tier of ["pr", "wrong", undefined]) {
      inputs.push({
        id: `invalid:${index}:${String(tier)}`,
        make: () => [plan, available, "", tier],
      });
    }
  }
  for (const selected of [[browser, browser], ["unknown.test.mjs"], [null], [1]]) {
    inputs.push({
      id: `tests:${JSON.stringify(selected)}`,
      make: () => [{ tier: "pr", version: 1, tests: selected }, available, "", "pr"],
    });
  }
  inputs.push(
    {
      id: "invalid-tier-does-not-read-plan",
      make: (trace) => [
        Object.defineProperty({}, "tier", {
          get() {
            trace.push("tier");
            throw new Error("unexpected tier read");
          },
        }),
        available,
        "",
        "wrong",
      ],
    },
    {
      id: "valid-tier-preserves-getter-error",
      make: (trace) => [
        Object.defineProperty({}, "tier", {
          get() {
            trace.push("tier");
            const error = new Error("getter-stop", { cause: { original: true } });
            Object.defineProperty(error, "unknownField", {
              value: ["retained"],
              enumerable: false,
            });
            throw error;
          },
        }),
        available,
        "",
        "pr",
      ],
    },
    {
      id: "delegated-version-before-tests",
      make: (trace) => [
        {
          get tier() {
            trace.push("tier");
            return "pr";
          },
          get version() {
            trace.push("version");
            return 0;
          },
          get tests() {
            trace.push("tests");
            throw new Error("unexpected tests read");
          },
        },
        available,
        "",
        "pr",
      ],
    },
    {
      id: "valid-plan-full-getter-order",
      make: (trace) => [
        {
          get tier() {
            trace.push("tier");
            return "pr";
          },
          get version() {
            trace.push("version");
            return 1;
          },
          get tests() {
            trace.push("tests");
            return [browser];
          },
        },
        available,
        "",
        "pr",
      ],
    },
    {
      id: "undefined-available-retains-existing-default",
      make: () => [{ version: 1, tier: "pr", tests: [browser] }, undefined, undefined, "pr"],
    },
    {
      id: "null-available-retains-native-error",
      make: () => [{ version: 1, tier: "pr", tests: [] }, null, "", "pr"],
    },
  );
  const previousLimit = Error.stackTraceLimit;
  Error.stackTraceLimit = 0; // Declared observer setting, applied to both complete Error records.
  let observations: { before: ReturnType<typeof observe>; after: ReturnType<typeof observe> }[];
  try {
    observations = inputs.map((input) => ({
      before: observe(before, input),
      after: observe(needsFormatterCssBrowser, input),
    }));
  } finally {
    Error.stackTraceLimit = previousLimit;
  }
  t.diagnostic(
    `formatter-css-plan-whole-functions ${JSON.stringify({ source, node: process.version, errorStackTraceLimit: 0, observations })}`,
  );
  for (const row of observations) assert.deepEqual(row.after, row.before, row.before.id);
  unchangedDependencies(source);
  const skipped = observations.find((row) => row.before.id === "invalid-tier-does-not-read-plan")!;
  assert.deepEqual(skipped.before.trace, []);
  const getter = observations.find((row) => row.before.id === "valid-tier-preserves-getter-error")!;
  assert.deepEqual(getter.before.trace, ["tier"]);
  const version = observations.find((row) => row.before.id === "delegated-version-before-tests")!;
  assert.deepEqual(version.before.trace, ["tier", "version"]);
});

test("actual Node CLI actors retain raw status, output append and first errors before verdict", (t) => {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-plan-cli-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const source = originalModule(directory);
  const planFile = path.join(directory, "plan.json");
  const output = path.join(directory, "github-output");
  const available = toolingTestFiles();
  const base: NodeJS.ProcessEnv = {};
  for (const key of ["PATH", "SystemRoot", "TEMP", "TMP", "TMPDIR"]) {
    if (process.env[key] !== undefined) base[key] = process.env[key];
  }
  const common = {
    ...base,
    VIZE_TOOLING_TEST_PLAN: planFile,
    GITHUB_OUTPUT: output,
    VIZE_TOOLING_TEST_TIER: "pr",
    VIZE_TOOLING_TEST_SHARD: "",
  };
  const cases: {
    id: string;
    text: string;
    env?: Record<string, string | undefined>;
    required?: boolean;
  }[] = [
    {
      id: "browser-pr",
      text: JSON.stringify({ version: 1, tier: "pr", tests: [browser] }),
      required: true,
    },
    {
      id: "no-browser-pr",
      text: JSON.stringify({ version: 1, tier: "pr", tests: [] }),
      required: false,
    },
    {
      id: "whole-merge",
      text: JSON.stringify({ version: 1, tier: "merge", tests: available }),
      env: { VIZE_TOOLING_TEST_TIER: "merge" },
      required: true,
    },
    { id: "malformed-json", text: "{malformed" },
    { id: "valid-tier-null", text: "null" },
    { id: "invalid-tier-null", text: "null", env: { VIZE_TOOLING_TEST_TIER: "wrong" } },
    { id: "missing-plan-variable", text: "{}", env: { VIZE_TOOLING_TEST_PLAN: undefined } },
    {
      id: "missing-plan-file",
      text: "{}",
      env: { VIZE_TOOLING_TEST_PLAN: path.join(directory, "absent.json") },
    },
    {
      id: "missing-output-variable",
      text: JSON.stringify({ version: 1, tier: "pr", tests: [] }),
      env: { GITHUB_OUTPUT: undefined },
    },
    {
      id: "missing-output-parent",
      text: JSON.stringify({ version: 1, tier: "pr", tests: [] }),
      env: { GITHUB_OUTPUT: path.join(directory, "absent", "out") },
    },
    {
      id: "tier-before-invalid-shard",
      text: JSON.stringify({ version: 1, tier: "pr", tests: [browser] }),
      env: { VIZE_TOOLING_TEST_TIER: "merge", VIZE_TOOLING_TEST_SHARD: "bad" },
    },
  ];
  const preload = "data:text/javascript," + encodeURIComponent("Error.stackTraceLimit=0;");
  const sourceFrames = {
    before: nullTierFrame(source.file),
    after: nullTierFrame(path.join(root, helper)),
  };
  const observations = cases.map((input) => {
    const actors = [source.file, path.join(root, helper)].map((file) => {
      fs.writeFileSync(planFile, input.text);
      fs.writeFileSync(output, "previous=retained\n");
      const env = { ...common, ...input.env };
      const result = spawnSync(process.execPath, ["--import", preload, file], {
        cwd: root,
        env,
        maxBuffer: 1024 * 1024,
      });
      return {
        status: result.status,
        signal: result.signal,
        error: captureWholeValue(result.error),
        stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
        stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
        outputBase64: fs.readFileSync(output).toString("base64"),
      };
    });
    return { id: input.id, expectedRequired: input.required, before: actors[0], after: actors[1] };
  });
  t.diagnostic(
    `formatter-css-plan-whole-cli ${JSON.stringify({ source, node: process.version, errorStackTraceLimit: 0, controlledEnvironmentKeys: Object.keys(common), sourceFrames, observations })}`,
  );
  for (const row of observations) {
    if (row.id === "valid-tier-null") {
      const { stderrBase64: beforeStderr, ...before } = row.before;
      const { stderrBase64: afterStderr, ...after } = row.after;
      assert.deepEqual(after, before, row.id);
      assert.equal(Buffer.from(beforeStderr, "base64").toString(), sourceFrames.before);
      assert.equal(Buffer.from(afterStderr, "base64").toString(), sourceFrames.after);
    } else assert.deepEqual(row.after, row.before, row.id);
    if (row.expectedRequired !== undefined) {
      assert.equal(row.after.status, 0);
      assert.equal(row.after.signal, null);
      assert.equal(row.after.stdoutBase64, "");
      assert.equal(row.after.stderrBase64, "");
      assert.equal(
        Buffer.from(row.after.outputBase64, "base64").toString(),
        `previous=retained\nrequired=${row.expectedRequired}\n`,
      );
    } else {
      assert.equal(row.after.status, 1);
      assert.equal(row.after.outputBase64, Buffer.from("previous=retained\n").toString("base64"));
    }
  }
  unchangedDependencies(source);
});

test("the source pin, single live caller and raw Git input custody remain explicit", (t) => {
  const action = fs.readFileSync(
    path.join(root, ".github/actions/install-formatter-css-browser/action.yml"),
    "utf8",
  );
  assert.equal(action.split(helper).length - 1, 1);
  assert.equal(action.includes(oldHelper), false);
  const existingTest = fs.readFileSync(
    path.join(root, "tests/tooling/formatter-css-browser-plan.test.mjs"),
    "utf8",
  );
  assert.equal(existingTest.split(helper).length - 1, 1);
  assert.equal(existingTest.includes(oldHelper), false);
  assert.equal(
    fs.readFileSync(path.join(root, ".gitattributes"), "utf8").split(`${fixture} -text`).length - 1,
    1,
  );
  const inputBytes = fs.readFileSync(path.join(root, fixture));
  const gitModes = ["false", "true"].map((mode) => {
    const result = spawnSync(
      "git",
      ["-c", `core.autocrlf=${mode}`, "hash-object", `--path=${fixture}`, "--stdin"],
      { cwd: root, input: inputBytes },
    );
    return {
      mode,
      status: result.status,
      signal: result.signal,
      error: captureWholeValue(result.error),
      stdout: (result.stdout ?? Buffer.alloc(0)).toString(),
      stderr: (result.stderr ?? Buffer.alloc(0)).toString(),
    };
  });
  t.diagnostic(`formatter-css-plan-git-input-custody ${JSON.stringify(gitModes)}`);
  for (const result of gitModes) {
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(result.stderr, "");
    assert.equal(result.stdout, "d4cc23fe9dc8f36de5ba3add1b6da2a0a1291171\n");
  }
});
