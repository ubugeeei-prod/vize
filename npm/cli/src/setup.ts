import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import {
  addDefaultScripts,
  atomicWriteFile,
  DEFAULT_OXLINT_CONFIG,
  DEFAULT_VIZE_CONFIG,
  dependencyNames,
  detectJsonIndent,
  OXLINT_CONFIG_FILES,
  parsePackageJson,
  planGeneratedConfig,
  readRequiredFile,
  REQUIRED_DEV_DEPENDENCIES,
  VIZE_CONFIG_FILES,
  type PlannedFile,
} from "./setup/config.js";
import { planViteMigration } from "./setup/vite.js";

export interface SetupCommand {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
}

export interface SetupResult {
  readonly root: string;
  readonly createdFiles: readonly string[];
  readonly preservedFiles: readonly string[];
  readonly addedScripts: readonly string[];
  readonly preservedScripts: readonly string[];
  readonly migratedViteConfig: string | null;
  readonly enabledVitePlusLint: boolean;
  readonly installCommand: SetupCommand | null;
  readonly removeCommand: SetupCommand | null;
}

export interface SetupOptions {
  readonly root: string;
  readonly install?: boolean;
  readonly runCommand?: (command: SetupCommand) => void;
  readonly writeFile?: (filename: string, source: string) => void;
}

export function setupProject(options: SetupOptions): SetupResult {
  const root = path.resolve(options.root);
  const packagePath = path.join(root, "package.json");
  const packageSource = readRequiredFile(packagePath, "No package.json found");
  const packageJson = parsePackageJson(packagePath, packageSource);
  const packageIndent = detectJsonIndent(packageSource);
  const existingDependencies = dependencyNames(packageJson);
  const missingDependencies = REQUIRED_DEV_DEPENDENCIES.filter(
    (dependency) => !existingDependencies.has(dependency),
  );

  const createdFiles: string[] = [];
  const preservedFiles: string[] = [];
  const plannedFiles: PlannedFile[] = [];
  planGeneratedConfig(
    root,
    VIZE_CONFIG_FILES,
    "vize.config.ts",
    DEFAULT_VIZE_CONFIG,
    plannedFiles,
    createdFiles,
    preservedFiles,
  );

  const existingOxlintConfig = OXLINT_CONFIG_FILES.find((candidate) =>
    fs.existsSync(path.join(root, candidate)),
  );
  const viteMigration = planViteMigration(root, existingOxlintConfig === undefined);
  if (viteMigration.file) {
    plannedFiles.push(viteMigration.file);
  }
  if (viteMigration.preserved) {
    preservedFiles.push(viteMigration.preserved);
  }
  if (
    viteMigration.usesVitePlus &&
    !viteMigration.hasVitePlusLint &&
    existingOxlintConfig === undefined
  ) {
    preservedFiles.push("Vite+ lint configuration");
  }
  if (existingOxlintConfig) {
    preservedFiles.push(existingOxlintConfig);
  } else if (!viteMigration.hasVitePlusLint && !viteMigration.usesVitePlus) {
    planGeneratedConfig(
      root,
      OXLINT_CONFIG_FILES,
      "oxlint.config.ts",
      DEFAULT_OXLINT_CONFIG,
      plannedFiles,
      createdFiles,
      preservedFiles,
    );
  }

  const { addedScripts, preservedScripts } = addDefaultScripts(packageJson);
  if (addedScripts.length > 0) {
    plannedFiles.push({
      filename: packagePath,
      source: `${JSON.stringify(packageJson, null, packageIndent)}\n`,
    });
  }
  writePlannedFiles(root, plannedFiles, options.writeFile ?? atomicWriteFile);

  const runCommand = options.runCommand ?? runSetupCommand;
  let installCommand: SetupCommand | null = null;
  if (options.install !== false && missingDependencies.length > 0) {
    installCommand = {
      command: "vp",
      args: ["add", "-D", ...missingDependencies],
      cwd: root,
    };
    runCommand(installCommand);
  }

  let removeCommand: SetupCommand | null = null;
  if (
    options.install !== false &&
    viteMigration.removesOfficialPlugin &&
    existingDependencies.has("@vitejs/plugin-vue")
  ) {
    removeCommand = {
      command: "vp",
      args: ["remove", "@vitejs/plugin-vue"],
      cwd: root,
    };
    runCommand(removeCommand);
  }

  return {
    root,
    createdFiles,
    preservedFiles,
    addedScripts,
    preservedScripts,
    migratedViteConfig:
      viteMigration.file && viteMigration.removesOfficialPlugin
        ? path.basename(viteMigration.file.filename)
        : null,
    enabledVitePlusLint: viteMigration.enablesVitePlusLint,
    installCommand,
    removeCommand,
  };
}

export function runSetupCli(args: readonly string[]): void {
  if (args.includes("--help") || args.includes("-h")) {
    process.stdout.write(setupHelp());
    return;
  }

  let install = true;
  let root: string | undefined;
  for (const arg of args) {
    if (arg === "--no-install") {
      install = false;
      continue;
    }
    if (arg.startsWith("-")) {
      throw new Error(`Unknown setup option: ${arg}`);
    }
    if (root) {
      throw new Error(`Unexpected setup argument: ${arg}`);
    }
    root = arg;
  }

  const result = setupProject({ root: root ?? process.cwd(), install });
  printSetupResult(result, install);
}

function setupHelp(): string {
  return `Configure Vize in an existing Vite or Vite+ project

Usage: vize setup [ROOT] [OPTIONS]

Arguments:
  [ROOT]  Project root containing package.json (default: current directory)

Options:
  --no-install  Write project configuration without changing dependencies
  -h, --help    Print help
`;
}

function printSetupResult(result: SetupResult, install: boolean): void {
  let dependenciesMissing = false;
  for (const filename of result.createdFiles) {
    process.stdout.write(`[vize setup] created ${filename}\n`);
  }
  if (result.migratedViteConfig) {
    process.stdout.write(
      `[vize setup] migrated ${result.migratedViteConfig} to @vizejs/vite-plugin\n`,
    );
  }
  if (result.enabledVitePlusLint) {
    process.stdout.write("[vize setup] enabled oxlint-plugin-vize for vp lint\n");
  }
  if (result.addedScripts.length > 0) {
    process.stdout.write(`[vize setup] added scripts: ${result.addedScripts.join(", ")}\n`);
  }
  for (const filename of result.preservedFiles) {
    process.stdout.write(`[vize setup] preserved existing ${filename}\n`);
  }
  if (result.preservedScripts.length > 0) {
    process.stdout.write(`[vize setup] preserved scripts: ${result.preservedScripts.join(", ")}\n`);
  }
  if (!install) {
    const packagePath = path.join(result.root, "package.json");
    const packageJson = parsePackageJson(packagePath, fs.readFileSync(packagePath, "utf8"));
    const missing = REQUIRED_DEV_DEPENDENCIES.filter(
      (dependency) => !dependencyNames(packageJson).has(dependency),
    );
    if (missing.length > 0) {
      dependenciesMissing = true;
      process.stdout.write(
        `[vize setup] install dependencies with: vp add -D ${missing.join(" ")}\n`,
      );
    }
  }
  if (result.removeCommand) {
    process.stdout.write("[vize setup] removed @vitejs/plugin-vue\n");
  }
  process.stdout.write(
    dependenciesMissing
      ? "[vize setup] configuration written; install dependencies before running Vize\n"
      : "[vize setup] ready; run vp run vize:ready\n",
  );
}

function runSetupCommand(command: SetupCommand): void {
  execFileSync(command.command, [...command.args], {
    cwd: command.cwd,
    stdio: "inherit",
  });
}

function writePlannedFiles(
  root: string,
  plannedFiles: readonly PlannedFile[],
  writeFile: (filename: string, source: string) => void,
): void {
  const writtenFiles: string[] = [];
  for (const file of plannedFiles) {
    try {
      writeFile(file.filename, file.source);
    } catch (error) {
      if (writtenFiles.length === 0) {
        throw error;
      }
      const failedFile = path.relative(root, file.filename);
      throw new Error(
        `Setup partially completed: wrote ${writtenFiles.join(", ")} before ${failedFile} failed. Run setup again to finish.`,
        { cause: error },
      );
    }
    writtenFiles.push(path.relative(root, file.filename));
  }
}
