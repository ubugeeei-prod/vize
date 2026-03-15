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
  const tempDir = path.join(cwd, "__oxlint_plugin_vize_temp__", String(process.pid));
  const ignoreArgs: string[] = [];
  const tempArgs: string[] = [];
  const pathReplacements = new Map<string, string>();
  let counter = 0;

  for (const filename of filenames) {
    const source = fs.readFileSync(filename, "utf8");
    if (hasScriptLikeBlock(source)) {
      continue;
    }

    const relativeFilename = path.relative(cwd, filename);
    const tempFilename = path.join(tempDir, `${counter}-${path.basename(filename)}`);
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
      if (pathReplacements.size === 0) {
        return;
      }

      fs.rmSync(tempDir, { force: true, recursive: true });
    },
    pathReplacements,
    usedScriptlessWorkaround: pathReplacements.size > 0,
  };
}

function toCliPath(filename: string): string {
  return filename.split(path.sep).join("/");
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
