// Nuxt's vite-node client rejects any module request that outlives
// `vite.viteNode.requestTimeout` (60s by default), then destroys the IPC socket on
// its retry, which fails every in-flight request with `IPC connection closed`. The
// playground's first SSR module fetch waits on Vite's optimize-deps pass, so
// smaller CI runners blow that default and the bridge looks dead while it is only
// slow, and rebooting just pays the same cold cost again. Give the bridge a budget
// wider than our own probes so a real socket loss stays the only dead-bridge signal.
export const VITE_NODE_REQUEST_TIMEOUT_MS = 600_000;

/**
 * Add the widened vite-node request budget to a Nuxt config source, reusing the
 * playground's existing `vite` block when it has one. Idempotent.
 */
export function withViteNodeRequestBudget(config: string): string {
  if (config.includes("viteNode:")) return config;

  const budget = `viteNode: { requestTimeout: ${VITE_NODE_REQUEST_TIMEOUT_MS} },`;
  const viteBlock = config.match(/^([ \t]*)vite: \{$/m);
  if (viteBlock) {
    return config.replace(viteBlock[0], `${viteBlock[0]}\n${viteBlock[1]}  ${budget}`);
  }
  if (config.includes("compatibilityDate:")) {
    return config.replace(
      "compatibilityDate:",
      `vite: {\n    ${budget}\n  },\n\n  compatibilityDate:`,
    );
  }
  return config;
}
