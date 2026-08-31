import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type CompositeAction = {
  runs?: {
    steps?: Array<{
      name?: string;
      run?: string;
    }>;
  };
};

function setupRustScriptInstaller(): string {
  const action = parse(
    readRepoFile(".github", "actions", "setup-rust-script", "action.yml"),
  ) as CompositeAction;
  const script = action.runs?.steps?.find((step) => step.name === "Install rust-script")?.run;
  assert.ok(script, "setup-rust-script action must define an install step");
  return script;
}

function writeExecutable(file: string, contents: string): void {
  fs.writeFileSync(file, contents);
  fs.chmodSync(file, 0o755);
}

function readIfExists(file: string): string {
  return fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "";
}

function runInstaller(initialVersion: string | null) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-setup-rust-script-"));
  const fakeBin = path.join(root, "bin");
  const cargoHome = path.join(root, "cargo-home");
  const cargoBin = path.join(cargoHome, "bin");
  const cargoLog = path.join(root, "cargo.log");
  const githubPath = path.join(root, "github-path");
  const versionState = path.join(root, "rust-script-version");

  fs.mkdirSync(fakeBin);
  fs.mkdirSync(cargoBin, { recursive: true });
  if (initialVersion) {
    fs.writeFileSync(versionState, initialVersion);
    writeExecutable(
      path.join(fakeBin, "rust-script"),
      `#!/usr/bin/env bash
set -euo pipefail
if [ "\${1:-}" = "--version" ]; then
  printf 'rust-script %s\\n' "$(cat "$VERSION_STATE")"
  exit 0
fi
exit 64
`,
    );
  }
  writeExecutable(
    path.join(fakeBin, "cargo"),
    `#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >> "$CARGO_LOG"
if [ "\${1:-}" != "install" ] || [ "\${2:-}" != "rust-script" ]; then
  exit 65
fi
printf '0.34.0' > "$VERSION_STATE"
mkdir -p "$CARGO_HOME/bin"
cat > "$CARGO_HOME/bin/rust-script" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ "\${1:-}" = "--version" ]; then
  printf 'rust-script %s\\n' "$(cat "$VERSION_STATE")"
  exit 0
fi
exit 64
SH
chmod +x "$CARGO_HOME/bin/rust-script"
`,
  );

  const result = spawnSync("/bin/bash", ["-c", setupRustScriptInstaller()], {
    encoding: "utf8",
    env: {
      PATH: `${fakeBin}:/usr/bin:/bin`,
      CARGO_HOME: cargoHome,
      CARGO_LOG: cargoLog,
      GITHUB_PATH: githubPath,
      VERSION_STATE: versionState,
    },
  });

  const output = {
    cargoBin,
    cargoLog: readIfExists(cargoLog),
    githubPath: readIfExists(githubPath),
    result,
  };
  fs.rmSync(root, { force: true, recursive: true });
  return output;
}

test("setup-rust-script reuses an already pinned runner", () => {
  const { cargoLog, githubPath, result } = runInstaller("0.34.0");

  assert.equal(result.status, 0, result.stderr);
  assert.equal(cargoLog, "");
  assert.equal(githubPath, "");
  assert.match(result.stdout, /^rust-script 0\.34\.0$/m);
});

test("setup-rust-script replaces a mismatched runner with the pinned version", () => {
  const { cargoBin, cargoLog, githubPath, result } = runInstaller("0.33.0");

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /Found rust-script 0\.33\.0; installing 0\.34\.0/);
  assert.equal(cargoLog, "install rust-script --version 0.34.0 --locked --force\n");
  assert.equal(githubPath, `${cargoBin}\n`);
  assert.match(result.stdout, /^rust-script 0\.34\.0$/m);
});

test("setup-rust-script installs the pinned runner when none exists", () => {
  const { cargoLog, result } = runInstaller(null);

  assert.equal(result.status, 0, result.stderr);
  assert.equal(cargoLog, "install rust-script --version 0.34.0 --locked --force\n");
  assert.match(result.stdout, /^rust-script 0\.34\.0$/m);
});
