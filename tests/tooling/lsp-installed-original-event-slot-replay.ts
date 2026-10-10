import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadPublicAuthority } from "./support/lsp/installed-alias-replay/authority.ts";
import { loadSignedSourceDelivery } from "./support/lsp/installed-alias-replay/source-delivery.ts";
import { replayInstalledEventSlotScopes } from "./support/lsp/installed-event-slot-replay/runner.ts";
import { loadEventFixtureVueAuthority } from "./support/lsp/installed-event-slot-replay/stock-authority.ts";

const allowed = new Set([
  "authority",
  "authority-sha256",
  "source-delivery",
  "source-delivery-sha256",
  "vue-authority",
  "vue-authority-sha256",
  "output",
]);
const values = new Map<string, string>();
for (let index = 2; index < process.argv.length; index += 2) {
  const flag = process.argv[index];
  const key = flag?.slice(2);
  const value = process.argv[index + 1];
  assert.ok(
    flag?.startsWith("--") && key && allowed.has(key) && value && !values.has(key),
    "explicit known unique argument/value pairs required",
  );
  values.set(key, value);
}
assert.equal(
  values.size,
  allowed.size,
  `required arguments: ${[...allowed].map((key) => `--${key}`).join(" ")}`,
);
const required = (name: string): string => values.get(name)!;
const sourceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const signedSourceMerge = loadSignedSourceDelivery(
  required("source-delivery"),
  required("source-delivery-sha256"),
);
const authority = loadPublicAuthority(
  required("authority"),
  required("authority-sha256"),
  sourceRoot,
  signedSourceMerge,
);
const vue = loadEventFixtureVueAuthority({
  receiptPath: required("vue-authority"),
  receiptSha256: required("vue-authority-sha256"),
  sourceRoot,
});
const receipt = await replayInstalledEventSlotScopes({
  authority,
  sourceRoot,
  vuePath: vue.vuePath,
  vueCustody: { receipt: vue.receipt, sourceComparison: vue.sourceComparison },
  recheckVue: vue.recheck,
  outputRoot: required("output"),
});
process.stdout.write(
  `${JSON.stringify({
    schema: receipt.schema,
    version: authority.version,
    source: authority.source,
    signedSourceMerge,
    output: required("output"),
    completeSessions: receipt.outcomes.length,
    success: true,
    scope: receipt.scope,
  })}\n`,
);
