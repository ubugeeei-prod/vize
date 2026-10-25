import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const experimentalRouterFixture = fileURLToPath(
  new URL("../../fixtures/typechecker/experimental-router-5.4/", import.meta.url),
);
export const experimentalRouterPage = "src/pages/products/[id=int].vue";

type FileIdentity = { path: string; sha256: string; bytes: number };
type Provider = {
  schema: string;
  version: number;
  issue: number;
  provider: {
    name: string;
    version: string;
    tarball: string;
    integrity: string;
    archiveSha256: string;
  };
  providerFiles: FileIdentity[];
  files: FileIdentity[];
};

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function readExperimentalRouterFixture(directory = experimentalRouterFixture) {
  const authority = JSON.parse(
    fs.readFileSync(path.join(directory, "provider.json"), "utf8"),
  ) as Provider;
  assert.equal(authority.schema, "vize.router-experimental-provider");
  assert.equal(authority.version, 1);
  assert.equal(authority.provider.name, "vue-router");
  assert.equal(authority.provider.version, "5.4.0");
  assert.equal(
    authority.provider.integrity,
    "sha512-pFvcYcxinDHjZUlf7AzFBrT8r8Xqvv4uMHoJnjc+EqPawXQvHTkF8S4tfSmQLpriVHUGfkaKlI7Et0KRjTMTdw==",
  );
  const files = new Map<string, Buffer>();
  for (const entry of authority.files) {
    assert.ok(
      entry.path && !path.isAbsolute(entry.path) && !entry.path.split(/[\\/]/u).includes(".."),
    );
    assert.equal(files.has(entry.path), false, `duplicate fixture input ${entry.path}`);
    const bytes = fs.readFileSync(path.join(directory, "project", `${entry.path}.txt`));
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(sha256(bytes), entry.sha256, entry.path);
    files.set(entry.path, bytes);
  }
  const lock = JSON.parse(files.get("package-lock.json")!.toString("utf8"));
  assert.equal(lock.packages["node_modules/vue-router"].version, authority.provider.version);
  assert.equal(lock.packages["node_modules/vue-router"].integrity, authority.provider.integrity);
  return { authority, files };
}

/** Fresh pinned provider; the complete route declaration is produced by its real scanner. */
export async function prepareExperimentalRouterProject(outputRoot: string) {
  const { authority, files } = readExperimentalRouterFixture();
  fs.mkdirSync(outputRoot, { recursive: true });
  const root = fs.mkdtempSync(path.join(outputRoot, "router-experimental-"));
  const restore = () => {
    for (const [relative, bytes] of files) {
      const target = path.join(root, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, bytes);
    }
  };
  restore();
  const commands: unknown[] = [];
  const run = (label: string, executable: string, args: string[]) => {
    const result = spawnSync(executable, args, {
      cwd: root,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    const record = {
      label,
      executable,
      args,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
    commands.push(record);
    fs.writeFileSync(
      path.join(outputRoot, "provider-processes.json"),
      `${JSON.stringify(commands, null, 2)}\n`,
    );
    assert.ifError(result.error);
    assert.equal(result.signal, null, label);
    assert.equal(result.status, 0, `${label}: ${result.stderr}`);
  };
  run("locked-provider-install", process.platform === "win32" ? "npm.cmd" : "npm", [
    "ci",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--prefer-offline",
    "--registry=https://registry.npmjs.org",
  ]);
  const providerRoot = path.join(root, "node_modules/vue-router");
  for (const entry of authority.providerFiles) {
    const bytes = fs.readFileSync(path.join(providerRoot, entry.path));
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(sha256(bytes), entry.sha256, entry.path);
  }
  const dependencies = ["vue", "vue-router", "typescript", "vue-tsc"].map((name) => {
    const manifest = fs.readFileSync(path.join(root, "node_modules", name, "package.json"));
    const identity = JSON.parse(manifest.toString("utf8")) as { name: string; version: string };
    assert.equal(identity.name, name);
    return { name: identity.name, version: identity.version, manifestSha256: sha256(manifest) };
  });
  assert.deepEqual(
    dependencies.map(({ name, version }) => ({ name, version })),
    [
      { name: "vue", version: "3.5.35" },
      { name: "vue-router", version: "5.4.0" },
      { name: "typescript", version: "5.9.3" },
      { name: "vue-tsc", version: "3.3.13" },
    ],
  );
  run("official-route-generation", process.execPath, ["generate.mjs"]);
  const generatedDeclaration = fs.readFileSync(path.join(root, "typed-router.d.ts"), "utf8");
  assert.equal(
    generatedDeclaration,
    files.get("typed-router.d.ts")!.toString("utf8"),
    "complete officially generated declarations",
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(root, "package-lock.json"))),
    authority.files.find((entry) => entry.path === "package-lock.json")!.sha256,
  );
  fs.writeFileSync(
    path.join(outputRoot, "provider-identity.json"),
    `${JSON.stringify({ authority, dependencies, generatedDeclarationSha256: sha256(Buffer.from(generatedDeclaration)) }, null, 2)}\n`,
  );
  return {
    root,
    pagePath: path.join(root, experimentalRouterPage),
    pageSource: files.get(experimentalRouterPage)!.toString("utf8"),
    generatedDeclaration,
    dependencies,
    restore,
    cleanup() {
      fs.rmSync(path.join(root, "node_modules"), { recursive: true, force: true });
    },
  };
}
