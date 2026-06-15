import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Resolves how to launch the `vize` CLI, mirroring the sibling CLI contract
 * tests: a prebuilt binary wins, otherwise fall back to cargo so fresh
 * checkouts still work.
 */
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

/**
 * The diagnostics below run through the Corsa/tsgo type-checker. Auto-discover a
 * real checker from node_modules/.bin; if neither is present (e.g. a checkout
 * that has not run install) the corsa-dependent tests skip gracefully rather
 * than fail, mirroring production-readiness.test.ts.
 */
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

type ParsedCheck = {
  files: Array<{ file: string; virtualTs: string; diagnostics: string[] }>;
  errorCount: number;
  warningCount: number;
  fileCount: number;
};

// `vize check --format json` writes only the JSON envelope to stdout; the
// "Building Corsa..." progress lines are emitted on stderr, so stdout parses
// cleanly without stripping.
function runJsonCheck(args: string[], cwd: string): { result: CheckResult; parsed: ParsedCheck } {
  const result = runCheck([...args, "--format", "json", "--corsa-path", CHECKER as string], cwd);
  const parsed = JSON.parse(result.stdout) as ParsedCheck;
  return { result, parsed };
}

// Workspaces live under os.tmpdir() so `vize check` never walks up into the
// repo's own tsconfig and starts type-checking the repository.
function withWorkspace<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-diag-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// Match the constant-string color reset/style sequences the text renderer emits
// so --quiet assertions compare against the underlying message text.
// eslint-disable-next-line no-control-regex
const ANSI = /\[[0-9;]*m/g;
function stripAnsi(input: string): string {
  return input.replace(ANSI, "");
}

const corsaSkip = CHECKER == null ? { skip: "no corsa/tsgo checker in node_modules/.bin" } : {};

const ANT_DESIGN_VUE_FIXTURE = path.join(root, "tests/_fixtures/_git/ant-design-vue");
const ANT_DESIGN_VUE_COMPONENTS = path.join(ANT_DESIGN_VUE_FIXTURE, "components");
const ANT_DESIGN_VUE_SEMANTIC_ERROR =
  '<script setup lang="ts">\nconst value: string = 1;\n</script>\n\n<template>\n  <div>{{ value }}</div>\n</template>\n';
const antDesignVueSkip =
  CHECKER == null
    ? corsaSkip
    : fs.existsSync(ANT_DESIGN_VUE_COMPONENTS)
      ? {}
      : { skip: "ant-design-vue fixture checkout unavailable" };

test(
  "SFC template parse errors reported with stable 1-based position and message",
  corsaSkip,
  () => {
    withWorkspace((dir) => {
      fs.writeFileSync(
        path.join(dir, "Broken.vue"),
        "<template>\n  <div>{{ unclosed\n</template>\n",
        "utf8",
      );
      const { result, parsed } = runJsonCheck(["Broken.vue"], dir);

      assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
      assert.equal(parsed.errorCount, 2);
      assert.equal(parsed.files.length, 1);

      const diagnostics = parsed.files[0]?.diagnostics ?? [];
      assert.ok(
        diagnostics.some((d) => /Template parse error: Element is missing end tag/.test(d)),
        `missing "Element is missing end tag" in ${JSON.stringify(diagnostics)}`,
      );
      assert.ok(
        diagnostics.some((d) => /Interpolation is missing its closing delimiter/.test(d)),
        `missing "Interpolation is missing its closing delimiter" in ${JSON.stringify(diagnostics)}`,
      );
      // The element-end-tag error is reported first, at line 2 column 3 (1-based).
      assert.ok(
        (diagnostics[0] ?? "").startsWith("error:2:3 "),
        `expected diagnostics[0] to start at 2:3, got ${JSON.stringify(diagnostics[0])}`,
      );
    });
  },
);

test("diagnostic line/col stays 1-based and stable under CRLF", corsaSkip, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(
      path.join(dir, "crlf.ts"),
      "export const a = 1;\r\nexport const b: string = 99;\r\n",
      "utf8",
    );
    const { result, parsed } = runJsonCheck(["crlf.ts"], dir);

    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    const diagnostics = parsed.files[0]?.diagnostics ?? [];
    assert.ok(
      diagnostics.some((d) => d.startsWith("error:2:14 ") && /TS2322/.test(d)),
      `expected a TS2322 at 2:14, got ${JSON.stringify(diagnostics)}`,
    );
  });
});

test("empty .ts type-checks cleanly with verbatim virtualTs (exit 0)", corsaSkip, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "empty.ts"), "", "utf8");
    const { result, parsed } = runJsonCheck(["empty.ts"], dir);

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(parsed.errorCount, 0);
    assert.equal(parsed.files.length, 1);
    assert.equal(parsed.files[0]?.diagnostics.length, 0);
    assert.equal(parsed.files[0]?.virtualTs, "");
  });
});

test("empty .vue type-checks cleanly with no diagnostics (exit 0)", corsaSkip, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "Empty.vue"), "", "utf8");
    const { result, parsed } = runJsonCheck(["Empty.vue"], dir);

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(parsed.fileCount, 1);
    assert.equal(parsed.files.length, 1);
    // .vue virtualTs is the generated scaffold; only assert it produces no
    // diagnostics, never its content.
    assert.equal(parsed.files[0]?.diagnostics.length, 0);
  });
});

test("TS2322 in plain .ts: exit 1 with verbatim virtualTs passthrough", corsaSkip, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(path.join(dir, "bad.ts"), "export const x: string = 123;\n", "utf8");
    const { result, parsed } = runJsonCheck(["bad.ts"], dir);

    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.equal(parsed.errorCount, 1);
    assert.equal(
      parsed.files[0]?.diagnostics[0],
      "error:1:14 [TS2322] Type 'number' is not assignable to type 'string'.",
    );
    // Plain .ts inputs are passed through to the checker verbatim.
    assert.equal(parsed.files[0]?.virtualTs, "export const x: string = 123;\n");
  });
});

test(
  "explicit ant-design-vue SFC checks report semantic TypeScript diagnostics",
  antDesignVueSkip,
  () => {
    const probeName = `__vize_semantic_probe_${process.pid}.vue`;
    const probeRelativePath = path.join("components", probeName);
    const probePath = path.join(ANT_DESIGN_VUE_FIXTURE, probeRelativePath);
    try {
      fs.writeFileSync(probePath, ANT_DESIGN_VUE_SEMANTIC_ERROR, "utf8");
      const { result, parsed } = runJsonCheck(
        [probeRelativePath, "--servers", "1"],
        ANT_DESIGN_VUE_FIXTURE,
      );

      assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
      assert.ok(parsed.errorCount > 0, result.stdout);
      const diagnostics = parsed.files[0]?.diagnostics ?? [];
      assert.ok(
        diagnostics.some((d) => d.includes("[TS2322]")),
        `expected semantic TS2322 diagnostic, got ${JSON.stringify(diagnostics)}`,
      );
    } finally {
      fs.rmSync(probePath, { force: true });
    }
  },
);

test("--quiet suppresses per-file diagnostics but keeps summary and exit code", corsaSkip, () => {
  withWorkspace((dir) => {
    fs.writeFileSync(
      path.join(dir, "Broken.vue"),
      "<template>\n  <div>{{ unclosed\n</template>\n",
      "utf8",
    );
    const result = runCheck(["Broken.vue", "--quiet", "--corsa-path", CHECKER as string], dir);

    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    // The summary renders to stdout with ANSI styling; strip it before matching.
    const text = stripAnsi(`${result.stdout}${result.stderr}`);
    assert.match(text, /Type checked 1 files/);
    assert.match(text, /error\(s\)/);
    assert.doesNotMatch(text, /Template parse error/);
  });
});
