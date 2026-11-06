import assert from "node:assert/strict";

export function assertComponentModesHtml(html: string, label: string): void {
  assert.match(
    html,
    /<span[^>]*id="mode-probe"[^>]*data-mode="server"[^>]*>server registration<\/span>/,
    `${label} must render the server registration`,
  );
  assert.doesNotMatch(
    html,
    /data-mode="client"/,
    `${label} must not render the client registration`,
  );
  assert.match(
    html,
    /<section[^>]*id="island-probe"[^>]*data-render="server"[^>]*data-island-uid="[^"]+"/,
    `${label} must render the standalone server component through NuxtIsland`,
  );
  assert.match(
    html,
    /<strong[^>]*id="island-slot"[^>]*>slot transport<\/strong>/,
    `${label} must transport the authored slot into the server island`,
  );
}
