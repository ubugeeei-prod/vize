import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import {
  pathWith,
  readRecordedCalls,
  runToolCommand,
  writeRecordingCommand,
} from "./_helpers/tool-commands.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("github/collect-native-artifacts copies .node files and skips node_modules", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "github-collect-node-"));
  const sourceDir = path.join(tempDir, "source");
  const outputDir = path.join(tempDir, "out");

  try {
    fs.mkdirSync(path.join(sourceDir, "nested"), { recursive: true });
    fs.mkdirSync(path.join(sourceDir, "node_modules", "ignored"), { recursive: true });
    fs.mkdirSync(outputDir, { recursive: true });
    writeFileSync(path.join(sourceDir, "nested", "first.node"), "first");
    writeFileSync(path.join(sourceDir, "node_modules", "ignored", "skip.node"), "skip");
    writeFileSync(path.join(outputDir, "keep.txt"), "keep");

    const result = runToolCommand(
      "tools/support/github/collect-native-artifacts.ts",
      [sourceDir, outputDir, "example"],
      {
        cwd: tempDir,
      },
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(path.join(outputDir, "first.node"), "utf8"), "first");
    assert.equal(fs.existsSync(path.join(outputDir, "skip.node")), false);
    assert.equal(fs.readFileSync(path.join(outputDir, "keep.txt"), "utf8"), "keep");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/create-site-structure assembles the Pages output tree", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "github-site-"));

  try {
    fs.mkdirSync(path.join(tempDir, "artifacts", "docs"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "artifacts", "playground"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "artifacts", "musea-examples"), { recursive: true });
    fs.mkdirSync(path.join(tempDir, "playground", "public"), { recursive: true });

    writeFileSync(path.join(tempDir, "artifacts", "docs", "index.html"), "docs");
    writeFileSync(path.join(tempDir, "artifacts", "playground", "app.js"), "play");
    writeFileSync(path.join(tempDir, "artifacts", "musea-examples", "index.html"), "musea");
    writeFileSync(path.join(tempDir, "playground", "public", "CNAME"), "example.com");

    const result = runToolCommand("tools/support/github/create-site-structure.ts", [], {
      cwd: tempDir,
    });

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

test("github/install-playwright-browsers resolves Playwright from its installed owner", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "github-playwright-install-"));
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

    for (const workspace of ["", "./tests"]) {
      const result = runToolCommand("tools/support/github/install-playwright-browsers.ts", [], {
        cwd: tempDir,
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          PLAYWRIGHT_WORKSPACE: workspace,
        },
      });

      assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
      assert.equal(
        fs.readFileSync(argsPath, "utf8"),
        [
          "exec",
          "--filter",
          workspace || "./playground",
          "--",
          "playwright",
          "install",
          "chromium",
          "--with-deps",
        ].join("\n"),
      );
    }
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/build-napi-package builds Apple targets with cargo and writes the expected .node artifact", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "github-build-napi-apple-"));
  const binDir = path.join(tempDir, "bin");
  const packageDir = path.join(tempDir, "npm", "native");
  const artifactPath = path.join(
    tempDir,
    "target",
    "aarch64-apple-darwin",
    "release",
    "libvize_vitrine.dylib",
  );
  const commandArgsPath = path.join(tempDir, "cargo-args.txt");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    fs.mkdirSync(packageDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        `const argsPath = ${JSON.stringify(commandArgsPath)};`,
        `const artifactPath = ${JSON.stringify(artifactPath)};`,
        "fs.writeFileSync(argsPath, process.argv.slice(2).join('\\n'));",
        "fs.mkdirSync(path.dirname(artifactPath), { recursive: true });",
        "fs.writeFileSync(artifactPath, 'apple-native');",
      ].join("\n"),
    );

    const result = runToolCommand(
      "tools/support/github/build-napi-package.ts",
      [packageDir, "../../crates/vize_vitrine/Cargo.toml", "vize_vitrine", "aarch64-apple-darwin"],
      {
        cwd: tempDir,
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        },
      },
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(
      fs.readFileSync(commandArgsPath, "utf8"),
      [
        "build",
        "--release",
        "--manifest-path",
        "../../crates/vize_vitrine/Cargo.toml",
        "-p",
        "vize_vitrine",
        "--features",
        "napi,legacy",
        "--target",
        "aarch64-apple-darwin",
      ].join("\n"),
    );
    assert.equal(
      fs.readFileSync(path.join(packageDir, "vize-vitrine.darwin-arm64.node"), "utf8"),
      "apple-native",
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

void test("github/vp-install appends caller filters to the frozen offline install", (t) => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "github-vp-install-"));
  t.after(() => rmSync(tempDir, { recursive: true, force: true }));
  const log = path.join(tempDir, "calls.log");
  writeRecordingCommand(path.join(tempDir, "bin"), "vp", log, { exitCodes: [13] });

  const result = runToolCommand(
    "tools/support/github/vp-install.ts",
    ["--filter", "./playground...", "--filter", "./npm/ui..."],
    { cwd: tempDir, env: { PATH: pathWith(path.join(tempDir, "bin")) } },
  );

  assert.equal(result.status, 13);
  assert.deepEqual(
    readRecordedCalls(log).map(({ argv, cwd }) => ({ argv, cwd })),
    [
      {
        argv: [
          "install",
          "--frozen-lockfile",
          "--prefer-offline",
          "--filter",
          "./playground...",
          "--filter",
          "./npm/ui...",
        ],
        cwd: fs.realpathSync(tempDir),
      },
    ],
  );
});

void test("github/install-cross-compile-tools stops after a failed apt update", (t) => {
  for (const [exitCodes, status, calls] of [
    [[0], 0, 2],
    [[100], 100, 1],
    [[0, 4], 4, 2],
  ] as const) {
    const tempDir = mkdtempSync(path.join(tmpdir(), "github-cross-tools-"));
    t.after(() => rmSync(tempDir, { recursive: true, force: true }));
    const log = path.join(tempDir, "calls.log");
    writeRecordingCommand(path.join(tempDir, "bin"), "sudo", log, { exitCodes });

    const result = runToolCommand("tools/support/github/install-cross-compile-tools.ts", [], {
      cwd: tempDir,
      env: { PATH: pathWith(path.join(tempDir, "bin")) },
    });

    assert.equal(result.status, status);
    assert.deepEqual(
      readRecordedCalls(log).map(({ argv }) => argv),
      [
        ["apt-get", "update"],
        ["apt-get", "install", "-y", "gcc-aarch64-linux-gnu"],
      ].slice(0, calls),
    );
  }
});

void test("github/create-cli-archive archives from the target release directory", (t) => {
  const tempDir = fs.realpathSync(mkdtempSync(path.join(tmpdir(), "github-cli-archive-")));
  t.after(() => rmSync(tempDir, { recursive: true, force: true }));
  const bin = path.join(tempDir, "bin");
  const log = path.join(tempDir, "calls.log");
  writeRecordingCommand(bin, "tar", log, { exitCodes: [2] });
  writeRecordingCommand(bin, "7z", log);
  const releaseDir = path.join(tempDir, "target", "x86_64-pc-windows-msvc", "release");
  fs.mkdirSync(releaseDir, { recursive: true });
  const run = (archive: string, executable: string) =>
    runToolCommand(
      "tools/support/github/create-cli-archive.ts",
      ["x86_64-pc-windows-msvc", archive, executable],
      { cwd: tempDir, env: { PATH: pathWith(bin) } },
    );

  assert.equal(run("vize-x86_64-pc-windows-msvc.zip", "vize.exe").status, 0);
  assert.equal(run("dist/vize.tar.gz", "vize").status, 2);
  assert.equal(run("vize.zip", "vize.exe extra").status, 0);
  const usage = runToolCommand("tools/support/github/create-cli-archive.ts", ["a", "b"]);
  assert.equal(usage.status, 1);
  assert.match(
    usage.stdout,
    /^Usage: .*create-cli-archive\.ts <target> <archive> <executable>\n$/u,
  );
  assert.deepEqual(
    readRecordedCalls(log).map(({ name, argv, cwd }) => ({ name, argv, cwd })),
    [
      {
        name: "7z",
        argv: ["a", path.join(tempDir, "vize-x86_64-pc-windows-msvc.zip"), "vize.exe"],
        cwd: releaseDir,
      },
      {
        name: "tar",
        argv: ["-czvf", path.join(tempDir, "dist", "vize.tar.gz"), "vize"],
        cwd: releaseDir,
      },
      {
        name: "7z",
        argv: ["a", path.join(tempDir, "vize.zip"), "vize.exe extra"],
        cwd: releaseDir,
      },
    ],
  );
});

void test("github/commit-vrt-snapshots commits and pushes only staged snapshot changes", (t) => {
  const identity = [
    ["config", "user.name", "github-actions[bot]"],
    ["config", "user.email", "github-actions[bot]@users.noreply.github.com"],
    ["add", "playground/e2e/vrt/__snapshots__/"],
  ];
  for (const { exitCodes, status, stdout, calls } of [
    { exitCodes: [0], status: 0, stdout: "No snapshot changes\n", calls: 4 },
    { exitCodes: [0, 0, 0, 1, 0, 5], status: 5, stdout: "", calls: 6 },
    { exitCodes: [0, 0, 0, 128], status: 128, stdout: "", calls: 4 },
    { exitCodes: [0, 0, 0, 1, 9], status: 1, stdout: "", calls: 5 },
    { exitCodes: [3], status: 1, stdout: "", calls: 1 },
    { exitCodes: [0, 3], status: 1, stdout: "", calls: 2 },
    { exitCodes: [0, 0, 3], status: 1, stdout: "", calls: 3 },
  ]) {
    const tempDir = mkdtempSync(path.join(tmpdir(), "github-vrt-snapshots-"));
    t.after(() => rmSync(tempDir, { recursive: true, force: true }));
    const log = path.join(tempDir, "calls.log");
    writeRecordingCommand(path.join(tempDir, "bin"), "git", log, { exitCodes });

    const result = runToolCommand("tools/support/github/commit-vrt-snapshots.ts", ["feature/vrt"], {
      cwd: tempDir,
      env: { PATH: pathWith(path.join(tempDir, "bin")) },
    });

    assert.equal(result.status, status);
    assert.equal(result.stdout, stdout);
    assert.deepEqual(
      readRecordedCalls(log).map(({ argv }) => argv),
      [
        ...identity,
        ["diff", "--cached", "--quiet"],
        ["commit", "-m", "chore: update VRT snapshots"],
        ["push", "origin", "HEAD:feature/vrt"],
      ].slice(0, calls),
    );
  }
  const usage = runToolCommand("tools/support/github/commit-vrt-snapshots.ts");
  assert.equal(usage.status, 1);
  assert.match(usage.stdout, /^Usage: .*commit-vrt-snapshots\.ts <branch>\n$/u);
});
