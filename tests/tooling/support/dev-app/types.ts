export type Target = "playground" | "misskey" | "npmx" | "elk" | "vuefes";

export type LaunchConfig = {
  target: Target;
  url: string;
  setup?: () => void;
  beforeStart?: () => void;
  cwd: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
};

export type MisskeyBeforeStartSteps = {
  startLocalServices: (misskeyRoot: string) => void;
  ensureBackendBuilt: (misskeyRoot: string, configName: string) => void;
  ensureNativeDependencies: (misskeyRoot: string) => void;
  waitForLocalServices: (misskeyRoot: string, configName: string) => void;
  ensureMigrated: (misskeyRoot: string, configName: string) => void;
};

/**
 * Normalizes the target name accepted by the MoonBit `dev_app` wrapper.
 *
 * The wrapper receives user input through environment variables rather than
 * command-line flags because Vite+ forwards task arguments through several
 * layers. Keeping validation here gives both tests and the real launcher the
 * same exact set of supported application targets.
 */
export function normalizeTarget(value: string | undefined): Target {
  switch (value ?? "playground") {
    case "playground":
      return "playground";
    case "misskey":
      return "misskey";
    case "npmx":
      return "npmx";
    case "elk":
      return "elk";
    case "vuefes":
      return "vuefes";
    default:
      throw new Error(
        `Unsupported target "${value}". Expected one of: playground, misskey, npmx, elk, vuefes.`,
      );
  }
}
