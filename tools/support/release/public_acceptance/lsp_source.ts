import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readRawBlob } from "./plan.ts";

export const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export const targets = ["lsp_named_model_transactions_cli", "lsp_bound_event_casing_cli"];
export const corpus = "tests/_fixtures/differential/lsp/event-rename/4075/";
const prefixes = [
  ...targets.flatMap((name) => [`crates/vize/tests/${name}.rs`, `crates/vize/tests/${name}/`]),
  "crates/vize/tests/support/lsp_process.rs",
  "crates/vize/tests/support/lsp_process/",
  "crates/vize/tests/fixtures/content_mapper_project/tsconfig.json",
  `${corpus}named-model/`,
  `${corpus}bound-emitter/`,
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
  const sourceTree = path.join(consumer, "lsp-source");
  const archive = path.join(output, "lsp-source.tar");
  execFileSync(
    "git",
    ["--no-replace-objects", "archive", "--format=tar", `--output=${archive}`, head],
    { cwd: root, timeout: 30_000 },
  );
  fs.mkdirSync(sourceTree);
  execFileSync("tar", ["-xf", archive, "-C", sourceTree], { timeout: 30_000 });
  const files = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-r", "--name-only", head, "--", ...prefixes],
    { cwd: root, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  )
    .trim()
    .split("\n");
  assert.equal(files.length, 53, "the complete original two-target source closure");
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
    .sort()
    .map((file) => {
      const bytes = readRawBlob(root, head, file);
      assert.equal(sha256(fs.readFileSync(path.join(sourceTree, file))), sha256(bytes), file);
      return { file, sha256: sha256(bytes) };
    });
  const adapter = path.join(consumer, "crates/vize");
  fs.mkdirSync(adapter, { recursive: true });
  const quote = (text: string) => JSON.stringify(text);
  const p = metadata.package;
  const dependencies = ["serde_json", "tempfile", "sha2"].map((name) => {
    const value = metadata.dependencies[name];
    const version = typeof value === "string" ? value : value.version;
    assert.match(version, /^=\d+\.\d+\.\d+$/u);
    return `${name} = ${quote(version)}`;
  });
  const manifest =
    `[workspace]\n[package]\nname = "vize_public_lsp_contracts"\nversion = ${quote(p.version)}\nedition = ${quote(p.edition)}\nrust-version = ${quote(p["rust-version"])}\nautobins = false\nautolib = false\nautotests = false\n[dependencies]\n${dependencies.join("\n")}\nvize_carton = { path = ${quote(path.join(sourceTree, "crates/vize_carton"))} }\nvize_l0 = { path = ${quote(path.join(sourceTree, "davinci/vize_l0"))} }\n` +
    targets
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
    sourceTree,
    adapter,
    metadata,
    sourceFiles,
    originalFiles: files,
    archiveSha256: sha256(fs.readFileSync(archive)),
    manifestSha256: sha256(Buffer.from(manifest)),
  };
}

export function assertLspSource(preparation: ReturnType<typeof prepareLspSource>) {
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
