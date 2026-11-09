import { existsSync, lstatSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, parse, resolve } from "node:path";

import { projectTemplates } from "./templates.js";
import type {
  CreateFrescoProjectOptions,
  CreateFrescoProjectResult,
  PackageManager,
} from "./types.js";

function normalizedPackageName(directory: string): string {
  const normalized = basename(directory)
    .toLowerCase()
    .replaceAll(/[^a-z0-9._-]+/gu, "-")
    .replaceAll(/^[._-]+|[._-]+$/gu, "");
  return normalized || "fresco-app";
}

function detectPackageManager(userAgent = process.env.npm_config_user_agent): PackageManager {
  const name = userAgent?.split("/")[0];
  return name === "npm" || name === "pnpm" || name === "yarn" || name === "bun" ? name : "pnpm";
}

function assertSafeTarget(targetDirectory: string): void {
  if (targetDirectory === parse(targetDirectory).root) {
    throw new Error("CREATE_FRESCO_UNSAFE_TARGET: refusing to scaffold into a filesystem root");
  }
  if (targetDirectory === homedir()) {
    throw new Error("CREATE_FRESCO_UNSAFE_TARGET: refusing to scaffold into the home directory");
  }
  if (existsSync(targetDirectory) && !lstatSync(targetDirectory).isDirectory()) {
    throw new Error(`CREATE_FRESCO_TARGET_FILE: ${targetDirectory} is not a directory`);
  }
}

function assertWritableTarget(targetDirectory: string, force: boolean): void {
  if (!existsSync(targetDirectory)) return;
  const entries = readdirSync(targetDirectory);
  if (entries.length > 0 && !force) {
    throw new Error(
      `CREATE_FRESCO_TARGET_NOT_EMPTY: ${targetDirectory} is not empty; pass --force to overwrite scaffold-owned files`,
    );
  }
}

/** Create a project without installing dependencies or deleting unrelated files. */
export function createFrescoProject(
  options: CreateFrescoProjectOptions,
): CreateFrescoProjectResult {
  const cwd = resolve(options.cwd ?? process.cwd());
  const targetDirectory = resolve(cwd, options.directory);
  const packageManager = options.packageManager ?? detectPackageManager();
  const packageName = normalizedPackageName(targetDirectory);
  const templates = projectTemplates({ packageName, packageManager });

  assertSafeTarget(targetDirectory);
  assertWritableTarget(targetDirectory, options.force === true);
  mkdirSync(targetDirectory, { recursive: true });

  for (const [relativePath, contents] of templates) {
    const destination = resolve(targetDirectory, relativePath);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, contents, "utf8");
  }

  return {
    targetDirectory,
    packageName,
    packageManager,
    files: [...templates.keys()],
  };
}
