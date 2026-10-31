import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type VersionResult = {
  status: number | null;
  signal?: string | null;
  stdout: string;
  pid?: number;
};
type VersionProbe = (command: string, args: string[], cwd: string) => VersionResult;
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const inside = (root: string, file: string) => {
  const relative = path.relative(root, file);
  return (
    relative !== "" &&
    relative !== ".." &&
    !relative.startsWith(`..${path.sep}`) &&
    !path.isAbsolute(relative)
  );
};
function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected object");
  return value as Record<string, unknown>;
}
function readRecord(file: string) {
  return record(JSON.parse(fs.readFileSync(file, "utf8")) as unknown);
}
function identity(file: string) {
  const before = fs.statSync(file, { bigint: true });
  const body = fs.readFileSync(file);
  const after = fs.statSync(file, { bigint: true });
  for (const field of ["dev", "ino", "size", "mtimeNs"] as const)
    assert.equal(after[field], before[field], "provider changed while reading");
  return { path: file, bytes: body.length, sha256: hash(body), body };
}
function nativeFormat(body: Buffer): string | null {
  const magic = body.subarray(0, 4).toString("hex");
  if (magic === "7f454c46") return "ELF";
  if (body.subarray(0, 2).toString() === "MZ") return "PE";
  if (
    [
      "feedface",
      "feedfacf",
      "cefaedfe",
      "cffaedfe",
      "cafebabe",
      "bebafeca",
      "cafebabf",
      "bfbafeca",
    ].includes(magic)
  )
    return "Mach-O";
  return null;
}
function declaredScript(entry: string, expectedVersion: string) {
  for (let root = path.dirname(entry); ; root = path.dirname(root)) {
    const manifestPath = path.join(root, "package.json");
    if (fs.existsSync(manifestPath)) {
      const manifest = readRecord(manifestPath);
      assert.equal(manifest.name, "pnpm", "active script must belong to pnpm");
      assert.equal(manifest.version, expectedVersion, "pnpm manifest version mismatch");
      const declared = record(manifest.bin).pnpm;
      assert.equal(typeof declared, "string", "pnpm must declare its bin entry");
      const lexical = path.resolve(root, declared as string);
      assert.ok(inside(root, lexical), "declared pnpm entry escaped its package");
      assert.equal(lexical, entry, "active script is not the declared pnpm entry");
      assert.equal(fs.realpathSync(manifestPath), manifestPath, "pnpm manifest must be owned");
      const { body: _body, ...manifestIdentity } = identity(manifestPath);
      return manifestIdentity;
    }
    assert.notEqual(path.dirname(root), root, "active script has no owning pnpm manifest");
  }
}
const versionProbe: VersionProbe = (command, args, cwd) => {
  const result = spawnSync(command, args, {
    cwd,
    env: process.env,
    encoding: "utf8",
    timeout: 120_000,
  });
  if (result.error) throw result.error;
  return { status: result.status, signal: result.signal, stdout: result.stdout, pid: result.pid };
};

/** Use the manager that launched the package script, never the PATH's Vite+ shim. */
export function resolvePnpmPackProvider(
  workspaceRoot: string,
  execpath: string | undefined,
  probe: VersionProbe = versionProbe,
  observe: (value: unknown) => void = () => {},
) {
  const workspaceManifest = path.join(workspaceRoot, "package.json");
  const manager = readRecord(workspaceManifest).packageManager;
  assert.equal(typeof manager, "string", "workspace must declare its package manager");
  const declaration = /^pnpm@(\d+\.\d+\.\d+)$/.exec(manager as string);
  assert.ok(declaration, "workspace must pin an exact pnpm version");
  const expectedVersion = declaration[1];
  assert.ok(execpath, "npm_execpath is missing from the package-test task environment");
  assert.ok(
    path.isAbsolute(execpath),
    "npm_execpath must identify the active absolute pnpm provider",
  );
  const entry = path.resolve(execpath);
  assert.ok(
    fs.lstatSync(entry).isFile(),
    "active pnpm provider must be a regular file, not a shim symlink",
  );
  assert.equal(fs.realpathSync(entry), entry, "active pnpm provider must be its physical path");
  const { body, ...image } = identity(entry);
  const format = nativeFormat(body);
  let manifest: ReturnType<typeof declaredScript> | undefined;
  if (!format) {
    manifest = declaredScript(entry, expectedVersion);
    assert.ok(
      /\.[cm]?js$/.test(entry) || /^#![^\r\n]*\bnode(?:\s|$)/.test(body.toString("utf8")),
      "declared pnpm script must identify JavaScript",
    );
  }
  const command = format ? entry : process.execPath;
  const prefix = format ? [] : [entry];
  const { body: _workspaceBody, ...workspace } = identity(workspaceManifest);
  const providerIdentity = {
    declaration: manager,
    version: expectedVersion,
    workspace,
    format: format ?? "owned JavaScript",
    image,
    manifest,
    node: format ? undefined : process.execPath,
  };
  observe({ phase: "selected-before-version", versionVerified: false, provider: providerIdentity });
  const result = probe(command, [...prefix, "--version"], workspaceRoot);
  observe({
    phase: "version-result",
    status: result.status,
    signal: result.signal ?? null,
    pid: result.pid ?? null,
    matchesPin: result.stdout.trim() === expectedVersion,
    stdout: { bytes: Buffer.byteLength(result.stdout), sha256: hash(Buffer.from(result.stdout)) },
  });
  assert.equal(result.status, 0, "active pnpm version probe failed");
  assert.ok(!result.signal, "active pnpm version probe was signalled");
  assert.equal(
    result.stdout.trim(),
    expectedVersion,
    "active pnpm version differs from the workspace pin",
  );
  const { body: _afterBody, ...after } = identity(entry);
  assert.deepEqual(after, image, "active pnpm provider changed during version probe");
  return { command, prefix, identity: providerIdentity };
}

export function packWithPnpm(
  workspaceRoot: string,
  packageRoot: URL,
  packDir: string,
  timeout: number,
) {
  const provider = resolvePnpmPackProvider(
    workspaceRoot,
    process.env.npm_execpath,
    versionProbe,
    (value) => console.log(`VIZE_PNPM_PACK_SELECTION ${JSON.stringify(value)}`),
  );
  console.log(`VIZE_PNPM_PACK_PROVIDER ${JSON.stringify(provider.identity)}`);
  try {
    const output = execFileSync(
      provider.command,
      [...provider.prefix, "pack", "--pack-destination", packDir],
      { cwd: packageRoot, env: process.env, stdio: "pipe", timeout },
    );
    console.log('VIZE_PNPM_PACK_OUTCOME {"status":0,"signal":null,"pid":null}');
    return output;
  } catch (error) {
    if (error && typeof error === "object") {
      const result = error as Record<string, unknown>;
      console.log(
        `VIZE_PNPM_PACK_OUTCOME ${JSON.stringify({
          status: typeof result.status === "number" ? result.status : null,
          signal:
            typeof result.signal === "string" && /^SIG[A-Z0-9]+$/.test(result.signal)
              ? result.signal
              : null,
          pid: typeof result.pid === "number" ? result.pid : null,
        })}`,
      );
    }
    throw error;
  }
}
