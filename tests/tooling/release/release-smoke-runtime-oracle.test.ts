import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

test("the Rust release command forwards all tarballs to the fresh-project oracle", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-dispatch-"));
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
      `import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
export function runFreshProjectInitChecks(context) {
  assert.deepEqual([...context.packed.keys()], ["@vizejs/other-platform", "vize"]);
  assert.deepEqual([...context.versions.values()], ["0.0.0-oracle", "0.0.0-oracle"]);
  for (const tarball of context.packed.values()) {
    assert.ok(fs.statSync(tarball).isFile());
    assert.equal(tarball, fs.realpathSync.native(tarball));
  }
  assert.equal(context.repoRoot, fs.realpathSync(process.env.VIZE_REPO_ROOT));
  assert.equal(context.installDir, process.cwd());
  assert.equal(context.tempDir, path.dirname(context.installDir));
  assert.equal(process.env.COREPACK_HOME, path.join(context.tempDir, "corepack"));
  assert.equal(process.env.COREPACK_ENABLE_DOWNLOAD_PROMPT, "0");
  for (const name of ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"]) {
    assert.equal(process.env[name], undefined);
  }
  assert.ok(fs.statSync(context.vizeBin).isFile());
  assert.deepEqual(Object.keys(context.peers).sort(), ["typescript", "vite", "vite-plus", "vue"]);
  throw new Error("fresh-project oracle reached with complete packed context");
}
`,
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
    const result = spawnSync(
      "rust-script",
      [
        "--force",
        "tools/commands/release/npm/smoke-release-install.rs",
        "--runtime-checks",
        cli,
        platform,
      ],
      {
        cwd: root,
        encoding: "utf8",
        timeout: 180_000,
        env: {
          ...process.env,
          VIZE_REPO_ROOT: fs.realpathSync(temporary),
          COREPACK_HOME: path.join(temporary, "stale-host-corepack"),
          CORSA_PATH: "/unrelated/host/corsa",
          CORSA_EXECUTABLE: "/unrelated/host/corsa",
          TSGO_PATH: "/unrelated/host/tsgo",
          TSGO_EXECUTABLE: "/unrelated/host/tsgo",
        },
      },
    );
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
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-oracle-"));
  try {
    const packageRoot = path.join(temporary, "package");
    fs.mkdirSync(path.join(packageRoot, "bin"), { recursive: true });
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({ name: "vize", version: "0.0.0-oracle", bin: { vize: "bin/vize" } }),
    );
    // This deliberately broken checker handles setup successfully, then always
    // returns a clean report. The real release entrypoint must catch that lie.
    fs.writeFileSync(
      path.join(packageRoot, "bin/vize"),
      `#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
if (args[0] === "init") {
  const project = args[1] && !args[1].startsWith("-") ? args[1] : process.cwd();
  const filename = path.join(project, "package.json");
  const manifest = JSON.parse(fs.readFileSync(filename, "utf8"));
  manifest.scripts = { "vize:check": "vize check" };
  fs.writeFileSync(filename, JSON.stringify(manifest));
  const js = !manifest.devDependencies.typescript;
  fs.writeFileSync(path.join(project, "tsconfig.json"), JSON.stringify({
    compilerOptions: js ? { allowJs: true, checkJs: true } : {},
  }));
  fs.writeFileSync(path.join(project, "vize.config.ts"), "export default {};\\n");
} else {
  console.log(JSON.stringify({ errorCount: 0, files: [] }));
}
`,
      { mode: 0o755 },
    );
    const result = spawnSync(
      "rust-script",
      [
        "--force",
        "tools/commands/release/npm/smoke-release-install.rs",
        "--runtime-checks",
        packageRoot,
      ],
      { cwd: root, encoding: "utf8", timeout: 180_000 },
    );
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
