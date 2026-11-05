/** The pinned Go hosts require explicit consent before launching a content mapper. */
export function normalHostArguments(command: string, config: string): string[] {
  return [
    command,
    "--project",
    config,
    "--runExternalCode",
    "--noEmit",
    "--incremental",
    "false",
    "--pretty",
    "false",
  ];
}
