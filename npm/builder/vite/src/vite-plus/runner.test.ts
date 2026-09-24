import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runTools } from "./runner.ts";
import type { VizeTaskConfig } from "./types.ts";

async function project(run: () => Promise<void>) {
  const cwd = process.cwd();
  const dir = mkdtempSync(path.join(os.tmpdir(), "vize-native-tasks-"));
  try {
    process.chdir(dir);
    writeFileSync("App.vue", "<template><div /></template>\n");
    writeFileSync("script.ts", "export const value = 1\n");
    await run();
    assert.ok(!readdirSync(dir).some((name) => name.startsWith(".vize-vp-")));
  } finally {
    process.chdir(cwd);
    rmSync(dir, { recursive: true, force: true });
  }
}

void test("check runs native checks, native lint and Oxlint, and both formatters after failures", async () => {
  await project(async () => {
    const calls: string[][] = [];
    const commands: string[] = [];
    const metadata: VizeTaskConfig = {
      config: (env) => {
        commands.push(env.command);
        return { linter: { preset: "essential" } };
      },
      options: {},
    };
    const code = await runTools(
      "check",
      [],
      metadata,
      "consumer/vp",
      "vize/bin",
      async (command, args) => {
        assert.equal(command, process.execPath);
        calls.push(args);
        if (args[0] === "vize/bin") {
          const file = args[args.indexOf("--config") + 1];
          assert.equal(path.dirname(file), process.cwd());
          assert.equal(JSON.parse(readFileSync(file, "utf8")).linter.preset, "essential");
        }
        return args[1] === "lint" ? 1 : 0;
      },
    );
    assert.equal(code, 1);
    assert.deepEqual(
      calls.map((args) => args.slice(0, 2)),
      [
        ["vize/bin", "check"],
        ["vize/bin", "lint"],
        ["consumer/vp", "lint"],
        ["vize/bin", "fmt"],
        ["consumer/vp", "fmt"],
      ],
    );
    assert.deepEqual(commands, ["check", "lint", "fmt"]);
    assert.deepEqual(calls[3].slice(4), ["--check", "App.vue"]);
    assert.deepEqual(calls[4], ["consumer/vp", "fmt", "--check"]);
  });
});

void test("formatting owns only selected Vue files and never expands an empty list", async () => {
  await project(async () => {
    const calls: string[][] = [];
    const execute = async (_: string, args: string[]) => {
      calls.push(args);
      return 0;
    };
    await runTools("fmt", ["script.ts"], { config: {}, options: {} }, "vp", "native", execute);
    assert.deepEqual(calls, [["vp", "fmt", "--write", "script.ts"]]);
    calls.length = 0;
    await runTools("fmt", ["App.vue"], { config: {}, options: {} }, "vp", "native", execute);
    assert.deepEqual(calls[0].slice(4), ["--write", "App.vue"]);
  });
});

void test("fmt.ignorePatterns excludes Vue files from check and write, even for explicit paths", async () => {
  await project(async () => {
    const calls: string[][] = [];
    const execute = async (_: string, args: string[]) => {
      calls.push(args);
      return 0;
    };
    const metadata: VizeTaskConfig = {
      config: {},
      options: {},
      fmtIgnorePatterns: ["generated/**"],
    };
    mkdirSync("generated");
    writeFileSync("generated/Generated.vue", "<template><div /></template>\n");
    for (const task of ["fmt:check", "fmt"] as const) {
      calls.length = 0;
      await runTools(task, [], metadata, "vp", "native", execute);
      assert.deepEqual(calls[0].slice(4), [
        task === "fmt:check" ? "--check" : "--write",
        "App.vue",
      ]);
      assert.ok(!calls.some((args) => args.includes("generated/Generated.vue")));
      calls.length = 0;
      await runTools(task, ["generated/Generated.vue"], metadata, "vp", "native", execute);
      assert.deepEqual(calls, [
        ["vp", "fmt", task === "fmt:check" ? "--check" : "--write", "generated/Generated.vue"],
      ]);
    }
  });
});

void test("lint locale and help level are passed only to the native linter", async () => {
  await project(async () => {
    const calls: string[][] = [];
    await runTools(
      "lint",
      ["App.vue"],
      {
        config: { linter: { preset: "happy-path" } },
        options: {},
        lintLocale: "ja",
        lintHelpLevel: "short",
      },
      "vp",
      "native",
      async (_, args) => {
        calls.push(args);
        return 0;
      },
    );
    assert.deepEqual(calls[0].slice(4), ["--locale", "ja", "--help-level", "short", "App.vue"]);
    assert.deepEqual(calls[1], ["vp", "lint", "App.vue"]);
  });
});

void test("lint fix reaches both engines and native opt-outs restore Vite+ ownership", async () => {
  await project(async () => {
    const calls: string[][] = [];
    const execute = async (_: string, args: string[]) => {
      calls.push(args);
      return 0;
    };
    await runTools("lint:fix", ["script.ts"], { config: {}, options: {} }, "vp", "native", execute);
    assert.deepEqual(calls[0].slice(4), ["--fix", "script.ts"]);
    assert.deepEqual(calls[1], ["vp", "lint", "--fix", "script.ts"]);
    calls.length = 0;
    await runTools(
      "check",
      [],
      { config: {}, options: { check: false, lint: false, fmt: false } },
      "vp",
      "native",
      execute,
    );
    assert.deepEqual(calls, [
      ["vp", "lint"],
      ["vp", "fmt", "--check"],
    ]);
  });
});

void test("exceptions and interrupts remove native config and stop subsequent tools", async () => {
  await project(async () => {
    for (const interrupted of [false, true]) {
      let count = 0;
      await assert.rejects(
        runTools("check", [], { config: {}, options: {} }, "vp", "native", async () => {
          count++;
          if (interrupted) return 130;
          throw new Error("spawn failed");
        }),
        interrupted ? /interrupted/ : /spawn failed/,
      );
      assert.equal(count, 1);
      assert.ok(!readdirSync(".").some((name) => name.startsWith(".vize-vp-")));
    }
  });
});

void test("check --fix routes fixes only to lint and format and loads existing shared config", async () => {
  await project(async () => {
    writeFileSync("vize.config.json", JSON.stringify({ formatter: { singleQuote: true } }));
    const calls: string[][] = [];
    await runTools(
      "check",
      ["--fix", "App.vue"],
      { options: {} },
      "vp",
      "native",
      async (_, args) => {
        calls.push(args);
        if (args[0] === "native") {
          const config = JSON.parse(readFileSync(args[3], "utf8"));
          assert.equal(config.formatter.singleQuote, true);
        }
        return 0;
      },
    );
    assert.deepEqual(calls[0].slice(4), ["App.vue"]);
    assert.deepEqual(calls[1].slice(4), ["--fix", "App.vue"]);
    assert.deepEqual(calls[3].slice(4), ["--write", "App.vue"]);
    assert.deepEqual(calls[4], ["vp", "fmt", "--write", "App.vue"]);
  });
});

void test("typecheck is standalone or composed with lint exactly once", async () => {
  await project(async () => {
    for (const task of ["typecheck", "lint", "check"] as const) {
      const calls: string[][] = [];
      await runTools(
        task,
        [],
        { config: {}, options: {}, lintTypecheck: true },
        "vp",
        "native",
        async (_, args) => {
          calls.push(args);
          return 0;
        },
      );
      assert.equal(calls.filter((args) => args[0] === "native" && args[1] === "check").length, 1);
      if (task === "typecheck") assert.equal(calls.length, 1);
      else assert.ok(calls.some((args) => args[0] === "vp" && args[1] === "lint"));
    }
  });
});
