/** Actual piped-child inputs used by both pinned hosts' presentation selection. */
export const presentationInputs = [
  "CI",
  "FORCE_COLOR",
  "NO_COLOR",
  "GITHUB_ACTIONS",
  "AI_AGENT",
  "CURSOR_AGENT",
  "CLAUDECODE",
  "CLAUDE_CODE",
  "REPL_ID",
  "GEMINI_CLI",
  "CODEX_SANDBOX",
  "CODEX_THREAD_ID",
  "COPILOT_CLI",
  "OPENCODE",
  "JUNIE_DATA",
  "JUNIE_SHIM_PATH",
  "PATH",
  "EDITOR",
  "TERM_PROGRAM",
];

export function presentationContext(): Record<string, string | null> {
  return Object.fromEntries(presentationInputs.map((key) => [key, process.env[key] ?? null]));
}

/** Pinned c42d639/2ae2939 agent_detection and default_output_format contracts. */
export function implicitDefault(context: Record<string, string | null>): boolean {
  const agentKeys = presentationInputs.filter(
    (key) =>
      ![
        "CI",
        "FORCE_COLOR",
        "NO_COLOR",
        "GITHUB_ACTIONS",
        "PATH",
        "EDITOR",
        "TERM_PROGRAM",
      ].includes(key),
  );
  return (
    context.GITHUB_ACTIONS !== "true" &&
    !agentKeys.some((key) => Boolean(context[key])) &&
    !/[.]pi[/\\]agent/u.test(context.PATH ?? "") &&
    !(context.EDITOR ?? "").includes("devin") &&
    !(context.TERM_PROGRAM ?? "").includes("kiro")
  );
}
