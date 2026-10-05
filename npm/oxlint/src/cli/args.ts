// Oxlint 1.78 command/{lint,ignore,mod}.rs: scalar arguments, not plugin switches.
const OPTION_NAMES_WITH_VALUES = new Set([
  "-A",
  "-D",
  "-W",
  "-c",
  "-f",
  "--allow",
  "--config",
  "--debug",
  "--deny",
  "--format",
  "--ignore-path",
  "--ignore-pattern",
  "--max-warnings",
  "--report-unused-disable-directives-severity",
  "--threads",
  "--tsconfig",
  "--warn",
]);

const BOOLEAN_OPTIONS = new Set([
  "-h",
  "-V",
  "--help",
  "--version",
  "--init",
  "--fix",
  "--fix-suggestions",
  "--fix-dangerously",
  "--quiet",
  "--deny-warnings",
  "--no-ignore",
  "--silent",
  "--no-error-on-unmatched-pattern",
  "--print-config",
  "--rules",
  "--lsp",
  "--disable-nested-config",
  "--type-aware",
  "--type-check",
  "--type-check-only",
  "--suppress-all",
  "--prune-suppressions",
  "--report-unused-disable-directives",
  "--disable-unicorn-plugin",
  "--disable-oxc-plugin",
  "--disable-typescript-plugin",
  "--import-plugin",
  "--react-plugin",
  "--jsdoc-plugin",
  "--jest-plugin",
  "--vitest-plugin",
  "--jsx-a11y-plugin",
  "--nextjs-plugin",
  "--react-perf-plugin",
  "--promise-plugin",
  "--node-plugin",
  "--vue-plugin",
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

/** Preserve original option/value pairs while replacing the selected target set. */
export function withoutLintTargets(argv: readonly string[]): string[] {
  const options: string[] = [];
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--") {
      options.push(arg);
      break;
    }
    if (OPTION_NAMES_WITH_VALUES.has(arg)) {
      options.push(arg);
      const value = argv[++index];
      if (value == null || value === "--")
        throw new Error(`Scoped Vue transport requires a value for ${arg}.`);
      options.push(value);
    } else if (
      BOOLEAN_OPTIONS.has(arg) ||
      (arg.startsWith("--") &&
        arg.includes("=") &&
        OPTION_NAMES_WITH_VALUES.has(arg.slice(0, arg.indexOf("="))))
    ) {
      options.push(arg);
    } else if (arg.startsWith("-")) {
      throw new Error(`Scoped Vue transport cannot preserve unsupported option form: ${arg}`);
    }
  }
  return options;
}
