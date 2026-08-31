/**
 * Quotes a command fragment for POSIX shell interpolation.
 *
 * Vite+ tasks are command strings, so a few helpers intentionally compose shell
 * snippets. This function keeps that composition centralized and prevents
 * environment-derived paths from breaking commands that are later wrapped in
 * `sh -c`.
 */
export const shellQuote = (command: string) => `'${command.replaceAll("'", `'"'"'`)}'`;

const darwinUnsupportedUtf8Locale = "C.UTF-8";
const darwinFallbackUtf8Locale = "en_US.UTF-8";
const taskShellLocaleVariables = ["LC_ALL", "LC_CTYPE", "LANG"] as const;

const needsDarwinLocaleFallback = (platform: NodeJS.Platform, env: NodeJS.ProcessEnv) =>
  platform === "darwin" &&
  taskShellLocaleVariables.some((name) => env[name] === darwinUnsupportedUtf8Locale);

export const getTaskShellLocaleAssignments = (
  platform: NodeJS.Platform = process.platform,
  env: NodeJS.ProcessEnv = process.env,
) =>
  needsDarwinLocaleFallback(platform, env)
    ? taskShellLocaleVariables.map((name) => `${name}=${shellQuote(darwinFallbackUtf8Locale)}`)
    : [];

export const normalizeTaskShellLocale = (
  platform: NodeJS.Platform = process.platform,
  env: NodeJS.ProcessEnv = process.env,
) => {
  if (!needsDarwinLocaleFallback(platform, env)) {
    return;
  }

  for (const name of taskShellLocaleVariables) {
    env[name] = darwinFallbackUtf8Locale;
  }
};

const taskShellLocaleAssignments = getTaskShellLocaleAssignments();
// macOS does not ship C.UTF-8, but Nix shells often export it by default.
normalizeTaskShellLocale();

export const shellCommand = (
  command: string,
  environmentAssignments: readonly string[] = taskShellLocaleAssignments,
) =>
  `${environmentAssignments.length === 0 ? "" : `env ${environmentAssignments.join(" ")} `}sh -c ${shellQuote(command)}`;

export const shellCommandForwardingArguments = (
  command: string,
  environmentAssignments: readonly string[] = taskShellLocaleAssignments,
) => `${shellCommand(command, environmentAssignments)} --`;

const darwinLibiconvLibraryPath = process.env.VIZE_DARWIN_LIBICONV_LIB;
const rustTaskEnvironment =
  darwinLibiconvLibraryPath == null
    ? []
    : [
        `export LIBRARY_PATH=${shellQuote(darwinLibiconvLibraryPath)}\${LIBRARY_PATH:+:$LIBRARY_PATH}`,
        `export RUSTFLAGS=${shellQuote(`-L native=${darwinLibiconvLibraryPath}`)}\${RUSTFLAGS:+ $RUSTFLAGS}`,
      ];

/**
 * Applies the optional macOS libiconv environment to Rust-oriented task
 * commands.
 *
 * The environment is injected only when explicitly requested so regular Linux
 * CI and developer machines keep the shortest possible command path. When the
 * variable is present, both Cargo and any nested Rust build script see the same
 * library search path.
 */
export const withRustTaskEnvironment = (
  command: string,
  options: {
    forwardArguments?: boolean;
  } = {},
) => {
  const commandWithEnvironment =
    rustTaskEnvironment.length === 0 ? command : `${rustTaskEnvironment.join("; ")}; ${command}`;

  return options.forwardArguments
    ? shellCommandForwardingArguments(commandWithEnvironment)
    : rustTaskEnvironment.length === 0
      ? command
      : shellCommand(commandWithEnvironment);
};
