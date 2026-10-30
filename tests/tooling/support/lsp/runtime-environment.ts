import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { stableTypeScriptRuntimePath } from "../typecheck-dependency.ts";

const explicitRuntimeVariables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];

/** Carry the checkout's locked test provider explicitly into source LSP fixtures. */
export function sourceLspRuntimeEnvironment(
  repoRoot: string,
  environment: NodeJS.ProcessEnv = process.env,
): NodeJS.ProcessEnv {
  const result = { ...environment };
  if (explicitRuntimeVariables.some((name) => environment[name])) return result;

  const name = `@typescript/typescript-${process.platform}-${process.arch}`;
  const directory = path.join(repoRoot, "node_modules", name);
  if (!fs.existsSync(directory)) return result;

  const metadata = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
  assert.equal(
    metadata.name,
    name,
    "source LSP test runtime must be the selected platform package",
  );
  assert.equal(
    typeof metadata.version,
    "string",
    "source LSP test runtime needs a package version",
  );
  const binary = stableTypeScriptRuntimePath(repoRoot);
  assert.ok(fs.statSync(binary).isFile(), "source LSP test runtime must be a regular executable");
  fs.accessSync(binary, fs.constants.X_OK);
  result.CORSA_PATH = fs.realpathSync(binary);
  return result;
}
