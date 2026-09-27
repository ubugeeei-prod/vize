import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const workflow = parse(readFileSync(join(root, ".github/workflows/pr-source-checks.yml"), "utf8"));
const selectedJs = "${{ needs.pr-source-plan.outputs.js == 'true' }}";
const selectedMerge =
  "${{ github.event_name == 'merge_group' && needs.pr-source-plan.outputs.js == 'true' }}";

function recordingCli(directory, source = "process.exit(0);") {
  const calls = join(directory, "calls.jsonl");
  writeFileSync(
    join(directory, "vp"),
    `#!${process.execPath}\nimport fs from 'node:fs'; import path from 'node:path'; import {spawnSync} from 'node:child_process'; const args=process.argv.slice(2); fs.appendFileSync(${JSON.stringify(calls)},JSON.stringify({args,cwd:process.cwd()})+'\\n'); ${source}\n`,
    { mode: 0o755 },
  );
  return {
    calls,
    env: { ...process.env, PATH: `${directory}:${process.env.PATH}` },
    records: () =>
      existsSync(calls) ? readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse) : [],
  };
}

void test("PR keeps native declaration/type checks; merge queue preserves the complete JS tail", () => {
  const names = [
    "Test JS packages",
    "Check Fresco native declarations and consumer types",
    "Check UI package conformance in merge queue",
  ];
  const steps = workflow.jobs["pr-js-packages"].steps.filter((step) => names.includes(step.name));
  assert.deepEqual(
    steps.map((step) => step.name),
    names,
  );
  assert.deepEqual(
    steps.map((step) => step.if),
    [selectedJs, selectedJs, selectedMerge],
  );
  assert.ok(workflow.jobs["source-report"].needs.includes("pr-js-packages"));
  for (const result of ["failure", "cancelled", "skipped"]) {
    assert.equal(aggregateNeedsResults({ "pr-js-packages": { result } }).exitCode, 1);
  }
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-tier-"));
  try {
    const cli = recordingCli(fixture);
    for (const [event, js] of [
      ["pull_request", true],
      ["merge_group", true],
      ["pull_request", false],
      ["merge_group", false],
    ]) {
      rmSync(cli.calls, { force: true });
      for (const step of steps) {
        if (!js || (step.if === selectedMerge && event !== "merge_group")) continue;
        const result = spawnSync("/bin/sh", ["-c", step.run], {
          cwd: root,
          env: cli.env,
          encoding: "utf8",
        });
        assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      }
      const expected = js
        ? [
            ["run", "--workspace-root", "test:js"],
            ["run", "--filter", "./npm/fresco-native", "check:generated"],
            ["run", "--filter", "./npm/fresco-native", "check:types"],
            ...(event === "merge_group"
              ? [
                  ["run", "--filter", "./npm/native", "build:ci"],
                  ["run", "--filter", "./npm/ui", "check"],
                ]
              : []),
          ]
        : [];
      assert.deepEqual(
        cli.records().map((record) => record.args),
        expected,
      );
    }
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("Fresco checks generate declarations and invoke static TypeScript without executing the loader", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-fresco-static-"));
  const packageDir = join(fixture, "package");
  const originalPackage = join(root, "npm/fresco-native");
  const tsc = createRequire(join(originalPackage, "package.json")).resolve("typescript/bin/tsc");
  try {
    for (const file of [
      "package.json",
      "index.js",
      "index.d.ts",
      "tsconfig.types.json",
      "tests/types/consumer.ts",
      "scripts/verify-consumer-types.mjs",
    ]) {
      mkdirSync(dirname(join(packageDir, file)), { recursive: true });
      copyFileSync(join(originalPackage, file), join(packageDir, file));
    }
    const cli = recordingCli(
      fixture,
      `
if(args[0]==='exec' && args[1]==='napi') {
  const out=args[args.indexOf('--output-dir')+1]; fs.mkdirSync(out,{recursive:true});
  for(const name of ['index.js','index.d.ts']) fs.copyFileSync(path.join(process.cwd(),name),path.join(out,name));
} else if(args[0]==='exec' && args[1]==='tsc') {
  process.exit(spawnSync(process.execPath,[${JSON.stringify(tsc)},...args.slice(2)],{stdio:'inherit'}).status ?? 1);
} else { throw new Error('unexpected CLI'); }
`,
    );
    const generated = spawnSync(
      process.execPath,
      [
        join(originalPackage, "scripts/verify-generated-contract.mjs"),
        "--package-dir",
        packageDir,
        "--profile",
        "ci",
      ],
      { env: cli.env, encoding: "utf8" },
    );
    assert.equal(generated.status, 0, `${generated.stdout}\n${generated.stderr}`);
    const generation = cli.records()[0];
    assert.equal(realpathSync(generation.cwd), realpathSync(packageDir));
    assert.deepEqual(generation.args.slice(0, -1), [
      "exec",
      "napi",
      "build",
      "--platform",
      "--profile",
      "ci",
      "--manifest-path",
      "../../crates/vize_fresco/Cargo.toml",
      "-p",
      "vize_fresco",
      "--features",
      "napi",
      "--output-dir",
    ]);
    assert.equal(existsSync(generation.args.at(-1)), false);
    writeFileSync(
      join(packageDir, "index.js"),
      "throw new Error('runtime loader must not execute');\n",
    );
    const checkTypes = () =>
      spawnSync(process.execPath, [join(packageDir, "scripts/verify-consumer-types.mjs")], {
        env: cli.env,
        encoding: "utf8",
      });
    const typed = checkTypes();
    assert.equal(typed.status, 0, `${typed.stdout}\n${typed.stderr}`);
    const typeCall = cli.records()[1];
    assert.equal(realpathSync(typeCall.cwd), realpathSync(packageDir));
    assert.deepEqual(typeCall.args.slice(0, -1), ["exec", "tsc", "--noEmit", "-p"]);
    assert.equal(existsSync(typeCall.args.at(-1)), false);
    writeFileSync(
      join(packageDir, "tests/types/consumer.ts"),
      "export const broken: string = 1;\n",
    );
    const invalidTypes = checkTypes();
    assert.notEqual(invalidTypes.status, 0);
    assert.match(invalidTypes.stderr, /Fresco native consumer type check failed/);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
