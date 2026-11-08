type FrescoNative = typeof import("@vizejs/fresco-native");

const CI_ENVIRONMENT_KEYS = ["CI", "CONTINUOUS_INTEGRATION", "BUILD_NUMBER", "RUN_ID"] as const;

function isTruthyEnvironmentValue(value: string | undefined): boolean {
  if (!value) return false;
  return !["0", "false", "no"].includes(value.toLowerCase());
}

function isCiEnvironment(env: NodeJS.ProcessEnv = process.env): boolean {
  return CI_ENVIRONMENT_KEYS.some((key) => isTruthyEnvironmentValue(env[key]));
}

/** Resolve whether an application can own the interactive terminal lifecycle. */
export function detectInteractiveMode(
  interactive: boolean | undefined,
  stdout: NodeJS.WriteStream,
): boolean {
  return interactive ?? (stdout.isTTY === true && !isCiEnvironment());
}

/** Initialize the native terminal modes required by an interactive application. */
export function initializeTerminal(
  native: FrescoNative,
  options: { alternateScreen: boolean; mouse: boolean },
): void {
  if (typeof native.initTerminalWithOptions === "function") {
    native.initTerminalWithOptions({
      alternateScreen: options.alternateScreen,
      bracketedPaste: true,
      hideCursor: true,
      mouse: options.mouse,
      rawMode: true,
    });
  } else if (options.mouse) {
    native.initTerminalWithMouse();
  } else {
    native.initTerminal();
  }

  native.enableIme?.();
  native.initLayout();
}
