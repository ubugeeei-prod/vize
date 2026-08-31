import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

test("Nix isolates the pinned Blacksmith CLI from the default dev shell", () => {
  const devShellModule = readRepoFile("tools/nix/dev-shell.nix");
  const blacksmithModule = readRepoFile("tools/nix/blacksmith.nix");
  const contributing = readRepoFile("docs/content/contributing.md");
  const defaultShell = devShellModule.match(
    /devShell = pkgs\.mkShell \{([\s\S]*?)\n\s*\};\n\s*testboxDevShell/,
  )?.[1];
  const testboxShell = devShellModule.match(
    /testboxDevShell = devShell\.overrideAttrs \(previous: \{([\s\S]*?)\n\s*\}\);/,
  )?.[1];
  // Every artifact the flake pins, not just Blacksmith's: a moving `latest`
  // URL anywhere would let a fixed-output derivation change under the lock.
  const sourceUrls = fs
    .readdirSync(path.join(repoRoot, "tools/nix"))
    .flatMap((entry) => [
      ...readRepoFile(`tools/nix/${entry}`).matchAll(/url = "([^"]+)";/g),
    ])
    .map((match) => match[1]);

  assert.ok(defaultShell, "default dev shell");
  assert.ok(testboxShell, "Testbox dev shell");
  assert.doesNotMatch(defaultShell, /blacksmith/i);
  assert.doesNotMatch(defaultShell, /VIZE_TESTBOX_SHELL/);
  assert.match(defaultShell, /\$\{clearTestboxEnvironment\}/);
  assert.match(testboxShell, /previous\.nativeBuildInputs/);
  assert.match(testboxShell, /pkgs\.gh/);
  assert.match(testboxShell, /pkgs\.rsync/);
  assert.match(testboxShell, /config\.packages\.blacksmith/);
  assert.match(testboxShell, /\$\{activateTestboxEnvironment\}/);
  assert.match(blacksmithModule, /unset VIZE_TESTBOX_SHELL VIZE_BLACKSMITH_BIN/);
  assert.match(blacksmithModule, /export VIZE_TESTBOX_SHELL=1/);
  assert.match(blacksmithModule, /export VIZE_BLACKSMITH_BIN="\$\{blacksmith\}\/bin\/blacksmith"/);
  assert.match(blacksmithModule, /testbox-environment = testboxEnvironmentCheck/);
  assert.match(blacksmithModule, /blacksmithVersion = "0\.4\.46";/);
  assert.ok(sourceUrls.length > 0, "fixed-output source URLs");
  for (const sourceUrl of sourceUrls) assert.doesNotMatch(sourceUrl, /\/latest\//);
  assert.match(
    blacksmithModule,
    /clireleases\.blacksmith\.sh\/cli\/v\$\{blacksmithVersion\}\/darwin\/arm64\/blacksmith/,
  );
  assert.match(
    blacksmithModule,
    /clireleases\.blacksmith\.sh\/cli\/v\$\{blacksmithVersion\}\/linux\/amd64\/blacksmith/,
  );
  assert.match(
    devShellModule,
    /devShells = \{[\s\S]*default = devShell;[\s\S]*testbox = testboxDevShell;/,
  );
  assert.match(contributing, /nix develop\s+# local development/);
  assert.match(contributing, /nix develop \.#testbox\s+# hosted Testbox workflows/);
});
