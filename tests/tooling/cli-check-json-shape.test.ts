import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { typecheckDependencySkip } from "./support/typecheck-dependency.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function resolveVizeCommand(): { command: string; prefix: string[] } {
  const candidates = [
    path.join(root, "target/ci/vize"),
    path.join(root, "target/release/vize"),
    path.join(root, "target/debug/vize"),
  ];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: root, encoding: "utf8" });
    if (probe.status === 0) {
      return { command: candidate, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

const VIZE = resolveVizeCommand();

function resolveCheckerPath(): string | null {
  const candidates = [
    path.join(root, "node_modules/.bin/corsa"),
    path.join(root, "node_modules/.bin/tsgo"),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

const CHECKER = resolveCheckerPath();
const checkerOptions = {
  skip: typecheckDependencySkip(
    CHECKER,
    "a corsa/tsgo checker for the CLI JSON gates",
    "no corsa/tsgo checker discoverable",
  ),
};

type CheckResult = { status: number | null; stdout: string; stderr: string };

function runCheck(args: string[], cwd: string): CheckResult {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, "check", ...args], {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

// Type-checking walks up looking for a tsconfig.json, so the workspace must live
// outside the repository tree (a repo-local temp dir would inherit the workspace
// config). os.tmpdir() keeps every fixture isolated.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-json-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

type CheckJson = {
  files: Array<{ file: string; virtualTs?: string; diagnostics: string[] }>;
  programs: Array<{
    root: string;
    tsconfig?: string;
    compilerOptions?: Record<string, unknown>;
    files: string[];
  }>;
  errorCount: number;
  warningCount: number;
  fileCount: number;
  declarations?: string[];
};

function parseJson(result: CheckResult): CheckJson {
  return JSON.parse(result.stdout) as CheckJson;
}

// A trivial canonical TS2322: `number` assigned to a `string` binding. Used only
// to exercise the JSON envelope when diagnostics are present; the cases below
// never assert the checker's message text, only the stable JSON shape.
const BAD_TS = "export const x: string = 123;";
const GOOD_TS = "export const answer = 42 as const;\n";
const PROJECT_REF_COMPILER_OPTIONS = {
  composite: true,
  module: "ESNext",
  moduleResolution: "Bundler",
  noEmit: true,
  strict: true,
  target: "ES2022",
};

test("vize check --format json has a stable top-level shape and key names", checkerOptions, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "bad.ts"), BAD_TS, "utf8");
    const result = runCheck(["bad.ts", "--format", "json", "--corsa-path", CHECKER as string], dir);
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);

    const parsed = parseJson(result);
    assert.deepEqual(
      Object.keys(parsed).sort(),
      ["errorCount", "fileCount", "files", "programs", "warningCount"],
      "top-level keys should be exactly the documented camelCase envelope",
    );
    // `--declaration` is absent, so the emitter must not surface a declarations key.
    assert.equal("declarations" in parsed, false, "no declarations key without --declaration");

    assert.ok(Array.isArray(parsed.files), "files should be an array");
    for (const entry of parsed.files) {
      assert.deepEqual(
        Object.keys(entry).sort(),
        ["diagnostics", "file"],
        "each file entry should expose file/diagnostics without virtualTs by default",
      );
      assert.equal(typeof entry.file, "string");
      assert.equal("virtualTs" in entry, false);
      assert.ok(Array.isArray(entry.diagnostics));
    }

    assert.equal(typeof parsed.errorCount, "number");
    assert.equal(typeof parsed.warningCount, "number");
    assert.equal(typeof parsed.fileCount, "number");
    assert.deepEqual(parsed.programs, [
      {
        root: ".",
        files: ["bad.ts"],
      },
    ]);
  });
});

test(
  "vize check --format json exposes declaration outputs when --declaration succeeds",
  checkerOptions,
  () => {
    withWorkspace((dir) => {
      fs.writeFileSync(path.join(dir, "good.ts"), GOOD_TS, "utf8");
      const result = runCheck(
        [
          "good.ts",
          "--declaration",
          "--declaration-dir",
          "types",
          "--format",
          "json",
          "--corsa-path",
          CHECKER as string,
        ],
        dir,
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);

      const parsed = parseJson(result);
      assert.deepEqual(parsed.declarations, ["types/good.d.ts"]);
      const declarationPath = path.join(dir, parsed.declarations[0] as string);
      assert.equal(fs.existsSync(declarationPath), true, "declaration file should exist on disk");
      const declaration = fs.readFileSync(declarationPath, "utf8");
      assert.match(declaration, /export declare const answer/u);
      assert.equal(parsed.declarations[0]?.includes("\\"), false, "path should use '/'");
    });
  },
);

test(
  "vize check --format json skips declaration outputs when type errors exist",
  checkerOptions,
  () => {
    withWorkspace((dir) => {
      fs.writeFileSync(path.join(dir, "bad.ts"), BAD_TS, "utf8");
      const result = runCheck(
        [
          "bad.ts",
          "--declaration",
          "--declaration-dir",
          "types",
          "--format",
          "json",
          "--corsa-path",
          CHECKER as string,
        ],
        dir,
      );
      assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);

      const parsed = parseJson(result);
      assert.equal("declarations" in parsed, false, "failed checks must not report d.ts outputs");
      assert.match(result.stderr, /Skipping declaration emit because type errors were reported\./u);
      assert.equal(
        fs.existsSync(path.join(dir, "types", "bad.d.ts")),
        false,
        "failed checks must not leave a declaration file",
      );
    });
  },
);

test(
  "vize check --format json reports files cwd-relative, '/'-separated and sorted",
  checkerOptions,
  () => {
    withWorkspace((dir) => {
      fs.mkdirSync(path.join(dir, "src"));
      fs.writeFileSync(
        path.join(dir, "src/Good.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ x }}</div></template>\n',
        "utf8",
      );
      fs.writeFileSync(
        path.join(dir, "src/Bad.vue"),
        '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ unclosed </div></template>\n',
        "utf8",
      );

      // Pass Good before Bad on the command line; the report must come back sorted.
      const result = runCheck(
        ["src/Good.vue", "src/Bad.vue", "--format", "json", "--corsa-path", CHECKER as string],
        dir,
      );
      const parsed = parseJson(result);

      const reported = parsed.files.map((f) => f.file);
      assert.deepEqual(
        reported,
        ["src/Bad.vue", "src/Good.vue"],
        "files should be reported cwd-relative and sorted ascending",
      );
      assert.equal(parsed.fileCount, parsed.files.length);
      assert.equal(parsed.fileCount, 2);
      for (const file of reported) {
        assert.equal(file.includes("\\"), false, `path should use '/' separators: ${file}`);
      }
    });
  },
);

test("vize check --format json reports only the requested subset of files", checkerOptions, () => {
  withWorkspace((dir) => {
    fs.mkdirSync(path.join(dir, "src"));
    fs.writeFileSync(
      path.join(dir, "src/Good.vue"),
      '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ x }}</div></template>\n',
      "utf8",
    );
    // Sibling with an unterminated interpolation: it must never appear in the
    // report when only Good.vue is checked.
    fs.writeFileSync(
      path.join(dir, "src/Bad.vue"),
      '<script setup lang="ts">\nconst x: number = 1\n</script>\n<template><div>{{ unclosed </div></template>\n',
      "utf8",
    );

    const result = runCheck(
      ["src/Good.vue", "--format", "json", "--corsa-path", CHECKER as string],
      dir,
    );
    const parsed = parseJson(result);

    assert.equal(parsed.files.length, 1, "only the explicitly requested file should be reported");
    assert.equal(parsed.files[0]?.file, "src/Good.vue");
    assert.ok(
      parsed.files.every((f) => f.file !== "src/Bad.vue"),
      "the unrequested sibling must not appear in the report",
    );
  });
});

test("vize check --format json exposes project-reference program inputs", checkerOptions, () => {
  withWorkspace((dir) => {
    fs.mkdirSync(path.join(dir, "packages/alpha/src"), { recursive: true });
    fs.mkdirSync(path.join(dir, "packages/bravo/src"), { recursive: true });
    fs.writeFileSync(
      path.join(dir, "tsconfig.json"),
      JSON.stringify({
        files: [],
        references: [{ path: "./packages/alpha" }, { path: "./packages/bravo" }],
      }),
      "utf8",
    );
    for (const name of ["alpha", "bravo"]) {
      fs.writeFileSync(
        path.join(dir, `packages/${name}/tsconfig.json`),
        JSON.stringify({
          compilerOptions: {
            composite: true,
            strict: true,
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "Bundler",
            noEmit: true,
          },
          include: ["src/**/*.ts"],
        }),
        "utf8",
      );
      fs.writeFileSync(
        path.join(dir, `packages/${name}/src/index.ts`),
        `export const ${name} = 1;\n`,
        "utf8",
      );
    }

    const result = runCheck(["--format", "json", "--corsa-path", CHECKER as string], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);

    const parsed = parseJson(result);
    assert.deepEqual(
      parsed.programs,
      [
        {
          root: "packages/alpha",
          tsconfig: "packages/alpha/tsconfig.json",
          compilerOptions: PROJECT_REF_COMPILER_OPTIONS,
          files: ["packages/alpha/src/index.ts"],
        },
        {
          root: "packages/bravo",
          tsconfig: "packages/bravo/tsconfig.json",
          compilerOptions: PROJECT_REF_COMPILER_OPTIONS,
          files: ["packages/bravo/src/index.ts"],
        },
      ],
      "project-reference runs should expose each effective program and its registered input set",
    );
    assert.deepEqual(
      parsed.files.map((file) => file.file),
      ["packages/alpha/src/index.ts", "packages/bravo/src/index.ts"],
    );
    assert.equal(parsed.fileCount, 2);
  });
});
