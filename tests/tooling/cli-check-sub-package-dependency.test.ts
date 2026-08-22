/**
 * Regression oracle for #3366: a dependency installed only in a sub-package of
 * a pnpm workspace, checked through a `tsconfig.json` at the workspace root.
 *
 * The canon mirrors checked files under `<project_root>/.vize/canon` and Corsa
 * resolves bare specifiers by walking the `node_modules` directories
 * on the ancestor chain of the *mirrored* path. That chain used to contain only
 * `<canon>/node_modules` (the Vue/Vite runtime mirror) and then, because Node's
 * algorithm skips `node_modules` path components, `<project_root>/node_modules`.
 * With pnpm's default isolated linker a dependency declared by `apps/app` is
 * linked only into `apps/app/node_modules`, which that chain never reaches, so
 * a specifier `tsc`/`tsgo` resolve cleanly reported a false `TS2307`.
 *
 * The failure mode is a *spurious* diagnostic, so the primary assertion is that
 * `TS2307` is absent. A control in the same layout imports a package that is
 * genuinely not installed, which keeps the test from passing by suppressing
 * module-resolution errors altogether.
 */
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
    "vize",
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
  return candidates.find((candidate) => fs.existsSync(candidate)) ?? null;
}

const CHECKER = resolveCheckerPath();
const corsaSkip = {
  skip: typecheckDependencySkip(
    CHECKER,
    "a corsa/tsgo checker for the sub-package dependency gate",
    "no corsa/tsgo checker in node_modules/.bin",
  ),
};

type ParsedCheck = {
  files: Array<{ file: string; diagnostics: string[] }>;
  errorCount: number;
};

const TSCONFIG = {
  compilerOptions: {
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    strict: true,
    target: "ESNext",
    types: [] as string[],
  },
  include: ["apps/**/*.ts"],
};

/**
 * A pnpm-shaped workspace: `tsconfig.json` at the root, and `acme-config`
 * installed only under `apps/app`.
 *
 * The sub-package `node_modules` is a real directory holding a symlink to the
 * package in a virtual store, which is exactly what pnpm's isolated linker
 * writes — and, because the walker does not follow links, the reason the
 * package's own declarations are never swept in as check inputs.
 */
function createWorkspace(caseName: string, extraSource: string): string {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), `vize-subpkg-${caseName}-`));
  const app = path.join(workspace, "apps/app");
  fs.mkdirSync(path.join(app, "src"), { recursive: true });
  // A real `node_modules` at the root: the canon is written to
  // `.vize/canon`, and reaching that through a link would move the
  // mirrored tree outside the project root.
  fs.mkdirSync(path.join(workspace, "node_modules"), { recursive: true });

  fs.writeFileSync(
    path.join(workspace, "package.json"),
    `${JSON.stringify({ name: "repro", private: true, type: "module" }, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(workspace, "pnpm-workspace.yaml"), "packages:\n  - apps/*\n");
  fs.writeFileSync(path.join(workspace, "tsconfig.json"), `${JSON.stringify(TSCONFIG, null, 2)}\n`);
  fs.writeFileSync(
    path.join(app, "package.json"),
    `${JSON.stringify(
      {
        name: "app",
        private: true,
        type: "module",
        devDependencies: { "acme-config": "1.0.0" },
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(path.join(app, "src/main.ts"), "export const answer: number = 42;\n");
  fs.writeFileSync(
    path.join(app, "vize.config.ts"),
    `import { defineConfig } from "acme-config";\n${extraSource}`,
  );

  const store = path.join(app, "node_modules/.pnpm/acme-config@1.0.0/node_modules/acme-config");
  fs.mkdirSync(store, { recursive: true });
  fs.writeFileSync(
    path.join(store, "package.json"),
    `${JSON.stringify(
      {
        name: "acme-config",
        version: "1.0.0",
        type: "module",
        types: "./index.d.ts",
        exports: { ".": { types: "./index.d.ts", import: "./index.js" } },
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(store, "index.d.ts"),
    "export interface AcmeConfig {\n  enabled?: boolean;\n}\n" +
      "export declare function defineConfig(config: AcmeConfig): AcmeConfig;\n",
  );
  fs.writeFileSync(
    path.join(store, "index.js"),
    "export function defineConfig(c) {\n  return c;\n}\n",
  );
  fs.symlinkSync(
    ".pnpm/acme-config@1.0.0/node_modules/acme-config",
    path.join(app, "node_modules/acme-config"),
    "dir",
  );

  return workspace;
}

function runCheck(workspace: string): ParsedCheck {
  const result = spawnSync(
    VIZE.command,
    [
      ...VIZE.prefix,
      "check",
      "--tsconfig",
      "../../tsconfig.json",
      "--corsa-path",
      CHECKER as string,
      "--format",
      "json",
    ],
    {
      cwd: path.join(workspace, "apps/app"),
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  );
  if (result.error) {
    throw result.error;
  }
  assert.ok(
    result.stdout.trim().startsWith("{"),
    `expected a JSON envelope on stdout, got:\n${result.stdout}\n${result.stderr}`,
  );
  return JSON.parse(result.stdout) as ParsedCheck;
}

function diagnosticsFor(parsed: ParsedCheck, file: string): string[] {
  const entry = parsed.files.find((candidate) => candidate.file === file);
  assert.ok(entry, `${file} must be part of the checked program: ${JSON.stringify(parsed.files)}`);
  return entry.diagnostics;
}

test(
  "a dependency installed only in a sub-package resolves under a workspace-root tsconfig",
  corsaSkip,
  () => {
    const workspace = createWorkspace(
      "resolves",
      "\nexport default defineConfig({ enabled: true });\n",
    );
    try {
      const parsed = runCheck(workspace);
      const diagnostics = diagnosticsFor(parsed, "vize.config.ts");
      assert.deepEqual(
        diagnostics.filter((entry) => entry.includes("TS2307")),
        [],
        "the sub-package dependency must resolve from the mirrored ancestor chain",
      );
      assert.equal(parsed.errorCount, 0, JSON.stringify(parsed.files));
    } finally {
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  },
);

test(
  "a module that is genuinely not installed still reports TS2307 in the same layout",
  corsaSkip,
  () => {
    const workspace = createWorkspace(
      "control",
      'import { nope } from "totally-not-installed";\n\nexport default defineConfig({ enabled: nope });\n',
    );
    try {
      const parsed = runCheck(workspace);
      const diagnostics = diagnosticsFor(parsed, "vize.config.ts");
      const unresolved = diagnostics.filter((entry) => entry.includes("TS2307"));
      assert.equal(unresolved.length, 1, JSON.stringify(diagnostics));
      assert.match(unresolved[0] as string, /Cannot find module 'totally-not-installed'/);
      assert.ok(
        !diagnostics.some((entry) => entry.includes("'acme-config'")),
        `the installed sub-package dependency must still resolve: ${JSON.stringify(diagnostics)}`,
      );
    } finally {
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  },
);
