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
  const recipe = (name: string) =>
    fs.readFileSync(path.join(repoRoot, "tools/nix/scripts", name + ".sh"), "utf8");
  assert.match(
    blacksmithModule,
    /clearTestboxEnvironment = builtins\.readFile \.\/scripts\/clear-testbox\.sh/u,
  );
  assert.match(
    blacksmithModule,
    /renderScript \.\/scripts\/set-testbox\.sh[\s\S]*?BLACKSMITH = "\$\{blacksmith\}"/u,
  );
  assert.match(recipe("activate-workspace"), /@CLEAR_TESTBOX@/u);
  assert.match(recipe("activate-testbox"), /@ACTIVATE_TESTBOX@/u);
  const defaultShell = devShellModule.match(
    /devShell = pkgs\.mkShell \{([\s\S]*?)\n\s*\};\n\s*testboxDevShell/,
  )?.[1];
  const testboxShell = devShellModule.match(
    /testboxDevShell = devShell\.overrideAttrs \(previous: \{([\s\S]*?)\n\s*\}\);/,
  )?.[1];
  // Every artifact the flake pins, not just Blacksmith's: a moving `latest`
  // URL anywhere would let a fixed-output derivation change under the lock.
  // Subflakes (tools/nix/creusot) included.
  const sourceUrls = fs
    .readdirSync(path.join(repoRoot, "tools/nix"), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".nix"))
    .flatMap((entry) => [
      ...fs
        .readFileSync(path.join(entry.parentPath, entry.name), "utf8")
        .matchAll(/url = "([^"]+)";/g),
    ])
    .map((match) => match[1]);

  assert.ok(defaultShell, "default dev shell");
  assert.ok(testboxShell, "Testbox dev shell");
  assert.doesNotMatch(defaultShell, /blacksmith/i);
  assert.doesNotMatch(defaultShell, /VIZE_TESTBOX_SHELL/);
  assert.match(defaultShell, /CLEAR_TESTBOX = clearTestboxEnvironment/u);
  assert.match(testboxShell, /previous\.nativeBuildInputs/);
  assert.match(testboxShell, /pkgs\.gh/);
  assert.match(testboxShell, /pkgs\.rsync/);
  assert.match(testboxShell, /config\.packages\.blacksmith/);
  assert.match(testboxShell, /ACTIVATE_TESTBOX = activateTestboxEnvironment/u);
  assert.match(recipe("clear-testbox"), /unset VIZE_TESTBOX_SHELL VIZE_BLACKSMITH_BIN/);
  assert.match(recipe("set-testbox"), /export VIZE_TESTBOX_SHELL=1/);
  assert.match(
    recipe("set-testbox"),
    /export VIZE_BLACKSMITH_BIN="@BLACKSMITH@\/bin\/blacksmith"/u,
  );
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
