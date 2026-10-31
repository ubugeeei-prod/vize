/** One unchanged-program ownership experiment; no code/type/map migration is admitted. */
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
  base: "a193c99c824b54fbe7e900bb88a19f549cd25ea7",
  head: "468c9beb2437029fed15746a20b8373307b3925e",
} as const;
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
  assert.equal(git(base, ["rev-parse", "HEAD"]), strictPairSources.base);
  assert.equal(git(head, ["rev-parse", "HEAD"]), strictPairSources.head);
  assert.deepEqual(
    git(base, ["diff", "--name-only", strictPairSources.base, strictPairSources.head]).split("\n"),
    [
      "crates/vize/src/commands/check/imports.rs",
      "crates/vize/src/commands/check/imports/deferred.rs",
      "crates/vize/src/commands/check/imports/deferred/tests.rs",
    ],
  );
  for (const root of [base, head])
    assert.equal(git(root, ["status", "--porcelain", "--untracked-files=all"]), "");
  const require = createRequire(path.join(provider, "package.json"));
  return {
    sources: strictPairSources,
    sdk: verifySourceSdkPair(base, head, require("typescript") as SdkScanner),
    scope:
      "only the three route-ownership Rust files differ; Cargo, canonical helpers and native launch policy unchanged",
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
