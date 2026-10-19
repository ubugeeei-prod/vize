import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256, supportTargets, targets, tomlJson } from "./lsp_source.ts";

type Package = {
  id: string;
  name: string;
  version: string;
  source: string | null;
  manifest_path: string;
  targets: { name: string; kind: string[] }[];
};
type Metadata = {
  packages: Package[];
  resolve: {
    root: string;
    nodes: {
      id: string;
      deps: { pkg: string; dep_kinds: { kind: string | null }[] }[];
    }[];
  };
};
export function assertSupportGraph(
  metadata: Metadata,
  sourceTree: string,
  originalLock: string,
  expectedTargets = targets,
) {
  const locked = tomlJson(fs.readFileSync(originalLock, "utf8")).package;
  const allowed = new Map([
    ["vize_carton", "crates/vize_carton"],
    ["vize_l0", "davinci/vize_l0"],
    ["vize_l0_derive", "davinci/vize_l0_derive"],
  ]);
  const ids = new Set<string>();
  const visit = (id: string) => {
    if (ids.has(id)) return;
    ids.add(id);
    const pkg = metadata.packages.find((p) => p.id === id);
    assert.ok(pkg);
    if (id !== metadata.resolve.root) {
      if (pkg.source === null) {
        const relative = allowed.get(pkg.name);
        assert.ok(relative, `source product forbidden: ${pkg.name}`);
        assert.equal(
          fs.realpathSync(pkg.manifest_path),
          fs.realpathSync(path.join(sourceTree, relative, "Cargo.toml")),
        );
      } else
        assert.ok(
          locked.some(
            (entry: Package) =>
              entry.name === pkg.name &&
              entry.version === pkg.version &&
              entry.source === pkg.source,
          ),
          `dependency absent from H lock: ${pkg.id}`,
        );
    } else {
      assert.equal(pkg.name, "vize_public_lsp_contracts");
      assert.deepEqual(
        pkg.targets
          .map((t) => ({ name: t.name, kind: t.kind }))
          .sort((a, b) => a.name.localeCompare(b.name)),
        expectedTargets
          .map((name) => ({ name, kind: ["test"] }))
          .sort((a, b) => a.name.localeCompare(b.name)),
      );
    }
    const node = metadata.resolve.nodes.find((n) => n.id === id);
    assert.ok(node);
    for (const dep of node.deps)
      if (dep.dep_kinds.some((kind) => kind.kind === null || kind.kind === "build")) visit(dep.pkg);
  };
  visit(metadata.resolve.root);
  return [...ids].sort();
}

export function cargoCommand(
  args: string[],
  cwd: string,
  environment: NodeJS.ProcessEnv,
  output: string,
  name: string,
) {
  const stdout = fs.openSync(path.join(output, `${name}.stdout`), "wx");
  const stderr = fs.openSync(path.join(output, `${name}.stderr`), "wx");
  let result;
  try {
    result = spawnSync("cargo", args, {
      cwd,
      env: environment,
      timeout: 15 * 60_000,
      stdio: ["ignore", stdout, stderr],
    });
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
  }
  const receipt = {
    executable: "cargo",
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error && String(result.error),
    stdoutSha256: sha256(fs.readFileSync(path.join(output, `${name}.stdout`))),
    stderrSha256: sha256(fs.readFileSync(path.join(output, `${name}.stderr`))),
  };
  fs.writeFileSync(path.join(output, `${name}.json`), JSON.stringify(receipt, null, 2) + "\n", {
    flag: "wx",
  });
  assert.equal(result.error, undefined, name);
  assert.equal(result.signal, null, name);
  assert.equal(result.status, 0, name);
  return receipt;
}

export function freezeSupportLock(
  adapter: string,
  sourceTree: string,
  environment: NodeJS.ProcessEnv,
  output: string,
) {
  // Cargo metadata adds this new adapter root while preserving compatible H
  // lock entries. A wholesale generate-lockfile would unnecessarily update them.
  cargoCommand(
    [
      "metadata",
      "--format-version",
      "1",
      "--filter-platform",
      "x86_64-unknown-linux-gnu",
      "--manifest-path",
      path.join(adapter, "Cargo.toml"),
    ],
    adapter,
    environment,
    output,
    "cargo-lock",
  );
  cargoCommand(
    [
      "metadata",
      "--locked",
      "--format-version",
      "1",
      "--filter-platform",
      "x86_64-unknown-linux-gnu",
      "--manifest-path",
      path.join(adapter, "Cargo.toml"),
    ],
    adapter,
    environment,
    output,
    "cargo-metadata",
  );
  const metadata = JSON.parse(fs.readFileSync(path.join(output, "cargo-metadata.stdout"), "utf8"));
  const allowedBuildIds = assertSupportGraph(
    metadata,
    sourceTree,
    path.join(sourceTree, "Cargo.lock"),
    supportTargets,
  );
  const lock = fs.readFileSync(path.join(adapter, "Cargo.lock"));
  fs.writeFileSync(path.join(output, "adapter-Cargo.lock"), lock, { flag: "wx" });
  return {
    allowedBuildIds,
    lockSha256: sha256(lock),
    metadataSha256: sha256(fs.readFileSync(path.join(output, "cargo-metadata.stdout"))),
  };
}

export function assertBuiltSupport(stdout: string, allowed: string[]) {
  const messages = stdout
    .split("\n")
    .filter((line) => line.startsWith('{"reason":'))
    .map((line) => JSON.parse(line));
  const built = messages.filter(
    (m) => m.reason === "compiler-artifact" || m.reason === "build-script-executed",
  );
  assert.ok(built.length > 0, "actual compiler-artifact graph required");
  for (const message of built)
    assert.ok(
      allowed.includes(message.package_id),
      `compiled product outside support graph: ${message.package_id}`,
    );
  assert.ok(messages.some((m) => m.reason === "build-finished" && m.success === true));
  return built.map((message) => ({
    packageId: message.package_id,
    ...(Object.hasOwn(message, "target") ? { target: message.target } : {}),
    ...(Object.hasOwn(message, "filenames") ? { filenames: message.filenames } : {}),
  }));
}
