/** Seal the clean baseline CLI before adding the identical test-only observer. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { parseArgs } from "node:util";
import { identify, save } from "./canon-instance-directives-io.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
const { values } = parseArgs({
  options: {
    root: { type: "string" },
    cargo: { type: "string" },
    image: { type: "string" },
    out: { type: "string" },
    source: { type: "string" },
  },
});
assert(values.root && values.cargo && values.image && values.out && values.source);
const root = fs.realpathSync(values.root);
const git = (...args: string[]) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
assert.match(values.source, /^[a-f0-9]{40}$/u);
assert.equal(git("rev-parse", "HEAD"), values.source);
assert.equal(
  git("status", "--porcelain", "--untracked-files=all"),
  "",
  "baseline must still be clean before the observer import",
);
const rows: unknown[] = fs
  .readFileSync(values.cargo, "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line) as unknown);
const images = rows.filter(
  (row) =>
    isRecord(row) &&
    row.reason === "compiler-artifact" &&
    isRecord(row.target) &&
    row.target.name === "vize" &&
    Array.isArray(row.target.kind) &&
    row.target.kind.includes("bin") &&
    typeof row.executable === "string",
);
assert.equal(images.length, 1);
const artifact = images[0];
assert(isRecord(artifact));
assert.equal(artifact.fresh, false);
assert(typeof artifact.executable === "string");
assert(
  Array.isArray(artifact.features) &&
    artifact.features.includes("default") &&
    artifact.features.includes("maestro") &&
    artifact.features.includes("glyph"),
);
assert(!artifact.features.includes("profiling") && !artifact.features.includes("cache-custody"));
const sourceImage = identify(artifact.executable);
assert(!fs.existsSync(values.image));
fs.copyFileSync(artifact.executable, values.image);
const retainedImage = identify(values.image);
assert.equal(retainedImage.sha256, sourceImage.sha256);
save(values.out, {
  schema: "vize.ref-factory-clean-source-image",
  source: values.source,
  tree: git("rev-parse", "HEAD^{tree}"),
  cleanBeforeObserverCopy: true,
  productionOverlay: false,
  observerImportedYet: false,
  cargoArtifact: artifact,
  cargoLog: identify(values.cargo),
  image: retainedImage,
  sourceImage,
  cargoArgs: [
    "build",
    "--locked",
    "--profile",
    "ci-opt",
    "-p",
    "vize",
    "--bin",
    "vize",
    "--message-format=json",
  ],
  sourceInputs: [
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_canon/src/virtual_ts/helpers/preamble.rs",
  ].map((file) => ({ file, ...identify(path.join(root, file)) })),
  environment: Object.fromEntries(
    ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL", "RUSTC_WRAPPER"].map((key) => [
      key,
      process.env[key] ?? null,
    ]),
  ),
  timingComparison: false,
});
