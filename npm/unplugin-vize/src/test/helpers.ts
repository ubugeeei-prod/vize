import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const currentDir = path.dirname(fileURLToPath(import.meta.url));

export const packageRoot = path.resolve(currentDir, "..", "..");
export const workspaceRoot = path.resolve(packageRoot, "..", "..");
const outputRoot = path.join(workspaceRoot, "__agent_only", "test-output", "unplugin-vize");
const snapshotRoots = Array.from(new Set([workspaceRoot, fs.realpathSync(workspaceRoot)])).map(
  (root) => root.replaceAll("\\", "/"),
);

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
