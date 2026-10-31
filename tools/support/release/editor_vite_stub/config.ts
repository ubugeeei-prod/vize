import assert from "node:assert/strict";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
  type JsonRecord,
} from "../../json-value.ts";
import { cleanEnv, npmArgv, Recorder, success } from "../../config/windows_consumer/common.ts";
import { digest, inside, inventory, writeFixture } from "./filesystem.ts";
import { hook, observed, verifyJournal } from "./custody.ts";
import { auditConfigCache, cacheSnapshot } from "./cold-cache.ts";

/** The reported command is npx, resolved from the actual npm shipped with Node. */
export function npxCommand(record: Recorder): string[] {
  const root = realpathSync(resolve(dirname(npmArgv()[1]), ".."));
  const manifestPath = join(root, "package.json"),
    manifest = readJsonRecord(manifestPath);
  assert.equal(manifest.name, "npm");
  const declared = asString(asRecord(manifest.bin).npx),
    lexical = resolve(root, declared);
  assert.ok(inside(root, lexical));
  const entry = realpathSync(lexical);
  assert.ok(inside(root, entry) && statSync(entry).isFile());
  record.json("npx-identity.json", {
    npmVersion: asString(manifest.version),
    manifest: { path: manifestPath, ...digest(manifestPath) },
    entry: { path: entry, ...digest(entry) },
    node: process.execPath,
  });
  return [process.execPath, entry];
}
export function bindLocalNpx(
  root: string,
  target: { packageName: string; bin: string; version: string; entry: string },
  record: Recorder,
) {
  const packageRoot = realpathSync(join(root, "node_modules", target.packageName));
  const manifest = readJsonRecord(join(packageRoot, "package.json"));
  assert.equal(manifest.name, target.packageName);
  assert.equal(manifest.version, target.version);
  const declared =
    typeof manifest.bin === "string" ? manifest.bin : asString(asRecord(manifest.bin)[target.bin]);
  assert.equal(realpathSync(resolve(packageRoot, declared)), target.entry);
  const shim = join(
    root,
    "node_modules/.bin",
    target.bin + (process.platform === "win32" ? ".cmd" : ""),
  );
  assert.ok(existsSync(shim));
  const expected = digest(target.entry),
    stat = lstatSync(shim);
  let shimBytes: string | null = null;
  if (stat.isSymbolicLink()) assert.equal(realpathSync(shim), target.entry);
  else {
    assert.ok(stat.isFile());
    const raw = readFileSync(shim);
    shimBytes = raw.toString("base64");
    const text = raw.toString("utf8").replaceAll("\\", "/");
    const candidates = [...text.matchAll(/\.\.\/([^"'\s]+)/g)].map((match) =>
      resolve(dirname(shim), "../" + match[1]),
    );
    if (text.includes(target.entry.replaceAll("\\", "/"))) candidates.push(target.entry);
    assert.ok(
      candidates.some(
        (candidate) =>
          existsSync(candidate) &&
          inside(packageRoot, realpathSync(candidate)) &&
          statSync(candidate).isFile() &&
          digest(candidate).sha256 === expected.sha256,
      ),
      "actual local shim must reference the exact installed CLI bytes",
    );
  }
  const cache = join(record.root, "isolated-npx-cache");
  mkdirSync(cache);
  assert.equal(inventory(cache).length, 1);
  const env = {
    ...cleanEnv(),
    NPM_CONFIG_OFFLINE: "true",
    npm_config_offline: "true",
    NPM_CONFIG_CACHE: cache,
    npm_config_cache: cache,
    PATH: [join(root, "node_modules/.bin"), dirname(process.execPath), process.env.PATH ?? ""].join(
      delimiter,
    ),
  };
  record.json("npx-local-binding.json", {
    target,
    packageRoot,
    declared,
    shim: { path: shim, ...digest(shim), rawBase64: shimBytes },
    expected,
    cache,
    environmentAdaptations: { offline: true, freshEmptyNpmCache: true, localBinFirst: true },
    assertion:
      "bundled npx, existing exact local package/bin bytes and fresh offline cache prevent a downloaded/cached replacement",
  });
  return { command: npxCommand(record), env, cache };
}
export function npxCacheSnapshot(cache: string) {
  const rows = inventory(cache);
  assert.equal(
    rows.some((row) => /(?:^|\/)node_modules(?:\/|$)/.test(row.path)),
    false,
    "npx must not install or use a cached replacement package",
  );
  return rows;
}

export function configAcceptance(
  root: string,
  identity: JsonRecord,
  original: JsonRecord,
  record: Recorder,
): void {
  const modules = join(root, "node_modules"),
    stub = join(modules, "vite");
  const files = asRecord(original.files);
  const npx = bindLocalNpx(
    root,
    {
      packageName: "vize",
      bin: "vize",
      version: asString(identity.cliVersion),
      entry: asString(identity.cli),
    },
    record,
  );
  const protectedFiles = ["package.json", "pnpm-lock.yaml", "vite.config.ts", "src/app.vue"].map(
    (name) => [name, readFileSync(join(root, name))] as const,
  );
  function phase(label: string, negative = false) {
    const before = inventory(modules);
    record.json(label + ".before.json", before);
    const version = asString(identity.cliVersion);
    const plainBefore = cacheSnapshot(root, version, record, label + "-npx-before");
    const plain = record.run(
      label + "-original-cli",
      [...npx.command, "vize", "lint", "src/app.vue"],
      root,
      npx.env,
    );
    record.json(label + ".npx-cache.json", npxCacheSnapshot(npx.cache));
    auditConfigCache(
      root,
      plainBefore,
      cacheSnapshot(root, version, record, label + "-npx-after"),
      record,
      label + "-original-cli",
    );
    const cliCustody = observed(root, identity, record, label + "-cli");
    const cliBefore = cacheSnapshot(root, version, record, label + "-direct-before");
    const cli = record.run(
      label + "-json-cli",
      [
        process.execPath,
        "--require",
        hook,
        asString(identity.cli),
        "lint",
        "src/app.vue",
        "--format",
        "json",
      ],
      root,
      cliCustody.env,
    );
    auditConfigCache(
      root,
      cliBefore,
      cacheSnapshot(root, version, record, label + "-direct-after"),
      record,
      label + "-direct-cli",
    );
    const apiCustody = observed(root, identity, record, label + "-api");
    const output = join(record.root, label + ".api.json");
    const apiBefore = inventory(modules);
    const api = record.run(
      label + "-api",
      [
        process.execPath,
        "--require",
        hook,
        fileURLToPath(new URL("./config-probe.ts", import.meta.url)),
        root,
        output,
      ],
      root,
      apiCustody.env,
    );
    assert.deepEqual(
      inventory(modules),
      apiBefore,
      "fresh JS API actor must preserve the entire installation",
    );
    const value = readJsonRecord(output);
    if (negative) {
      for (const packet of [plain, cli, api]) {
        assert.equal(packet.status, packet === api ? 1 : 2, packet.label);
        assert.equal(packet.error, null);
        assert.equal(packet.signal, null);
        assert.ok((packet.stdout + packet.stderr).includes("runtime-owner-11145"), packet.label);
      }
      assert.equal(asRecord(value.error).name, "Error");
      assert.equal(asRecord(value.error).message, "runtime-owner-11145");
    } else {
      for (const packet of [plain, cli, api]) success(packet);
      const report = asArray(parseJson(cli.stdout));
      assert.equal(report.length, 1, "the original app must actually be linted");
      for (const item of report) {
        const row = asRecord(item);
        assert.equal(asString(row.file).replaceAll("\\", "/"), "src/app.vue");
        assert.equal(row.errorCount, 0);
        assert.equal(row.warningCount, 0);
        assert.deepEqual(row.messages, []);
      }
      assert.notEqual(value.value, null);
      assert.ok(Object.hasOwn(value, "value"));
      assert.equal(
        asRecord(asRecord(value.value).typeChecker).jsxTypecheck,
        true,
        "the original project export must produce its actual project defaults",
      );
    }
    // Rust's lint config loader exits(2) directly, bypassing Node's exit event.
    // The actual process status is checked above; its successful .node load remains required.
    verifyJournal(cliCustody.journal, identity, negative ? null : 0);
    // A runtime-owner import error precedes the JS API's native normalizer.
    // Keep that honest journal; do not preload a binding to fabricate a native call.
    verifyJournal(apiCustody.journal, identity, negative ? 1 : 0, !negative);
    const after = inventory(modules);
    record.json(label + ".after.json", after);
    for (const [name, bytes] of protectedFiles)
      assert.deepEqual(readFileSync(join(root, name)), bytes, label + " changed " + name);
    for (const name of ["vite.config.ts", "src/app.vue"])
      assert.equal(readFileSync(join(root, name), "utf8"), files[name]);
    return { config: value, report: negative ? null : parseJson(cli.stdout) };
  }
  const baseline = phase("baseline");
  mkdirSync(stub);
  writeFixture(root, files, (name) => name.startsWith("node_modules/"));
  const withStub = phase("types-only");
  assert.deepEqual(
    withStub,
    baseline,
    "full CLI report and imported config must survive the exact original stub",
  );
  writeFileSync(
    join(stub, "package.json"),
    '{"name":"vite","type":"module","main":"./index.mjs"}\n',
  );
  writeFileSync(
    join(stub, "index.mjs"),
    'export function loadConfigFromFile(){throw new Error("runtime-owner-11145")}\n',
  );
  phase("runtime-owner", true);
  rmSync(join(stub, "index.mjs"));
  writeFixture(root, files, (name) => name.startsWith("node_modules/"));
  assert.deepEqual(phase("stub-repaired"), baseline);
  rmSync(stub, { recursive: true });
  assert.deepEqual(phase("removed-repaired"), baseline);
  record.json("config-acceptance.json", {
    originalCommand: [...npx.command, "vize", "lint", "src/app.vue"],
    originalReportedCommand: "npx vize lint src/*.vue",
    adaptations: [
      "single original file glob expansion",
      "fresh offline npm cache and bound local bin prevent replacement fallback",
      "additional direct Node JSON CLI/native custody/API observations",
      "separate tooling-only PNPM fixture explains previous adapted-wrapper metadata; the original consumer is not primed by PNPM exec",
    ],
    originalGlobExpansion: "src/*.vue expands to the single original src/app.vue",
    phases: ["baseline", "types-only", "runtime-owner", "stub-repaired", "removed-repaired"],
    baseline,
    assertion:
      "complete positive CLI/config objects equal; runtime authority error is fatal; per-actor complete vectors and declared cache transitions audited; every package/Vite byte/link exact; entire JS API and editor vectors immutable",
  });
}
