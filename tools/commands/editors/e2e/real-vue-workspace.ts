import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  prepareRealVueWorkspace,
  resolveCorsaPath,
  resolveRealServerPath,
  resolveVuePackagePath,
} from "../../../support/compat/editor-e2e/real-vue-workspace.ts";

const usage = `Usage: node tools/commands/editors/e2e/real-vue-workspace.ts <prepare|server-path|corsa-path|vue-path> [workspace]

prepare <workspace> materializes the shared real-vue editor fixture.`;

function requireWorkspace(args: readonly string[]): string {
  const workspace = args[1];
  if (workspace === undefined) throw new Error(`prepare requires a workspace path\n\n${usage}`);
  return workspace;
}

function main(args: readonly string[]): number {
  switch (args[0]) {
    case "prepare":
      process.stdout.write(`${prepareRealVueWorkspace(requireWorkspace(args))}\n`);
      return 0;
    case "server-path":
      process.stdout.write(`${resolveRealServerPath()}\n`);
      return 0;
    case "corsa-path":
      process.stdout.write(`${resolveCorsaPath()}\n`);
      return 0;
    case "vue-path":
      process.stdout.write(`${resolveVuePackagePath()}\n`);
      return 0;
    case "--help":
    case "-h":
      process.stdout.write(`${usage}\n`);
      return 0;
    case undefined:
      throw new Error(usage);
    default:
      throw new Error(`unknown command ${args[0]}\n\n${usage}`);
  }
}

runEntrypoint(() => main(process.argv.slice(2)));
