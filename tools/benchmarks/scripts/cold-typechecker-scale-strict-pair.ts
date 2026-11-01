/** One unchanged-program import-fact experiment; no code/type/map migration is admitted. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
export interface SdkScanner {
  version: string;
  preProcessFile(
    text: string,
    readImports: boolean,
    detectJsImports: boolean,
  ): {
    referencedFiles: { fileName: string }[];
    importedFiles: { fileName: string }[];
  };
}

export const strictPairSources = {
  base: "ec8474e0ea9ed05062542c57c857f36b9467ac15",
  head: "d2579c761944c5a1480f74443188bb4ba10c3913",
} as const;
export const strictPairPaths = [
  ".gitattributes",
  ".github/workflows/setup-binding-anchor-profile.yml",
  "crates/vize_canon/src/virtual_ts/generator.rs",
  "crates/vize_canon/src/virtual_ts/generator/anchors.rs",
  "crates/vize_canon/src/virtual_ts/generator/anchors/value_import_names.rs",
  "crates/vize_canon/src/virtual_ts/generator/imports.rs",
  "crates/vize_canon/src/virtual_ts/generator/type_only_imports.rs",
  "crates/vize_canon/src/virtual_ts/generator/type_only_imports/prepared_tests.rs",
  "crates/vize_canon/src/virtual_ts/generator/type_only_imports/prepared_tests/reference.rs",
  "tests/_fixtures/differential/typechecker/manifest.json",
  "tests/_fixtures/differential/typechecker/prepared-type-only-import-facts/Imports.vue.txt",
  "tests/tooling/differential-dialect-coverage.test.ts",
  "tests/tooling/setup-binding-anchor-evidence.test.ts",
  "tools/benchmarks/crates/vize/tests/setup_binding_anchor_profile.rs",
  "tools/benchmarks/scripts/setup-binding-anchor-evidence.ts",
] as const;
const cargoIdentity = [
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
] as const;
const files = [
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "virtual-ts-slots.d.ts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
];
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const git = (root: string, args: string[]) =>
  execFileSync("git", args, { cwd: root }).toString().trim();

export function prepareStrictOutput(output: string) {
  assert(path.isAbsolute(output));
  fs.mkdirSync(output); // No recursive/reuse mode: preserve every prior attempt.
}

export function verifyStrictSourceInputs(input: {
  sources: { base: string; head: string };
  changedPaths: string[];
  cargo: {
    base: Record<"Cargo.toml" | "Cargo.lock", Buffer>;
    head: Record<"Cargo.toml" | "Cargo.lock", Buffer>;
  };
}) {
  assert.deepEqual(input.sources, strictPairSources);
  assert.deepEqual(input.changedPaths, [...strictPairPaths]);
  for (const role of ["base", "head"] as const)
    for (const expected of cargoIdentity) {
      const body = input.cargo[role][expected.file];
      assert.equal(body.length, expected.bytes, `${role} whole ${expected.file} length`);
      assert.equal(hash(body), expected.sha256, `${role} whole SDK4 ${expected.file} identity`);
    }
  return { sdkVersion: "2.0.0-beta.4", files: cargoIdentity };
}

export function verifySourceSdkPair(base: string, head: string, typescript: SdkScanner) {
  assert.equal(typescript.version, "6.0.3");
  const edges: { file: string; specifier: string; target: string }[] = [];
  const declarations = files.map((file) => {
    const before = fs.readFileSync(path.join(base, "npm/cli", file));
    const after = fs.readFileSync(path.join(head, "npm/cli", file));
    assert(before.equals(after), `whole source SDK declaration changed: ${file}`);
    const imports = typescript.preProcessFile(after.toString("utf8"), true, true);
    assert.equal(imports.referencedFiles.length, 0);
    for (const reference of imports.importedFiles) {
      if (!reference.fileName.startsWith(".")) continue;
      assert(reference.fileName.endsWith(".js"));
      const target = path.posix.normalize(
        path.posix.join(path.posix.dirname(file), reference.fileName.slice(0, -3) + ".d.ts"),
      );
      assert(files.includes(target), "SDK relative edge escaped the ten-file source closure");
      edges.push({ file, specifier: reference.fileName, target });
    }
    return { file, bytes: before.length, sha256: hash(before) };
  });
  assert.equal(edges.length, 13);
  const before = fs.readFileSync(path.join(base, "npm/cli/package.json"));
  assert(before.equals(fs.readFileSync(path.join(head, "npm/cli/package.json"))));
  return {
    declarations,
    edges,
    packageManifest: { bytes: before.length, sha256: hash(before) },
    scope: "whole current source SDK identity, not installed-public consumer qualification",
  };
}

export function verifyStrictSources(base: string, head: string, provider: string) {
  const readCargo = (root: string) => ({
    "Cargo.toml": fs.readFileSync(path.join(root, "Cargo.toml")),
    "Cargo.lock": fs.readFileSync(path.join(root, "Cargo.lock")),
  });
  const cargo = verifyStrictSourceInputs({
    sources: { base: git(base, ["rev-parse", "HEAD"]), head: git(head, ["rev-parse", "HEAD"]) },
    changedPaths: git(base, [
      "diff",
      "--name-only",
      strictPairSources.base,
      strictPairSources.head,
    ]).split("\n"),
    cargo: { base: readCargo(base), head: readCargo(head) },
  });
  for (const root of [base, head])
    assert.equal(git(root, ["status", "--porcelain", "--untracked-files=all"]), "");
  const require = createRequire(path.join(provider, "package.json"));
  return {
    sources: strictPairSources,
    changedPaths: strictPairPaths,
    cargo,
    sdk: verifySourceSdkPair(base, head, require("typescript") as SdkScanner),
    scope:
      "only three import-fact production paths and twelve reviewed support paths differ; exact whole SDK4 Cargo identity, canonical helpers and native launch policy unchanged",
  };
}

/** output.rs prints one program in absolute UTF-8 byte order, separately from JSON display order. */
export function verifyStrictDebug(stderr: Buffer, value: unknown, cwd: string) {
  assert(value && typeof value === "object" && !Array.isArray(value));
  const packet = value as {
    programs: unknown[];
    files: { file: string; virtualTs?: string | null }[];
  };
  assert(Array.isArray(packet.programs) && packet.programs.length === 1);
  assert(Array.isArray(packet.files));
  const suffix = Buffer.from(
    packet.files
      .filter((file) => {
        assert.equal(typeof file.file, "string");
        assert(file.virtualTs == null || typeof file.virtualTs === "string");
        return typeof file.virtualTs === "string";
      })
      .toSorted((a, b) =>
        Buffer.compare(
          Buffer.from(path.resolve(cwd, a.file)),
          Buffer.from(path.resolve(cwd, b.file)),
        ),
      )
      .map((file) => `\n=== ${path.resolve(cwd, file.file)} ===\n${file.virtualTs}\n`)
      .join(""),
  );
  const header = Buffer.from("\n=== __vize_helpers.d.ts ===\n");
  assert(suffix.length > 0 && stderr.length > header.length + suffix.length);
  assert(stderr.subarray(0, header.length).equals(header));
  assert(
    stderr.subarray(-suffix.length).equals(suffix),
    `whole generated debug suffix mismatch: ${stderr.length}/${hash(stderr)}`,
  );
  return {
    bytes: stderr.length,
    sha256: hash(stderr),
    helperSha256: hash(stderr.subarray(header.length, -suffix.length)),
  };
}
