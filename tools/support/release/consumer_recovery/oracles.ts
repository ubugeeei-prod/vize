import assert from "node:assert/strict";
import { lstatSync, readFileSync } from "node:fs";
import path from "node:path";
import { sha256, same } from "./policy.ts";
import { effectiveDenoConfig } from "./runtime.ts";

// Closed source/config/workflow closure for signing AND consuming recovery evidence.
// Include all materialized public harness modules as well as static imports.
export const policyFiles = [
  ".github/workflows/release-consumer-recovery.yml",
  ".github/workflows/release-operator.yml",
  ".github/workflows/release-public-acceptance.yml",
  "npm/cli/src/corsa-runtime.ts",
  "tests/performance/support/warm-type-backed-public-archives.ts",
  "tools/support/release/consumer_recovery/archive.ts",
  "tools/support/release/consumer_recovery/consumer.ts",
  "tools/support/release/consumer_recovery/custody.ts",
  "tools/support/release/consumer_recovery/evidence.ts",
  "tools/support/release/consumer_recovery/main.ts",
  "tools/support/release/consumer_recovery/oracles.ts",
  "tools/support/release/consumer_recovery/policy.ts",
  "tools/support/release/consumer_recovery/runtime.ts",
  "tools/support/release/jsr/consumer-deno.json",
  "tools/support/release/marketplace_query.ts",
  "tools/support/release/public_acceptance/a11y.ts",
  "tools/support/release/public_acceptance/accept.ts",
  "tools/support/release/public_acceptance/browser.ts",
  "tools/support/release/public_acceptance/browser_observation.ts",
  "tools/support/release/public_acceptance/gallery.ts",
  "tools/support/release/public_acceptance/github.ts",
  "tools/support/release/public_acceptance/http.ts",
  "tools/support/release/public_acceptance/inputs.ts",
  "tools/support/release/public_acceptance/installed.ts",
  "tools/support/release/public_acceptance/lsp.ts",
  "tools/support/release/public_acceptance/lsp_assets.ts",
  "tools/support/release/public_acceptance/lsp_atomic.ts",
  "tools/support/release/public_acceptance/lsp_cargo.ts",
  "tools/support/release/public_acceptance/lsp_case.ts",
  "tools/support/release/public_acceptance/lsp_packets.ts",
  "tools/support/release/public_acceptance/lsp_source.ts",
  "tools/support/release/public_acceptance/native.ts",
  "tools/support/release/public_acceptance/plan.ts",
  "tools/support/release/public_acceptance/registry.ts",
  "tools/support/release/public_acceptance/vrt_api.ts",
  "tools/support/release/public_acceptance/vrt_artifacts.ts",
  "tools/support/release/public_acceptance/vrt_audit.ts",
  "tools/support/release/public_acceptance/vrt_cli.ts",
  "tools/support/release/public_acceptance/vrt_fixtures.ts",
  "tools/support/release/public_acceptance/vrt_host.ts",
  "tools/support/release/public_acceptance/vrt_hosted.ts",
  "tools/support/release/public_acceptance/vrt_literal.ts",
  "tools/support/release/public_acceptance/vrt_observer.ts",
  "tools/support/release/public_acceptance/vrt_retained.ts",
  "tools/support/release/public_acceptance/vrt_retained_scan.ts",
  "tools/support/release/public_acceptance/vrt_secrets.ts",
  "tools/support/release/public_acceptance/vrt_service_announcement.ts",
  "tools/support/release/public_acceptance/vrt_service_process.ts",
  "tools/support/release/public_acceptance/vrt_trusted_color_build.ts",
  "tools/support/release/public_acceptance/vrt_trusted_color_capture.ts",
  "tools/support/release/public_acceptance/vrt_trusted_colors.ts",
  "tools/support/release/public_acceptance/vrt_trusted_failures.ts",
  "tools/support/release/public_acceptance/vrt_trusted_gallery.ts",
  "tools/support/release/public_acceptance/vrt_trusted_network.ts",
  "tools/support/release/public_acceptance/vrt_trusted_observation.ts",
  "tools/support/release/public_acceptance/vrt_trusted_report_isolation.ts",
  "tools/support/release/public_acceptance/vrt_trusted_surface.ts",
  "tools/support/release/public_acceptance/vrt_trusted_ui.ts",
  "tools/support/release/public_acceptance/vrt_tunnel.ts",
  "tools/support/release/public_acceptance/vrt_tunnel_metadata.ts",
  "tools/support/release/public_acceptance/vrt_tunnel_process.ts",
  "tools/support/release/public_acceptance/vrt_tunnel_setup.ts",
  "tools/support/release/windows_hover_acceptance.ts",
] as const;

export interface OraclePolicy {
  consumerSha256: string;
  denoConfigSha256: string;
  custodyProbeSha256: string;
}
function expected(policy: OraclePolicy, deno: boolean) {
  same(policy.custodyProbeSha256, sha256(custodyProbe), "reviewed custody oracle required");
  return {
    "consumer.ts": policy.consumerSha256,
    "custody.ts": policy.custodyProbeSha256,
    ...(deno ? { "deno.json": sha256(effectiveDenoConfig) } : {}),
  };
}
export function verifyOracleHashes(actual: unknown, policy: OraclePolicy, deno: boolean) {
  same(actual, expected(policy, deno), "immutable consumer/config/custody oracle changed");
}
/** Used in the live project around every command, and on retained files before/after signing. */
export function assertOracleFiles(directory: string, policy: OraclePolicy, deno: boolean) {
  const hashes: Record<string, string> = {};
  for (const name of Object.keys(expected(policy, deno))) {
    const file = path.join(directory, name);
    assert.ok(
      lstatSync(file).isFile() && !lstatSync(file).isSymbolicLink(),
      "normal oracle file required",
    );
    hashes[name] = sha256(readFileSync(file));
  }
  verifyOracleHashes(hashes, policy, deno);
  return hashes;
}

export const custodyProbe = String.raw`
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
const [root, version, output, cache] = process.argv.slice(2);
const require = createRequire(import.meta.url);
const native = await import(root + "/native");
const result = native.compileSfc("<template><p>custody</p></template>", {filename:"Custody.vue"});
assert.equal(result.errors.length, 0);
const bindings = Object.entries(require.cache).filter(([name, module]) =>
  name.endsWith(".node") && module?.loaded && name.includes("vize"));
assert.equal(bindings.length, 1, "one actual loaded public Vize native provider required");
const entry = fs.realpathSync(bindings[0][0]);
const packages = [];
const wanted = new Set(["vize", "@vizejs/native", ...(cache ? [] : ["@vizejs/vite-plugin"])]);
let cursor = path.dirname(entry);
while (cursor !== path.dirname(cursor)) {
  const manifest = path.join(cursor, "package.json");
  if (fs.existsSync(manifest)) {
    const value = JSON.parse(fs.readFileSync(manifest, "utf8"));
    if (value.name?.startsWith("@vizejs/native-")) {
      wanted.add(value.name);
      break;
    }
  }
  cursor = path.dirname(cursor);
}
function walk(directory, depth = 0) {
  if (depth > 8) return;
  const manifest = path.join(directory, "package.json");
  if (fs.existsSync(manifest)) {
    const value = JSON.parse(fs.readFileSync(manifest, "utf8"));
    if (wanted.has(value.name)) {
      assert.equal(value.version, version, value.name);
      packages.push({name:value.name, version:value.version, directory:fs.realpathSync(directory),
        manifestSha256:createHash("sha256").update(fs.readFileSync(manifest)).digest("hex")});
      wanted.delete(value.name);
    }
    // npm cache packages have their own dependencies but never contain our other packages.
    if (!directory.endsWith("node_modules") && !directory.includes("node_modules/.")) return;
  }
  for (const item of fs.readdirSync(directory, {withFileTypes:true})) {
    if (item.isDirectory() && item.name !== ".bin") walk(path.join(directory,item.name),depth+1);
  }
}
walk(cache || path.join(process.cwd(),"node_modules"));
assert.equal(wanted.size, 0, "every public Vize package manifest must be observed");
assert.ok(packages.some(p => p.name.startsWith("@vizejs/native-") &&
  entry.startsWith(p.directory + path.sep)), "loaded binary belongs to public native package");
fs.writeFileSync(output, JSON.stringify({packages,native:{path:entry,loaded:true,
  sha256:createHash("sha256").update(fs.readFileSync(entry)).digest("hex")},
  runtime:{node:process.version,deno:globalThis.Deno?.version,bun:process.versions.bun,
    platform:process.platform,arch:process.arch}},null,2)+"\n");
`;
