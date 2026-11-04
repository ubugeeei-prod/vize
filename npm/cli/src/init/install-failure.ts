const MAX_INSTALL_OUTPUT = 4_000;

/**
 * The install command's own output names the missing package. Keep it on the
 * error init throws so a failed install cannot continue into type checking.
 */
export function failedInstallMessage(
  command: string,
  args: readonly string[],
  output: string,
  status: string | number | null,
): string {
  const commandText = [command, ...args].join(" ");
  const detail = output.trim();
  const rendered =
    detail.length > MAX_INSTALL_OUTPUT ? `${detail.slice(0, MAX_INSTALL_OUTPUT)}…` : detail;
  const body = rendered.length > 0 ? rendered : `exit ${status ?? "unknown"}`;
  return `Install command failed (${commandText}). Type checking was not run.\n${body}`;
}
