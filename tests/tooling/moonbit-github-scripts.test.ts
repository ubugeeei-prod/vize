import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";

function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

test("github/run_many executes command groups in order", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-run-many-"));
  const outputPath = path.join(tempDir, "output.txt");

  try {
    const result = runMoonScript(
      "github/run_many",
      [
        "node",
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'a')`,
        "--",
        "node",
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'b')`,
      ],
      { cwd: tempDir },
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(outputPath, "utf8"), "ab");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/clean_node_binaries removes only top-level .node files", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-clean-node-"));
  const firstDir = path.join(tempDir, "first");
  const secondDir = path.join(tempDir, "second");

  try {
    fs.mkdirSync(firstDir, { recursive: true });
    fs.mkdirSync(secondDir, { recursive: true });
    writeFileSync(path.join(firstDir, "native.node"), "native");
    writeFileSync(path.join(firstDir, "keep.txt"), "keep");
    writeFileSync(path.join(secondDir, "addon.node"), "addon");

    const result = runMoonScript("github/clean_node_binaries", [firstDir, secondDir], {
      cwd: tempDir,
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.existsSync(path.join(firstDir, "native.node")), false);
    assert.equal(fs.existsSync(path.join(secondDir, "addon.node")), false);
    assert.equal(fs.existsSync(path.join(firstDir, "keep.txt")), true);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/collect_native_artifacts copies .node files and skips node_modules", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-collect-node-"));
  const sourceDir = path.join(tempDir, "source");
  const outputDir = path.join(tempDir, "out");

  try {
    fs.mkdirSync(path.join(sourceDir, "nested"), { recursive: true });
    fs.mkdirSync(path.join(sourceDir, "node_modules", "ignored"), { recursive: true });
    writeFileSync(path.join(sourceDir, "nested", "first.node"), "first");
    writeFileSync(path.join(sourceDir, "node_modules", "ignored", "skip.node"), "skip");

    const result = runMoonScript(
      "github/collect_native_artifacts",
      [sourceDir, outputDir, "example"],
      {
        cwd: tempDir,
      },
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(path.join(outputDir, "first.node"), "utf8"), "first");
    assert.equal(fs.existsSync(path.join(outputDir, "skip.node")), false);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/create_site_structure assembles the Pages output tree", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-site-"));

  try {
    fs.mkdirSync(path.join(tempDir, "artifacts", "docs"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "artifacts", "playground"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "artifacts", "musea-examples"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "playground", "public"), { recursive: true });

    writeFileSync(path.join(tempDir, "artifacts", "docs", "index.html"), "docs");
    writeFileSync(path.join(tempDir, "artifacts", "playground", "app.js"), "play");
    writeFileSync(path.join(tempDir, "artifacts", "musea-examples", "index.html"), "musea");
    writeFileSync(path.join(tempDir, "playground", "public", "CNAME"), "example.com");

    const result = runMoonScript("github/create_site_structure", [], { cwd: tempDir });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(path.join(tempDir, "site", "index.html"), "utf8"), "docs");
    assert.equal(fs.readFileSync(path.join(tempDir, "site", "play", "app.js"), "utf8"), "play");
    assert.equal(
      fs.readFileSync(path.join(tempDir, "site", "musea-examples", "index.html"), "utf8"),
      "musea",
    );
    assert.equal(fs.readFileSync(path.join(tempDir, "site", "CNAME"), "utf8"), "example.com");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/install_playwright_browsers uses the playground-local Playwright CLI", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-playwright-install-"));
  const binDir = path.join(tempDir, "bin");
  const argsPath = path.join(tempDir, "vp-args.txt");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "vp",
      [
        "require('node:fs').writeFileSync(",
        `  ${JSON.stringify(argsPath)},`,
        "  process.argv.slice(2).join('\\n'),",
        ");",
      ].join("\n"),
    );

    const result = runMoonScript("github/install_playwright_browsers", [], {
      cwd: tempDir,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(
      fs.readFileSync(argsPath, "utf8"),
      [
        "exec",
        "--filter",
        "./playground",
        "--",
        "playwright",
        "install",
        "chromium",
        "--with-deps",
      ].join("\n"),
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/write_coverage_summary appends the tail of coverage output", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-coverage-summary-"));
  const binDir = path.join(tempDir, "bin");
  const summaryPath = path.join(tempDir, "step-summary.md");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const args = process.argv.slice(2);",
        "if (args.join(' ') !== 'run -p vize_test_runner --bin coverage --release') process.exit(99);",
        "process.stdout.write([",
        "  'Coverage report',",
        "  'line-1',",
        "  'line-2',",
        "  'line-3',",
        "  'line-4',",
        "  'line-5',",
        "  'line-6',",
        "  'line-7',",
        "  'line-8',",
        "].join('\\n'));",
      ].join("\n"),
    );
    writeFileSync(summaryPath, "existing\n");

    const result = runMoonScript("github/write_coverage_summary", [], {
      cwd: tempDir,
      env: {
        GITHUB_STEP_SUMMARY: summaryPath,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(
      fs.readFileSync(path.join(tempDir, "coverage-report.txt"), "utf8"),
      [
        "Coverage report",
        "line-1",
        "line-2",
        "line-3",
        "line-4",
        "line-5",
        "line-6",
        "line-7",
        "line-8",
      ].join("\n"),
    );
    assert.equal(
      fs.readFileSync(summaryPath, "utf8"),
      [
        "existing",
        "### Coverage Summary",
        "line-2",
        "line-3",
        "line-4",
        "line-5",
        "line-6",
        "line-7",
        "line-8",
      ].join("\n") + "\n",
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
