import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readRawBlob } from "./plan.ts";

export const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export const targets = ["lsp_named_model_transactions_cli", "lsp_bound_event_casing_cli"];
export const atomicTarget = "lsp_component_attribute_authority_cli";
export const atomicTest =
  "library_refusal::original_emit_string_keeps_atomic_owned_updates_and_unsafe_name_refusals_without_file_writes";
export const supportTargets = [...targets, atomicTarget];
export function supportDependencyVersions(
  dependencies: Record<string, string | { version: string }>,
) {
  return Object.fromEntries(
    ["serde_json", "tempfile", "sha2", "lsp-types", "url"].map((name) => {
      const value = dependencies[name];
      assert.ok(value, `support dependency absent from H workspace: ${name}`);
      const version = typeof value === "string" ? value : value.version;
      assert.match(version, /^=\d+\.\d+\.\d+$/u);
      return [name, version];
    }),
  );
}
export const corpus = "tests/_fixtures/differential/lsp/event-rename/4075/";
const prefixes = [
  ...targets.flatMap((name) => [`crates/vize/tests/${name}.rs`, `crates/vize/tests/${name}/`]),
  "crates/vize/tests/support/lsp_process.rs",
  "crates/vize/tests/support/lsp_process/",
  "crates/vize/tests/fixtures/content_mapper_project/tsconfig.json",
  `${corpus}named-model/`,
  `${corpus}bound-emitter/`,
];
const atomicPrefixes = [
  `crates/vize/tests/${atomicTarget}.rs`,
  `crates/vize/tests/${atomicTarget}/`,
  "crates/vize/tests/lsp_library_rename_refusal_cli/",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/",
  "tests/_fixtures/differential/lsp/event-rename/8010/",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/",
];
export function python(script: string, args: string[]) {
  return JSON.parse(
    execFileSync("python3", ["-c", script, ...args], {
      maxBuffer: 64 * 1024 * 1024,
      timeout: 30_000,
      encoding: "utf8",
    }),
  );
}
export function sourceMetadata(root: string, head: string) {
  const read = (file: string) => readRawBlob(root, head, file);
  const metadata = python(
    `import json,sys,tomllib
cargo,toolchain=sys.argv[1:]
c=tomllib.loads(cargo); t=tomllib.loads(toolchain)
print(json.dumps({'package':c['workspace']['package'],'dependencies':c['workspace']['dependencies'],'toolchain':t['toolchain']['channel']}))`,
    [read("Cargo.toml").toString(), read("rust-toolchain.toml").toString()],
  );
  assert.match(metadata.toolchain, /^\d+\.\d+\.\d+$/u);
  const cli = JSON.parse(read("npm/cli/package.json").toString());
  const name = "@typescript/typescript-linux-x64";
  assert.equal(cli.optionalDependencies[name], "catalog:corsa-runtime");
  const catalog = read("pnpm-workspace.yaml").toString();
  const section = /^  corsa-runtime:\n([\s\S]*?)(?=^  [a-z]|$(?![\s\S]))/mu.exec(catalog)?.[1];
  const entries = [
    ...(section ?? "").matchAll(/^    "@typescript\/typescript-linux-x64": "([^"]+)"$/gmu),
  ];
  assert.equal(entries.length, 1);
  assert.match(entries[0][1], /^7\.\d+\.\d+$/u);
  return { ...metadata, provider: { name, version: entries[0][1] } };
}

type SourceIsolation = {
  sourceRoot: string;
  sourceTree: string;
  isolation: { checkout: string; consumer: string; output: string; markerSha256: string };
};
const ownershipFile = "source-owner.json";
const canonicalDirectory = (directory: string) => {
  assert.ok(path.isAbsolute(directory), "absolute actor root required");
  const stat = fs.lstatSync(directory);
  assert.ok(stat.isDirectory() && !stat.isSymbolicLink(), "normal actor directory required");
  assert.equal(fs.realpathSync(directory), directory, "canonical actor root required");
  return directory;
};
const contains = (parent: string, child: string) => {
  const relative = path.relative(parent, child);
  return (
    relative === "" ||
    (relative !== ".." && !relative.startsWith(".." + path.sep) && !path.isAbsolute(relative))
  );
};
const overlaps = (left: string, right: string) => contains(left, right) || contains(right, left);

/** Own a fresh source sibling so browser discovery sees only its authored consumer inputs. */
export function prepareLspSourceRoot(checkout: string, consumer: string, output: string) {
  for (const directory of [checkout, consumer, output]) canonicalDirectory(directory);
  assert.ok(
    !overlaps(consumer, checkout) && !overlaps(consumer, output),
    "isolated consumer root required",
  );
  const sourceRoot = fs.mkdtempSync(path.join(path.dirname(consumer), "vize-public-lsp-source-"));
  const sourceTree = path.join(sourceRoot, "lsp-source");
  fs.mkdirSync(sourceTree);
  const marker = Buffer.from(
    JSON.stringify({
      schema: "vize-public-lsp-source-owner-v1",
      sourceRoot,
      sourceTree,
      checkout,
      consumer,
      output,
    }) + "\n",
  );
  fs.writeFileSync(path.join(sourceRoot, ownershipFile), marker, { flag: "wx" });
  const preparation = {
    sourceRoot,
    sourceTree,
    isolation: { checkout, consumer, output, markerSha256: sha256(marker) },
  };
  assertLspSourceRoot(preparation, { checkout, consumer, output });
  return preparation;
}

export function assertLspSourceRoot(
  preparation: SourceIsolation,
  expected?: { checkout: string; consumer: string; output: string },
) {
  const { sourceRoot, sourceTree, isolation } = preparation;
  for (const directory of [
    sourceRoot,
    sourceTree,
    isolation.checkout,
    isolation.consumer,
    isolation.output,
  ])
    canonicalDirectory(directory);
  assert.match(path.basename(sourceRoot), /^vize-public-lsp-source-[A-Za-z0-9]+$/u);
  assert.equal(
    path.dirname(sourceRoot),
    path.dirname(isolation.consumer),
    "owned source must remain a consumer sibling",
  );
  assert.equal(
    sourceTree,
    path.join(sourceRoot, "lsp-source"),
    "owned source tree identity changed",
  );
  for (const directory of [isolation.checkout, isolation.consumer, isolation.output])
    assert.ok(
      !overlaps(sourceRoot, directory),
      "source actor overlaps browser/checkout/output root",
    );
  if (expected)
    for (const key of ["checkout", "consumer", "output"] as const)
      assert.equal(isolation[key], expected[key], `source owner ${key} changed`);
  const markerPath = path.join(sourceRoot, ownershipFile);
  assert.ok(
    fs.lstatSync(markerPath).isFile() && !fs.lstatSync(markerPath).isSymbolicLink(),
    "normal source ownership marker required",
  );
  const marker = fs.readFileSync(markerPath);
  assert.equal(sha256(marker), isolation.markerSha256, "source ownership bytes changed");
  assert.deepEqual(
    JSON.parse(marker.toString()),
    {
      schema: "vize-public-lsp-source-owner-v1",
      sourceRoot,
      sourceTree,
      checkout: isolation.checkout,
      consumer: isolation.consumer,
      output: isolation.output,
    },
    "borrowed source ownership marker",
  );
}

/** Preserve the actual H tree; only the isolated adapter's test support is compiled. */
export function prepareLspSource(root: string, head: string, consumer: string, output: string) {
  execFileSync(
    "git",
    [
      "--no-replace-objects",
      "merge-base",
      "--is-ancestor",
      "9edc2a4459ced6200b12bc47b08e84c384c8e59d",
      head,
    ],
    { cwd: root },
  );
  const metadata = sourceMetadata(root, head);
  const ownedSource = prepareLspSourceRoot(root, consumer, output);
  const { sourceTree } = ownedSource;
  const archive = path.join(output, "lsp-source.tar");
  execFileSync(
    "git",
    ["--no-replace-objects", "archive", "--format=tar", `--output=${archive}`, head],
    { cwd: root, timeout: 30_000 },
  );
  execFileSync("tar", ["-xf", archive, "-C", sourceTree], { timeout: 30_000 });
  const files = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-r", "--name-only", head, "--", ...prefixes],
    { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  )
    .trim()
    .split("\n");
  assert.equal(files.length, 53, "the complete original two-target source closure");
  const atomicFiles = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-r", "--name-only", head, "--", ...atomicPrefixes],
    { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  )
    .trim()
    .split("\n");
  assert.equal(atomicFiles.length, 45, "the complete unchanged atomic-original source closure");
  const atomicSourceFiles = atomicFiles.map((file) => {
    const bytes = readRawBlob(root, head, file);
    assert.equal(sha256(fs.readFileSync(path.join(sourceTree, file))), sha256(bytes), file);
    return { file, sha256: sha256(bytes) };
  });
  // Retain all support bodies too, so inherited helpers cannot change after preparation.
  const support = execFileSync(
    "git",
    [
      "--no-replace-objects",
      "ls-tree",
      "-r",
      "--name-only",
      head,
      "--",
      "crates/vize_carton/",
      "davinci/vize_l0/",
      "davinci/vize_l0_derive/",
    ],
    { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  )
    .trim()
    .split("\n");
  const sourceFiles = [
    ...new Set([
      ...files,
      ...support,
      "Cargo.toml",
      "Cargo.lock",
      "rust-toolchain.toml",
      "clippy.toml",
    ]),
  ]
    .sort((left, right) => (left < right ? -1 : Number(left > right)))
    .map((file) => {
      const bytes = readRawBlob(root, head, file);
      assert.equal(sha256(fs.readFileSync(path.join(sourceTree, file))), sha256(bytes), file);
      return { file, sha256: sha256(bytes) };
    });
  const adapter = path.join(consumer, "crates/vize");
  fs.mkdirSync(adapter, { recursive: true });
  const quote = (text: string) => JSON.stringify(text);
  const p = metadata.package;
  const dependencies = Object.entries(supportDependencyVersions(metadata.dependencies)).map(
    ([name, version]) => `${name} = ${quote(version)}`,
  );
  const manifest =
    `[workspace]\n[package]\nname = "vize_public_lsp_contracts"\nversion = ${quote(p.version)}\nedition = ${quote(p.edition)}\nrust-version = ${quote(p["rust-version"])}\nautobins = false\nautolib = false\nautotests = false\n[dependencies]\n${dependencies.join("\n")}\nvize_carton = { path = ${quote(path.join(sourceTree, "crates/vize_carton"))} }\nvize_l0 = { path = ${quote(path.join(sourceTree, "davinci/vize_l0"))} }\n` +
    supportTargets
      .map(
        (name) =>
          `[[test]]\nname = ${quote(name)}\npath = ${quote(path.join(sourceTree, `crates/vize/tests/${name}.rs`))}\n`,
      )
      .join("");
  fs.writeFileSync(path.join(adapter, "Cargo.toml"), manifest, { flag: "wx" });
  fs.copyFileSync(path.join(sourceTree, "Cargo.lock"), path.join(adapter, "Cargo.lock"));
  fs.mkdirSync(path.join(consumer, "playground/node_modules"), { recursive: true });
  fs.symlinkSync(
    path.join(consumer, "node_modules/vue"),
    path.join(consumer, "playground/node_modules/vue"),
  );
  return {
    ...ownedSource,
    adapter,
    metadata,
    sourceFiles,
    originalFiles: files,
    atomicSourceFiles,
    archiveSha256: sha256(fs.readFileSync(archive)),
    manifestSha256: sha256(Buffer.from(manifest)),
  };
}

export function assertLspSource(preparation: ReturnType<typeof prepareLspSource>) {
  assertLspSourceRoot(preparation);
  for (const entry of preparation.sourceFiles)
    assert.equal(
      sha256(fs.readFileSync(path.join(preparation.sourceTree, entry.file))),
      entry.sha256,
      entry.file,
    );
  assert.equal(
    sha256(fs.readFileSync(path.join(preparation.adapter, "Cargo.toml"))),
    preparation.manifestSha256,
  );
}
