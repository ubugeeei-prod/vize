import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";
import { writeVueTscFixture } from "./_helpers/typecheck-divergence-checker-fakes.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function runProvider(label: string, args: string[], cwd?: string) {
  const result = spawnSync(process.execPath, args, {
    cwd,
    encoding: "buffer",
    timeout: 10_000,
  });
  const error = result.error;
  const describe = (value: unknown) =>
    inspect(value, { depth: null, showHidden: true, customInspect: false });
  // Retain every child result before an assertion can stop this control.
  console.log(
    JSON.stringify({
      baselineProviderChild: label,
      command: [process.execPath, ...args],
      cwd: cwd ?? null,
      timeoutMs: 10_000,
      pid: result.pid,
      status: result.status,
      signal: result.signal,
      processError: error
        ? {
            name: error.name,
            message: error.message,
            stack: error.stack ?? null,
            properties: Object.fromEntries(
              Object.getOwnPropertyNames(error).map((key) => [
                key,
                describe(Reflect.get(error, key)),
              ]),
            ),
            enumerableProperties: Object.fromEntries(
              Object.entries(error).map(([key, value]) => [key, describe(value)]),
            ),
            cause: describe(error.cause),
            raw: describe(error),
          }
        : null,
      stdoutBase64: result.stdout?.toString("base64") ?? null,
      stderrBase64: result.stderr?.toString("base64") ?? null,
      stdout: result.stdout?.toString("utf8") ?? null,
      stderr: result.stderr?.toString("utf8") ?? null,
    }),
  );
  return {
    ...result,
    stdout: result.stdout?.toString("utf8") ?? "",
    stderr: result.stderr?.toString("utf8") ?? "",
  };
}

// This executes the declared official JS provider, not the synthetic checkers.
test("official vue-tsc keeps syntax failures under noCheck while retaining semantic full baselines", () => {
  const requireTests = createRequire(path.join(root, "tests/package.json"));
  const manifest = requireTests.resolve("vue-tsc/package.json");
  const provider = path.join(path.dirname(manifest), "bin/vue-tsc.js");
  const version = runProvider("official version", [provider, "--version"]);
  assert.equal(version.error, undefined, version.stderr);
  assert.equal(version.status, 0);
  assert.match(version.stdout, /^Version \S+\s*$/u);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-official-baseline-"));
  try {
    const rows = [];
    for (const [name, filename, source] of [
      ["semantic", "input.ts", "const text: string = 42;\n"],
      ["syntax", "input.d.ts", "declare namespace External {}\n}\n"],
    ] as const) {
      const cwd = path.join(directory, name);
      fs.mkdirSync(cwd);
      fs.writeFileSync(path.join(cwd, filename), source);
      const config = path.join(cwd, "tsconfig.json");
      fs.writeFileSync(
        config,
        JSON.stringify({
          compilerOptions: { noEmit: true, target: "ES2022", types: [] },
          files: [filename],
        }),
      );
      const outputs = [];
      for (const preSemantic of [false, true]) {
        const args = [
          provider,
          "--noEmit",
          "--pretty",
          "false",
          "-p",
          config,
          ...(preSemantic ? ["--noCheck"] : []),
        ];
        const result = runProvider(
          `official ${name} ${preSemantic ? "noCheck" : "full"}`,
          args,
          cwd,
        );
        assert.equal(result.error, undefined, result.stderr);
        assert.equal(result.stderr, "");
        outputs.push(result);
        rows.push({
          name,
          preSemantic,
          provider,
          typescriptVersionOutput: version.stdout,
          status: result.status,
          stdout: result.stdout,
        });
      }
      const [full, probe] = outputs;
      assert.ok(full && probe);
      assert.equal(full.status, 2);
      if (name === "semantic") {
        assert.match(full.stdout, /input\.ts\(1,7\): error TS2322:/u);
        assert.equal(probe.status, 0);
        assert.equal(probe.stdout, "");
      } else {
        assert.match(full.stdout, /input\.d\.ts\(2,1\): error TS1128:/u);
        assert.equal(probe.status, 2);
        assert.equal(probe.stdout, full.stdout);
      }
    }
    console.log(JSON.stringify({ officialPreSemanticProviderControls: rows }));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("synthetic providers preserve official clean and diagnostic exits without changing explicit statuses", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-protocol-"));
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(path.join(directory, "src/App.vue"), "<template />\n");
    const provider = path.join(directory, "vue-tsc.mjs");
    for (const [output, exitCode, expected] of [
      ["", undefined, 0],
      ["src/App.vue(1,1): error TS2322: shared\n", undefined, 2],
      ["src/App.vue(1,1): error TS2322: shared\n", 1, 1],
    ] as const) {
      writeVueTscFixture(
        provider,
        {
          baselineOutput: output,
          exitCode,
          files: ["src/App.vue"],
          fixtureRoot: directory,
          mutation: "match",
        },
        path.join(directory, "invocation.json"),
      );
      for (const preSemantic of [false, true]) {
        const result = runProvider(
          `synthetic exit ${expected} ${preSemantic ? "noCheck" : "full"}`,
          [provider, ...(preSemantic ? ["--noCheck"] : [])],
          directory,
        );
        assert.equal(result.error, undefined);
        assert.equal(result.status, preSemantic ? 0 : expected);
        assert.equal(result.stdout, preSemantic ? "" : output);
        assert.equal(result.stderr, "");
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
