import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const versionPattern = /^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

export interface PreviewCommand {
  readonly executable: string;
  readonly arguments: readonly string[];
  readonly log: string;
}

function assertVersion(version: string): void {
  if (!versionPattern.test(version)) throw new Error(`Invalid JSR preview version ${version}`);
}

/** Commands used to install and execute one immutable public release with Bun. */
export function bunPreviewCommands(version: string): readonly PreviewCommand[] {
  assertVersion(version);
  return [
    {
      executable: "bun",
      arguments: [
        "add",
        "--ignore-scripts",
        "--exact",
        `@vizejs/babel-plugin@${version}`,
        `oxlint-plugin-vize@${version}`,
        "vite@8.2.4",
        "vue@3.5.42",
      ],
      log: "install.log",
    },
    {
      executable: "bunx",
      arguments: ["--bun", "jsr@0.14.3", "add", `@vizejs/vize@${version}`],
      log: "jsr-add.log",
    },
    {
      executable: "bun",
      arguments: [
        "run",
        "consumer.ts",
        "@vizejs/vize",
        "--vite",
        "--babel-specifier",
        "@vizejs/babel-plugin",
        "--oxlint-specifier",
        "oxlint-plugin-vize",
      ],
      log: "runtime.log",
    },
  ];
}

/** Creates the minimal fresh project used by the hosted Bun acceptance check. */
export function prepareBunPreviewConsumer(output: string, sourceRoot: string = repository): void {
  mkdirSync(output, { recursive: true });
  copyFileSync(
    resolve(sourceRoot, "tools/support/release/jsr/consumer.ts"),
    resolve(output, "consumer.ts"),
  );
  writeFileSync(
    resolve(output, "package.json"),
    `${JSON.stringify({ name: "vize-jsr-bun-preview", private: true, type: "module" }, null, 2)}\n`,
  );
}

function runLoggedCommand(directory: string, command: PreviewCommand): void {
  const result = spawnSync(command.executable, command.arguments, {
    cwd: directory,
    encoding: "utf8",
    env: process.env,
    maxBuffer: 16 * 1024 * 1024,
  });
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  writeFileSync(resolve(directory, command.log), output);
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.signal) {
    throw new Error(`${command.executable} received ${result.signal}; see ${command.log}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command.executable} exited with ${result.status}; see ${command.log}`);
  }
}

/** Runs the complete public package graph from a fresh Bun project. */
export function verifyPublicBunPreview(
  output: string,
  version: string,
  sourceRoot: string = repository,
): void {
  prepareBunPreviewConsumer(output, sourceRoot);
  for (const command of bunPreviewCommands(version)) runLoggedCommand(output, command);
}

/** Imports the locally staged facade through Deno with the repository policy. */
export function prepareStagedDenoPreview(
  output: string,
  sourceRoot: string = repository,
): { readonly config: string; readonly consumer: string; readonly lock: string } {
  mkdirSync(output, { recursive: true });
  const consumer = resolve(output, "consumer.ts");
  const config = resolve(output, "deno.json");
  const lock = resolve(output, "deno.lock");
  copyFileSync(resolve(sourceRoot, "tools/support/release/jsr/consumer.ts"), consumer);
  copyFileSync(resolve(sourceRoot, "tools/support/release/jsr/consumer-deno.json"), config);
  return { config, consumer, lock };
}

export function stagedDenoArguments(output: string, stagedPackage: string): readonly string[] {
  const { config, consumer, lock } = prepareStagedDenoPreview(output);
  return [
    "run",
    "--config",
    config,
    "--lock",
    lock,
    "--allow-all",
    consumer,
    `${pathToFileURL(resolve(stagedPackage)).href}/`,
  ];
}

/** Imports the locally staged facade through Deno with the repository policy. */
export function verifyStagedDenoPreview(output: string, stagedPackage: string): void {
  const result = spawnSync("deno", stagedDenoArguments(output, stagedPackage), {
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.signal) throw new Error(`deno received ${result.signal}`);
  if (result.status !== 0) throw new Error(`deno exited with ${result.status}`);
}

function usage(): never {
  throw new Error(
    "Usage: public-preview-consumers.ts <bun|staged-deno> <output> <version|staged-package>",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, output, value] = process.argv.slice(2);
  if (!output || !value) usage();
  if (command === "bun") verifyPublicBunPreview(output, value);
  else if (command === "staged-deno") verifyStagedDenoPreview(output, value);
  else usage();
}
