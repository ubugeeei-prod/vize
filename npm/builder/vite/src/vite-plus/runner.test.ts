import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runNative, runTools, withConfigMetadataResolution } from "./runner.ts";
import type { VizeTaskConfig } from "./types.ts";

async function project(run: () => Promise<void>) {
  const cwd = process.cwd();
  const dir = mkdtempSync(path.join(os.tmpdir(), "vize-native-tasks-"));
  try {
    process.chdir(dir);
    writeFileSync("App.vue", "<template><div /></template>\n");
    writeFileSync("script.ts", "export const value = 1\n");
    await run();
  } finally {
    process.chdir(cwd);
    rmSync(dir, { recursive: true, force: true });
  }
}

void test("config metadata resolution marks only the config load", async () => {
  const previous = process.env["VP_RESOLVING_CONFIG_METADATA"];
  delete process.env["VP_RESOLVING_CONFIG_METADATA"];
  try {
    assert.equal(
      await withConfigMetadataResolution(async () => {
        assert.equal(process.env["VP_RESOLVING_CONFIG_METADATA"], "1");
        return "loaded";
      }),
      "loaded",
    );
    assert.equal(process.env["VP_RESOLVING_CONFIG_METADATA"], undefined);

    process.env["VP_RESOLVING_CONFIG_METADATA"] = "caller";
    await assert.rejects(
      withConfigMetadataResolution(async () => {
        assert.equal(process.env["VP_RESOLVING_CONFIG_METADATA"], "1");
        throw new Error("load failed");
      }),
      { message: "load failed" },
    );
    assert.equal(process.env["VP_RESOLVING_CONFIG_METADATA"], "caller");
  } finally {
    if (previous === undefined) delete process.env["VP_RESOLVING_CONFIG_METADATA"];
    else process.env["VP_RESOLVING_CONFIG_METADATA"] = previous;
  }
});

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
      async (command, args, stdin) => {
        assert.equal(command, process.execPath);
        calls.push(args);
        if (args[0] === "vize/bin") {
          assert.ok(args.includes("--config-stdin"));
          assert.equal(JSON.parse(stdin!).vue.linter.preset, "essential");
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
    assert.deepEqual(calls[3].slice(3), ["--check", "App.vue"]);
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
    assert.deepEqual(calls[0].slice(3), ["--write", "App.vue"]);
    assert.equal(calls.length, 1);
    calls.length = 0;
    await runTools(
      "fmt:check",
      ["App.vue", "script.ts"],
      { config: {}, options: {} },
      "vp",
      "native",
      execute,
    );
    assert.deepEqual(calls[0].slice(3), ["--check", "App.vue"]);
    assert.deepEqual(calls[1], ["vp", "fmt", "--check", "script.ts"]);
  });
});

void test("formatting opt-out forwards every selected path to the host formatter", async () => {
  await project(async () => {
    for (const task of ["fmt", "fmt:check", "check"] as const) {
      for (const patterns of [[], ["App.vue"], ["**/*.vue"], ["App.vue", "script.ts"]]) {
        const calls: string[][] = [];
        const metadata: VizeTaskConfig = {
          config: { formatter: { printWidth: 120 } },
          options: { check: false, lint: false, fmt: false },
        };
        const status = await runTools(task, patterns, metadata, "vp", "native", async (_, args) => {
          calls.push(args);
          return 0;
        });
        assert.equal(status, 0);
        assert.deepEqual(calls, [
          ...(task === "check" ? [["vp", "lint", ...patterns]] : []),
          ["vp", "fmt", task === "fmt" ? "--write" : "--check", ...patterns],
        ]);
      }
    }
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
      assert.deepEqual(calls[0].slice(3), [
        task === "fmt:check" ? "--check" : "--write",
        "App.vue",
      ]);
      assert.ok(!calls.some((args) => args.includes("generated/Generated.vue")));
      calls.length = 0;
      await runTools(task, ["generated/Generated.vue"], metadata, "vp", "native", execute);
      assert.deepEqual(calls, []);
    }
  });
});

void test("vue.ignores excludes Vue files before native fmt can warn about them", async () => {
  await project(async () => {
    const calls: string[][] = [];
    const execute = async (_: string, args: string[]) => {
      calls.push(args);
      return 0;
    };
    mkdirSync("legacy");
    writeFileSync("legacy/Old.vue", "<template><div /></template>\n");
    writeFileSync("legacy/Older.vue", "<template><div /></template>\n");
    const metadata: VizeTaskConfig = {
      config: {},
      options: {},
      vueFmtIgnorePatterns: ["legacy/**"],
    };
    await runTools("fmt:check", [], metadata, "vp", "native", execute);
    assert.deepEqual(calls[0].slice(3), ["--check", "App.vue"]);
    assert.ok(!calls.some((args) => args.some((arg) => arg.includes("legacy/"))));
    calls.length = 0;
    await runTools("check", [], metadata, "vp", "native", execute);
    assert.ok(!calls.some((args) => args.some((arg) => arg.includes("legacy/"))));
  });
});

void test("lint presentation and ordered host ignores reach only the native linter", async () => {
  await project(async () => {
    const calls: string[][] = [];
    await runTools(
      "lint",
      ["App.vue"],
      {
        config: { ignores: ["src/**", "generated/**"], linter: { preset: "happy-path" } },
        options: {},
        lintLocale: "ja",
        lintHelpLevel: "short",
        lintIgnorePatterns: ["**/*.ts", "!src/**/*.ts", "generated/**"],
      },
      "vp",
      "native",
      async (_, args, stdin) => {
        if (args[0] === "native") {
          assert.deepEqual(JSON.parse(stdin!).vue.ignores, [
            path.resolve("**/*.ts").replaceAll("\\", "/"),
            `!${path.resolve("src/**/*.ts").replaceAll("\\", "/")}`,
            path.resolve("generated/**").replaceAll("\\", "/"),
            path.resolve("src/**").replaceAll("\\", "/"),
            path.resolve("generated/**").replaceAll("\\", "/"),
          ]);
        }
        calls.push(args);
        return 0;
      },
    );
    assert.deepEqual(calls[0].slice(3), ["--locale", "ja", "--help-level", "short", "App.vue"]);
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
    assert.deepEqual(calls[0].slice(3), ["--fix", "script.ts"]);
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

void test("exceptions and interrupts stop subsequent tools", async () => {
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
    }
  });
});

void test("published CLIs without the stdin marker are refused without a config file", async () => {
  await project(async () => {
    const native = path.resolve("legacy/bin/vize");
    mkdirSync(path.dirname(native), { recursive: true });
    writeFileSync(native, "legacy native placeholder\n");
    let spawned = false;
    await assert.rejects(
      runNative("check", ["App.vue"], { config: {}, options: {} }, native, async () => {
        spawned = true;
        return 0;
      }),
      {
        message: `${native} predates --config-stdin; install the vize CLI version that matches @vizejs/vite-plugin.`,
      },
    );
    assert.equal(spawned, false);
  });
});

void test("check --fix routes fixes only to lint and format and loads existing shared config", async () => {
  await project(async () => {
    writeFileSync(
      "vite.config.mjs",
      "export default { vue: { formatter: { singleQuote: true } } }",
    );
    const calls: string[][] = [];
    await runTools(
      "check",
      ["--fix", "App.vue"],
      { options: {} },
      "vp",
      "native",
      async (_, args, stdin) => {
        calls.push(args);
        if (args[0] === "native") {
          const config = JSON.parse(stdin!);
          assert.equal(config.vue.formatter.singleQuote, true);
        }
        return 0;
      },
    );
    assert.deepEqual(calls[0].slice(3), ["App.vue"]);
    assert.deepEqual(calls[1].slice(3), ["--fix", "App.vue"]);
    assert.deepEqual(calls[3].slice(3), ["--write", "App.vue"]);
    assert.equal(calls.length, 4);
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

void test("lint rejects native-only flags before invoking either checker", async () => {
  for (const task of ["lint", "lint:fix"] as const) {
    const calls: string[][] = [];
    await assert.rejects(
      runTools(
        task,
        ["--cross-file"],
        { config: {}, options: {} },
        "vp",
        "native",
        async (_command, args) => {
          calls.push(args);
          return 0;
        },
      ),
      {
        message: `Vize ${task} tasks accept paths and --fix. Put native tool options in lint.vue.`,
      },
    );
    assert.deepEqual(calls, []);
  }
});
