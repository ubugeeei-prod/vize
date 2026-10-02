import type { A11yViolation } from "../types/index.js";
import { escapeHtml } from "../utils.js";

export function nodeDetails(result: A11yViolation): string {
  return (result.targets ?? [])
    .map((node) => {
      const selectors = node.target
        .map((target) => (typeof target === "string" ? target : JSON.stringify(target)))
        .join(", ");
      const checks = JSON.stringify({ any: node.any, all: node.all, none: node.none }, null, 2);
      return `<details class="node-details"><summary><code>${escapeHtml(selectors)}</code></summary>
      <pre>${escapeHtml(node.html)}</pre>
      ${node.failureSummary ? `<pre>${escapeHtml(node.failureSummary)}</pre>` : ""}
      <pre>${escapeHtml(checks)}</pre></details>`;
    })
    .join("");
}
