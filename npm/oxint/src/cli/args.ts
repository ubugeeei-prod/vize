const OPTION_NAMES_WITH_VALUES = new Set([
  "-A",
  "-D",
  "-W",
  "-c",
  "-f",
  "--config",
  "--cwd",
  "--deny",
  "--fix-suggestions",
  "--format",
  "--ignore-path",
  "--ignore-pattern",
  "--import-plugin",
  "--jsx-a11y-plugin",
  "--max-warnings",
  "--nextjs-plugin",
  "--node-plugin",
  "--promise-plugin",
  "--react-perf-plugin",
  "--react-plugin",
  "--threads",
  "--tsconfig",
  "--typescript-plugin",
  "--unicorn-plugin",
  "--vitest-plugin",
  "--warn",
]);

export function getLintTargets(argv: readonly string[]): string[] {
  const targets: string[] = [];
  let collectEverything = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (collectEverything) {
      targets.push(arg);
      continue;
    }

    if (arg === "--") {
      collectEverything = true;
      continue;
    }

    if (arg.startsWith("--") && arg.includes("=")) {
      continue;
    }

    if (OPTION_NAMES_WITH_VALUES.has(arg)) {
      index += 1;
      continue;
    }

    if (arg.startsWith("-")) {
      continue;
    }

    targets.push(arg);
  }

  return targets.length === 0 ? ["."] : targets;
}
