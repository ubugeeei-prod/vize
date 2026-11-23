import { readToolingProgram } from "../support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const command = process.execPath;
const entry = "tools/commands/release/npm/smoke-release-install.ts";

test("the release command forwards all tarballs to the fresh-project oracle", () => {
  const temporary = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-dispatch-")),
  );
  try {
    const helpers = path.join(temporary, "tools/support/compat/npm");
    fs.mkdirSync(helpers, { recursive: true });
    fs.writeFileSync(path.join(temporary, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(temporary, "pnpm-workspace.yaml"), "packages: []\n");
    fs.writeFileSync(
      path.join(helpers, "smoke-release-init-typecheck.mjs"),
      "export function runInitTypecheckChecks() {}\n",
    );
    fs.writeFileSync(
      path.join(helpers, "smoke-release-init-fresh.mjs"),
      readToolingProgram("commands/release-smoke-runtime-oracle/verify-fresh-init.js.in"),
    );
    const cli = path.join(temporary, "cli");
    const platform = path.join(temporary, "platform");
    fs.mkdirSync(cli);
    fs.mkdirSync(platform);
    fs.writeFileSync(
      path.join(cli, "package.json"),
      JSON.stringify({ name: "vize", version: "0.0.0-oracle", bin: { vize: "vize.cjs" } }),
    );
    fs.writeFileSync(path.join(cli, "vize.cjs"), "#!/usr/bin/env node\n", { mode: 0o755 });
    fs.writeFileSync(
      path.join(platform, "package.json"),
      JSON.stringify({
        name: "@vizejs/other-platform",
        version: "0.0.0-oracle",
        os: [process.platform === "win32" ? "linux" : "win32"],
      }),
    );
    const result = spawnSync(command, [entry, "--runtime-checks", cli, platform], {
      cwd: root,
      encoding: "utf8",
      timeout: 180_000,
      env: {
        ...process.env,
        VIZE_REPO_ROOT: fs.realpathSync.native(temporary),
        COREPACK_HOME: path.join(temporary, "stale-host-corepack"),
        CORSA_PATH: "/unrelated/host/corsa",
        CORSA_EXECUTABLE: "/unrelated/host/corsa",
        TSGO_PATH: "/unrelated/host/tsgo",
        TSGO_EXECUTABLE: "/unrelated/host/tsgo",
        TSRS_PATH: "/unrelated/host/tsrs",
        TSRS_EXECUTABLE: "/unrelated/host/tsrs",
      },
    });
    assert.ifError(result.error);
    assert.notEqual(result.status, 0);
    assert.match(
      `${result.stdout}\n${result.stderr}`,
      /fresh-project oracle reached with complete packed context/u,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("the release command rejects a packed CLI that accepts every broken project", () => {
  const temporary = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-oracle-")),
  );
  try {
    const packageRoot = path.join(temporary, "package");
    fs.mkdirSync(path.join(packageRoot, "bin"), { recursive: true });
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({ name: "vize", version: "0.0.0-oracle", bin: { vize: "bin/vize" } }),
    );
    // This deliberately broken checker handles setup successfully, then always
    // returns a clean report. The real release entrypoint must catch that lie.
    const historicalFalseCleanCli = readToolingProgram(
      "commands/release-smoke-runtime-oracle/historical-false-clean-cli.cjs.in",
    );
    const configFreeFalseCleanCli = historicalFalseCleanCli.replace(
      '  fs.writeFileSync(path.join(project, "vize.config.ts"), "export default {};\\n");\n',
      "",
    );
    fs.writeFileSync(path.join(packageRoot, "bin/vize"), configFreeFalseCleanCli, { mode: 0o755 });
    const result = spawnSync(command, [entry, "--runtime-checks", packageRoot], {
      cwd: root,
      encoding: "utf8",
      timeout: 180_000,
    });
    assert.ifError(result.error);
    assert.notEqual(
      result.status,
      0,
      `false-clean packed CLI passed the release gate\n${result.stdout}`,
    );
    assert.match(
      `${result.stdout}\n${result.stderr}`,
      /typescript standalone source unexpectedly passed/u,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
