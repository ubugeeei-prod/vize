import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const currentDir = path.dirname(fileURLToPath(import.meta.url));

export const packageRoot = path.resolve(currentDir, "..", "..");
export const workspaceRoot = path.resolve(packageRoot, "..", "..", "..");
const outputRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "test-output",
  "rspack-vize-plugin",
);
const snapshotRoots = Array.from(new Set([workspaceRoot, fs.realpathSync(workspaceRoot)])).map(
  (root) => root.replaceAll("\\", "/"),
);

type PackageLoaderAlias =
  | "@vizejs/rspack-plugin/loader"
  | "@vizejs/rspack-plugin/jsx-loader"
  | "@vizejs/rspack-plugin/scope-loader"
  | "@vizejs/rspack-plugin/style-loader";

export const packageLoaderAliases = {
  "@vizejs/rspack-plugin/loader": path.join(packageRoot, "dist", "loader", "index.mjs"),
  "@vizejs/rspack-plugin/jsx-loader": path.join(packageRoot, "dist", "loader", "jsx-loader.mjs"),
  "@vizejs/rspack-plugin/scope-loader": path.join(
    packageRoot,
    "dist",
    "loader",
    "scope-loader.mjs",
  ),
  "@vizejs/rspack-plugin/style-loader": path.join(
    packageRoot,
    "dist",
    "loader",
    "style-loader.mjs",
  ),
} satisfies Record<PackageLoaderAlias, string>;

export function resolveFixturePath(name: string, file: string): string {
  return path.join(packageRoot, "src", "test", "fixtures", name, file);
}

export function prepareOutputDir(name: string): string {
  const outputDir = path.join(outputRoot, name);
  fs.rmSync(outputDir, { recursive: true, force: true });
  fs.mkdirSync(outputDir, { recursive: true });
  return outputDir;
}

export function normalizeSnapshot(value: string): string {
  let normalized = value.replaceAll("\\", "/");
  for (const root of snapshotRoots) {
    normalized = normalized.replaceAll(root, "<WORKSPACE>");
    normalized = normalized.replaceAll(encodeURIComponent(root), "<WORKSPACE>");
  }
  return normalized;
}
