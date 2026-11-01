/** Finite current source/Cargo/SDK closure; no resolver or provider substitution. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";

export const currentProfileSource = {
  sha: "d2579c761944c5a1480f74443188bb4ba10c3913",
  tree: "ee0528edef279d84d86fec02a52ff64f419b44d1",
  cargo: [
    {
      file: "Cargo.toml",
      bytes: 13168,
      sha256: "6400cafce2abb1c16bb3c43dfc6498d0a0c695c88849bd5a6bdd4063b91377de",
    },
    {
      file: "Cargo.lock",
      bytes: 128818,
      sha256: "e9a8659e79a919541b6cf29340886dabad787a18fa3051bd77873fc3a76194a4",
    },
  ],
  sdk: [
    {
      file: "virtual-ts.d.ts",
      bytes: 15334,
      sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
    },
    {
      file: "virtual-ts.d.mts",
      bytes: 15334,
      sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
    },
    {
      file: "virtual-ts.d.cts",
      bytes: 15334,
      sha256: "d5e5eee803afd4fdb74912647d453322d36ce476daef487012eb1eca25f750df",
    },
    {
      file: "virtual-ts-globals.d.ts",
      bytes: 18966,
      sha256: "d647737ea035f166c94c131013af3265a3a33783dbad307b2eb3c6d558fc476a",
    },
    {
      file: "virtual-ts-patterns.d.ts",
      bytes: 12522,
      sha256: "e42c2c431553563663e28f75b4e68f8728d928a2867c857c76ef8cf30fe0f545",
    },
    {
      file: "virtual-ts-slots.d.ts",
      bytes: 4134,
      sha256: "3b2f3387f8e720bdffd4ed864b74af6fa19895a89ba6f2f64151fd8dd98bdf9f",
    },
    {
      file: "content-mapper-helpers.d.ts",
      bytes: 1109,
      sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
    },
    {
      file: "content-mapper-helpers.d.mts",
      bytes: 1109,
      sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
    },
    {
      file: "content-mapper-helpers.d.cts",
      bytes: 1109,
      sha256: "561fd0ad08013b3d2e80eb8255e7a55bfea529ec3f9e4c460b8fd5cbbcec24b0",
    },
    {
      file: "generated/content-mapper-types.d.ts",
      bytes: 11202,
      sha256: "7457fd86aafb4b2a5b1edbddedf538a45a072517645dd0ece2fd4c074a0f632e",
    },
  ],
  edges: [
    {
      file: "virtual-ts.d.ts",
      specifier: "./virtual-ts-globals.js",
      target: "virtual-ts-globals.d.ts",
    },
    {
      file: "virtual-ts.d.ts",
      specifier: "./virtual-ts-patterns.js",
      target: "virtual-ts-patterns.d.ts",
    },
    {
      file: "virtual-ts.d.ts",
      specifier: "./virtual-ts-slots.js",
      target: "virtual-ts-slots.d.ts",
    },
    {
      file: "virtual-ts.d.mts",
      specifier: "./virtual-ts-globals.js",
      target: "virtual-ts-globals.d.ts",
    },
    {
      file: "virtual-ts.d.mts",
      specifier: "./virtual-ts-patterns.js",
      target: "virtual-ts-patterns.d.ts",
    },
    {
      file: "virtual-ts.d.mts",
      specifier: "./virtual-ts-slots.js",
      target: "virtual-ts-slots.d.ts",
    },
    {
      file: "virtual-ts.d.cts",
      specifier: "./virtual-ts-globals.js",
      target: "virtual-ts-globals.d.ts",
    },
    {
      file: "virtual-ts.d.cts",
      specifier: "./virtual-ts-patterns.js",
      target: "virtual-ts-patterns.d.ts",
    },
    {
      file: "virtual-ts.d.cts",
      specifier: "./virtual-ts-slots.js",
      target: "virtual-ts-slots.d.ts",
    },
    {
      file: "virtual-ts-slots.d.ts",
      specifier: "./generated/content-mapper-types.js",
      target: "generated/content-mapper-types.d.ts",
    },
    {
      file: "content-mapper-helpers.d.ts",
      specifier: "./generated/content-mapper-types.js",
      target: "generated/content-mapper-types.d.ts",
    },
    {
      file: "content-mapper-helpers.d.mts",
      specifier: "./generated/content-mapper-types.js",
      target: "generated/content-mapper-types.d.ts",
    },
    {
      file: "content-mapper-helpers.d.cts",
      specifier: "./generated/content-mapper-types.js",
      target: "generated/content-mapper-types.d.ts",
    },
  ],
  packageManifest: {
    bytes: 4062,
    sha256: "00845d6bef7571d1b554247e0b3fc5808557e910c10b331b63e970fb88919058",
  },
  sdkRoles: [
    {
      name: "corsa",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "a88d1e7cd3fa6919d9f2ab4b51de40645547606317ac3f04dfccc1ded0ce9dcc",
    },
    {
      name: "corsa_client",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "48683d933fc2fd2b2bf9f8563103c2bbcb249f0ec6a6ace1831644ca0a84d780",
    },
    {
      name: "corsa_core",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "c64b958bb6e54a0f6a13cde2f5df28ce95234b6bb90c289fd2a3b6027af951c7",
    },
    {
      name: "corsa_jsonrpc",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "5603d3a1ca6c9f0caadbf8e89c853b740f31f44e818a9c1a01f09c2b93da5c8f",
    },
    {
      name: "corsa_lsp",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "b6949c82fac11db1376c361c7e70a6ac6019dedce1d379110385229eb643ea25",
    },
    {
      name: "corsa_orchestrator",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "56a610d7d8fec2e576d461751bd7ccb7bf2348a030418b5dc6756473353e2c0c",
    },
    {
      name: "corsa_runtime",
      version: "2.0.0-beta.4",
      source: "registry+https://github.com/rust-lang/crates.io-index",
      checksum: "71b6fd0a075158abbb1c513e754eb73b368066bbccf5967c9a2be5d6305df637",
    },
  ],
} as const;
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
export function assertProfileBodies(input: {
  source: string;
  tree: string;
  cargo: { file: string; body: Buffer }[];
  sdk: { file: string; body: Buffer }[];
  packageManifest: Buffer;
  edges: unknown;
}) {
  const pin = currentProfileSource;
  assert.equal(input.source, pin.sha);
  assert.equal(input.tree, pin.tree);
  const identities = (rows: { file: string; body: Buffer }[]) =>
    rows.map(({ file, body }) => ({ file, bytes: body.length, sha256: hash(body) }));
  assert.deepEqual(identities(input.cargo), pin.cargo, "whole fixed Cargo SDK4 identity");
  assert.deepEqual(identities(input.sdk), pin.sdk, "whole ten-declaration source SDK identity");
  assert.deepEqual(
    { bytes: input.packageManifest.length, sha256: hash(input.packageManifest) },
    pin.packageManifest,
  );
  assert.deepEqual(input.edges, pin.edges, "all thirteen relative edges");
  return {
    source: input.source,
    tree: input.tree,
    cargo: pin.cargo,
    sdk: pin.sdk,
    sdkRoles: pin.sdkRoles,
    edges: input.edges,
    packageManifest: pin.packageManifest,
    textBodies: [
      ...input.cargo,
      ...input.sdk.map((row) => ({ ...row, file: "npm/cli/" + row.file })),
      { file: "npm/cli/package.json", body: input.packageManifest },
    ].map(({ file, body }) => ({
      file,
      body: new TextDecoder("utf-8", { fatal: true }).decode(body),
    })),
    scope: "current source availability, not installed-public/native-loaded declaration authority",
  };
}

export function verifyProfileSource(root: string, provider: string) {
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root }).toString().trim();
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  const require = createRequire(path.join(provider, "package.json"));
  const ts = require("typescript") as {
    version: string;
    preProcessFile: (
      body: string,
      readImports: boolean,
      detectJsImports: boolean,
    ) => { referencedFiles: unknown[]; importedFiles: { fileName: string }[] };
  };
  assert.equal(ts.version, "6.0.3");
  const edges: { file: string; specifier: string; target: string }[] = [];
  const sdk = currentProfileSource.sdk.map(({ file }) => {
    const body = fs.readFileSync(path.join(root, "npm/cli", file));
    const imports = ts.preProcessFile(body.toString("utf8"), true, true);
    assert.equal(imports.referencedFiles.length, 0);
    for (const reference of imports.importedFiles) {
      if (!reference.fileName.startsWith(".")) continue;
      assert(reference.fileName.endsWith(".js"));
      edges.push({
        file,
        specifier: reference.fileName,
        target: path.posix.normalize(
          path.posix.join(path.posix.dirname(file), reference.fileName.slice(0, -3) + ".d.ts"),
        ),
      });
    }
    return { file, body };
  });
  return assertProfileBodies({
    source: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    cargo: currentProfileSource.cargo.map(({ file }) => ({
      file,
      body: fs.readFileSync(path.join(root, file)),
    })),
    sdk,
    packageManifest: fs.readFileSync(path.join(root, "npm/cli/package.json")),
    edges,
  });
}

export function assertProfileBuild(
  build: Record<string, unknown>,
  binarySha256: string,
  custody: boolean,
) {
  assert.equal(build.sourceSha, currentProfileSource.sha);
  assert.equal(build.treeSha, currentProfileSource.tree);
  assert.equal(build.binarySha256, binarySha256);
  assert.equal(build.allocation, false);
  assert.equal(build.targetStartedEmpty, true);
  assert.deepEqual(
    build.manifests,
    Object.fromEntries(currentProfileSource.cargo.map((row) => [row.file, row.sha256])),
  );
  const artifact = build.cargoArtifact as { features: string[] };
  assert.deepEqual(
    artifact.features.toSorted(),
    ["default", "glyph", "maestro", ...(custody ? ["cache-custody"] : [])].sort(),
  );
  assert.equal(build.observerSourceSha, custody ? currentProfileSource.sha : undefined);
}
