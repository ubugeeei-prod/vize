import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "../_helpers/moonbit.ts";

export type CargoMetadata = {
  packages: CargoPackage[];
};

export type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
  manifest_path: string;
  publish: string[] | null;
  version: string;
};

export type CargoDependency = { name: string; kind: string | null; req: string };

export function getScriptCrateArray(variableName: string): string[] {
  const scriptPath = path.join(repoRoot, "tools", "moon", "cmd", "publish_crates", "main.mbt");
  const script = fs.readFileSync(scriptPath, "utf8");
  const arrayBody = script.match(
    new RegExp(
      `let ${variableName}\\s*:\\s*Array\\[String\\]\\s*=\\s*\\[(?<body>[\\s\\S]*?)\\n\\]`,
      "m",
    ),
  )?.groups?.body;

  assert.ok(arrayBody, `Failed to locate ${variableName} in publish_crates`);
  return Array.from(arrayBody.matchAll(/"([^"]+)"/g), ([, crateName]) => crateName);
}

export const getPublishedCrates = () => getScriptCrateArray("published_crates");
export const getManualPublishCrates = () => getScriptCrateArray("manual_publish_crates");
export const getBlockedByManualPublishCrates = () =>
  getScriptCrateArray("blocked_by_manual_publish_crates");

export function getMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}
