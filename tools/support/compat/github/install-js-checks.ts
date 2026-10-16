import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { testedPackageGroups } from "../../../config/vite-plus/tested-package-groups.ts";

export function jsInstallFilters(group: string): string[] {
  const selected = testedPackageGroups.find((entry) => entry.name === group);
  const packages: readonly string[] = selected
    ? [...selected.packages, ...selected.prebuilt]
    : group === "fresco-conformance"
      ? ["./npm/fresco-native"]
      : group === "ui-conformance"
        ? ["./npm/ui"]
        : group === "original-project"
          ? ["./npm/oxlint"]
          : [];
  if (!packages.length) throw new Error(`unknown JS check group: ${group}`);
  const root = JSON.parse(readFileSync("package.json", "utf8"));
  if (typeof root.name !== "string") throw new Error("missing root package name");
  return [...new Set([root.name, "./npm/native...", ...packages.map((pkg) => `${pkg}...`)])];
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const args = [
    "install",
    "--frozen-lockfile",
    "--prefer-offline",
    ...jsInstallFilters(process.argv[2]).flatMap((filter) => ["--filter", filter]),
  ];
  const result = spawnSync("vp", args, { stdio: "inherit", shell: false });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
}
