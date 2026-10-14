import { execFileSync } from "node:child_process";

import {
  blockedByManualPublishCrates,
  manualPublishCrates,
  publishedCrates,
} from "../../../tools/support/release/publish_crates/main.ts";
import { repoRoot } from "./release-command.ts";

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

export const getPublishedCrates = (): string[] => [...publishedCrates];
export const getManualPublishCrates = (): string[] => [...manualPublishCrates];
export const getBlockedByManualPublishCrates = (): string[] => [...blockedByManualPublishCrates];

export function getMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}
