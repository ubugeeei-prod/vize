/**
 * Compares the `vize` CLI package with the installed `@vizejs/native` package.
 *
 * A mismatch must fail before the native binding loads. Otherwise `vize check`
 * would type-check with a different release than the CLI the user invoked.
 */
export function nativeVersionMismatchMessage(
  cliVersion: unknown,
  nativeVersion: unknown,
  env: NodeJS.ProcessEnv = process.env,
): string | null {
  if (env.VIZE_ALLOW_NATIVE_VERSION_MISMATCH === "1") return null;
  if (typeof cliVersion !== "string" || cliVersion.length === 0) {
    return "vize package version is missing. Reinstall the vize package before type checking.";
  }
  if (typeof nativeVersion !== "string" || nativeVersion.length === 0) {
    return "@vizejs/native package version is missing. Reinstall dependencies so type checking uses one Vize release.";
  }
  if (cliVersion === nativeVersion) return null;
  return `vize@${cliVersion} and @vizejs/native@${nativeVersion} are different versions. Reinstall dependencies so type checking uses one Vize release.`;
}
