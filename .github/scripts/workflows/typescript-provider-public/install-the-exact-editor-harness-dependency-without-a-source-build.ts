import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
// editors/vscode is deliberately outside the root pnpm workspace.
// Its test runner needs only this declared package, not an editor build.
const manifest = JSON.parse(fs.readFileSync("editors/vscode/package.json", "utf8"));
const version = manifest.devDependencies["@vscode/test-electron"];
assert(/^\d+\.\d+\.\d+$/.test(version));
const npm = [
  process.env.npm_execpath,
  path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js"),
  path.resolve(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
].find((file) => file && fs.existsSync(file));
assert(npm, "Managed Node must provide npm");
const temporary = fs.mkdtempSync(
  path.join(requiredEnv("RUNNER_TEMP"), "typescript-provider-harness-"),
);
fs.writeFileSync(path.join(temporary, "package.json"), '{"private":true}');
execFileSync(
  process.execPath,
  [
    npm,
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    `@vscode/test-electron@${version}`,
  ],
  { cwd: temporary, stdio: "inherit" },
);
const installed = path.join(temporary, "node_modules/@vscode/test-electron");
assert.equal(
  JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).version,
  version,
);
const scope = path.resolve("editors/vscode/node_modules/@vscode");
fs.mkdirSync(scope, { recursive: true });
fs.symlinkSync(
  installed,
  path.join(scope, "test-electron"),
  process.platform === "win32" ? "junction" : "dir",
);
fs.copyFileSync(
  path.join(temporary, "package-lock.json"),
  path.join(requiredEnv("RUNNER_TEMP"), "typescript-provider-public/harness-package-lock.json"),
);
