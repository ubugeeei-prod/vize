export function createSyntaxAuditDeadline(
  environment: NodeJS.ProcessEnv,
  now: () => number = Date.now,
): () => void {
  const name = "SYNTAX_HIGHLIGHTER_ORACLE_TIMEOUT_MS";
  const timeoutMs = Number(environment[name] ?? "600000");
  if (!Number.isInteger(timeoutMs) || timeoutMs <= 0) throw new Error(`${name} must be positive`);
  const startedAt = now();
  return () => {
    if (now() - startedAt > timeoutMs)
      throw new Error(`syntax oracle shard exceeded ${timeoutMs}ms`);
  };
}
