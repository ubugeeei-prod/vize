import assert from "node:assert/strict";
import fs from "node:fs";

// Mirrors `HOST_TEST_SERVER_INFO_COMMAND` in `src/extension-core.ts`; the
// tooling tests assert both stay in sync.
export const HOST_TEST_SERVER_INFO_COMMAND = "vize.test.getServerInfo";

export function parseVizeVersion(output) {
  const match = output.match(/\bvize\s+([0-9]+\.[0-9]+\.[0-9]+(?:[-+][^\s]+)?)/);
  assert.ok(match, `could not parse vize version from ${JSON.stringify(output)}`);
  return match[1];
}

export function assertRealHostServerInfo(actual, { extensionVersion, serverPath, serverVersion }) {
  const expected = {
    extensionVersion,
    path: fs.realpathSync(serverPath),
    source: "configured",
    status: "ready",
    version: serverVersion,
  };

  assert.deepEqual(actual, expected);
  return actual;
}
