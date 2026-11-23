/** Quotes one argument in the command line passed to Vite+. */
export const shellQuote = (value: string) => "'" + value.replaceAll("'", "'\"'\"'") + "'";

const localeVariables = ["LC_ALL", "LC_CTYPE", "LANG"] as const;

export function normalizeTaskShellLocale(
  platform: NodeJS.Platform = process.platform,
  env: NodeJS.ProcessEnv = process.env,
): void {
  if (platform === "darwin" && localeVariables.some((name) => env[name] === "C.UTF-8")) {
    for (const name of localeVariables) env[name] = "en_US.UTF-8";
  }
}
// The Nix shell may export C.UTF-8, which macOS does not install.
normalizeTaskShellLocale();

export const rustEnvironmentKeys = ["VIZE_DARWIN_LIBICONV_LIB", "LIBRARY_PATH", "RUSTFLAGS"];

export function workspaceTaskEnvironment(
  platform: NodeJS.Platform,
  environment: NodeJS.ProcessEnv,
): NodeJS.ProcessEnv {
  const env = { ...environment };
  normalizeTaskShellLocale(platform, env);
  const library = env.VIZE_DARWIN_LIBICONV_LIB;
  if (library) {
    if (!env.LIBRARY_PATH?.split(":").includes(library))
      env.LIBRARY_PATH = library + (env.LIBRARY_PATH ? ":" + env.LIBRARY_PATH : "");
    const flag = "-L native=" + library;
    if (env.RUSTFLAGS !== flag && !env.RUSTFLAGS?.startsWith(flag + " "))
      env.RUSTFLAGS = flag + (env.RUSTFLAGS ? " " + env.RUSTFLAGS : "");
  }
  return env;
}
