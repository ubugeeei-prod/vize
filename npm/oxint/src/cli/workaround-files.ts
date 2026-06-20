import fs from "node:fs";
import path from "node:path";

import { hasScriptLikeBlock, appendScriptlessWorkaround } from "../workaround.js";

export interface PreparedWorkaroundFiles {
  appendedArgs: string[];
  cleanup(): void;
  pathReplacements: ReadonlyMap<string, string>;
  usedScriptlessWorkaround: boolean;
}

export function prepareScriptlessWorkaroundFiles(
  cwd: string,
  filenames: readonly string[],
): PreparedWorkaroundFiles {
  const nodeModulesDir = path.join(cwd, "node_modules");
  const tempRoot = path.join(nodeModulesDir, ".vize", "oxlint-plugin-vize");
  let tempDir: string | undefined;
  let createdNodeModules = false;
  const ignoreArgs: string[] = [];
  const tempArgs: string[] = [];
  const pathReplacements = new Map<string, string>();
  let counter = 0;

  for (const filename of filenames) {
    const source = fs.readFileSync(filename, "utf8");
    const isStandaloneHtml = isStandaloneHtmlFile(filename);
    if (!isStandaloneHtml && hasScriptLikeBlock(source)) {
      continue;
    }

    const relativeFilename = path.relative(cwd, filename);
    if (tempDir == null) {
      const created = createWorkaroundTempDir(nodeModulesDir, tempRoot);
      tempDir = created.tempDir;
      createdNodeModules = created.createdNodeModules;
    }
    const tempBasename = isStandaloneHtml
      ? `${path.basename(filename)}.vue`
      : path.basename(filename);
    const tempFilename = path.join(tempDir, `${counter}-${tempBasename}`);
    counter += 1;

    fs.mkdirSync(path.dirname(tempFilename), { recursive: true });
    fs.writeFileSync(tempFilename, appendScriptlessWorkaround(source, filename));

    ignoreArgs.push("--ignore-pattern", toCliPath(relativeFilename));
    tempArgs.push(tempFilename);
    registerPathReplacementVariants(pathReplacements, cwd, tempFilename, filename);
  }

  return {
    appendedArgs: [...ignoreArgs, ...tempArgs],
    cleanup() {
      if (tempDir == null) {
        return;
      }

      fs.rmSync(tempDir, { force: true, recursive: true });
      removeEmptyDirectory(tempRoot);
      removeEmptyDirectory(path.dirname(tempRoot));
      if (createdNodeModules) {
        removeEmptyDirectory(nodeModulesDir);
      }
    },
    pathReplacements,
    usedScriptlessWorkaround: pathReplacements.size > 0,
  };
}

function createWorkaroundTempDir(
  nodeModulesDir: string,
  tempRoot: string,
): { createdNodeModules: boolean; tempDir: string } {
  const createdNodeModules = !fs.existsSync(nodeModulesDir);
  fs.mkdirSync(tempRoot, { recursive: true });
  return {
    createdNodeModules,
    tempDir: fs.mkdtempSync(path.join(tempRoot, `${process.pid}-`)),
  };
}

function removeEmptyDirectory(dirname: string): void {
  try {
    fs.rmdirSync(dirname);
  } catch (error) {
    if (!isIgnorableRemoveDirError(error)) {
      throw error;
    }
  }
}

function isIgnorableRemoveDirError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    (error.code === "ENOENT" ||
      error.code === "ENOTDIR" ||
      error.code === "ENOTEMPTY" ||
      error.code === "EEXIST")
  );
}

function toCliPath(filename: string): string {
  return filename.split(path.sep).join("/");
}

function isStandaloneHtmlFile(filename: string): boolean {
  return filename.endsWith(".html") || filename.endsWith(".htm");
}

function registerPathReplacementVariants(
  replacements: Map<string, string>,
  cwd: string,
  tempFilename: string,
  originalFilename: string,
): void {
  const relativeTempFilename = path.relative(cwd, tempFilename);
  const relativeOriginalFilename = getReportedOriginalFilename(cwd, originalFilename);
  const variants = new Set([
    [tempFilename, relativeOriginalFilename],
    [toCliPath(tempFilename), toCliPath(relativeOriginalFilename)],
    [relativeTempFilename, relativeOriginalFilename],
    [toCliPath(relativeTempFilename), toCliPath(relativeOriginalFilename)],
  ]);

  for (const [from, to] of variants) {
    if (!from || !to) {
      continue;
    }

    replacements.set(from, to);
  }
}

function getReportedOriginalFilename(cwd: string, filename: string): string {
  const relativeFilename = path.relative(cwd, filename);

  if (
    relativeFilename &&
    !relativeFilename.startsWith(`..${path.sep}`) &&
    relativeFilename !== ".." &&
    !path.isAbsolute(relativeFilename)
  ) {
    return relativeFilename;
  }

  return filename;
}
