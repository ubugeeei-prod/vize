import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadPublicAuthority } from "./support/lsp/installed-alias-replay/authority.ts";
import { replayInstalledAliases } from "./support/lsp/installed-alias-replay/runner.ts";
import { loadSignedSourceDelivery } from "./support/lsp/installed-alias-replay/source-delivery.ts";
import { loadVueFixtureAuthority } from "./support/lsp/installed-alias-replay/vue.ts";

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
  const key = process.argv[index]?.replace(/^--/u, "");
  const value = process.argv[index + 1];
  assert.ok(
    allowed.has(key) && value && !values.has(key),
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
const vue = loadVueFixtureAuthority({
  receiptPath: required("vue-authority"),
  receiptSha256: required("vue-authority-sha256"),
  sourceRoot,
  sourceHead: "c2bc3a55a2b70dc4dd6c4e43ee0737c347476398",
});
const receipt = await replayInstalledAliases({
  authority,
  fixtureRoot: path.join(sourceRoot, "tests/_fixtures/differential/lsp/type-alias-navigation/8011"),
  vuePath: vue.vuePath,
  vueCustody: vue.receipt,
  recheckVue: vue.recheck,
  outputRoot: required("output"),
  timeoutMs: 30_000,
});
process.stdout.write(
  `${JSON.stringify({ schema: receipt.schema, version: authority.version, source: authority.source, signedSourceMerge, output: required("output"), completeSessions: receipt.outcomes.length, success: true })}\n`,
);
