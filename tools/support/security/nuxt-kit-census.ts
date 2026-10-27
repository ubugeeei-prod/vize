import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { installedBracesCensus } from "./braces-proof.ts";

const source = fileURLToPath(new URL("../../../", import.meta.url));
const root = process.argv[3] ? path.resolve(process.argv[3]) : source;
const artifacts = path.resolve(process.argv[2]);
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const identity = (directory: string) => {
  const bytes = fs.readFileSync(path.join(directory, "package.json"));
  const manifest = JSON.parse(bytes.toString("utf8")) as { name: string; version: string };
  return {
    directory,
    name: manifest.name,
    version: manifest.version,
    manifestSha256: hash(bytes),
    manifestBytes: bytes.toString("utf8"),
  };
};
// Resolve the public entry, then identify its owning manifest. Modern c12
// intentionally does not export its package.json subpath.
const resolve = (owner: string, name: string, actualName = name): string => {
  const entry = createRequire(path.join(owner, "package.json")).resolve(name);
  for (
    let directory = path.dirname(fs.realpathSync(entry));
    ;
    directory = path.dirname(directory)
  ) {
    const manifest = path.join(directory, "package.json");
    if (
      fs.existsSync(manifest) &&
      JSON.parse(fs.readFileSync(manifest, "utf8")).name === actualName
    )
      return directory;
    if (path.dirname(directory) === directory)
      throw new Error(`No owning manifest for ${name}: ${entry}`);
  }
};
fs.mkdirSync(artifacts, { recursive: true });
fs.writeFileSync(
  path.join(artifacts, "source-head.txt"),
  execFileSync("git", ["rev-parse", "HEAD"], { cwd: root }),
);
const census = installedBracesCensus(root);
fs.writeFileSync(
  path.join(artifacts, "installed-census-paths.json"),
  JSON.stringify(
    { instances: [...census.instances].sort(), consumers: [...census.consumers].sort() },
    null,
    2,
  ) + "\n",
);
const kit = resolve(path.join(root, "npm/framework/nuxt"), "@nuxt/kit");
const c12 = resolve(kit, "c12");
const chokidar = resolve(c12, "chokidar");
fs.mkdirSync(artifacts, { recursive: true });
fs.writeFileSync(
  path.join(artifacts, "installed-consumer-census.json"),
  JSON.stringify(
    {
      head: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
      root,
      lockSha256: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
      proofSourceSha256: hash(
        fs.readFileSync(path.join(root, "tools/support/security/braces-proof.ts")),
      ),
      executedProofSourceSha256: hash(
        fs.readFileSync(path.join(source, "tools/support/security/braces-proof.ts")),
      ),
      moduleKit: identity(kit),
      kitC12: identity(c12),
      kitChokidar: identity(chokidar),
      explicitDevOracle: fs.existsSync(path.join(root, "node_modules/chokidar-braces-oracle"))
        ? identity(resolve(root, "chokidar-braces-oracle", "chokidar"))
        : null,
      instances: [...census.instances].sort().map(identity),
      consumers: [...census.consumers].sort().map((directory) => ({
        ...identity(directory),
        braces: identity(resolve(directory, "braces")),
      })),
    },
    null,
    2,
  ) + "\n",
);
