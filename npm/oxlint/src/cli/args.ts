const OPTION_NAMES_WITH_VALUES = new Set([
  "-A",
  "-D",
  "-W",
  "-c",
  "-f",
  "--config",
  "--cwd",
  "--deny",
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

/**
 * Formats whose reports contain output even for a completely clean run.
 *
 * The auto, stylish, and unix formats legitimately print nothing when no
 * diagnostics are found, so an empty report proves nothing for them. These
 * formats always emit at least a summary or document skeleton, which makes a
 * fully silent exit-0 run attributable to a child that never linted at all.
 */
const ALWAYS_REPORTING_FORMATS = new Set(["checkstyle", "default", "json", "junit"]);

/** Whether the requested output format guarantees a non-empty report. */
export function expectsLintReport(argv: readonly string[]): boolean {
  let format: string | null = null;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--") {
      break;
    }

    if (arg === "-f" || arg === "--format") {
      format = argv[index + 1] ?? null;
      index += 1;
      continue;
    }

    if (arg.startsWith("--format=")) {
      format = arg.slice("--format=".length);
    }
  }

  return format != null && ALWAYS_REPORTING_FORMATS.has(format);
}

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
