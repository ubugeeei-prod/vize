import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import { isTrustedReleaseDownloadUrl } from "../../editors/vscode/src/release-download.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("vscode release downloads stay on GitHub HTTPS hosts", () => {
  assert.equal(
    isTrustedReleaseDownloadUrl(
      "https://github.com/ubugeeei-prod/vize/releases/download/v0.387.0/vize-x86_64-unknown-linux-gnu.tar.gz",
    ),
    true,
  );
  assert.equal(
    isTrustedReleaseDownloadUrl(
      "https://objects.githubusercontent.com/github-production-release-asset-2e65be/archive.tar.gz",
    ),
    true,
  );
  assert.equal(
    isTrustedReleaseDownloadUrl("https://release-assets.githubusercontent.com/vize.tar.gz"),
    true,
  );
  assert.equal(
    isTrustedReleaseDownloadUrl("http://github.com/ubugeeei-prod/vize/vize.tar.gz"),
    false,
  );
  assert.equal(isTrustedReleaseDownloadUrl("https://evil.example/vize.tar.gz"), false);
  assert.equal(isTrustedReleaseDownloadUrl("https://github.com.evil.example/vize.tar.gz"), false);
  assert.equal(isTrustedReleaseDownloadUrl("https://attacker@github.com/vize.tar.gz"), false);
  assert.equal(isTrustedReleaseDownloadUrl("https://github.com:8443/vize.tar.gz"), false);
});

test("vscode release download source imports without package module warnings", () => {
  const source = pathToFileURL(path.join(root, "editors/vscode/src/release-download.ts")).href;
  const result = spawnSync(
    process.execPath,
    ["--input-type=module", "--eval", `await import(${JSON.stringify(source)});`],
    { cwd: root, encoding: "utf8" },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.doesNotMatch(result.stderr, /MODULE_TYPELESS_PACKAGE_JSON|Reparsing as ES module/);
});
