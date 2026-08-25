import assert from "node:assert/strict";
import { test } from "node:test";

import { isTrustedReleaseDownloadUrl } from "../../editors/vscode/src/release-download.ts";

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
