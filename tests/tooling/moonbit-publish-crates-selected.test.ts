import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

function workspaceVersion(): string {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as { packages: Array<{ name: string; version: string }> };
  return metadata.packages.find((pkg) => pkg.name === "vize_atelier_jsx")?.version ?? "";
}

test("publish_crates can target the JSX and Patina handoff set", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-selected-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");
  const curlLogPath = path.join(tempDir, "curl.log");
  const version = workspaceVersion();
  assert.ok(version);

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
        "if (['package', 'publish', 'info'].includes(args[0])) process.exit(0);",
        "process.exit(1);",
      ].join("\n"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "if (process.env.CURL_LOG) fs.appendFileSync(process.env.CURL_LOG, args.join(' ') + '\\n');",
        "const endpoint = args.at(-1).split('/');",
        "const crateName = endpoint.at(-2);",
        "const version = endpoint.at(-1);",
        "const published = (process.env.TEST_PUBLISHED_CRATES || '').split(',').includes(crateName);",
        "const body = published ? { version: { crate: crateName, num: version } } : { errors: [{ detail: 'Not Found' }] };",
        "fs.writeSync(1, JSON.stringify(body) + 'VIZE_HTTP_STATUS:' + (published ? '200' : '404'));",
        "process.exit(published ? 0 : 22);",
      ].join("\n"),
    );

    const env = {
      PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      CARGO_LOG: cargoLogPath,
      PUBLISH_RETRY_LIMIT: "1",
      PUBLISH_RETRY_DELAY: "1",
    };
    const selectedPublish = runMoonScript(
      "publish_crates",
      ["--crate", "vize_atelier_jsx", "--crate", "vize_patina"],
      { cwd: repoRoot, env },
    );
    assert.equal(selectedPublish.status, 0, selectedPublish.stderr);
    assert.match(
      selectedPublish.stdout,
      /Selected crate publish plan: vize_atelier_jsx, vize_patina/,
    );
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      "publish --locked -p vize_atelier_jsx",
      `info --registry crates-io vize_atelier_jsx@${version}`,
      "publish --locked -p vize_patina",
      `info --registry crates-io vize_patina@${version}`,
    ]);

    fs.writeFileSync(cargoLogPath, "");
    fs.writeFileSync(curlLogPath, "");
    const selectedDryRun = runMoonScript(
      "publish_crates",
      ["--dry-run", "--crate", "vize_atelier_jsx", "--crate", "vize_patina"],
      {
        cwd: repoRoot,
        env: {
          ...env,
          CURL_LOG: curlLogPath,
          TEST_PUBLISHED_CRATES: "vize_atelier_jsx",
        },
      },
    );
    assert.equal(selectedDryRun.status, 0, selectedDryRun.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      ["package", "--locked", "--no-verify", "-p", "vize_atelier_jsx", "-p", "vize_patina"].join(
        " ",
      ),
      `info --registry crates-io vize_atelier_jsx@${version}`,
      "publish --dry-run --locked -p vize_patina",
    ]);
    assert.match(selectedDryRun.stdout, /registry-resolvable frontier vize_patina/i);

    for (const invalidArgs of [
      ["--crate"],
      ["--crate", "unknown_crate"],
      ["--crate", "vize_patina", "--crate", "vize_patina"],
    ]) {
      fs.writeFileSync(cargoLogPath, "");
      const invalid = runMoonScript("publish_crates", invalidArgs, {
        cwd: repoRoot,
        env,
      });
      assert.notEqual(invalid.status, 0);
      assert.match(invalid.stderr, /Usage: .*publish_crates.*--crate <crate>/);
      assert.equal(fs.readFileSync(cargoLogPath, "utf8"), "");
    }
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
