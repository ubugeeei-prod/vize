import { spawnSync } from "node:child_process";

function splitGroups(args: readonly string[]): string[][] {
  const groups: string[][] = [];
  let current: string[] = [];
  for (const arg of args) {
    if (arg === "--") {
      if (current.length > 0) groups.push(current);
      current = [];
      continue;
    }
    current.push(arg);
  }
  if (current.length > 0) groups.push(current);
  return groups;
}

function main(): number {
  const groups = splitGroups(process.argv.slice(2));
  if (groups.length === 0) {
    console.log("Usage: node tools/support/github/run-many.ts <cmd...> -- <cmd...>");
    return 1;
  }
  for (const group of groups) {
    const [command, ...args] = group;
    if (command == null) return 1;
    const status = spawnSync(command, args, { stdio: "inherit" });
    if (status.error != null) {
      console.error(`run-many: failed to run ${command}: ${status.error.message}`);
      return 1;
    }
    if (status.status !== 0) return status.status ?? 1;
  }
  return 0;
}

process.exit(main());
