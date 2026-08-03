function parseDefine(value: unknown): unknown {
  try {
    return typeof value === "string" ? JSON.parse(value) : value;
  } catch {
    return value;
  }
}

/** Resolve the Options API runtime define with plugin-vue's precedence. */
export function resolveOptionsApiFlag(option: boolean | undefined, define: unknown): unknown {
  return option ?? parseDefine(define) ?? true;
}
