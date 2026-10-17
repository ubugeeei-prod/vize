import assert from "node:assert/strict";
import { lstatSync, readFileSync } from "node:fs";
import path from "node:path";
import { sha256, same } from "./policy.ts";
import { effectiveDenoConfig } from "./runtime.ts";

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
